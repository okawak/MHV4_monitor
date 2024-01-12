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
    arg_required_else_help = true,
)]
struct MyArguments {
    #[clap(
        value_name = "COMMAND",
        help = "send command (String), ex. \"sc 0\"",
        required = true
    )]
    command: String,

    #[clap(short = 'p', long = "port_name", default_value = "/dev/ttyUSB0")]
    port_name: String,
}

#[allow(unused_assignments)]
fn main() -> Result<(), Box<dyn Error>> {
    let args = MyArguments::parse();

    let mut port = serialport::new(args.port_name, 9600)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()?;

    let mut buf: Vec<u8> = vec![0; 1000];
    let mut vec: Vec<String> = Vec::new();

    let send_str = format!("{}\r", args.command);

    match port.write(send_str.as_bytes()) {
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
            if vec.len() > 2 {
                println!("send command: {}", vec[0]);
                vec.remove(0);
                vec.pop();
                println!("Result:");
                println!("{:?}", vec);
            } else {
                println!("read error!");
                println!("{:?}", vec);
            }
            Ok(())
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(()),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(Box::new(e))
        }
    }
}
