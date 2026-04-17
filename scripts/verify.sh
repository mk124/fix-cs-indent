#!/usr/bin/env bash
# Smoke test: run the installed binary against a fixture and verify blank-line indent is restored.
set -euo pipefail

BIN="${INSTALL_DIR:-$HOME/.claude/bin}/fix-cs-indent"
if [[ ! -x "$BIN" ]]; then
    echo "verify FAILED: binary not found at $BIN" >&2
    exit 1
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

FIXTURE="$TMPDIR/sample.cs"
cat > "$FIXTURE" <<'EOF'
class A
{
    int x;

    int y;
}
EOF

# Send hook JSON to the binary, whitelist the tmpdir. Unset dry-run so verify
# actually writes back even when the parent shell exports it.
JSON=$(printf '{"tool_name":"Edit","tool_input":{"file_path":"%s"}}' "$FIXTURE")
echo "$JSON" | env -u FIX_CS_INDENT_DRY_RUN FIX_CS_INDENT_ROOTS="$TMPDIR" "$BIN"

# The blank line (line 4) should now contain 4 spaces.
if awk 'NR==4 && /^    $/ {found=1} END {exit !found}' "$FIXTURE"; then
    echo "verify OK"
else
    echo "verify FAILED: blank line 4 not indented as expected" >&2
    echo "--- fixture content (cat -e) ---" >&2
    cat -e "$FIXTURE" >&2
    exit 1
fi
