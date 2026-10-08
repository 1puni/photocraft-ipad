#!/bin/sh
# Never reset or overwrite an existing sibling checkout.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$ROOT/upstream.env"
ENGINE="$ROOT/../photocraft"
if [ -e "$ENGINE" ]; then
    exec "$ROOT/scripts/check-upstream.sh"
fi
git clone --no-checkout https://github.com/storytold/photocraft.git "$ENGINE"
git -C "$ENGINE" checkout --detach "$PHOTOCRAFT_BASE"
git -C "$ENGINE" -c user.name='PhotoCraft iPad setup' \
    -c user.email='setup@localhost' am "$ROOT"/patches/photocraft/*.patch
"$ROOT/scripts/check-upstream.sh"
