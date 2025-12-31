#!/usr/bin/env bash
set -euo pipefail

# Runs the Milestone 1 TeX build from repo root for a consistent working directory.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

cd "$REPO_ROOT/docs/tex"
make milestone-1

