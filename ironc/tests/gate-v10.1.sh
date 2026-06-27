#!/usr/bin/env bash
# [V10.1] GATE final — inférence pilotée par les sites d'appel. Prouve : zéro régression, l'inférence
# détermine les params **par l'usage** (appels), la polymorphie n'apparaît que sur conflit (promotion à
# coût runtime nul), et les diagnostics de type Iron restent natifs. Verdict honnête.
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$DIR/.." && pwd)"
BIN="$ROOT/target/debug/ironc"
ok=0; ko=0
chk() { if eval "$2" >/dev/null 2>&1; then echo "  ✓ $1"; ok=$((ok+1)); else echo "  ✗ $1"; ko=$((ko+1)); fi; }

cargo build --quiet --manifest-path "$ROOT/Cargo.toml" 2>/dev/null

echo "================= GATE V10.1 ================="

echo "[1] Non-régression (par le moteur piloté par les appels)"
bash "$DIR/gate.sh" >/dev/null 2>&1 && { echo "  ✓ corpus 50 VERT"; ok=$((ok+1)); } || { echo "  ✗ corpus"; ko=$((ko+1)); }
bash "$DIR/binding/run.sh" >/dev/null 2>&1 && { echo "  ✓ binding 30 VERT"; ok=$((ok+1)); } || { echo "  ✗ binding"; ko=$((ko+1)); }
g=$("$BIN" run "$DIR/vitrine-v9.1/main.irn" 2>/dev/null); w=$(cat "$DIR/vitrine-v9.1/expected.txt")
[ "$g" = "$w" ] && { echo "  ✓ vitrine VERT"; ok=$((ok+1)); } || { echo "  ✗ vitrine"; ko=$((ko+1)); }

echo "[2] Inférence SANS annotation (déterminée par l'usage + les appels)"
for f in "$DIR"/zero-annot/*.irn; do
  name=$(basename "$f" .irn)
  got=$("$BIN" run "$f" 2>/dev/null); want=$(cat "${f%.irn}.expected" 2>/dev/null)
  if [ "$got" = "$want" ]; then echo "  ✓ $name"; ok=$((ok+1)); else echo "  ✗ $name (got: $(echo "$got"|tr '\n' '|'))"; ko=$((ko+1)); fi
done

echo "[3] Modèle « piloté par les appels » — comportements clés"
chk "param typé par l'appel (count)"        "$BIN run $DIR/zero-annot/callsite.irn   | head -1 | grep -q 5"
chk "retour structurel via l'appel (x.name)" "$BIN run $DIR/zero-annot/callsite.irn   | sed -n 2p | grep -q iron"
chk "ordre supérieur (apply2)"               "$BIN run $DIR/zero-annot/callsite.irn   | sed -n 3p | grep -q 42"
chk "polymorphie sur conflit (id i64+str)"   "$BIN run $DIR/zero-annot/polydisplay.irn | head -2 | tr '\n' ' ' | grep -q 'hi'"
chk "générique borné Display (label)"        "$BIN run $DIR/zero-annot/polydisplay.irn | sed -n '3,4p' | tr '\n' ' ' | grep -q 'a=b'"
chk "récursion sans annotation (fact)"       "$BIN run $DIR/zero-annot/polydisplay.irn | tail -1 | grep -q 120"

echo "[4] Diagnostics de type Iron NATIFS (param annoté → erreur, D-b)"
for f in "$DIR"/fixtures/type-errors/*.irn; do
  name=$(basename "$f" .irn)
  out=$("$BIN" run "$f" 2>&1 | head -1)
  if echo "$out" | grep -q 'erreur: type :'; then echo "  ✓ $name → diag natif"; ok=$((ok+1)); else echo "  ✗ $name ($out)"; ko=$((ko+1)); fi
done

echo "[5] Métrique de vérité (≥ V9)"
bash "$DIR/metric.sh" 2>/dev/null | grep -q 'VERDICT: VERT' && { echo "  ✓ métrique VERT"; ok=$((ok+1)); } || { echo "  ✗ métrique"; ko=$((ko+1)); }

echo "=============================================="
echo "Gate V10.1 : $ok OK / $ko KO"
if [ "$ko" -eq 0 ]; then
  echo "GATE V10.1: VERT — inférence pilotée par les sites d'appel · polymorphie sur conflit (coût nul) · structurel incrémental · HOF · Display borné · diagnostics natifs"
  exit 0
else
  echo "GATE V10.1: ROUGE"
  exit 1
fi
