#!/usr/bin/env bash

CPATH=$(pwd)
sudo mkdir -p /var/lib/byeusb
sudo mkdir -p /var/lib/byeusb/bin
cd ByeUSB
cargo build --release
sudo mv ./target/release/ByeUSB /var/lib/byeusb/bin
cd $CPATH
cd ByeUSBMsg
cargo build --release
sudo mv ./target/release/ByeUSBMsg /var/lib/byeusb/bin
