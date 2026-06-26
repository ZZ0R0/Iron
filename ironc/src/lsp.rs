//! [71100/71300] Serveur LSP (V9 + V9.1) sur stdio JSON-RPC. Diagnostics **live** (lex/parse) +
//! **complétion / hover / go-to-def** (V9.1, 71300). Tout est dérivé du front-end Iron (lex→parse→AST) :
//! pas de type-checker maison — les symboles viennent de l'AST, les positions d'un `Span` Iron (I8).

use crate::ir::{CResult, CompileError, ItemKind, NumKind, Span, Type};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

pub fn lsp() -> CResult<()> {
    let stdin = io::stdin();
    let mut r = stdin.lock();
    let stdout = io::stdout();
    let mut w = stdout.lock();
    // Magasin de documents (uri → texte courant) : complétion/hover/def en ont besoin (le diag. seul, non).
    let mut docs: HashMap<String, String> = HashMap::new();
    while let Some(msg) = read_message(&mut r) {
        let id = msg.get("id").cloned();
        match msg["method"].as_str() {
            Some("initialize") => respond(
                &mut w,
                id,
                serde_json::json!({ "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": { "triggerCharacters": ["."] },
                    "hoverProvider": true,
                    "definitionProvider": true
                }}),
            ),
            Some("textDocument/didOpen") => {
                let td = &msg["params"]["textDocument"];
                let uri = td["uri"].as_str().unwrap_or("").to_string();
                let text = td["text"].as_str().unwrap_or("").to_string();
                publish(&mut w, &uri, &text);
                docs.insert(uri, text);
            }
            Some("textDocument/didChange") => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = msg["params"]["contentChanges"][0]["text"].as_str().unwrap_or("").to_string();
                publish(&mut w, &uri, &text);
                docs.insert(uri, text);
            }
            Some("textDocument/completion") => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
                let text = docs.get(uri).cloned().unwrap_or_default();
                respond(&mut w, id, completion(&text));
            }
            Some("textDocument/hover") => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
                let text = docs.get(uri).cloned().unwrap_or_default();
                let pos = &msg["params"]["position"];
                respond(&mut w, id, hover(&text, pos["line"].as_u64().unwrap_or(0) as usize, pos["character"].as_u64().unwrap_or(0) as usize));
            }
            Some("textDocument/definition") => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = docs.get(&uri).cloned().unwrap_or_default();
                let pos = &msg["params"]["position"];
                respond(&mut w, id, definition(&uri, &text, pos["line"].as_u64().unwrap_or(0) as usize, pos["character"].as_u64().unwrap_or(0) as usize));
            }
            Some("shutdown") => respond(&mut w, id, serde_json::Value::Null),
            Some("exit") => break,
            _ => {
                // Toute autre requête (avec `id`) reçoit une réponse nulle plutôt qu'un silence (timeout client).
                if id.is_some() {
                    respond(&mut w, id, serde_json::Value::Null);
                }
            }
        }
    }
    Ok(())
}

// ----------------------------------------------------------------- symboles (AST → table)

/// Un symbole top-level du fichier : nom, genre LSP, signature lisible (hover), `Span` (go-to-def).
struct Sym {
    name: String,
    kind: u8, // CompletionItemKind / mappé pour hover
    detail: String,
    span: Span,
}

/// Extrait les symboles top-level de l'AST (fonctions, classes/structs, enums, traits, méthodes).
/// Front-end pur : si le fichier ne parse pas, pas de symbole (les diagnostics signalent l'erreur).
fn symbols(text: &str) -> Vec<Sym> {
    let toks = match crate::lex::lex(text, 0) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let ast = match crate::parse::parse(toks) {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { name, params, ret, is_async, .. } => {
                let kw = if *is_async { "async def" } else { "def" };
                out.push(Sym { name: name.clone(), kind: 3, detail: fn_sig(kw, name, params, ret), span: it.span });
            }
            ItemKind::Struct { name, fields, .. } => {
                let fs = fields.iter().map(|(n, t)| format!("    {n}: {}", ty(t))).collect::<Vec<_>>().join("\n");
                let detail = if fs.is_empty() { format!("class {name}") } else { format!("class {name}:\n{fs}") };
                out.push(Sym { name: name.clone(), kind: 7, detail, span: it.span });
            }
            ItemKind::Enum { name, variants, .. } => {
                let vs = variants.iter().map(|(n, _)| format!("    {n}")).collect::<Vec<_>>().join("\n");
                out.push(Sym { name: name.clone(), kind: 13, detail: format!("enum {name}:\n{vs}"), span: it.span });
            }
            ItemKind::Trait { name, .. } => {
                out.push(Sym { name: name.clone(), kind: 8, detail: format!("trait {name}"), span: it.span });
            }
            ItemKind::Impl { type_name, methods, .. } => {
                for m in methods {
                    let recv = if m.self_param { "self, " } else { "" };
                    let ps = recv.to_string() + &m.params.iter().map(|(n, t)| format!("{n}: {}", ty(t))).collect::<Vec<_>>().join(", ");
                    let r = if matches!(m.ret, Type::Unit) { String::new() } else { format!(" -> {}", ty(&m.ret)) };
                    out.push(Sym {
                        name: m.name.clone(),
                        kind: 2,
                        detail: format!("def {type_name}.{}({}){}", m.name, ps.trim_end_matches(", "), r),
                        span: it.span,
                    });
                }
            }
        }
    }
    out
}

