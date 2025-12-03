#!/bin/bash
set -e

# Move to the tex build directory
cd "$(dirname "$0")"

# Run the make command
make all
