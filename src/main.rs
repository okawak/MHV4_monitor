mod mhv4;
mod shared;

use clap::Parser;
use futures::{Stream, StreamExt};
use mhv4::MHV4Data;
use serialport::SerialPort;
use shared::{CLArguments, OperationError, SharedData};
use std::io::{Read, Write};
use std::result::Result;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use tokio::time::{sleep, Duration};
use warp::{sse::Event, Filter, Reply};

static ARGS: OnceLock<CLArguments> = OnceLock::new();
static PORT: OnceLock<Arc<Mutex<Box<dyn SerialPort>>>> = OnceLock::new();
static DATA: OnceLock<Arc<Mutex<SharedData>>> = OnceLock::new();

// when the server started, this function will be read
async fn initialize_status() -> Result<(), OperationError> {
    log::info!("Initializing...");
    let mut mhv4_array: Vec<MHV4Data> = Vec::new();

    // Check RC mode or not
    let mut is_rc = false;
    let mut is_first = true; // flag of first process or not

    for bus in 0..2 {
        // scan command
        let command = format!("sc {}\r", bus);
        log::info!("command: {}", command);

        let mut buf: Vec<u8> = vec![0; 300];
        let size: usize;
        {
            let mut port = PORT.get().ok_or(OperationError::PortGetError)?.lock()?;
            port.write(command.as_bytes())?;

            std::thread::sleep(Duration::from_millis(100));
            size = port.read(buf.as_mut_slice())?;
        }
        let bytes = &buf[..size];
        let string = String::from_utf8(bytes.to_vec())?;
        let modules = string.split("\n\r").collect::<Vec<_>>();
        log::info!("result: {:?}", modules);

        for dev in 0..16 {
            let module = modules[dev + 2].to_string();
            let datas = module.split_whitespace().collect::<Vec<_>>();
            if datas[1] == "-" {
                continue;
            }

            let mut idc_str = datas[1].to_string();
            idc_str.pop();
            let idc: usize = idc_str.parse()?;
            if idc != 27 && idc != 17 {
                log::debug!("find not MHV4 module, idc = {}", idc);
                continue;
            }

            if is_first {
                let power_status = datas[2].to_string();
                if power_status == "ON" {
                    is_rc = true;
                    is_first = false;
                }
            }

            for ch in 0..4 {
                // read channel status ON/OFF
                let mut is_on = false;
                let command = format!("re {} {} {}\r", bus, dev, ch + 36);
                let read_array = port_write_and_read(command)?;

                if read_array.len() == 3 {
                    let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                    let status: usize = datas
                        .last()
                        .ok_or(OperationError::DataGetError)?
                        .to_string()
                        .parse()?;
                    if status == 1 {
                        is_on = true;
                    }
                } else {
                    return Err(OperationError::DataGetError);
                }

                // read polarity
                let mut is_positive = false;
                let command = format!("re {} {} {}\r", bus, dev, ch + 46);
                let read_array = port_write_and_read(command)?;

                if read_array.len() == 3 {
                    let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                    let status: usize = datas
                        .last()
                        .ok_or(OperationError::DataGetError)?
                        .to_string()
                        .parse()?;
                    if status == 1 {
                        is_positive = true;
                    }
                } else {
                    return Err(OperationError::DataGetError);
                }

                // read current HV
                let mut tmp: isize = 10_000;
                let current: isize;
                // sometimes read strange value, so check the stability using loop
                loop {
                    // read Voltage command
                    let command = format!("re {} {} {}\r", bus, dev, ch + 32);
                    let read_array = port_write_and_read(command)?;
                    if read_array.len() != 3 {
                        continue;
                    } else {
                        let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                        let voltage: isize = datas
                            .last()
                            .ok_or(OperationError::DataGetError)?
                            .to_string()
                            .parse()?;
                        if voltage != tmp {
                            tmp = voltage;
                            continue;
                        } else if voltage.abs()
                            > ARGS.get().ok_or(OperationError::ArgumentError)?.max_voltage
                        {
                            // check if it is over maximum voltage (for reading error)
                            continue;
                        } else {
                            current = voltage.abs();
                            break;
                        }
                    }
                }

                mhv4_array.push(MHV4Data::new(
                    idc,
                    bus,
                    dev,
                    ch,
                    current,
                    is_on,
                    is_positive,
                ));
            }
        }
    }
    //let shared_data = Arc::new(Mutex::new(SharedData::new(mhv4_array.clone(), is_rc)));
    let shared_data = Arc::new(Mutex::new(SharedData::new(mhv4_array, is_rc)));
    DATA.set(shared_data)
        .map_err(|_| OperationError::OnceLockError)?;

    log::info!("Initialization is completed!");
    Ok(())
}

