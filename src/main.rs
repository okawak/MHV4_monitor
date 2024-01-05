mod mhv4;

use clap::Parser;
use futures::StreamExt;
use mhv4::MHV4Data;
use serialport::SerialPort;
use std::env;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::time::{self, Duration};
use tokio_stream::wrappers::IntervalStream;
use warp::Filter;
use warp::Reply;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct MyArguments {
    #[clap(short = 'p', long = "port_name", default_value = "/dev/ttyUSB0")]
    port_name: String,

    #[clap(short = 'r', long = "port_rate", default_value = "9600")]
    port_rate: u32,

    #[clap(short = 'i', long = "sse_interval_ms", default_value = "1000")]
    sse_interval: u64,

    #[clap(short = 's', long = "apply_hv_step", default_value = "5")] // 1 -> 0.1 V
    voltage_step: isize,

    #[clap(short = 'w', long = "waiting_time_ms", default_value = "500")]
    waiting_time: u64,

    #[clap(short = 't', long = "port_read_time_ms", default_value = "50")]
    read_time: u64,
}

lazy_static! {
    static ref ARGS: MyArguments = MyArguments::parse();
}

// MHV4 data array
struct SharedData {
    mhv4_data_array: Vec<MHV4Data>,
}

impl SharedData {
    fn new() -> SharedData {
        SharedData {
            mhv4_data_array: Vec::new(),
        }
    }
}

async fn initialize_status(
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    shared_data: Arc<Mutex<SharedData>>,
) -> Result<(), io::Error> {
    let mut shared_data = shared_data.lock().unwrap();

    for bus in 0..2 {
        let command = format!("sc {}\r", bus);
        println!("{}", command);
        {
            let mut port = port.lock().unwrap();
            port.write(command.as_bytes()).expect("Write failed!");
        }

        std::thread::sleep(Duration::from_millis(100));

        let mut buf: Vec<u8> = vec![0; 300];
        let size: usize;
        {
            let mut port = port.lock().unwrap();
            size = port.read(buf.as_mut_slice()).expect("Found no data!");
        }
        let bytes = &buf[..size];
        let string = String::from_utf8(bytes.to_vec()).expect("Failed to convert");
        let modules = string.split("\n\r").collect::<Vec<_>>();

        for dev in 0..16 {
            let module = modules[dev + 2].to_string();
            let datas = module.split_whitespace().collect::<Vec<_>>();
            if datas[1] == "-" {
                continue;
            }

            let mut idc_str = datas[1].to_string();
            idc_str.pop();
            let idc: usize = idc_str.parse().unwrap();
            if idc != 27 && idc != 17 {
                continue;
            }

            let power_status = datas[2].to_string();
            let is_rc: bool;
            if power_status == "ON" {
                is_rc = true;
            } else {
                is_rc = false;
            }

            for ch in 0..4 {
                let is_on: bool;
                let command = format!("re {} {} {}\r", bus, dev, ch + 36);
                println!("{}", command);
                let read_array = port_write_and_read(port.clone(), command)
                    .expect("Error in port communication");
                if read_array.len() != 3 {
                    is_on = false;
                } else {
                    let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                    let status: usize = datas.last().unwrap().to_string().parse().unwrap();
                    if status == 0 {
                        is_on = false;
                    } else {
                        is_on = true;
                    }
                }

                // read current HV
                let mut tmp: isize = 10_000;
                let current: isize;
                loop {
                    let command = format!("re {} {} {}\r", bus, dev, ch + 32);
                    println!("{}", command);
                    let read_array = port_write_and_read(port.clone(), command)
                        .expect("Error in port communication");
                    if read_array.len() != 3 {
                        continue;
                    } else {
                        let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                        let voltage: isize = datas.last().unwrap().to_string().parse().unwrap();
                        if voltage != tmp {
                            tmp = voltage;
                            continue;
                        } else {
                            current = voltage;
                            break;
                        }
                    }
                }

                shared_data
                    .mhv4_data_array
                    .push(MHV4Data::new(idc, bus, dev, ch, current, is_on, is_rc));
            }
        }
    }

    Ok(())
}

fn get_mhv4_data(
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    shared_data: Arc<Mutex<SharedData>>,
) -> impl warp::Reply {
    let mut shared_data = shared_data.lock().unwrap();

    if shared_data.mhv4_data_array[0].get_progress() {
        let mhv4_data_array = shared_data.mhv4_data_array.to_vec();
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            // read current HV
            let mut tmp: isize = 10_000;
            let current: isize;
            loop {
                let command = format!("re {} {} {}\r", bus, dev, ch + 32);
                println!("{}", command);
                let read_array = port_write_and_read(port.clone(), command)
                    .expect("Error in port communication");
                if read_array.len() != 3 {
                    continue;
                } else {
                    let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                    let voltage: isize = datas.last().unwrap().to_string().parse().unwrap();
                    if voltage != tmp {
                        tmp = voltage;
                        continue;
                    } else {
                        current = voltage;
                        break;
                    }
                }
            }
            shared_data.mhv4_data_array[i].set_current(current);
        }

        shared_data.mhv4_data_array[0].set_progress(false);
    }

    let mhv4_data_array = &shared_data.mhv4_data_array;
    let data_json = serde_json::to_string(mhv4_data_array).unwrap_or_else(|_| "[]".to_string());
    warp::reply::json(&data_json).into_response()
}

