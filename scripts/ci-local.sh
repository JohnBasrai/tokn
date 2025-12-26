#!/bin/bash
set -euo pipefail

act --rm "$@" | cat
