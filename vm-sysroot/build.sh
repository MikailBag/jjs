#!/usr/bin/env bash

if [ "x$2" != x ] || [ "x$1" == "x--help" ]
then cat >&2 << EOF
usage: $0 [sysroot_path]

Build a jjs sysroot at \$sysroot_path (default = ./sysroot)
This script assumes that you have working sudo command. Don't run directly as root!
EOF
exit 1
fi

if [ ! -f ../target/jjs.tgz ]
then cat >&2 <<EOF
ERROR: JJS package (../target/jjs.tgz) has not been built.
To build it, do the following:

    cd ../target
    ../configure --enable-archive
    make
EOF
exit 1
fi

bash -c 'cd ..; rm -rf pkg; mkdir -p pkg; cd pkg; tar -xvf ../target/jjs.tgz; mv jjs ar_data'

SYSROOT="$1"

if [ "x$SYSROOT" == x ]
then SYSROOT=sysroot
fi

if [ "${SYSROOT:0:1}" != "/" ]
then SYSROOT="$(pwd)/$SYSROOT"
fi

export SYSROOT

export LANG=POSIX # prevent packaging a non-English locale

sudo rm -rf "$SYSROOT" 2>&1
sudo mkdir "$SYSROOT" || exit 1

for i in scripts/sysroot/*
do bash -x "$i"
done | sort | uniq | tee /dev/stderr | while read -r path
do
    sudo mkdir -p "$SYSROOT/$path"
    if ! sudo test -d "$path"
    then
        sudo rmdir "$SYSROOT/$path"
        sudo cp "$path" "$SYSROOT/$path"
        sudo chown "$(whoami):$(whoami)" "$SYSROOT/$path"
    fi
done

for i in scripts/post-sysroot/*
do bash -x "$i"
done
