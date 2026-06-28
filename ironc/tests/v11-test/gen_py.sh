#!/usr/bin/env bash
# Derive each <prog>/<prog>.py from <prog>/<prog>.irn (mechanical rebind, opposite direction).
DIR="$(cd "$(dirname "$0")" && pwd)"
for irn in "$DIR"/*/*.irn; do
  prog=$(basename "$irn" .irn)
  py="$(dirname "$irn")/$prog.py"
  sed -e 's/^import std$/import sys/' \
      -e 's/args()/sys.argv/g' \
      -e 's/read_text(\([^)]*\))/open(\1).read()/g' "$irn" > "$py"
  printf '\nmain()\n' >> "$py"
done
echo "généré $(ls "$DIR"/*/*.py | wc -l) fichiers .py"
