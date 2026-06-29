#!/bin/bash
set -euo pipefail

version="$1"

# Assemble the landing page and the generated documentation under target/page.
# The root holds the hand-written landing page assets and the Sphinx
# documentation is built into the docs/ subdirectory.

# A missing glob match must not abort the build, so unmatched patterns expand
# to nothing instead of staying literal and failing cp under set -e.
shopt -s nullglob

mkdir -p target/page
assets=(docs/*.html docs/*.svg docs/*.gif)
if [ ${#assets[@]} -gt 0 ]; then
    cp -rf "${assets[@]}" target/page/
fi

./scripts/generate-docs.sh "$version"
sphinx-build docs target/page/docs
