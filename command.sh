#!/bin/sh

port_name="/dev/ttyUSB0"

cargo run --release --bin command -- "$@" -p $port_name
