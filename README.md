# fix-cs-indent

`fix-cs-indent` is a Claude Code `PostToolUse` hook that restores indentation
on blank lines in C# files. It parses each edited file with Tree-sitter, leaves
blank lines inside strings, comments, and malformed regions untouched, and
ignores non-C# and non-UTF-8 files.

## Install

```sh
make install
```

Installation runs the complete verification suite before copying the binary to
`~/.claude/bin/fix-cs-indent`. To use another directory:

```sh
make install INSTALL_DIR="$HOME/bin"
```

When overriding `INSTALL_DIR`, update the hook `command` below to use the same
directory.

Run `make verify` to verify the release binary without installing it.

## Configure Claude Code

Merge this hook into `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "\"$HOME/.claude/bin/fix-cs-indent\""
          }
        ]
      }
    ]
  }
}
```

If the settings file already contains a `hooks` object or other
`PostToolUse` entries, merge this entry instead of replacing them. See the
[Claude Code hooks documentation](https://code.claude.com/docs/en/hooks) for
the complete configuration schema.

## Diagnostics

The hook is silent by default. To record its decisions while troubleshooting,
set `FIX_CS_INDENT_LOG` to a log file in the hook command:

```json
"command": "FIX_CS_INDENT_LOG=\"$HOME/.claude/logs/fix-cs-indent.log\" \"$HOME/.claude/bin/fix-cs-indent\""
```
