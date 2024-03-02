#!/bin/bash
if [[ -f /etc/systemd/system/elktool.service ]]; then systemctl stop elktool; else echo "na"; fi;
if [[ -d /var/lib/elktool/ ]]; then echo '/var/lib/elktool/ exists'; else mkdir /var/lib/elktool/; fi;
cargo build --release
cp elktool.service /etc/systemd/system/elktool.service
systemctl daemon-reload
if [[ -f /etc/systemd/system/elktool.service ]]; then systemctl restart elktool; else echo "na"; fi;
