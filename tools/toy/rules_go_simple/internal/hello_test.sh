#!/usr/bin/env bash

set -euo pipefail

program="$1"
got=$("$program")
want="Hello, World!"

if [ "$got" != "$want" ]; then
  cat >&2 <<EOF
got:
$got
want:
$want
EOF
  exit 1
fi
