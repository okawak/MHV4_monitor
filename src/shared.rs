use crate::mhv4::MHV4Data;
use clap::Parser;
use serde::Serialize;
use std::error::Error;
use std::fmt;
use std::sync::MutexGuard;
use std::sync::PoisonError;

#[derive(Serialize, Debug, Clone)]
pub struct SharedData {
    mhv4_data_array: Vec<MHV4Data>,
    pub is_rc: bool,
    pub is_progress: bool,
}

impl SharedData {
    pub fn new(in_vec: Vec<MHV4Data>, in_is_rc: bool) -> SharedData {
        SharedData {
            mhv4_data_array: in_vec,
            is_rc: in_is_rc,
            is_progress: false,
        }
    }

    pub fn get_data(&self) -> Vec<MHV4Data> {
        self.mhv4_data_array.clone()
    }

    pub fn set_current(&mut self, id: usize, in_current: isize) {
        self.mhv4_data_array[id].set_current(in_current);
    }

    pub fn set_onoff(&mut self, id: usize, do_on: bool) {
        self.mhv4_data_array[id].is_on = do_on;
    }
}

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct CLArguments {
    #[clap(short = 'p', long = "port_name", default_value = "/dev/ttyUSB0")]
    pub port_name: String,

    #[clap(short = 'r', long = "port_rate", default_value = "9600")]
    pub port_rate: u32,

    #[clap(short = 's', long = "apply_hv_step", default_value = "5")] // 1 -> 0.1 V
    pub voltage_step: isize,

    #[clap(short = 'w', long = "waiting_time_ms", default_value = "500")]
    pub waiting_time: u64,

    #[clap(short = 'm', long = "max_voltage", default_value = "3000")] // 1 -> 0.1 V
    pub max_voltage: isize,

    #[clap(short = 'l', long = "localhost")] // 1 -> 0.1 V
    pub is_localhost: bool,
}

// This error is used only for initialize part
#[derive(Debug)]
pub enum OperationError {
    ArgumentError,
    OnceLockError,
    SerialPortError(String),
    Utf8Error(std::string::FromUtf8Error),
    ParseIntError(std::num::ParseIntError),
    PortGetError,
    MutexPoisonError,
    PortIOError,
    DataGetError,
    JSONSerializeError(serde_json::Error),
    ReadingError,
    SharedDataError,
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OperationError::ArgumentError => write!(f, "Could not get Argument variable"),
            OperationError::OnceLockError => write!(f, "OnceLockError"),
            OperationError::SerialPortError(ref err) => write!(f, "SerialPortError: {}", err),
            OperationError::Utf8Error(ref err) => write!(f, "Utf8 port read Error: {}", err),
            OperationError::ParseIntError(ref err) => write!(f, "Parse Error: {}", err),
            OperationError::PortGetError => write!(f, "Port Get Error"),
            OperationError::MutexPoisonError => write!(f, "Mutex Error"),
            OperationError::PortIOError => write!(f, "Port IO Error"),
            OperationError::DataGetError => write!(f, "Data Get Error, no read data?"),
            OperationError::JSONSerializeError(ref err) => {
                write!(f, "JSON Serialize Error: {}", err)
            }
            OperationError::ReadingError => write!(f, "SSE Reading Error"),
            OperationError::SharedDataError => write!(f, "Could not get shared data"),
        }
    }
}

impl Error for OperationError {}
impl warp::reject::Reject for OperationError {}

impl From<serialport::Error> for OperationError {
    fn from(err: serialport::Error) -> Self {
        OperationError::SerialPortError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for OperationError {
    fn from(err: std::string::FromUtf8Error) -> OperationError {
        OperationError::Utf8Error(err)
    }
}

impl From<std::num::ParseIntError> for OperationError {
    fn from(err: std::num::ParseIntError) -> OperationError {
        OperationError::ParseIntError(err)
    }
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for OperationError {
    fn from(_: PoisonError<MutexGuard<'_, T>>) -> Self {
        OperationError::MutexPoisonError
    }
}

impl From<std::io::Error> for OperationError {
    fn from(_: std::io::Error) -> Self {
        OperationError::PortIOError
    }
}
