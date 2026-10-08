#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
. "$ROOT/upstream.env"
ENGINE="$ROOT/../photocraft"
if [ "$(git -C "$ENGINE" rev-parse 'HEAD^{tree}' 2>/dev/null || true)" != "$PHOTOCRAFT_TREE" ] ||
   ! git -C "$ENGINE" diff --quiet || ! git -C "$ENGINE" diff --cached --quiet; then
    echo 'The sibling photocraft checkout differs from the tested source tree.' >&2
    echo 'Keep your work. Clone photocraft-ipad into a fresh parent and run ./setup.sh there.' >&2
    exit 1
fi
printf '%s\n' "PhotoCraft source tree verified: $PHOTOCRAFT_TREE"
