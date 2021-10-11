#!/usr/bin/env bash

set -euo pipefail

program="$1"
got=$("$program")
want="tools/toy/rules_go_simple/internal/bar.txt
tools/toy/rules_go_simple/internal/foo.txt"

if [ "$got" != "$want" ]; then
  cat >&2 <<EOF
got:
$got
want:
$want
EOF
  exit 1
fi
