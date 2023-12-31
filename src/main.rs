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
use warp::sse::Event;
use warp::Filter;
use warp::Reply;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
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

// 共有データ構造体
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

struct InitializationState {
    initialized: bool,
}

impl InitializationState {
    fn new() -> InitializationState {
        InitializationState { initialized: false }
    }

    fn set_initialized(&mut self) {
        self.initialized = true;
    }
}

async fn initialize_serial_port(
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    init_state: Arc<Mutex<InitializationState>>,
    shared_data: Arc<Mutex<SharedData>>,
) -> Result<(), io::Error> {
    let mut port = port.lock().unwrap();

    let mut shared_data = shared_data.lock().unwrap();

    for bus in 0..=1 {
        let command = format!("sc {}", bus);
        port.write(command.as_bytes()).expect("Write failed!");

        std::thread::sleep(Duration::from_millis(100));

        let mut buf: Vec<u8> = vec![0; 100];
        let size = port.read(buf.as_mut_slice()).expect("Found no data!");
        let bytes = &buf[..size];
        let string = String::from_utf8(bytes.to_vec()).expect("Failed to convert");
        let modules = string.split("\n\r").collect::<Vec<_>>();

        for j in 0..16 {
            let module = modules[j + 2].to_string();
            let datas = module.split_whitespace().collect::<Vec<_>>();
            if datas[1] == "-" {
                continue;
            }

            let idc: usize = (&datas[1][..2]).parse().unwrap();
            if idc != 27 || idc != 17 {
                continue;
            }

            let mut tmp = datas[0].to_string();
            tmp.pop();
            let dev: usize = tmp.parse().unwrap();

            shared_data
                .mhv4_data_array
                .push(MHV4Data::new(idc, bus, dev));
        }
    }

    let mut init_state = init_state.lock().unwrap();
    init_state.set_initialized();

    Ok(())
}

// mhv4_data_array の情報を取得するエンドポイント
fn get_mhv4_data(
    shared_data: Arc<Mutex<SharedData>>,
    init_state: Arc<Mutex<InitializationState>>,
) -> impl warp::Reply {
    let init_state = init_state.lock().unwrap();

    if !init_state.initialized {
        std::thread::sleep(Duration::from_millis(100));
        return warp::reply::with_status("Not available", warp::http::StatusCode::BAD_REQUEST)
            .into_response();
    }

    let shared_data = shared_data.lock().unwrap();
    let mhv4_data_array = &shared_data.mhv4_data_array;

    let data_json = serde_json::to_string(mhv4_data_array).unwrap_or_else(|_| "[]".to_string());

    warp::reply::json(&data_json).into_response()
}

// SSEエンドポイントのハンドラ
fn sse_handler(port: Arc<Mutex<Box<dyn SerialPort>>>) -> impl warp::Reply {
    let interval = time::interval(Duration::from_millis(100));
    let stream = IntervalStream::new(interval).map(move |_| {
        let mut port = port.lock().unwrap();
        // ここでシリアルポートからのデータを読み取り
        // 例: let data = port.read(...);
        // 仮のデータを返す
        Ok::<_, warp::Error>(Event::default().data("Some data from serial port"))
    });

    warp::sse::reply(stream)
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
    let init_state = Arc::new(Mutex::new(InitializationState::new()));

    // 初期化処理の実行
    initialize_serial_port(port.clone(), init_state.clone(), shared_data.clone())
        .await
        .expect("Failed to initialize serial port");

    let get_mhv4_data_route = warp::get()
        .and(warp::path("mhv4_data"))
        .map(move || get_mhv4_data(shared_data.clone(), init_state.clone()));

    let port_for_sse = port.clone();
    let sse_route = warp::path("sse").map(move || sse_handler(port_for_sse.clone()));

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
