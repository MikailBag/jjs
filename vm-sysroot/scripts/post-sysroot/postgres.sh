#!/usr/bin/env bash

firstof ()
{
    echo -n "$1"
}

sudo mkdir -p "$SYSROOT/usr/bin"
sudo tee "$SYSROOT/usr/bin/postgres" >/dev/null << EOF
#!/bin/sh

exec /usr/lib/postgresql/*/bin/postgres "\$@"
EOF
sudo tee "$SYSROOT/usr/bin/psql" >/dev/null << EOF
#!/bin/sh

exec /usr/lib/postgresql/*/bin/psql "\$@"
EOF
sudo chmod +x "$SYSROOT"/usr/bin/{postgres,psql}

sudo rm -rf "$SYSROOT"/var/lib/postgresql/*/main/*
sudo chown "$(whoami):$(whoami)" "$SYSROOT"/var/lib/postgresql/*/main
"$(firstof /usr/lib/postgresql/*/bin/initdb)" -U postgres "$SYSROOT"/var/lib/postgresql/*/main
sudo sed -i 's/.*timezone.*//g' "$SYSROOT"/var/lib/postgresql/*/main/postgresql.conf

sudo rm -rf tmp
mkdir tmp
"$(firstof /usr/lib/postgresql/*/bin/postgres)" -D "$SYSROOT"/var/lib/postgresql/*/main -k "$(pwd)/tmp" &
sleep 15
psql -h "$(pwd)/tmp" -U postgres -c "create role jjs with password 'internal';"
psql -h "$(pwd)/tmp" -U postgres -c "alter role jjs with login;"
psql -h "$(pwd)/tmp" -U postgres -c "create database jjs;"
psql -h "$(pwd)/tmp" -U postgres -d jjs -c "grant all on all tables in schema public to jjs;"
psql -h "$(pwd)/tmp" -U postgres -d jjs -c "grant all on all sequences in schema public to jjs;"
echo "postgresql://jjs:internal@$(echo -n "$(pwd)" | xxd -p | sed 's/\(..\)/%\1/g')%2ftmp/jjs"
cargo run -p setup -- - upgrade << EOF
install-dir: ../pkg/ar_data
pg:
  db-name: jjs
  conn-string: "postgresql://jjs:internal@$(echo -n "$(pwd)" | xxd -p | tr '\n' ' ' | sed 's/ //g' | sed 's/\(..\)/%\1/g')%2ftmp/jjs"
EOF
kill %1
sleep 1
sudo rm -rf tmp

sudo chown -R 2:2 "$SYSROOT/var/lib/postgresql"
sudo chmod -R 0700 "$SYSROOT/var/lib/postgresql"
sudo rm -rf "$SYSROOT/var/run/postgresql"
sudo mkdir -p "$SYSROOT/var/run/postgresql"
sudo chown 2:2 "$SYSROOT/var/run/postgresql"
sudo chmod 0700 "$SYSROOT/var/run/postgresql"

sudo fuser -k "$SYSROOT"/usr/lib/postgresql/*/bin/postgres
