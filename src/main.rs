mod mhv4;

use clap::Parser;
use futures::{Future, StreamExt};
use mhv4::MHV4Data;
use serialport::SerialPort;
use std::env;
use std::io::{self, Read, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use tokio_stream::wrappers::IntervalStream;
use warp::reject::Reject;
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
}

#[derive(Debug)]
struct SerialPortError {
    message: String,
}

impl SerialPortError {
    fn new(message: &str) -> SerialPortError {
        SerialPortError {
            message: message.to_string(),
        }
    }
}

impl Reject for SerialPortError {}

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

async fn initialize_serial_port(
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    shared_data: Arc<Mutex<SharedData>>,
) -> Result<(), io::Error> {
    let mut port = port.lock().unwrap();

    let mut shared_data = shared_data.lock().unwrap();

    for bus in 0..2 {
        let command = format!("sc {}\r", bus);
        port.write(command.as_bytes()).expect("Write failed!");

        std::thread::sleep(Duration::from_millis(100));

        let mut buf: Vec<u8> = vec![0; 300];
        let size = port.read(buf.as_mut_slice()).expect("Found no data!");
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

            for ch in 0..4 {
                shared_data
                    .mhv4_data_array
                    .push(MHV4Data::new(idc, bus, dev, ch));
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
) -> impl warp::Reply {
    let mhv4_data_array: Vec<MHV4Data>;
    {
        let shared_data = shared_data.lock().unwrap();
        mhv4_data_array = shared_data.mhv4_data_array.to_vec();
    }

    let interval = time::interval(Duration::from_millis(1000));
    let stream = IntervalStream::new(interval).map(move |_| {
        let mut v_array: Vec<isize> = Vec::new();
        let mut c_array: Vec<isize> = Vec::new();

        for i in 0..mhv4_data_array.len() {
            let (bus, dev, ch) = mhv4_data_array[i].get_module_id();

            let command = format!("re {} {} {}\r", bus, dev, ch + 32);
            let read_array =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
            if read_array.len() != 3 {
                v_array.push(-1000);
            } else {
                let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                let voltage = datas.last().unwrap().to_string();
                v_array.push(voltage.parse().unwrap());
            }

            let command = format!("re {} {} {}\r", bus, dev, ch + 50);
            let read_array =
                port_write_and_read(port.clone(), command).expect("Error in port communication");
            if read_array.len() != 3 {
                c_array.push(-1000);
            } else {
                let datas = read_array[1].split_whitespace().collect::<Vec<_>>();
                let current = datas.last().unwrap().to_string();
                c_array.push(current.parse().unwrap());
            }
        }

        let datas = (v_array, c_array);
        //let datas = (vec![0; 8], vec![0; 8]);
        let sse_json = serde_json::to_string(&datas).unwrap();

        Ok::<_, warp::Error>(warp::sse::Event::default().data(sse_json))
    });

    warp::sse::reply(stream)
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
// シリアルポートにデータを送信するハンドラ
fn send_to_serial_port(
    data: String,
    port: Arc<Mutex<Box<dyn SerialPort>>>,
) -> Pin<Box<dyn Future<Output = Result<impl warp::Reply, warp::Rejection>> + Send>> {
    Box::pin(async move {
        let values: Vec<f32> = match serde_json::from_str(&data) {
            Ok(val) => val,
            Err(_) => {
                return Err(warp::reject::custom(SerialPortError::new(
                    "Invalid data format",
                )));
            }
        };

        let mut port = port.lock().unwrap();
        // ここで values をシリアルポートに送信する処理を行う
        // ...
        //        // 何らかの処理
        //        let processed_value = number * 2.0; // 例: 値を2倍にする
        //
        //        // 処理した値を文字列に変換
        //        let string_value = processed_value.to_string();
        //
        //        // 文字列をシリアルポートに送信
        //        port.write_all(string_value.as_bytes()).map_err(|e| {
        //            eprintln!("Error sending data to serial port: {}", e);
        //            warp::reject::custom(SerialPortError::new(&e.to_string()))
        //        })?;

        Ok(warp::reply::with_status(
            "Data sent to serial port",
            warp::http::StatusCode::OK,
        ))
    })
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args: MyArguments = MyArguments::parse();

    let port = serialport::new(args.port_name, args.port_rate)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Failed to open serial port");

    let port = Arc::new(Mutex::new(port));
    let shared_data = Arc::new(Mutex::new(SharedData::new()));

    // 初期化処理の実行
    initialize_serial_port(port.clone(), shared_data.clone())
        .await
        .expect("Failed to initialize serial port");

    let shared_for_sse = shared_data.clone();
    let get_mhv4_data_route = warp::get()
        .and(warp::path("mhv4_data"))
        .map(move || get_mhv4_data(shared_data.clone()));

    let port_for_sse = port.clone();
    let sse_route =
        warp::path("sse").map(move || sse_handler(port_for_sse.clone(), shared_for_sse.clone()));

    let port_for_send = port.clone();
    let send_route = warp::post()
        .and(warp::path("send"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and_then(move |data| send_to_serial_port(data, port_for_send.clone()));

    let static_files = warp::fs::dir("www");

    let routes = static_files
        .or(sse_route)
        .or(send_route)
        .or(get_mhv4_data_route);

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
