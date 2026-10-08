#!/bin/sh
# Ship the exact upstream notices plus dependency license texts with each build.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
DEST=${1:?usage: package-notices.sh output-directory}
ABOUT=${CARGO_ABOUT:-cargo-about}
ENGINE="$ROOT/../photocraft"
mkdir -p "$DEST"
cp "$ROOT/LICENSE-MIT" "$ROOT/LICENSE-APACHE" "$ROOT/NOTICE" "$DEST/"
cp "$ENGINE/ATTRIBUTION.md" "$DEST/"
for REL in assets/fonts assets/icons assets/dict assets/app-icon docs/brand crates/ui-egui/src/i18n; do
    mkdir -p "$DEST/$REL"
    for LICENSE in "$ENGINE/$REL/"*LICENSE*.txt "$ENGINE/$REL/"OFL-*.txt; do
        [ -f "$LICENSE" ] || continue
        cp "$LICENSE" "$DEST/$REL/"
    done
done
"$ABOUT" generate --locked --fail --manifest-path "$ROOT/Cargo.toml" \
    --config "$ROOT/about.toml" "$ROOT/scripts/licenses.hbs" \
    --output-file "$DEST/dependency-licenses.html"
