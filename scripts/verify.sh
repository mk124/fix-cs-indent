#!/usr/bin/env bash
# Smoke test the release binary against a complete expected file.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
if [[ "$TARGET_DIR" != /* ]]; then
    TARGET_DIR="$ROOT_DIR/$TARGET_DIR"
fi
BIN="$TARGET_DIR/release/fix-cs-indent"
if [[ ! -x "$BIN" ]]; then
    echo "verify FAILED: binary not found at $BIN" >&2
    exit 1
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

FIXTURE="$TMPDIR/sample.cs"
EXPECTED="$TMPDIR/expected.cs"
cat > "$FIXTURE" <<'EOF'
class A
{
    int x;

    int y;
}
EOF
printf 'class A\n{\n    int x;\n    \n    int y;\n}\n' > "$EXPECTED"

JSON=$(printf '{"tool_name":"Edit","tool_input":{"file_path":"%s"}}' "$FIXTURE")
printf '%s\n' "$JSON" | "$BIN"

if cmp -s "$EXPECTED" "$FIXTURE"; then
    echo "verify OK"
else
    echo "verify FAILED: output did not match expected content" >&2
    diff -u "$EXPECTED" "$FIXTURE" >&2 || true
    exit 1
fi
