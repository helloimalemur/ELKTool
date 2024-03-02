#!/bin/bash
if [[ -f /etc/systemd/system/elktool.service ]]; then systemctl stop elktool; else echo "na"; fi;
if [[ -d /var/lib/elktool/ ]]; then echo '/var/lib/elktool/ exists'; else mkdir /var/lib/elktool/; fi;
if [[ -f Cargo.toml ]]; then cargo build --release; else cd /var/lib/elktool/ && cargo build --release; fi
CURDIR=$(pwd)
if [ "$CURDIR" != "/var/lib/elktool/" ]; then
  cp -r ./config/ /var/lib/elktool/
  cp -r ./target/ /var/lib/elktool/
fi
cp ./run.sh /var/lib/elktool/
cp elktool.service /etc/systemd/system/elktool.service
systemctl daemon-reload
if [[ -f /etc/systemd/system/elktool.service ]]; then systemctl restart elktool; else echo "na"; fi;
