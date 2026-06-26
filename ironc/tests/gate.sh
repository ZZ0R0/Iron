#!/usr/bin/env bash
# Gate / runner du corpus (V6/6.1 + harnais 5.1).
#
# Compile + exécute chaque programme `.irn` du corpus, agrège correction / densité de clones / `unsafe`.
# NB : 4 fixtures sont des tests NÉGATIFS (remontée d'erreur) — listées ci-dessous, exclues du décompte
# « positif ». La **métrique de vérité** (correction sémantique chiffrée) reste différée jusqu'au gel du
# langage + corpus ≫ 37 (cf. specs/V6/realisation.md) ; ce script en est le harnais.
set -u
cd "$(dirname "$0")/.." || exit 1

NEGATIVE="type_error nonexhaustive bad_extern p_forvar"  # échecs attendus (négatifs / D7b différé)
pass=0; fail=0; clones=0; unsafe=0; failed=""

is_negative() { case " $NEGATIVE " in *" $1 "*) return 0;; *) return 1;; esac; }

for f in tests/fixtures/*.irn; do
  stem=$(basename "$f" .irn)
  is_negative "$stem" && continue
  if cargo run --quiet -- run "$f" >/dev/null 2>&1; then pass=$((pass+1)); else fail=$((fail+1)); failed="$failed $stem"; fi
  m=$(ls /tmp/ironc-out/${stem}_*/src/main.rs 2>/dev/null | head -1)
  [ -f "$m" ] && clones=$((clones + $(grep -o '\.clone()' "$m" | wc -l)))
  [ -f "$m" ] && unsafe=$((unsafe + $(grep -E '\bunsafe\b' "$m" | grep -vc 'forbid')))
done

echo "Corpus positif : $pass OK / $fail KO$([ -n "$failed" ] && echo "  (KO:$failed)")"
echo "Clones générés : $clones   |   unsafe réel : $unsafe"
[ "$fail" -eq 0 ] && [ "$unsafe" -eq 0 ] && { echo "GATE: VERT"; exit 0; } || { echo "GATE: ROUGE"; exit 1; }