// SSE endpoint
fn sse_handler(
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    shared_data: Arc<Mutex<SharedData>>,
) -> impl warp::Reply {
    let interval = time::interval(Duration::from_millis(ARGS.sse_interval));
    let stream = IntervalStream::new(interval).map(move |_| {
        let mhv4_data_array: Vec<MHV4Data>;
        let is_progress: bool;
        {
            let shared_data = shared_data.lock().unwrap();
            mhv4_data_array = shared_data.mhv4_data_array.to_vec();
            is_progress = shared_data.mhv4_data_array[0].get_progress();
        }

        let mut v_array: Vec<isize> = Vec::new();
        let mut c_array: Vec<isize> = Vec::new();

        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();

            let command = format!("re {} {} {}\r", bus, dev, ch + 32);
            println!("{}", command);
            let read_array =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
            if read_array.len() > 1 {
                let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                let voltage = match datas.last() {
                    Some(str) => str.to_string(),
                    None => String::from("-100_000"),
                };
                match voltage.parse() {
                    Ok(num) => v_array.push(num),
                    Err(_) => {
                        println!("{:?}", read_array);
                        v_array.push(-100_000);
                    }
                }
            } else {
                println!("{:?}", read_array);
                v_array.push(-100_000);
            }

            let command = format!("re {} {} {}\r", bus, dev, ch + 50);
            println!("{}", command);
            let read_array =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
            if read_array.len() > 1 {
                let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                let current = match datas.last() {
                    Some(str) => str.to_string(),
                    None => String::from("-100_000"),
                };
                match current.parse() {
                    Ok(num) => c_array.push(num),
                    Err(_) => {
                        println!("{:?}", read_array);
                        c_array.push(-100_000);
                    }
                }
            } else {
                println!("{:?}", read_array);
                c_array.push(-100_000);
            }
        }

        let datas = (v_array, c_array, is_progress);
        let sse_json = serde_json::to_string(&datas).unwrap();

        Ok::<_, warp::Error>(warp::sse::Event::default().data(sse_json))
    });

    warp::sse::reply(stream)
}

// 0: RC on, 1: RC off, 2: Power on, 3: Power off
fn set_status(
    num: u32,
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    shared_data: Arc<Mutex<SharedData>>,
) -> bool {
    let mhv4_data_array: Vec<MHV4Data>;
    let current_rc: bool;
    let current_on: bool;
    {
        let shared_data = shared_data.lock().unwrap();
        mhv4_data_array = shared_data.mhv4_data_array.to_vec();
        (current_on, current_rc) = mhv4_data_array[0].get_status();
    }

    // remote ON
    if num == 0 && !current_rc {
        // if you use IDC=27 MHV4, please prepare polarity list
        // example) 0: negative, 1: positive
        // let mhv4_1 = [0, 0, 1, 1]; // 1ch: neg, 2ch: neg, 3ch: pos, 4ch: pos
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            let command = format!("on {} {}\r", bus, dev);
            println!("{}", command);
            let _ =
                port_write_and_read(port.clone(), command).expect("Error in port communication");

            // current limit
            let command = format!("se {} {} {} 10000\r", bus, dev, ch + 8);
            println!("{}", command);
            let _ =
                port_write_and_read(port.clone(), command).expect("Error in port communication");

            // if you use IDC=27 MHV4, you can set polarity in here
            //let idc = mhv4_data_array[i].get_idc();
            //if idc == 27 {
            //    let command = format!("se {} {} {} {}\r", bus, dev, ch + 14, mhv4_1[ch]);
            //    let _ = port_write_and_read(port.clone(), command).expect("Error in port communication");
            //}
        }

        {
            let mut shared_data = shared_data.lock().unwrap();
            shared_data.mhv4_data_array[0].set_status(current_on, true);
        }
    // remote OFF
    } else if num == 1 && current_rc {
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, _) = mhv4_data_array[i].get_module_id();
            let command = format!("off {} {}\r", bus, dev);
            println!("{}", command);
            let _ =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
        }

        {
            let mut shared_data = shared_data.lock().unwrap();
            shared_data.mhv4_data_array[0].set_status(current_on, false);
        }
    // power ON
    } else if num == 2 && !current_on {
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            let command = format!("se {} {} {} 1\r", bus, dev, ch + 4);
            println!("{}", command);
            let _ =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
        }

        {
            let mut shared_data = shared_data.lock().unwrap();
            shared_data.mhv4_data_array[0].set_status(true, current_rc);
        }
    // power OFF
    } else if num == 3 && current_on {
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            let command = format!("se {} {} {} 0\r", bus, dev, ch + 4);
            println!("{}", command);
            let _ =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
        }

        {
            let mut shared_data = shared_data.lock().unwrap();
            shared_data.mhv4_data_array[0].set_status(false, current_rc);
        }
    }
    true
}

