set -eu
if grep -R -n -E '^[[:space:]]*impl(<[^>]*>)?[[:space:]]' "$src/src/lib.rs" "$src/src/generated" | grep -v ' for ' | grep -E '\{[[:space:]]*$'; then
  echo "Signal contract Rust must home behavior in traits" >&2
  exit 1
fi
touch "$out"
