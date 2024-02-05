mod mhv4;
mod shared;

use clap::Parser;
use futures::{Stream, StreamExt};
use mhv4::MHV4Data;
use serialport::SerialPort;
use shared::SharedData;
use std::env;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::result::Result;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use tokio::time::{sleep, Duration};
use warp::sse::Event;
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

    #[clap(short = 's', long = "apply_hv_step", default_value = "5")] // 1 -> 0.1 V
    voltage_step: isize,

    #[clap(short = 'w', long = "waiting_time_ms", default_value = "500")]
    waiting_time: u64,

    #[clap(short = 'm', long = "max_voltage", default_value = "3000")] // 1 -> 0.1 V
    max_voltage: isize,

    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
}

#[derive(Debug)]
enum MyError {
    DataGetError,
    DataLockError,
    PortGetError,
    PortLockError,
    PortWriteError,
    PortReadError,
    JSONSerializeError,
    ReadingError,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MyError::DataGetError => write!(f, "Data Get Error"),
            MyError::DataLockError => write!(f, "Data Lock Error"),
            MyError::PortGetError => write!(f, "Port Get Error"),
            MyError::PortLockError => write!(f, "Port Lock Error"),
            MyError::PortWriteError => write!(f, "Port Write Error"),
            MyError::PortReadError => write!(f, "Port Read Error"),
            MyError::JSONSerializeError => write!(f, "JSON Serialize Error"),
            MyError::ReadingError => write!(f, "Reading Error"),
        }
    }
}

impl Error for MyError {}

static ARGS: OnceLock<MyArguments> = OnceLock::new();
static PORT: OnceLock<Arc<Mutex<Box<dyn SerialPort>>>> = OnceLock::new();
static DATA: OnceLock<Arc<Mutex<SharedData>>> = OnceLock::new();

async fn initialize_status() -> Result<(), io::Error> {
    let mut mhv4_array: Vec<MHV4Data> = Vec::new();

    let mut is_rc = false;
    let mut is_rc_first = true;
    let mut is_on = false;
    let mut is_on_first = true;

    for bus in 0..2 {
        let command = format!("sc {}\r", bus);
        println!("Init: {}", command);
        let mut buf: Vec<u8> = vec![0; 300];
        let size: usize;
        {
            let mut port = PORT.get().unwrap().lock().unwrap();
            port.write(command.as_bytes()).expect("Write failed!");

            std::thread::sleep(Duration::from_millis(100));

            size = port.read(buf.as_mut_slice()).expect("Found no data!");
        }
        let bytes = &buf[..size];
        let string = String::from_utf8(bytes.to_vec()).expect("Failed to convert");
        let modules = string.split("\n\r").collect::<Vec<_>>();
        println!("{:?}", modules);

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

            if is_rc_first {
                let power_status = datas[2].to_string();
                if power_status == "ON" {
                    is_rc = true;
                    is_rc_first = false;
                }
            }

            for ch in 0..4 {
                if is_on_first {
                    let command = format!("re {} {} {}\r", bus, dev, ch + 36);
                    let read_array =
                        port_write_and_read(command).expect("Error in port communication");
                    if read_array.len() == 3 {
                        let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                        let status: usize = datas.last().unwrap().to_string().parse().unwrap();
                        if status == 1 {
                            is_on = true;
                        }
                        is_on_first = false
                    }
                }

                // read current HV
                let mut tmp: isize = 10_000;
                let current: isize;
                loop {
                    let command = format!("re {} {} {}\r", bus, dev, ch + 32);
                    let read_array =
                        port_write_and_read(command).expect("Error in port communication");
                    if read_array.len() != 3 {
                        continue;
                    } else {
                        let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                        let voltage: isize = datas.last().unwrap().to_string().parse().unwrap();
                        if voltage != tmp {
                            tmp = voltage;
                            continue;
                        } else if voltage.abs() > ARGS.get().unwrap().max_voltage {
                            // maximum voltage (for reading error)
                            continue;
                        } else {
                            current = voltage.abs();
                            break;
                        }
                    }
                }

                mhv4_array.push(MHV4Data::new(idc, bus, dev, ch, current));
            }
        }
    }
    let shared_data = Arc::new(Mutex::new(SharedData::new(
        mhv4_array.clone(),
        is_on,
        is_rc,
    )));
    DATA.set(shared_data).expect("Failed to set DATA OnceLock");

    Ok(())
}

