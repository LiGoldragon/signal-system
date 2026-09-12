set -eu
if grep -R -n -E '^(pub(\([^)]*\))? )?fn ' "$src/src/lib.rs" "$src/src/generated"; then
  echo "Signal contract Rust must not use module-level free functions" >&2
  exit 1
fi
touch "$out"
