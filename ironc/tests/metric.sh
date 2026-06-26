#!/usr/bin/env bash
# [90100] Métrique de vérité CHIFFRÉE (V9, 6.2). Agrège la correction sémantique Iron→Rust sur le corpus
# offline + la batterie de vrais crates, plus le coût clone-fallback (I4) et l'invariant `unsafe` (I2).
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR/.." || exit 1
MANIFEST="$DIR/../Cargo.toml"

# --- 1. Corpus offline (fixtures) : compile+run, compte clones + unsafe ---
NEG="type_error nonexhaustive bad_extern p_forvar"
is_neg() { case " $NEG " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }
cpass=0; cfail=0; clones=0; unsafe=0
for f in tests/fixtures/*.irn; do
  stem=$(basename "$f" .irn); is_neg "$stem" && continue
  if cargo run --quiet --manifest-path "$MANIFEST" -- run "$f" >/dev/null 2>&1; then cpass=$((cpass+1)); else cfail=$((cfail+1)); fi
  m=$(ls /tmp/ironc-out/${stem}_*/src/main.rs 2>/dev/null | head -1)
  [ -f "$m" ] && clones=$((clones + $(grep -o '\.clone()' "$m" | wc -l)))
  [ -f "$m" ] && unsafe=$((unsafe + $(grep -cE '\bunsafe\b' "$m" | grep -v forbid || true)))
done

# --- 2. Batterie de vrais crates : sortie == oracle ---
bpass=0; bfail=0
for d in tests/binding/*/; do
  [ -f "$d/main.irn" ] || continue
  got=$(cargo run --quiet --manifest-path "$MANIFEST" -- run "$d/main.irn" 2>/dev/null)
  [ "$got" = "$(cat "$d/expected.txt")" ] && bpass=$((bpass+1)) || bfail=$((bfail+1))
done

tot=$((cpass + cfail + bpass + bfail))
ok=$((cpass + bpass))
pct=$(( ok * 1000 / (tot>0?tot:1) ))
echo "================ MÉTRIQUE DE VÉRITÉ (90100) ================"
printf "Correction Iron→Rust : %d/%d programmes (%d.%d%%)\n" "$ok" "$tot" $((pct/10)) $((pct%10))
printf "  · corpus offline   : %d OK / %d KO\n" "$cpass" "$cfail"
printf "  · vrais crates     : %d OK / %d KO\n" "$bpass" "$bfail"
printf "Coût clone-fallback  : %d clones générés (corpus)\n" "$clones"
printf "Invariant I2         : %d unsafe réel (doit être 0)\n" "$unsafe"
echo "==========================================================="
[ "$cfail" -eq 0 ] && [ "$bfail" -eq 0 ] && [ "$unsafe" -eq 0 ] && echo "VERDICT: VERT" || echo "VERDICT: ROUGE"