fn get_mhv4_data() -> impl warp::Reply {
    let mut shared_data = DATA.get().unwrap().lock().unwrap();

    if shared_data.get_progress() {
        let mhv4_data_array = shared_data.get_data();
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            // read current HV
            let mut tmp: isize = 10_000;
            let current: isize;
            loop {
                let command = format!("re {} {} {}\r", bus, dev, ch + 32);
                let read_array = port_write_and_read(command).expect("Error in port communication");
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
            shared_data.set_current(i, current);
        }
        shared_data.set_progress(false);
    }

    let obj = &shared_data.clone();
    let data_json = serde_json::to_string(obj).unwrap_or_else(|_| "[]".to_string());
    warp::reply::json(&data_json).into_response()
}

// SSE endpoint
fn get_sse_stream() -> impl Stream<Item = Result<Event, MyError>> {
    futures::stream::unfold((), |()| async move {
        match read_monitor_value().await {
            Ok(result) => match serde_json::to_string(&result) {
                Ok(sse_json) => {
                    // normal process
                    let sse_data = warp::sse::Event::default().data(sse_json);
                    sleep(Duration::from_millis(100)).await;
                    Some((Ok::<_, MyError>(sse_data), ()))
                }
                Err(e) => {
                    eprintln!("JSON serialization error: {:?}", e);
                    Some((Err(MyError::JSONSerializeError), ()))
                }
            },
            Err(e) => {
                eprintln!("Error reading monitor value: {:?}", e);
                Some((Err(MyError::ReadingError), ()))
            }
        }
    })
    .filter_map(|result| async move {
        match result {
            Ok(event) => Some(Ok(event)),
            Err(_) => None,
        }
    })
}

async fn read_monitor_value() -> Result<(Vec<isize>, Vec<isize>, bool), MyError> {
    let mhv4_data_array: Vec<MHV4Data>;
    let is_progress: bool;
    {
        match DATA.get() {
            Some(data) => match data.lock() {
                Ok(data) => {
                    mhv4_data_array = data.get_data();
                    is_progress = data.get_progress();
                }
                Err(_) => return Err(MyError::DataLockError),
            },
            None => return Err(MyError::DataGetError),
        }
    }

    let mut v_array: Vec<isize> = Vec::new();
    let mut c_array: Vec<isize> = Vec::new();

    for i in 0..mhv4_data_array.len() {
        let (bus, dev, ch) = mhv4_data_array[i].get_module_id();

        let command = format!("re {} {} {}\r", bus, dev, ch + 32);
        let read_array = port_write_and_read(command)?;
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
        let read_array = port_write_and_read(command)?;
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
    Ok((v_array, c_array, is_progress))
}

// 0: RC on, 1: RC off, 2: Power on, 3: Power off
fn set_status(num: u32) -> bool {
    let mhv4_data_array: Vec<MHV4Data>;
    let current_rc: bool;
    let current_on: bool;
    {
        let shared_data = DATA.get().unwrap().lock().unwrap();
        mhv4_data_array = shared_data.get_data();
        (current_on, current_rc) = shared_data.get_status();
    }

    // remote ON
    if num == 0 && !current_rc {
        // if you use IDC=27 MHV4, please prepare polarity list
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            let command = format!("on {} {}\r", bus, dev);
            let _ = port_write_and_read(command).expect("Error in port communication");

            // current limit
            let command = format!("se {} {} {} 20000\r", bus, dev, ch + 8);
            let _ = port_write_and_read(command).expect("Error in port communication");

            // if you use IDC=27 MHV4, you can set polarity or something in here
            let idc = mhv4_data_array[i].get_idc();
            if idc == 27 {
                // ramp speed setting
                let command = format!("se {} {} 80 0\r", bus, dev);
                let _ = port_write_and_read(command).expect("Error in port communication");
            } else {
                // HV range setting
                let command = format!("se {} {} 13 1\r", bus, dev);
                let _ = port_write_and_read(command).expect("Error in port communication");
            }
        }

        {
            let mut shared_data = DATA.get().unwrap().lock().unwrap();
            shared_data.set_status(current_on, true);
        }
    // remote OFF
    } else if num == 1 && current_rc {
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, _) = mhv4_data_array[i].get_module_id();
            let command = format!("off {} {}\r", bus, dev);
            let _ = port_write_and_read(command).expect("Error in port communication");
        }

        {
            let mut shared_data = DATA.get().unwrap().lock().unwrap();
            shared_data.set_status(current_on, false);
        }
    // power ON
    } else if num == 2 && !current_on {
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            let command = format!("se {} {} {} 1\r", bus, dev, ch + 4);
            let _ = port_write_and_read(command).expect("Error in port communication");
        }

        {
            let mut shared_data = DATA.get().unwrap().lock().unwrap();
            shared_data.set_status(true, current_rc);
        }
    // power OFF
    } else if num == 3 && current_on {
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            let command = format!("se {} {} {} 0\r", bus, dev, ch + 4);
            let _ = port_write_and_read(command).expect("Error in port communication");
        }

        {
            let mut shared_data = DATA.get().unwrap().lock().unwrap();
            shared_data.set_status(false, current_rc);
        }
    }
    true
}