// when the page is loaded, this function will be read.
async fn get_mhv4_data() -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("getting current MHV4 status...");
    let shared_data = DATA
        .get()
        .ok_or(OperationError::SharedDataError)?
        .lock()
        .map_err(|_| warp::reject::custom(OperationError::MutexPoisonError))?;

    let obj = shared_data.clone();
    let data_json = serde_json::to_string(&obj)
        .map_err(|e| warp::reject::custom(OperationError::JSONSerializeError(e)))?;

    log::info!("complete to read the status, send to the web browser...");
    Ok(warp::reply::json(&data_json).into_response())
}

// SSE endpoint
fn get_sse_stream() -> impl Stream<Item = Result<Event, OperationError>> {
    log::debug!("SSE handler start...");
    futures::stream::unfold((), |()| async move {
        match read_monitor_value().await {
            Ok(result) => {
                let json = match serde_json::to_string(&result) {
                    Ok(json) => json,
                    Err(e) => return Some((Err(OperationError::JSONSerializeError(e)), ())),
                };
                let sse_data = warp::sse::Event::default().data(json);
                sleep(Duration::from_millis(100)).await;
                Some((Ok::<_, OperationError>(sse_data), ()))
            }
            Err(e) => {
                log::error!("Error in the sse stream: {:?}", e);
                Some((Err(OperationError::ReadingError), ()))
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

async fn read_monitor_value() -> Result<(Vec<isize>, Vec<isize>, bool), OperationError> {
    let mhv4_data_array: Vec<MHV4Data>;
    let is_progress: bool;
    {
        let shared_data = DATA.get().ok_or(OperationError::SharedDataError)?.lock()?;
        mhv4_data_array = shared_data.get_data();
        is_progress = shared_data.is_progress;
    }

    let mut v_array: Vec<isize> = Vec::new();
    let mut c_array: Vec<isize> = Vec::new();

    for i in 0..mhv4_data_array.len() {
        let (bus, dev, ch) = mhv4_data_array[i].get_module_id();

        // read HV value
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
                    log::error!("SSE read error: {:?}", read_array);
                    v_array.push(-100_000);
                }
            }
        } else {
            log::error!("SSE read error: {:?}", read_array);
            v_array.push(-100_000);
        }

        // read current value
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
                    log::error!("SSE read error: {:?}", read_array);
                    c_array.push(-100_000);
                }
            }
        } else {
            log::error!("SSE read error: {:?}", read_array);
            c_array.push(-100_000);
        }
    }
    Ok((v_array, c_array, is_progress))
}

// 0: RC on, 1: RC off
fn set_rcstatus(do_rc: bool) -> Result<bool, OperationError> {
    log::info!("set_rcstatus is called");
    let mhv4_data_array: Vec<MHV4Data>;
    let current_rc: bool;
    {
        let shared_data = DATA.get().ok_or(OperationError::SharedDataError)?.lock()?;
        mhv4_data_array = shared_data.get_data();
        current_rc = shared_data.is_rc;
    }

    // remote ON
    if do_rc && !current_rc {
        // if you use IDC=27 MHV4, please prepare polarity list
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
            let command = format!("on {} {}\r", bus, dev);
            let _ = port_write_and_read(command)?;

            // current limit
            let command = format!("se {} {} {} 20000\r", bus, dev, ch + 8);
            let _ = port_write_and_read(command)?;

            // if you use IDC=27 MHV4, you can set polarity or something in here
            let idc = mhv4_data_array[i].idc;
            if idc == 27 {
                // ramp speed setting
                let command = format!("se {} {} 80 0\r", bus, dev);
                let _ = port_write_and_read(command)?;
            } else {
                // HV range setting
                let command = format!("se {} {} 13 1\r", bus, dev);
                let _ = port_write_and_read(command)?;
            }
        }

        {
            let mut shared_data = DATA.get().ok_or(OperationError::SharedDataError)?.lock()?;
            shared_data.is_rc = true;
        }
    // remote OFF
    } else if !do_rc && current_rc {
        for i in 0..mhv4_data_array.len() {
            let (bus, dev, _) = mhv4_data_array[i].get_module_id();
            let command = format!("off {} {}\r", bus, dev);
            let _ = port_write_and_read(command)?;
        }

        {
            let mut shared_data = DATA.get().ok_or(OperationError::SharedDataError)?.lock()?;
            shared_data.is_rc = false;
        }
    }
    Ok(true)
}

fn set_onoff(arr: Vec<bool>) -> Result<bool, OperationError> {
    log::info!("set_onoff is called");
    let mhv4_data_array: Vec<MHV4Data>;
    {
        let shared_data = DATA.get().ok_or(OperationError::SharedDataError)?.lock()?;
        mhv4_data_array = shared_data.get_data();
    }

    for i in 0..mhv4_data_array.len() {
        let is_on = mhv4_data_array[i].is_on;
        if is_on != arr[i] {
            if arr[i] {
                let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
                let command = format!("se {} {} {} 1\r", bus, dev, ch + 4);
                let _ = port_write_and_read(command)?;
            } else {
                let (bus, dev, ch) = mhv4_data_array[i].get_module_id();
                let command = format!("se {} {} {} 0\r", bus, dev, ch + 4);
                let _ = port_write_and_read(command)?;
            }
        }
    }

    {
        let mut shared_data = DATA.get().ok_or(OperationError::SharedDataError)?.lock()?;
        for i in 0..arr.len() {
            shared_data.set_onoff(i, arr[i]);
        }
    }
    Ok(true)
}

