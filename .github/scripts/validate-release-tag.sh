#!/usr/bin/env bash
set -euo pipefail

tag=${1:?release tag is required}
if [[ $tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || [[ $tag =~ ^test-release-[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
  exit 0
fi
echo "Invalid release tag: $tag (expected vMAJOR.MINOR.PATCH or test-release-SUFFIX)" >&2
exit 1
