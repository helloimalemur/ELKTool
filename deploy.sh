#!/bin/bash
if [[ -f /etc/systemd/system/elktool-core.service ]]; then systemctl stop elktool-core; else echo "na"; fi;
if [[ -f /etc/systemd/system/elktool-lifetimes.service ]]; then systemctl stop elktool-lifetimes; else echo "na"; fi;
if [[ -f /etc/systemd/system/elktool-sanitize.service ]]; then systemctl stop elktool-sanitize; else echo "na"; fi;
if [[ -f /etc/systemd/system/elktool-replicate.service ]]; then systemctl stop elktool-replicate; else echo "na"; fi;

if [[ -d /var/lib/elktool/ ]]; then echo '/var/lib/elktool/ exists'; else mkdir /var/lib/elktool/; fi;

cargo build --release

cp elktool-core.service /etc/systemd/system/elktool-core.service
cp elktool-lifetimes.service /etc/systemd/system/elktool-lifetimes.service
cp elktool-sanitize.service /etc/systemd/system/elktool-sanitize.service
cp elktool-replicate.service /etc/systemd/system/elktool-replicate.service

systemctl daemon-reload
if [[ -f /etc/systemd/system/elktool-core.service ]]; then systemctl enable elktool-core; else echo "na"; fi;
if [[ -f /etc/systemd/system/elktool-lifetimes.service ]]; then systemctl enable elktool-lifetimes; else echo "na"; fi;
if [[ -f /etc/systemd/system/elktool-sanitize.service ]]; then systemctl enable elktool-sanitize; else echo "na"; fi;
if [[ -f /etc/systemd/system/elktool-replicate.service ]]; then systemctl enable elktool-replicate; else echo "na"; fi;

if [[ -f /etc/systemd/system/elktool-core.service ]]; then systemctl restart elktool-core; else echo "na"; fi;
sleep 2s
if [[ -f /etc/systemd/system/elktool-lifetimes.service ]]; then systemctl restart elktool-lifetimes; else echo "na"; fi;
sleep 2s
if [[ -f /etc/systemd/system/elktool-sanitize.service ]]; then systemctl restart elktool-sanitize; else echo "na"; fi;
sleep 2s
if [[ -f /etc/systemd/system/elktool-replicate.service ]]; then systemctl restart elktool-replicate; else echo "na"; fi;
