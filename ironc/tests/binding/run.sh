#!/usr/bin/env bash
# Gate V8 (90300) : binde de vrais crates et vérifie l'e2e (compile + run + sortie = oracle).
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
MANIFEST="$DIR/../../Cargo.toml"
pass=0; fail=0; failed=""
for d in "$DIR"/*/; do
  name=$(basename "$d")
  [ -f "$d/main.irn" ] || continue
  got=$(cargo run --quiet --manifest-path "$MANIFEST" -- run "$d/main.irn" 2>/dev/null)
  want=$(cat "$d/expected.txt")
  if [ "$got" = "$want" ]; then pass=$((pass+1)); echo "  ✓ $name";
  else fail=$((fail+1)); failed="$failed $name"; echo "  ✗ $name (got: $(echo "$got" | tr '\n' '|'))"; fi
done
echo "Gate binding : $pass OK / $fail KO$([ -n "$failed" ] && echo "  (KO:$failed)")"
[ "$fail" -eq 0 ] && [ "$pass" -gt 0 ] && { echo "GATE BINDING: VERT"; exit 0; } || { echo "GATE BINDING: ROUGE"; exit 1; }
