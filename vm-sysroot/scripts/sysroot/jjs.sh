#!/usr/bin/env bash
cd ../pkg/ar_data/bin || exit 1
ldd ./* | grep ' => ' | sed 's/^.* => //g' | sed 's/ (0x[0-9a-f]*)$//g'
