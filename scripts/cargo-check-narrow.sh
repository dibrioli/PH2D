#!/usr/bin/env bash
# cargo-check-narrow.sh <crate> [extra cargo args] — narrowed inner-loop check.
# Prints ONLY the first 3 error-level diagnostics (full text each), dropping
# warnings + artifact/build-script noise, to cut the LLM token payload ~90% on
# a failing check. On a clean pass it prints nothing.
#
# CRITICAL: preserves cargo's REAL exit code. A naive `cargo | jq | head` would
# report head's exit 0 even when the compile FAILED, silently breaking the
# autonomous agent's inner-loop signal. This captures to a temp file and exits
# with cargo's code.
#
# This is a DEV convenience only — never used by ship.sh/CI (warnings are caught
# at the batched gate: clippy --all-targets + deny).
set -uo pipefail

crate="${1:?usage: cargo-check-narrow.sh <crate> [extra cargo args]}"
shift || true

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

cargo check -p "$crate" --message-format=json "$@" >"$tmp"
rc=$?

# First 3 error-level rendered diagnostics, full text each (never line-truncate
# a multi-line diagnostic — bound by error COUNT, not `head -n`).
jq -rs '
  [ .[] | select(.reason=="compiler-message") | .message
        | select(.level=="error") | .rendered ]
  | .[0:3] | .[]' "$tmp"

nerr=$(jq -rs '[ .[] | select(.reason=="compiler-message") | .message
                     | select(.level=="error") ] | length' "$tmp")
if [ "${nerr:-0}" -gt 3 ]; then
    echo "... (${nerr} total errors; showing first 3 — rerun \`cargo check -p ${crate}\` for full)"
fi

exit $rc
