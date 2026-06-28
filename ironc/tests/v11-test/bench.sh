#!/usr/bin/env bash
# [V11-test] Harnais différentiel + perf : CPython (python3) vs Iron→Rust, mono-cœur.
# Pour chaque tests/v11-test/<prog>/ contenant <prog>.py + <prog>.irn + arg :
#   1. compile le .irn (ironc → projet) puis `cargo build --release` (binaire natif)
#   2. correction : sortie python3 == sortie binaire Iron (sinon ROUGE, pas de chrono)
#   3. perf : N runs `taskset -c 0` de chacun, garde le MIN ; speedup = t_py / t_iron
#   4. vérifie 0 `unsafe` réel dans le Rust généré
# Tableau final + speedup géométrique global. Mode résumé (les programmes n'impriment qu'une ligne).
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$DIR/../.." && pwd)"
IRONC="$ROOT/target/release/ironc"
N="${N:-10}"                 # runs par mesure
ONLY="${1:-}"                # filtrer un seul programme (debug)

command -v taskset >/dev/null 2>&1 && PIN="taskset -c 0" || PIN=""

# min wall-time de N exécutions de "$@"
best_time() {
  local best=""
  for _ in $(seq 1 "$N"); do
    local t0 t1 dt
    t0=$(date +%s.%N)
    $PIN "$@" >/dev/null 2>&1
    t1=$(date +%s.%N)
    dt=$(echo "$t1 - $t0" | bc -l)
    if [ -z "$best" ] || (( $(echo "$dt < $best" | bc -l) )); then best=$dt; fi
  done
  echo "$best"
}

printf "%-12s | %-6s | %10s | %10s | %8s\n" "prog" "ok" "t_py(s)" "t_iron(s)" "speedup"
printf -- "-------------|--------|------------|------------|----------\n"

geo_sum=0; geo_n=0; pass=0; total=0; fails=""

for d in "$DIR"/*/; do
  prog=$(basename "$d")
  py="$d/$prog.py"; irn="$d/$prog.irn"
  [ -f "$py" ] && [ -f "$irn" ] || continue
  [ -n "$ONLY" ] && [ "$ONLY" != "$prog" ] && continue
  total=$((total+1))

  # argument unique : un nombre, ou un nom de dataset sous data/
  raw=$(cat "$d/arg" 2>/dev/null)
  if [ -f "$DIR/data/$raw" ]; then arg="$DIR/data/$raw"; else arg="$raw"; fi

  # 1. compile (ironc émet le projet + build debug = check de compilation)
  if ! "$IRONC" build "$irn" >/dev/null 2>&1; then
    printf "%-12s | %-6s | %10s | %10s | %8s\n" "$prog" "BUILD" "-" "-" "-"; fails="$fails $prog(build)"; continue
  fi
  out=$(ls -td /tmp/ironc-out/${prog}_*/ 2>/dev/null | head -1)
  ( cd "$out" && cargo build --release --quiet 2>/dev/null )
  bin="$out/target/release/iron_out"
  if [ ! -x "$bin" ]; then
    printf "%-12s | %-6s | %10s | %10s | %8s\n" "$prog" "RELBLD" "-" "-" "-"; fails="$fails $prog(rel)"; continue
  fi

  # 0 unsafe réel
  mainrs="$out/src/main.rs"
  if [ -f "$mainrs" ] && [ "$(grep -E '\bunsafe\b' "$mainrs" | grep -vc forbid)" -ne 0 ]; then
    fails="$fails $prog(unsafe)"
  fi

  # 2. correction
  out_py=$(python3 "$py" "$arg" 2>/dev/null)
  out_ir=$("$bin" "$arg" 2>/dev/null)
  if [ "$out_py" != "$out_ir" ]; then
    printf "%-12s | %-6s | %10s | %10s | %8s\n" "$prog" "DIFF" "-" "-" "-"
    fails="$fails $prog(diff)"; continue
  fi

  # 3. perf
  t_py=$(best_time python3 "$py" "$arg")
  t_ir=$(best_time "$bin" "$arg")
  sp=$(echo "scale=2; $t_py / $t_ir" | bc -l)
  printf "%-12s | %-6s | %10.3f | %10.3f | %7.2fx\n" "$prog" "OK" "$t_py" "$t_ir" "$sp"
  pass=$((pass+1))
  geo_sum=$(echo "$geo_sum + l($sp)" | bc -l); geo_n=$((geo_n+1))
done

printf -- "-------------|--------|------------|------------|----------\n"
if [ "$geo_n" -gt 0 ]; then
  geo=$(echo "scale=2; e($geo_sum / $geo_n)" | bc -l)
  printf "Programmes OK : %d/%d   |   speedup géométrique CPython→Iron : %.2fx\n" "$pass" "$total" "$geo"
fi
[ -n "$fails" ] && echo "Échecs :$fails"
exit 0
