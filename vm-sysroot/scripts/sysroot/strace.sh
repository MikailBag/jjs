#!/usr/bin/env bash

{ strace -f -o /dev/fd/3 strace >&2; } 3>&1 | python3 ../scripts/strace-parser.py | python3 strace/soft.py