fn fn_sig(kw: &str, name: &str, params: &[(String, Type)], ret: &Type) -> String {
    let ps = params.iter().map(|(n, t)| format!("{n}: {}", ty(t))).collect::<Vec<_>>().join(", ");
    let r = if matches!(ret, Type::Unit) { String::new() } else { format!(" -> {}", ty(ret)) };
    format!("{kw} {name}({ps}){r}")
}

/// `Type` → syntaxe de surface Iron (pour hover/signatures).
fn ty(t: &Type) -> String {
    match t {
        Type::I64 => "i64".into(),
        Type::F64 => "f64".into(),
        Type::Bool => "bool".into(),
        Type::Str => "str".into(),
        Type::Unit => "unit".into(),
        Type::Char => "char".into(),
        Type::Struct(n) | Type::Enum(n) | Type::Generic(n) => n.clone(),
        Type::Extern(p) => p.clone(),
        Type::Option(x) => format!("Option[{}]", ty(x)),
        Type::Result(x) => format!("Result[{}]", ty(x)),
        Type::List(x) => format!("list[{}]", ty(x)),
        Type::Set(x) => format!("set[{}]", ty(x)),
        Type::Shared(x) => format!("shared[{}]", ty(x)),
        Type::SharedSync(x) => format!("shared_sync[{}]", ty(x)),
        Type::Weak(x) => format!("Weak[{}]", ty(x)),
        Type::Ref(x) => format!("inout {}", ty(x)),
        Type::Dict(k, v) => format!("dict[{}, {}]", ty(k), ty(v)),
        Type::App(n, args) => format!("{}[{}]", n, args.iter().map(ty).collect::<Vec<_>>().join(", ")),
        Type::Tuple(xs) => format!("({})", xs.iter().map(ty).collect::<Vec<_>>().join(", ")),
        Type::Fn(ps, r) => format!("fn({}) -> {}", ps.iter().map(ty).collect::<Vec<_>>().join(", "), ty(r)),
        Type::Num(k) => num_name(*k).into(),
    }
}

fn num_name(k: NumKind) -> &'static str {
    match k {
        NumKind::I8 => "i8",
        NumKind::I16 => "i16",
        NumKind::I32 => "i32",
        NumKind::I128 => "i128",
        NumKind::Isize => "isize",
        NumKind::U8 => "u8",
        NumKind::U16 => "u16",
        NumKind::U32 => "u32",
        NumKind::U64 => "u64",
        NumKind::U128 => "u128",
        NumKind::Usize => "usize",
        NumKind::F32 => "f32",
    }
}

// ----------------------------------------------------------------- completion / hover / definition

const KEYWORDS: &[&str] = &[
    "def", "class", "enum", "trait", "impl", "if", "elif", "else", "while", "for", "in", "return",
    "break", "continue", "pass", "match", "import", "use", "extern", "lambda", "and", "or", "not",
    "True", "False", "async", "await", "yield", "raise", "try", "except", "assert", "inout", "print",
];
const BUILTIN_TYPES: &[&str] = &[
    "i64", "f64", "bool", "str", "char", "unit", "list", "dict", "set", "Option", "Result", "shared", "shared_sync",
];

/// `textDocument/completion` : mots-clés + types intégrés + symboles top-level du fichier. Le client
/// filtre par préfixe ; on renvoie l'ensemble (position-insensible, suffisant et robuste).
fn completion(text: &str) -> serde_json::Value {
    let mut items: Vec<serde_json::Value> = Vec::new();
    for k in KEYWORDS {
        items.push(serde_json::json!({ "label": k, "kind": 14 })); // 14 = Keyword
    }
    for t in BUILTIN_TYPES {
        items.push(serde_json::json!({ "label": t, "kind": 22 })); // 22 = Struct (type intégré)
    }
    for s in symbols(text) {
        items.push(serde_json::json!({ "label": s.name, "kind": s.kind, "detail": s.detail }));
    }
    serde_json::json!({ "isIncomplete": false, "items": items })
}

