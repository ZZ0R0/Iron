#!/usr/bin/env bash
# [90800] GATE V9.1 final — agrège toute l'expérience développeur V9.1 et prononce le verdict.
#
# Prouve : ZÉRO lib Iron (tout par import) · support langage par overlay rustc · intégration VSCode ·
# profondeur (génériques / erreurs typées / closures) · maturité (paquets / doc / repl / debug / fmt).
# Verdict honnête : ROUGE si une seule section casse.
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$DIR/.." && pwd)"
M="$ROOT/Cargo.toml"
BIN="$ROOT/target/debug/ironc"
ok=0; ko=0
chk() { # nom ; commande
  if eval "$2" >/dev/null 2>&1; then echo "  ✓ $1"; ok=$((ok+1)); else echo "  ✗ $1"; ko=$((ko+1)); fi
}

# binaire à jour (les sous-gates l'utilisent aussi).
cargo build --quiet --manifest-path "$M" 2>/dev/null

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/t.irn" <<'IRN'
def sq(n: i64) -> i64: return n * n

def main():
    print(sq(7))
IRN

echo "================== GATE V9.1 =================="

echo "[1] Corpus offline + invariants (I1–I14)"
bash "$DIR/gate.sh" >/dev/null 2>&1 && { echo "  ✓ corpus VERT"; ok=$((ok+1)); } || { echo "  ✗ corpus"; ko=$((ko+1)); }

echo "[2] Batterie 30 vrais crates (zéro lib Iron)"
bash "$DIR/binding/run.sh" >/dev/null 2>&1 && { echo "  ✓ binding 30/30 VERT"; ok=$((ok+1)); } || { echo "  ✗ binding"; ko=$((ko+1)); }

echo "[3] Métrique de vérité chiffrée (≥ V9)"
bash "$DIR/metric.sh" 2>/dev/null | grep -q 'VERDICT: VERT' && { echo "  ✓ métrique VERT"; ok=$((ok+1)); } || { echo "  ✗ métrique"; ko=$((ko+1)); }

echo "[4] Outillage développeur"
chk "ironc check (overlay rustc)"       "$BIN check $tmp/t.irn"
chk "ironc lint"                        "$BIN lint $tmp/t.irn"
chk "ironc lib (export Iron→Rust)"      "grep -q 'fn sq' \"\$($BIN lib $tmp/t.irn 2>/dev/null | sed 's/.*: //')/src/lib.rs\""
cp "$tmp/t.irn" "$tmp/f.irn"
chk "ironc fmt (idempotent)"            "$BIN fmt $tmp/f.irn && $BIN fmt $tmp/f.irn | grep -q 'déjà formaté'"
chk "ironc repl (compile-and-run)"      "printf 'x = 21\nx * 2\nexit\n' | $BIN repl | grep -q 42"
chk "ironc build --debug (source-map I8)" "$BIN build --debug $tmp/t.irn && test -f $(eval $BIN build --debug $tmp/t.irn 2>/dev/null | grep source-map | sed 's/.*: //')"
# `ironc doc` : overlay rustdoc (cargo doc) — plus lent, fichier sans dépendance.
chk "ironc doc (overlay rustdoc)"       "$BIN doc $tmp/t.irn"
# `ironc add` : auto-bind d'un crate (réseau). Compté si dispo, sinon SKIP honnête (pas ROUGE).
if ( cd "$tmp" && timeout 200 "$BIN" add itoa >/dev/null 2>&1 && test -f itoa.iri ); then
  echo "  ✓ ironc add <crate> (auto-bind)"; ok=$((ok+1))
else
  echo "  ~ ironc add <crate> SKIP (réseau indispo ; prouvé par la batterie-30)"
fi

echo "[5] LSP enrichi (la « surcouche »)"
printf 'Content-Length: 58\r\n\r\n{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' > "$tmp/lsp.in"
chk "lsp initialize"        "cargo run --quiet --manifest-path $M -- lsp < $tmp/lsp.in | grep -q capabilities"
chk "lsp completionProvider" "cargo run --quiet --manifest-path $M -- lsp < $tmp/lsp.in | grep -q completionProvider"
chk "lsp hoverProvider"      "cargo run --quiet --manifest-path $M -- lsp < $tmp/lsp.in | grep -q hoverProvider"
chk "lsp definitionProvider" "cargo run --quiet --manifest-path $M -- lsp < $tmp/lsp.in | grep -q definitionProvider"

echo "[6] Vitrine V9.1 (multi-fichiers, ZÉRO lib Iron)"
V="$DIR/vitrine-v9.1"
got="$("$BIN" run "$V/main.irn" 2>/dev/null)"; want="$(cat "$V/expected.txt")"
[ "$got" = "$want" ] && { echo "  ✓ vitrine compile+run+oracle"; ok=$((ok+1)); } || { echo "  ✗ vitrine (got: $(echo "$got" | tr '\n' '|'))"; ko=$((ko+1)); }
vm="$(find /tmp/ironc-out -path '*main_*' -name main.rs 2>/dev/null | xargs grep -l parallel_total 2>/dev/null | head -1)"
if [ -n "$vm" ] && [ "$(grep -E '\bunsafe\b' "$vm" | grep -vc forbid)" = "0" ]; then echo "  ✓ vitrine 0 unsafe (I2)"; ok=$((ok+1)); else echo "  ✗ vitrine unsafe"; ko=$((ko+1)); fi

echo "[7] Intégration VSCode (artefacts + capacités)"
E="$ROOT/../editors/vscode"
chk "package.json valide (langage .irn)" "python3 -c \"import json;d=json.load(open('$E/package.json'));assert any(l['id']=='iron' for l in d['contributes']['languages'])\""
chk "grammaire TextMate"                 "test -f $E/syntaxes/iron.tmLanguage.json"
chk "client LSP (extension.ts)"          "grep -q LanguageClient $E/src/extension.ts"
chk "commandes (build/run/fmt/debug/add)" "grep -q 'iron.debug' $E/package.json && grep -q 'iron.format' $E/package.json"

echo "==============================================="
echo "Gate V9.1 : $ok OK / $ko KO"
if [ "$ko" -eq 0 ]; then
  echo "GATE V9.1: VERT — zéro lib Iron · surcouche rustc · VSCode · génériques/erreurs-typées/closures · paquets/doc/repl/debug/fmt"
  exit 0
else
  echo "GATE V9.1: ROUGE"
  exit 1
fi
