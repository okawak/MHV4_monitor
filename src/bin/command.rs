use clap::Parser;
use std::error::Error;
use std::io::prelude::*;
use std::time::Duration;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct MyArguments {
    #[clap(short = 'c', long = "command", default_value = "sc")]
    command: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = MyArguments::parse();

    let mut port = serialport::new("/dev/ttyUSB0", 9600)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()?;

    let mut buf: Vec<u8> = vec![0; 1000];
    let mut vec: Vec<String> = Vec::new();

    match port.write(args.command.as_bytes()) {
        Ok(_) => std::io::stdout().flush()?,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    std::thread::sleep(Duration::from_millis(100));

    // read from the serial port
    match port.read(buf.as_mut_slice()) {
        Ok(t) => {
            let bytes = &buf[..t];
            let string = String::from_utf8(bytes.to_vec())?;
            let v = string.split("\n\r").collect::<Vec<_>>();
            vec = v.iter().map(|&s| s.to_string()).collect();
            println!("{:?}", vec);
            Ok(())
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(()),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(Box::new(e))
        }
    }
}
