mod mhv4;

use clap::Parser;
use futures::StreamExt;
use mhv4::MHV4Data;
use serialport::SerialPort;
use std::env;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use tokio_stream::wrappers::IntervalStream;
use warp::Filter;
use warp::Reply;

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

fn get_mhv4_data(shared_data: Arc<Mutex<SharedData>>) -> impl warp::Reply {
    let shared_data = shared_data.lock().unwrap();
    let mhv4_data_array = &shared_data.mhv4_data_array;

    let data_json = serde_json::to_string(mhv4_data_array).unwrap_or_else(|_| "[]".to_string());
    warp::reply::json(&data_json).into_response()
}

// SSE endpoint
fn sse_handler(
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    shared_data: Arc<Mutex<SharedData>>,
    sse_interval: u64,
) -> impl warp::Reply {
    let mhv4_data_array: Vec<MHV4Data>;
    {
        let shared_data = shared_data.lock().unwrap();
        mhv4_data_array = shared_data.mhv4_data_array.to_vec();
    }

    let interval = time::interval(Duration::from_millis(sse_interval));
    let stream = IntervalStream::new(interval).map(move |_| {
        let mut v_array: Vec<isize> = Vec::new();
        let mut c_array: Vec<isize> = Vec::new();

        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();

            let command = format!("re {} {} {}\r", bus, dev, ch + 32);
            let read_array =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
            if read_array.len() != 3 {
                v_array.push(-100_000);
            } else {
                let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                let voltage = datas.last().unwrap().to_string();
                v_array.push(voltage.parse().unwrap());
            }

            let command = format!("re {} {} {}\r", bus, dev, ch + 50);
            let read_array =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
            if read_array.len() != 3 {
                c_array.push(-100_000);
            } else {
                let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                let current = datas.last().unwrap().to_string();
                c_array.push(current.parse().unwrap());
            }
        }

        let datas = (v_array, c_array);
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
            port_write(port.clone(), command).expect("Error in port communication");

            // current limit
            let command = format!("se {} {} {} 2000\r", bus, dev, ch + 8);
            port_write(port.clone(), command).expect("Error in port communication");

            // if you use IDC=27 MHV4, you can set polarity in here
            //let idc = mhv4_data_array[i].get_idc();
            //if idc == 27 {
            //    let command = format!("se {} {} {} {}\r", bus, dev, ch + 14, mhv4_1[ch]);
            //    port_write(port.clone(), command).expect("Error in port communication");
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
            port_write(port.clone(), command).expect("Error in port communication");
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
            port_write(port.clone(), command).expect("Error in port communication");
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
            port_write(port.clone(), command).expect("Error in port communication");
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
        let shared_data = shared_data.lock().unwrap();
        mhv4_data_array = shared_data.mhv4_data_array.to_vec();
    }

    let mut voltage_now_array: Vec<isize> =
        mhv4_data_array.iter().map(|x| x.get_current()).collect();
    let mut count: usize = 0;

    loop {
        for i in 0..mhv4_data_array.len() {
            if voltage_now_array[i] == nums[i] {
                continue;
            } else if voltage_now_array[i] < nums[i] {
                voltage_now_array[i] += 1;
                let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
                let command = format!("se {} {} {} {}\r", bus, dev, ch + 4, voltage_now_array[i]);
                port_write(port.clone(), command).expect("Error in port communication");

                if voltage_now_array[i] == nums[i] {
                    count += 1;
                }
            } else {
                voltage_now_array[i] -= 1;
                let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
                let command = format!("se {} {} {} {}\r", bus, dev, ch + 4, voltage_now_array[i]);
                port_write(port.clone(), command).expect("Error in port communication");

                if voltage_now_array[i] == nums[i] {
                    count += 1;
                }
            }
        }
        println!("{:?}", voltage_now_array);
        println!("{:?}", nums);
        println!("---");

        if count == mhv4_data_array.len() {
            break;
        }
    }
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
    std::thread::sleep(Duration::from_millis(50));

    let mut v_buf: Vec<u8> = vec![0; 100];
    let size: usize;
    {
        let mut port = port.lock().unwrap();
        size = port.read(v_buf.as_mut_slice()).expect("Found no data!");
    }
    let bytes = &v_buf[..size];
    let string = String::from_utf8(bytes.to_vec()).expect("Failed to convert");
    let read_array = string.split("\n\r").collect::<Vec<_>>();
    let vec = read_array.iter().map(|&s| s.to_string()).collect();

    Ok(vec)
}

fn port_write(port: Arc<Mutex<Box<dyn SerialPort>>>, command: String) -> Result<(), io::Error> {
    {
        let mut port = port.lock().unwrap();
        port.write(command.as_bytes()).expect("Write failed!");
    }
    std::thread::sleep(Duration::from_millis(50));
    Ok(())
}

#[tokio::main]
async fn main() {
    // init the logger (not inpremented)
    pretty_env_logger::init();

    // argument parser using clap
    let args: MyArguments = MyArguments::parse();

    // port connection
    let port = serialport::new(args.port_name, args.port_rate)
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
        .map(move || get_mhv4_data(shared_data.clone()));

    let sse_route = warp::path("sse").map(move || {
        sse_handler(
            port_for_sse.clone(),
            shared_for_sse.clone(),
            args.sse_interval,
        )
    });

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
