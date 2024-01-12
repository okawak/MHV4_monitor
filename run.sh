#!/bin/sh

port_name="/dev/ttyUSB0"
port_rate="9600"
sse_interval="1000" # ms
voltage_step="5"    # 5 -> 0.5 V
waiting_time="500"  # ms
read_time="45"      # ms

cargo run --release --bin mhv4_monitor --  -v -p $port_name -r $port_rate -i $sse_interval -s $voltage_step -w $waiting_time -t $read_time
