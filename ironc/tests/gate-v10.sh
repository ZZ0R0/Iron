#!/usr/bin/env bash
# [22300] GATE V10 final — le moteur d'inférence par contraintes. Prouve : zéro régression (corpus/binding/
# vitrine), **inférence sans annotation** (zero-annot), **diagnostics de type Iron natifs** (type-errors),
# et que l'ancienne pré-passe heuristique `params.rs` a bien disparu. Verdict honnête.
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$DIR/.." && pwd)"
BIN="$ROOT/target/debug/ironc"
ok=0; ko=0
chk() { if eval "$2" >/dev/null 2>&1; then echo "  ✓ $1"; ok=$((ok+1)); else echo "  ✗ $1"; ko=$((ko+1)); fi; }

cargo build --quiet --manifest-path "$ROOT/Cargo.toml" 2>/dev/null

echo "================== GATE V10 =================="

echo "[1] Non-régression (l'ancien corpus passe via le moteur)"
bash "$DIR/gate.sh" >/dev/null 2>&1 && { echo "  ✓ corpus 50 VERT"; ok=$((ok+1)); } || { echo "  ✗ corpus"; ko=$((ko+1)); }
bash "$DIR/binding/run.sh" >/dev/null 2>&1 && { echo "  ✓ binding 30 VERT"; ok=$((ok+1)); } || { echo "  ✗ binding"; ko=$((ko+1)); }
g=$("$BIN" run "$DIR/vitrine-v9.1/main.irn" 2>/dev/null); w=$(cat "$DIR/vitrine-v9.1/expected.txt")
[ "$g" = "$w" ] && { echo "  ✓ vitrine VERT"; ok=$((ok+1)); } || { echo "  ✗ vitrine"; ko=$((ko+1)); }

echo "[2] Inférence SANS annotation (zero-annot)"
for f in "$DIR"/zero-annot/*.irn; do
  name=$(basename "$f" .irn)
  got=$("$BIN" run "$f" 2>/dev/null); want=$(cat "${f%.irn}.expected" 2>/dev/null)
  if [ "$got" = "$want" ]; then echo "  ✓ $name"; ok=$((ok+1)); else echo "  ✗ $name (got: $(echo "$got"|tr '\n' '|'))"; ko=$((ko+1)); fi
done

echo "[3] Diagnostics de type Iron NATIFS (avant rustc, D8)"
for f in "$DIR"/fixtures/type-errors/*.irn; do
  name=$(basename "$f" .irn)
  out=$("$BIN" run "$f" 2>&1 | head -1)
  if echo "$out" | grep -q 'erreur: type :'; then echo "  ✓ $name → diag natif"; ok=$((ok+1)); else echo "  ✗ $name (pas natif: $out)"; ko=$((ko+1)); fi
done

echo "[4] Le moteur a remplacé l'ancienne heuristique"
chk "params.rs supprimé (heuristiques retirées)" "! test -f $ROOT/src/infer/params.rs"
chk "moteur engine/ présent (gen+solve+assume)" "test -f $ROOT/src/infer/engine/gen.rs && test -f $ROOT/src/infer/engine/solve.rs"
chk "polymorphisme inféré (id à 2 types)" "$BIN run $DIR/zero-annot/poly.irn 2>/dev/null | head -1 | grep -q 7"

echo "[5] Métrique de vérité (≥ V9)"
bash "$DIR/metric.sh" 2>/dev/null | grep -q 'VERDICT: VERT' && { echo "  ✓ métrique VERT"; ok=$((ok+1)); } || { echo "  ✗ métrique"; ko=$((ko+1)); }

echo "=============================================="
echo "Gate V10 : $ok OK / $ko KO"
if [ "$ko" -eq 0 ]; then
  echo "GATE V10: VERT — moteur de contraintes (unification + traits + bindings + let-poly) · inférence sans annotation · diagnostics de type Iron natifs"
  exit 0
else
  echo "GATE V10: ROUGE"
  exit 1
fi
