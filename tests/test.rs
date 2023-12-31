use std::env;
use std::error::Error;
use std::io::prelude::*;
use std::time::Duration;

fn mrc_scan_result() -> Result<Vec<String>, Box<dyn Error>> {
    let mut port = serialport::new("/dev/ttyUSB0", 9600)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()?;

    let mut buf: Vec<u8> = vec![0; 1000];
    let mut vec: Vec<String> = Vec::new();
    println!("Write...");
    match port.write("sc 0\r".as_bytes()) {
        Ok(_) => std::io::stdout().flush()?,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    std::thread::sleep(Duration::from_millis(100));

    println!("Read...");
    match port.read(buf.as_mut_slice()) {
        Ok(t) => {
            let bytes = &buf[..t];
            let string = String::from_utf8(bytes.to_vec())?;
            let v = string.split("\n\r").collect::<Vec<_>>();
            vec = v.iter().map(|&s| s.to_string()).collect();
            println!("{:?}", vec);
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    std::thread::sleep(Duration::from_millis(1000));
    return Ok(vec);
}

#[test]
fn argument_test() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);
}

#[test]
fn scan_test() {
    let sc_result_vec = mrc_scan_result().expect("Cannot get the output");
    assert_eq!(sc_result_vec.last().unwrap(), &String::from("mrc-1>"));
}
