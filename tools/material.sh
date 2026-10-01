#!/bin/zsh
# Copies Material Symbols Rounded (Apache 2.0), weight 400, unfilled, into assets/icons, from
# the npm package @material-symbols/svg-400 pinned below. The package ships one optical size, 48.
# Usage: tools/material.sh <name>...   e.g. tools/material.sh check content_copy stop-fill
# The package is fetched once into /tmp. `-fill` names the filled form, for the one icon that needs it.
set -eu
VERSION=0.47.5
DIR=/tmp/material-symbols-svg-400-$VERSION
if [[ ! -d $DIR/package ]]; then
  mkdir -p $DIR
  (cd $DIR && npm pack "@material-symbols/svg-400@$VERSION" --silent >/dev/null && tar xzf "material-symbols-svg-400-$VERSION.tgz")
fi
for name in "$@"; do
  cp "$DIR/package/rounded/$name.svg" "assets/icons/$name.svg"
done
