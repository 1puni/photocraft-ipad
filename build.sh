#!/bin/sh
# Compile the extension; retain the last preview until a complete build is ready.
set -eu
INTEGRATION_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$INTEGRATION_ROOT"
"$INTEGRATION_ROOT/scripts/check-upstream.sh"
# A separate target per checkout prevents equal crate names linking stale UI.
export CARGO_TARGET_DIR="$INTEGRATION_ROOT/target"
mkdir -p "$INTEGRATION_ROOT/.cache"
env -u NO_COLOR trunk build --release --locked
STAGED=$(mktemp -d "$INTEGRATION_ROOT/.cache/ipad.XXXXXX")
cp -R "$INTEGRATION_ROOT/.cache/dist/." "$STAGED/"
"$INTEGRATION_ROOT/scripts/package-notices.sh" "$STAGED"
test -f "$STAGED/index.html"
if [ -e "$INTEGRATION_ROOT/public/ipad" ]; then
    mv "$INTEGRATION_ROOT/public/ipad" "$INTEGRATION_ROOT/.cache/previous-$(date +%s)"
fi
mv "$STAGED" "$INTEGRATION_ROOT/public/ipad"
printf '%s\n' 'Built locally. Run server.py and verify /ipad/?webgl before calling it ready.'
