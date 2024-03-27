#!/bin/sh

# Log type
log_type="debug" # info, debug, trace

# localhost server
localhost="" # -l or blank

# for read error
max_voltage="3000"
port_name="/dev/ttyUSB0"
port_rate="9600"
voltage_step="5"   # 5 -> 0.5 V
waiting_time="500" # ms

# kill the existing serial port process
pkill mhv4_monitor

# start the server
RUST_LOG="${log_type}" cargo run --release --bin mhv4_monitor -- "$localhost" -p "$port_name" -r "$port_rate" -s "$voltage_step" -w "$waiting_time" -m "$max_voltage"
