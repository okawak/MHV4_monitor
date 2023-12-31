use std::error::Error;
use std::io::prelude::*;
use std::time::Duration;

fn mrc_scan_result(bus: isize) -> Result<Vec<String>, Box<dyn Error>> {
    let mut port = serialport::new("/dev/ttyUSB0", 9600)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()?;

    let mut buf: Vec<u8> = vec![0; 1000];
    let mut vec: Vec<String> = Vec::new();
    println!("Write...");
    let command = format!("sc {}\r", bus);
    match port.write(command.as_bytes()) {
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
    return Ok(vec);
}

#[test]
fn scan_test() {
    let sc0_result_vec = mrc_scan_result(0).expect("Cannot get the output");
    let sc1_result_vec = mrc_scan_result(1).expect("Cannot get the output");

    // result of sc 0
    assert_eq!(sc0_result_vec.len(), 19);
    assert_eq!(sc0_result_vec[0], String::from("sc 0"));
    assert_eq!(sc0_result_vec[1], String::from("ID-SCAN BUS 0:"));
    assert_eq!(sc0_result_vec[18], String::from("mrc-1>"));

    // result of sc 1
    assert_eq!(sc1_result_vec.len(), 19);
    assert_eq!(sc1_result_vec[0], String::from("sc 1"));
    assert_eq!(sc1_result_vec[1], String::from("ID-SCAN BUS 1:"));
    assert_eq!(sc1_result_vec[18], String::from("mrc-1>"));
}
