#!/bin/sh
# Compile the extension; retain the last preview until a complete build is ready.
set -eu
INTEGRATION_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$INTEGRATION_ROOT"
"$INTEGRATION_ROOT/scripts/check-upstream.sh"
export CARGO_TARGET_DIR="$INTEGRATION_ROOT/../photocraft/target"
mkdir -p "$INTEGRATION_ROOT/.cache"
env -u NO_COLOR trunk build --release
STAGED=$(mktemp -d "$INTEGRATION_ROOT/.cache/ipad.XXXXXX")
cp -R "$INTEGRATION_ROOT/.cache/dist/." "$STAGED/"
cp "$INTEGRATION_ROOT/LICENSE-MIT" "$STAGED/"
cp "$INTEGRATION_ROOT/LICENSE-APACHE" "$STAGED/"
cp "$INTEGRATION_ROOT/NOTICE" "$STAGED/"
cp "$INTEGRATION_ROOT/../photocraft/ATTRIBUTION.md" "$STAGED/"
# Include the upstream notices and license texts without redistributing brand artwork.
for REL in assets/fonts assets/icons assets/dict assets/app-icon docs/brand; do
    mkdir -p "$STAGED/$REL"
    for LICENSE in "$INTEGRATION_ROOT/../photocraft/$REL/"*LICENSE*.txt "$INTEGRATION_ROOT/../photocraft/$REL/"OFL-*.txt; do
        [ -f "$LICENSE" ] || continue
        cp "$LICENSE" "$STAGED/$REL/"
    done
done
test -f "$STAGED/index.html"
if [ -e "$INTEGRATION_ROOT/public/ipad" ]; then
    mv "$INTEGRATION_ROOT/public/ipad" "$INTEGRATION_ROOT/.cache/previous-$(date +%s)"
fi
mv "$STAGED" "$INTEGRATION_ROOT/public/ipad"
printf '%s\n' 'Built locally. Run server.py and verify /ipad/?webgl before calling it ready.'
