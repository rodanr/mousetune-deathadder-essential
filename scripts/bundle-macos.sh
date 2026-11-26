#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Builds a universal (arm64 + x86_64) MouseTune.app, ad-hoc signs it and
# packages a .zip and a drag-to-Applications .dmg into dist/.
#
# Needs: Rust with both macOS targets (rustup target add aarch64-apple-darwin
# x86_64-apple-darwin) and the Xcode command line tools. No paid Apple
# Developer account is involved: the signature is ad hoc.
set -euo pipefail

cd "$(dirname "$0")/.."

export MACOSX_DEPLOYMENT_TARGET=13.0

version="$(cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="mousetune"))')"
name="MouseTune-${version}-macos-universal"
dist="dist"
app="${dist}/MouseTune.app"

echo "==> Building MouseTune ${version} for both architectures"
for target in aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build --release --locked --package mousetune --target "${target}"
done

echo "==> Assembling ${app}"
rm -rf "${app}" "${dist}/${name}.zip" "${dist}/${name}.dmg"
mkdir -p "${app}/Contents/MacOS" "${app}/Contents/Resources"
lipo -create \
  target/aarch64-apple-darwin/release/mousetune \
  target/x86_64-apple-darwin/release/mousetune \
  -output "${app}/Contents/MacOS/MouseTune"
sed "s/@VERSION@/${version}/g" packaging/macos/Info.plist > "${app}/Contents/Info.plist"
cp assets/AppIcon.icns "${app}/Contents/Resources/AppIcon.icns"

echo "==> Ad-hoc signing"
codesign --force --deep --sign - "${app}"
codesign --verify --deep --strict "${app}"

echo "==> Creating ${name}.zip"
ditto -c -k --keepParent "${app}" "${dist}/${name}.zip"

echo "==> Creating ${name}.dmg"
staging="$(mktemp -d)"
trap 'rm -rf "${staging}"' EXIT
cp -R "${app}" "${staging}/"
ln -s /Applications "${staging}/Applications"
hdiutil create -volname "MouseTune" -srcfolder "${staging}" \
  -fs HFS+ -format UDZO -ov "${dist}/${name}.dmg" >/dev/null
codesign --force --sign - "${dist}/${name}.dmg"

echo "==> Done"
ls -lh "${dist}/${name}".{zip,dmg}