fn set_voltage(nums: Vec<isize>) -> Result<bool, OperationError> {
    let mhv4_data_array: Vec<MHV4Data>;
    {
        let mut shared_data = DATA.get().ok_or(OperationError::SharedDataError)?.lock()?;
        shared_data.is_progress = true;
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
                let _ = port_write_and_read(command).expect("error in the thread");
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
            shared_data.is_progress = false;
            for i in 0..mhv4_data_array.len() {
                shared_data.set_current(i, nums[i]);
            }
        }
    });
    Ok(true)
}

fn port_write_and_read(command: String) -> Result<Vec<String>, OperationError> {
    log::trace!("command: {}", command);

    let mut buf: Vec<u8> = vec![0; 100];
    let size: usize;
    {
        let mut port = PORT.get().ok_or(OperationError::PortGetError)?.lock()?;
        port.write(command.as_bytes())?;
        std::thread::sleep(Duration::from_millis(50));

        size = port.read(buf.as_mut_slice())?;
        std::thread::sleep(Duration::from_millis(10));
    }
    let bytes = &buf[..size];
    let string = String::from_utf8(bytes.to_vec())?;
    let read_array = string.split("\n\r").collect::<Vec<_>>();
    let vec = read_array.iter().map(|&s| s.to_string()).collect();

    log::trace!("result: {:?}", vec);

    Ok(vec)
}

#[tokio::main]
async fn main() -> Result<(), OperationError> {
    // init the logger
    pretty_env_logger::init();
    log::info!("Started the MHV4_monitor server!");

    // argument parser
    log::debug!("trying to get command line arguments...");
    let args = CLArguments::parse();
    ARGS.set(args).map_err(|_| OperationError::OnceLockError)?;
    log::debug!("success to get command line arguments");

    // port connection
    log::debug!(
        "trying to open serial port from {}...",
        ARGS.get().ok_or(OperationError::ArgumentError)?.port_name
    );
    let port = serialport::new(
        &ARGS.get().ok_or(OperationError::ArgumentError)?.port_name,
        ARGS.get().ok_or(OperationError::ArgumentError)?.port_rate,
    )
    .stop_bits(serialport::StopBits::One)
    .data_bits(serialport::DataBits::Eight)
    .parity(serialport::Parity::None)
    .timeout(Duration::from_millis(100))
    .open()?;

    // get Mutex key
    let port = Arc::new(Mutex::new(port));
    PORT.set(port).map_err(|_| OperationError::OnceLockError)?;
    log::debug!("success to open serial port!");

    // main
    initialize_status().await?;

    log::info!("Setting the routing...");
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["POST"]);

    let mhv4_data_route = warp::path("mhv4_data")
        .and(warp::get())
        .and_then(get_mhv4_data);

    let sse_route = warp::path("sse").and(warp::get()).map(|| {
        let stream = get_sse_stream();
        warp::sse::reply(warp::sse::keep_alive().stream(stream))
    });

    let status_route = warp::path("status")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |do_rc: bool| {
            let result = match set_rcstatus(do_rc) {
                Ok(val) => val,
                Err(e) => {
                    log::error!("Error: {:?}", e);
                    false
                }
            };
            warp::reply::json(&result)
        })
        .with(cors.clone());

    let onoff_route = warp::path("onoff")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |arr: Vec<bool>| {
            let result = match set_onoff(arr) {
                Ok(val) => val,
                Err(e) => {
                    log::error!("Error: {:?}", e);
                    false
                }
            };
            warp::reply::json(&result)
        })
        .with(cors.clone());

    let apply_route = warp::path("apply")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |nums: Vec<isize>| {
            let result = match set_voltage(nums) {
                Ok(val) => val,
                Err(e) => {
                    log::error!("Error: {:?}", e);
                    false
                }
            };
            warp::reply::json(&result)
        })
        .with(cors.clone());

    if ARGS
        .get()
        .ok_or(OperationError::ArgumentError)?
        .is_localhost
    {
        let static_files = warp::fs::dir("public/out");
        let routes = static_files
            .or(mhv4_data_route)
            .or(sse_route)
            .or(status_route)
            .or(onoff_route)
            .or(apply_route);

        warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
    } else {
        let routes = mhv4_data_route
            .or(sse_route)
            .or(status_route)
            .or(onoff_route)
            .or(apply_route);

        warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
    }

    Ok(())
}