fn set_voltage(
    nums: Vec<isize>,
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    shared_data: Arc<Mutex<SharedData>>,
) -> bool {
    let mhv4_data_array: Vec<MHV4Data>;
    {
        let mut shared_data = shared_data.lock().unwrap();
        shared_data.mhv4_data_array[0].set_progress(true);
        mhv4_data_array = shared_data.mhv4_data_array.to_vec();
    }

    {
        let shared_data = shared_data.lock().unwrap();
        println!("{}", shared_data.mhv4_data_array[0].get_progress());
    }

    let mut voltage_now_array: Vec<isize> =
        mhv4_data_array.iter().map(|x| x.get_current()).collect();
    let mut count: usize = 0;
    let mut is_finish: Vec<bool> = vec![false; mhv4_data_array.len()];

    thread::spawn(move || {
        loop {
            for i in 0..mhv4_data_array.len() {
                if voltage_now_array[i] == nums[i] {
                    if !is_finish[i] {
                        is_finish[i] = true;
                        count += 1;
                    }
                    continue;
                } else if (voltage_now_array[i] - nums[i]).abs() < ARGS.voltage_step {
                    voltage_now_array[i] = nums[i];
                } else if voltage_now_array[i] < nums[i] {
                    voltage_now_array[i] += ARGS.voltage_step;
                } else {
                    voltage_now_array[i] -= ARGS.voltage_step;
                }
                let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
                let command = format!("se {} {} {} {}\r", bus, dev, ch, voltage_now_array[i]);
                println!("{}", command);
                let _ = port_write_and_read(port.clone(), command)
                    .expect("Error in port communication");
            }
            if count == mhv4_data_array.len() {
                break;
            }
            std::thread::sleep(Duration::from_millis(ARGS.waiting_time));
        }

        {
            let mut shared_data = shared_data.lock().unwrap();
            shared_data.mhv4_data_array[0].set_progress(false);
            for i in 0..mhv4_data_array.len() {
                shared_data.mhv4_data_array[i].set_current(nums[i]);
            }
        }
    });
    true
}

fn port_write_and_read(
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    command: String,
) -> Result<Vec<String>, io::Error> {
    {
        let mut port = port.lock().unwrap();
        port.write(command.as_bytes()).expect("Write failed!");
    }
    std::thread::sleep(Duration::from_millis(ARGS.read_time));

    let mut v_buf: Vec<u8> = vec![0; 100];
    let size: usize;
    {
        let mut port = port.lock().unwrap();
        size = match port.read(v_buf.as_mut_slice()) {
            Ok(t) => t,
            Err(_) => {
                let dummy: Vec<String> = Vec::new();
                return Ok(dummy);
            }
        };
    }
    let bytes = &v_buf[..size];
    let string = String::from_utf8(bytes.to_vec()).expect("Failed to convert");
    let read_array = string.split("\n\r").collect::<Vec<_>>();
    let vec = read_array.iter().map(|&s| s.to_string()).collect();

    Ok(vec)
}

#[tokio::main]
async fn main() {
    // init the logger (not inpremented)
    pretty_env_logger::init();

    // port connection
    let port = serialport::new(ARGS.port_name.clone(), ARGS.port_rate)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Failed to open serial port");

    // get Mutex key
    let port = Arc::new(Mutex::new(port));
    let shared_data = Arc::new(Mutex::new(SharedData::new()));

    // making clone for each route (because of move? I'm not sure about ownership yet...)
    let port_for_sse = port.clone();
    let port_for_status = port.clone();
    let port_for_apply = port.clone();
    let shared_for_sse = shared_data.clone();
    let shared_for_status = shared_data.clone();
    let shared_for_apply = shared_data.clone();

    // main
    initialize_status(port.clone(), shared_data.clone())
        .await
        .expect("Failed to initialize serial port");

    let get_mhv4_data_route = warp::get()
        .and(warp::path("mhv4_data"))
        .map(move || get_mhv4_data(port.clone(), shared_data.clone()));

    let sse_route =
        warp::path("sse").map(move || sse_handler(port_for_sse.clone(), shared_for_sse.clone()));

    let status_route = warp::path("status")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |num: u32| {
            let result = set_status(num, port_for_status.clone(), shared_for_status.clone());
            warp::reply::json(&result)
        });

    let apply_route = warp::path("apply")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |nums: Vec<isize>| {
            let result = set_voltage(nums, port_for_apply.clone(), shared_for_apply.clone());
            warp::reply::json(&result)
        });

    let static_files = warp::fs::dir("www");

    let routes = static_files
        .or(get_mhv4_data_route)
        .or(sse_route)
        .or(status_route)
        .or(apply_route);

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
