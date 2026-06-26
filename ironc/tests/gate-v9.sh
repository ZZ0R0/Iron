#!/usr/bin/env bash
# [90500] Gate V9 — agrège tous les gates + l'outillage pour prononcer un verdict honnête.
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
M="$DIR/../Cargo.toml"
ok=0; ko=0
chk() { # nom + commande : OK si exit 0
  if eval "$2" >/dev/null 2>&1; then echo "  ✓ $1"; ok=$((ok+1)); else echo "  ✗ $1"; ko=$((ko+1)); fi
}
echo "================ GATE V9 ================"
echo "[1] Corpus offline + invariants (gate.sh)"
bash "$DIR/gate.sh" >/dev/null 2>&1 && { echo "  ✓ corpus VERT"; ok=$((ok+1)); } || { echo "  ✗ corpus"; ko=$((ko+1)); }
echo "[2] Batterie de vrais crates (binding/run.sh)"
bash "$DIR/binding/run.sh" >/dev/null 2>&1 && { echo "  ✓ binding VERT"; ok=$((ok+1)); } || { echo "  ✗ binding"; ko=$((ko+1)); }
echo "[3] Métrique de vérité chiffrée"
bash "$DIR/metric.sh" 2>/dev/null | grep -q 'VERDICT: VERT' && { echo "  ✓ métrique VERT"; ok=$((ok+1)); } || { echo "  ✗ métrique"; ko=$((ko+1)); }
echo "[4] Outillage"
tmp=$(mktemp -d)
printf 'def add(a: i64, b: i64) -> i64:\n    return a + b\n' > "$tmp/m.irn"
chk "ironc lib (export Iron→Rust)" "cargo run --quiet --manifest-path $M -- lib $tmp/m.irn"
printf 'def main():\n    x = 1\n    y = 2\n    print(y)\n' > "$tmp/l.irn"
chk "ironc lint" "cargo run --quiet --manifest-path $M -- lint $tmp/l.irn"
chk "ironc run/build" "cargo run --quiet --manifest-path $M -- build $tmp/l.irn"
rm -rf "$tmp"
echo "========================================"
echo "Gate V9 : $ok OK / $ko KO"
[ "$ko" -eq 0 ] && echo "GATE V9: VERT (Pareto réalisé : interop bidir. + concurrence + polish + tooling + métrique)" \
  || echo "GATE V9: ROUGE"
echo "Reste (20% assumé, hors Pareto) : async · LSP · yield · fmt · source-maps-expr · type-errors · repl · registre · batterie-30"
