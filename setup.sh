#!/usr/bin/env bash
#
# Setup git hooks for this repository.
# Run once after cloning: ./setup.sh
#

set -euo pipefail

echo "Configuring git hooks path → .githooks/"
git config core.hooksPath .githooks
echo "Done. Pre-commit hook is now active."