fn set_voltage(nums: Vec<isize>) -> bool {
    let mhv4_data_array: Vec<MHV4Data>;
    {
        let mut shared_data = DATA.get().unwrap().lock().unwrap();
        shared_data.set_progress(true);
        mhv4_data_array = shared_data.get_data();
    }

    let mut voltage_now_array: Vec<isize> =
        mhv4_data_array.iter().map(|x| x.get_current()).collect();
    let mut count: usize = 0;
    let mut is_finish: Vec<bool> = vec![false; mhv4_data_array.len()];
    let step = ARGS.get().unwrap().voltage_step;
    let waiting_time = ARGS.get().unwrap().waiting_time;

    thread::spawn(move || {
        loop {
            for i in 0..mhv4_data_array.len() {
                if voltage_now_array[i] == nums[i] {
                    if !is_finish[i] {
                        is_finish[i] = true;
                        count += 1;
                    }
                    continue;
                } else if (voltage_now_array[i] - nums[i]).abs() < step {
                    voltage_now_array[i] = nums[i];
                } else if voltage_now_array[i] < nums[i] {
                    voltage_now_array[i] += step;
                } else {
                    voltage_now_array[i] -= step;
                }
                let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
                let command = format!("se {} {} {} {}\r", bus, dev, ch, voltage_now_array[i]);
                let _ = port_write_and_read(command).expect("Error in port communication");
            }
            if count == mhv4_data_array.len() {
                break;
            }

            if waiting_time > (60 * (mhv4_data_array.len() - count)) as u64 {
                std::thread::sleep(Duration::from_millis(
                    waiting_time - 60 * ((mhv4_data_array.len() - count) as u64),
                ));
            }
        }

        {
            let mut shared_data = DATA.get().unwrap().lock().unwrap();
            shared_data.set_progress(false);
            for i in 0..mhv4_data_array.len() {
                shared_data.set_current(i, nums[i]);
            }
        }
    });
    true
}

fn port_write_and_read(command: String) -> Result<Vec<String>, MyError> {
    if ARGS.get().unwrap().verbose {
        println!("{}", command);
    }

    let mut buf: Vec<u8> = vec![0; 100];
    let size: usize;
    {
        let mut port = PORT
            .get()
            .ok_or(MyError::PortGetError)?
            .lock()
            .map_err(|_| MyError::PortLockError)?;
        port.write(command.as_bytes())
            .map_err(|_| MyError::PortWriteError)?;
        std::thread::sleep(Duration::from_millis(50));

        size = port
            .read(buf.as_mut_slice())
            .map_err(|_| MyError::PortReadError)?;
        std::thread::sleep(Duration::from_millis(10));
    }
    let bytes = &buf[..size];
    let string = String::from_utf8(bytes.to_vec()).map_err(|_| MyError::PortReadError)?;
    let read_array = string.split("\n\r").collect::<Vec<_>>();
    let vec = read_array.iter().map(|&s| s.to_string()).collect();

    Ok(vec)
}

#[tokio::main]
async fn main() {
    // argument parser
    let args = MyArguments::parse();
    ARGS.set(args).expect("Failed to set ARGS OnceLock");

    // init the logger (not inpremented)
    pretty_env_logger::init();

    // port connection
    let port = serialport::new(
        &ARGS.get().expect("arguments not set").port_name,
        ARGS.get().expect("arguments not set").port_rate,
    )
    .stop_bits(serialport::StopBits::One)
    .data_bits(serialport::DataBits::Eight)
    .parity(serialport::Parity::None)
    .timeout(Duration::from_millis(100))
    .open()
    .expect("Failed to open serial port");

    // get Mutex key
    let port = Arc::new(Mutex::new(port));
    PORT.set(port).expect("Failed to set PORT OnceLock");

    // main
    initialize_status()
        .await
        .expect("Failed to initialize serial port");

    let get_mhv4_data_route = warp::path("mhv4_data")
        .and(warp::get())
        .map(|| get_mhv4_data());

    let sse_route = warp::path("sse").and(warp::get()).map(|| {
        let stream = get_sse_stream();
        warp::sse::reply(warp::sse::keep_alive().stream(stream))
    });

    let status_route = warp::path("status")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |num: u32| {
            let result = set_status(num);
            warp::reply::json(&result)
        });

    let apply_route = warp::path("apply")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |nums: Vec<isize>| {
            let result = set_voltage(nums);
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
