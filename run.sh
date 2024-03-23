#!/bin/sh

# Log type
log_type="debug" # info or debug

# for read error
max_voltage="3000"
port_name="/dev/ttyUSB0"
port_rate="9600"
voltage_step="5"   # 5 -> 0.5 V
waiting_time="500" # ms

# kill the existing serial port process
#pkill ${port_name}

# start the server
RUST_LOG="${log_type}" cargo run --release --bin mhv4_monitor -- -p $port_name -r $port_rate -s $voltage_step -w $waiting_time -m $max_voltage
