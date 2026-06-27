#!/usr/bin/env bash
# [V11] GATE de clôture — prouve que TOUS les trous de l'audit sont comblés (patterns, surface, traits,
# dunders, inférence enum/anti-unif), que le moteur est plus complet (DA-1..5), que les diagnostics de type
# riches sont natifs (champ/méthode/arité), sans aucune régression, et que le Rust généré reste sûr (0 unsafe).
# Verdict honnête. La comparaison-test utilisateur s'ajoute dans tests/comparison/ quand elle est fournie.
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$DIR/.." && pwd)"
BIN="$ROOT/target/debug/ironc"
ok=0; ko=0
chk() { if eval "$2" >/dev/null 2>&1; then echo "  ✓ $1"; ok=$((ok+1)); else echo "  ✗ $1"; ko=$((ko+1)); fi; }

cargo build --quiet --manifest-path "$ROOT/Cargo.toml" 2>/dev/null

echo "================== GATE V11 =================="

echo "[1] Non-régression (gate V10.1 : corpus 50 + binding 30 + vitrine + zero-annot + métrique + diag natifs)"
if bash "$DIR/gate-v10.1.sh" >/dev/null 2>&1; then echo "  ✓ gate V10.1 VERT"; ok=$((ok+1)); else echo "  ✗ gate V10.1"; ko=$((ko+1)); fi

echo "[2] Fixtures de comblement V11 (compile + run + oracle)"
for f in "$DIR"/v11/*.irn; do
  name=$(basename "$f" .irn)
  got=$("$BIN" run "$f" 2>/dev/null); want=$(cat "${f%.irn}.expected" 2>/dev/null)
  if [ "$got" = "$want" ]; then echo "  ✓ $name"; ok=$((ok+1)); else echo "  ✗ $name (got: $(echo "$got"|tr '\n' '|'))"; ko=$((ko+1)); fi
done

echo "[3] Couverture des features de l'audit (comportements clés)"
chk "or-patterns (1|2|3)"              "$BIN run $DIR/v11/patterns.irn | head -1 | grep -q bas"
chk "range-patterns (0..10 / 10..=20)" "$BIN run $DIR/v11/patterns.irn | sed -n 4p | grep -q petit"
chk "fn imbriquée (outer→dbl)"         "$BIN run $DIR/v11/surface.irn | sed -n 2p | grep -q 21"
chk "for-else (sans break → else)"     "$BIN run $DIR/v11/surface.irn | sed -n 5p | grep -q 'tous positifs'"
chk "affectation chaînée (a=b=5)"      "$BIN run $DIR/v11/surface.irn | head -1 | grep -q 10"
chk "const globale (MAX)"              "$BIN run $DIR/v11/surface.irn | sed -n 3p | grep -q 100"
chk "class(Trait) + méthode défaut"    "$BIN run $DIR/v11/traits.irn | head -1 | grep -q 'Rex!'"
chk "générique borné (T: Show)"        "$BIN run $DIR/v11/traits.irn | sed -n 2p | grep -q 'Tom!'"
chk "surcharge opérateur (Vec2 + Vec2)" "$BIN run $DIR/v11/dunders.irn | head -1 | grep -q 4"
chk "dunder == / < / str"              "$BIN run $DIR/v11/dunders.irn | tail -1 | grep -q vec"
chk "enum-construction inférée (area)" "$BIN run $DIR/v11/infer_enum.irn | head -1 | grep -q 12"
chk "anti-unification (count list[T])" "$BIN run $DIR/v11/infer_enum.irn | tail -1 | grep -q 2"

echo "[4] Diagnostics de type Iron NATIFS (DA-3 : champ / méthode / arité)"
for f in "$DIR"/v11/type-errors/*.irn; do
  name=$(basename "$f" .irn)
  out=$("$BIN" check "$f" 2>&1 | head -1)
  if echo "$out" | grep -q 'type :'; then echo "  ✓ $name → diag natif sur la ligne"; ok=$((ok+1)); else echo "  ✗ $name ($out)"; ko=$((ko+1)); fi
done

echo "[5] Audit (DA-5 couverture · flow.rs retiré · 0 unsafe)"
cov=$("$BIN" build --audit "$DIR/v11/infer_enum.irn" -o /dev/null 2>&1 | grep -o 'couverture moteur.*')
[ -n "$cov" ] && { echo "  ✓ $cov"; ok=$((ok+1)); } || { echo "  ✗ couverture moteur non reportée"; ko=$((ko+1)); }
[ ! -f "$ROOT/src/infer/flow.rs" ] && { echo "  ✓ flow.rs retiré (porté dans le moteur : engine/flux.rs)"; ok=$((ok+1)); } || { echo "  ✗ flow.rs encore présent"; ko=$((ko+1)); }
# 0 unsafe sur le Rust généré de toutes les fixtures V11 (forbid(unsafe_code) + aucun `unsafe` réel).
unsafe_tot=0
for f in "$DIR"/v11/*.irn; do
  "$BIN" build "$f" -o /dev/null >/dev/null 2>&1
  m=$(ls -d /tmp/ironc-out/$(basename "$f" .irn)_*/src/main.rs 2>/dev/null | head -1)
  [ -f "$m" ] && unsafe_tot=$((unsafe_tot + $(grep -E '\bunsafe\b' "$m" | grep -vc 'forbid')))
done
[ "$unsafe_tot" -eq 0 ] && { echo "  ✓ 0 unsafe (I2)"; ok=$((ok+1)); } || { echo "  ✗ $unsafe_tot unsafe"; ko=$((ko+1)); }

echo "[6] Comparaison-test utilisateur (tests/comparison/)"
if [ -d "$DIR/comparison" ] && ls "$DIR"/comparison/*.irn >/dev/null 2>&1; then
  for f in "$DIR"/comparison/*.irn; do
    name=$(basename "$f" .irn)
    got=$("$BIN" run "$f" 2>/dev/null); want=$(cat "${f%.irn}.expected" 2>/dev/null)
    if [ "$got" = "$want" ]; then echo "  ✓ comparaison $name"; ok=$((ok+1)); else echo "  ✗ comparaison $name"; ko=$((ko+1)); fi
  done
else
  echo "  ⏸️  en attente du test fourni par l'utilisateur (déposer .irn + .expected dans tests/comparison/)"
fi

echo "=============================================="
echo "Gate V11 : $ok OK / $ko KO"
if [ "$ko" -eq 0 ]; then
  echo "GATE V11: VERT — patterns · surface · traits · surcharge d'opérateurs · inférence enum+anti-unif · diagnostics riches · flow.rs retiré · 0 unsafe"
  exit 0
else
  echo "GATE V11: ROUGE"
  exit 1
fi
