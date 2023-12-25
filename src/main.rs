use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use futures::Future;
use futures::StreamExt;
use serialport::SerialPort;
use std::env;
use std::io::Write;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use tokio_stream::wrappers::IntervalStream;
use warp::reject::Reject;
use warp::sse::Event;
use warp::Filter;

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

// ブラウザからの入力をシリアル通信で送る
fn send_to_serial_port(
    data: Bytes,
    port: Arc<Mutex<Box<dyn SerialPort>>>,
) -> Pin<Box<dyn Future<Output = Result<impl warp::Reply, warp::Rejection>> + Send>> {
    Box::pin(async move {
        if data.len() != 4 {
            return Err(warp::reject::custom(SerialPortError::new(
                "Invalid data length",
            )));
        }

        let mut port = port.lock().unwrap();

        // バイナリデータをf32に変換
        let mut float_data = [0u8; 4];
        float_data.copy_from_slice(&data);
        let number = LittleEndian::read_f32(&float_data);

        // 何らかの処理
        let processed_value = number * 2.0; // 例: 値を2倍にする

        // 処理した値を文字列に変換
        let string_value = processed_value.to_string();

        // 文字列をシリアルポートに送信
        port.write_all(string_value.as_bytes()).map_err(|e| {
            eprintln!("Error sending data to serial port: {}", e);
            warp::reject::custom(SerialPortError::new(&e.to_string()))
        })?;

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

    let port_for_sse = port.clone();
    let sse_route = warp::path("sse").map(move || sse_handler(port_for_sse.clone()));

    let port_for_send = port.clone();
    let send_route = warp::post()
        .and(warp::path("send"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::bytes())
        .and_then(move |data| send_to_serial_port(data, port_for_send.clone()));

    let static_files = warp::fs::dir("www");

    let routes = static_files.or(sse_route).or(send_route);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