/// `textDocument/hover` : signature du symbole sous le curseur (en bloc ```iron```), ou `null`.
fn hover(text: &str, line: usize, ch: usize) -> serde_json::Value {
    let off = offset_of(text, line, ch);
    let Some(word) = word_at(text, off) else { return serde_json::Value::Null };
    if let Some(s) = symbols(text).into_iter().find(|s| s.name == word) {
        return serde_json::json!({
            "contents": { "kind": "markdown", "value": format!("```iron\n{}\n```", s.detail) }
        });
    }
    // mots-clés / types intégrés : hover minimal.
    if KEYWORDS.contains(&word.as_str()) {
        return serde_json::json!({ "contents": { "kind": "markdown", "value": format!("mot-clé Iron `{word}`") } });
    }
    if BUILTIN_TYPES.contains(&word.as_str()) {
        return serde_json::json!({ "contents": { "kind": "markdown", "value": format!("type intégré `{word}`") } });
    }
    serde_json::Value::Null
}

/// `textDocument/definition` : `Location` du symbole top-level sous le curseur, ou `null`.
fn definition(uri: &str, text: &str, line: usize, ch: usize) -> serde_json::Value {
    let off = offset_of(text, line, ch);
    let Some(word) = word_at(text, off) else { return serde_json::Value::Null };
    let Some(s) = symbols(text).into_iter().find(|s| s.name == word) else { return serde_json::Value::Null };
    let (sl, sc) = line_col(text, s.span.start as usize);
    // Plage : du début de l'item à la fin de son nom (approx. = début + longueur du nom sur la ligne).
    serde_json::json!({
        "uri": uri,
        "range": {
            "start": { "line": sl, "character": sc },
            "end": { "line": sl, "character": sc + 1 }
        }
    })
}

/// Position LSP (line, character) → offset octet dans `text`. `character` traité en *chars* (≈ UTF-16
/// pour de l'ASCII, ce que sont les identifiants Iron) — approximation assumée.
fn offset_of(text: &str, line: usize, ch: usize) -> usize {
    let mut off = 0usize;
    let mut l = 0usize;
    for li in text.split_inclusive('\n') {
        if l == line {
            let mut c = 0usize;
            for (bi, _) in li.char_indices() {
                if c == ch {
                    return off + bi;
                }
                c += 1;
            }
            return off + li.trim_end_matches('\n').len();
        }
        off += li.len();
        l += 1;
    }
    off
}

/// Identifiant (`[A-Za-z0-9_]+`) couvrant `off` (curseur dedans ou en bordure), ou `None`.
fn word_at(text: &str, off: usize) -> Option<String> {
    let b = text.as_bytes();
    let off = off.min(b.len());
    let is_id = |x: u8| x == b'_' || x.is_ascii_alphanumeric();
    let mut s = off;
    while s > 0 && is_id(b[s - 1]) {
        s -= 1;
    }
    let mut e = off;
    while e < b.len() && is_id(b[e]) {
        e += 1;
    }
    if s == e {
        None
    } else {
        Some(text[s..e].to_string())
    }
}

// ----------------------------------------------------------------- diagnostics (V9, inchangé)

/// Front-end Iron (lex+parse) → diagnostics `(line, char, message)`.
fn check(text: &str) -> Vec<(usize, usize, String)> {
    let toks = match crate::lex::lex(text, 0) {
        Ok(t) => t,
        Err(e) => return vec![err_pos(text, &e)],
    };
    match crate::parse::parse(toks) {
        Ok(_) => Vec::new(),
        Err(e) => vec![err_pos(text, &e)],
    }
}

fn err_pos(text: &str, e: &CompileError) -> (usize, usize, String) {
    let off = e.span.map(|s| s.start as usize).unwrap_or(0);
    let (l, c) = line_col(text, off);
    (l, c, e.msg.clone())
}

fn line_col(text: &str, off: usize) -> (usize, usize) {
    let (mut line, mut col) = (0usize, 0usize);
    for (i, ch) in text.char_indices() {
        if i >= off {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn publish(out: &mut impl Write, uri: &str, text: &str) {
    let diags: Vec<serde_json::Value> = check(text)
        .into_iter()
        .map(|(l, c, m)| {
            serde_json::json!({
                "range": { "start": {"line": l, "character": c}, "end": {"line": l, "character": c + 1} },
                "severity": 1, "source": "ironc", "message": m
            })
        })
        .collect();
    write_message(
        out,
        &serde_json::json!({
            "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": { "uri": uri, "diagnostics": diags }
        }),
    );
}

fn respond(out: &mut impl Write, id: Option<serde_json::Value>, result: serde_json::Value) {
    write_message(out, &serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }));
}

fn read_message(r: &mut impl BufRead) -> Option<serde_json::Value> {
    let mut len = 0usize;
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(n) = line.strip_prefix("Content-Length:") {
            len = n.trim().parse().ok()?;
        }
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

fn write_message(out: &mut impl Write, v: &serde_json::Value) {
    let s = v.to_string();
    let _ = write!(out, "Content-Length: {}\r\n\r\n{}", s.len(), s);
    let _ = out.flush();
}
