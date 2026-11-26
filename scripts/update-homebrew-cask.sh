#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Points the MouseTune Homebrew cask at a new release.
#
# Usage: scripts/update-homebrew-cask.sh <version> <dmg-sha256> <path/to/mousetune.rb>
set -euo pipefail

version="${1:?version, e.g. 0.2.0}"
sha256="${2:?SHA-256 of the macOS DMG}"
cask="${3:?path to the cask file}"

[[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "bad version: ${version}" >&2; exit 1; }
[[ "${sha256}" =~ ^[0-9a-f]{64}$ ]] || { echo "bad sha256: ${sha256}" >&2; exit 1; }
[[ -f "${cask}" ]] || { echo "no such cask: ${cask}" >&2; exit 1; }

sed -i.bak -E \
  -e "s/^  version \"[^\"]*\"/  version \"${version}\"/" \
  -e "s/^  sha256 .*/  sha256 \"${sha256}\"/" \
  "${cask}"
rm -f "${cask}.bak"

grep -q "^  version \"${version}\"$" "${cask}" || { echo "version not updated" >&2; exit 1; }
grep -q "^  sha256 \"${sha256}\"$" "${cask}" || { echo "sha256 not updated" >&2; exit 1; }
echo "updated ${cask} to ${version}"
