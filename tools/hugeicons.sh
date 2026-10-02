#!/usr/bin/env bash
# Copies Hugeicons (MIT), the free stroke-rounded set on a 24 grid, into assets/icons, from the npm package
# @iconify-json/hugeicons pinned below. Every UI icon comes from it; never draw one by hand.
# Usage: tools/hugeicons.sh <name>...   e.g. tools/hugeicons.sh mic-01 tick-02 stop:fill
# The package is fetched once into /tmp. `:fill` fills the outline, for the one icon that needs a solid shape.
set -euo pipefail
VERSION=1.2.35
DIR=/tmp/iconify-json-hugeicons-$VERSION
if [[ ! -d $DIR/package ]]; then
  mkdir -p $DIR
  (cd $DIR && npm pack "@iconify-json/hugeicons@$VERSION" --silent >/dev/null && tar xzf "iconify-json-hugeicons-$VERSION.tgz")
fi
node - "$DIR/package/icons.json" "$@" <<'JS'
const fs = require("fs");
const [file, ...names] = process.argv.slice(2);
const set = JSON.parse(fs.readFileSync(file, "utf8"));
for (const arg of names) {
  const [name, form] = arg.split(":");
  const icon = set.icons[name];
  if (!icon) throw new Error(`no hugeicon named ${name}`);
  let body = icon.body;
  if (form === "fill") body = body.replaceAll('fill="none"', 'fill="currentColor"');
  const size = set.width ?? 24;
  const out = form ? `${name}-${form}` : name;
  fs.writeFileSync(`assets/icons/${out}.svg`, `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">${body}</svg>\n`);
}
JS
