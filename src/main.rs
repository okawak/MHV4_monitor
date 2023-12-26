mod mhv4;

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
    response: String, // 応答を格納するフィールド
    mhv4_data_array: Vec<MHV4Data>,
}

impl SharedData {
    fn new() -> SharedData {
        SharedData {
            response: String::new(),
            mhv4_data_array: Vec::new(),
        }
    }
}

struct InitializationState {
    initialized: bool,
    get_mhv4_data_called: bool,
}

impl InitializationState {
    fn new() -> InitializationState {
        InitializationState {
            initialized: false,
            get_mhv4_data_called: false,
        }
    }

    fn set_initialized(&mut self) {
        self.initialized = true;
    }

    fn set_get_mhv4_data_called(&mut self) {
        self.get_mhv4_data_called = true;
    }
}

// コマンドライン引数からポート名を取得し、シリアルポートを開く
fn open_serial_port() -> Result<Box<dyn SerialPort>, serialport::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <serial port>", args[0]);
        std::process::exit(1);
    }
    let port_name = &args[1];
    serialport::new(port_name, 9600).open()
}

async fn initialize_serial_port(
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    init_state: Arc<Mutex<InitializationState>>,
    shared_data: Arc<Mutex<SharedData>>,
) -> Result<(), io::Error> {
    let mut port = port.lock().unwrap();

    // シリアルポートに何かしらの初期化信号を送信
    port.write_all(b"initialization signal")?;

    // ここで応答を受け取る処理を実装
    // ...
    // let mut response = String::new();
    // port.read_to_string(&mut response)?;

    // SharedData の response を更新
    let mut shared_data = shared_data.lock().unwrap();
    shared_data.response = "test".to_string();

    let mut init_state = init_state.lock().unwrap();
    init_state.set_initialized();

    Ok(())
}

// mhv4_data_array の情報を取得するエンドポイント
fn get_mhv4_data(
    shared_data: Arc<Mutex<SharedData>>,
    init_state: Arc<Mutex<InitializationState>>,
) -> impl warp::Reply {
    let mut init_state = init_state.lock().unwrap();

    if !init_state.initialized || init_state.get_mhv4_data_called {
        return warp::reply::with_status("Not available", warp::http::StatusCode::BAD_REQUEST)
            .into_response();
    }

    init_state.set_get_mhv4_data_called();

    let shared_data = shared_data.lock().unwrap();
    let mhv4_data_array = &shared_data.mhv4_data_array;

    // データを処理してJSON形式などに変換
    // ここでは例としてJSON文字列を返す
    let data_json = serde_json::to_string(mhv4_data_array).unwrap_or_else(|_| "[]".to_string());

    warp::reply::json(&data_json).into_response()
    // JSON に変換するには serde_json クレートなどが便利です
}

// SSEエンドポイントのハンドラ
fn sse_handler(port: Arc<Mutex<Box<dyn SerialPort>>>) -> impl warp::Reply {
    let interval = time::interval(Duration::from_micros(1));
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
    let port = open_serial_port().expect("Failed to open serial port");
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

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
