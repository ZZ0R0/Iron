//! [71100] Serveur LSP minimal (V9) : diagnostics **live** (lex/parse) sur stdio JSON-RPC.
//! Répond à `initialize`, publie des `publishDiagnostics` sur `didOpen`/`didChange`. Fondé sur le pipeline
//! Iron (front-end) ; les erreurs portent un `Span` Iron → position éditeur (I8).

use crate::ir::{CResult, CompileError};
use std::io::{self, BufRead, Write};

pub fn lsp() -> CResult<()> {
    let stdin = io::stdin();
    let mut r = stdin.lock();
    let stdout = io::stdout();
    let mut w = stdout.lock();
    while let Some(msg) = read_message(&mut r) {
        let id = msg.get("id").cloned();
        match msg["method"].as_str() {
            Some("initialize") => respond(
                &mut w,
                id,
                serde_json::json!({ "capabilities": { "textDocumentSync": 1 } }),
            ),
            Some("textDocument/didOpen") => {
                let td = &msg["params"]["textDocument"];
                publish(&mut w, td["uri"].as_str().unwrap_or(""), td["text"].as_str().unwrap_or(""));
            }
            Some("textDocument/didChange") => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = msg["params"]["contentChanges"][0]["text"].as_str().unwrap_or("").to_string();
                publish(&mut w, &uri, &text);
            }
            Some("shutdown") => respond(&mut w, id, serde_json::Value::Null),
            Some("exit") => break,
            _ => {}
        }
    }
    Ok(())
}

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
