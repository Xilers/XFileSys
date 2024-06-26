use crate::device::spec;
use serde_json::to_vec;
use std::io::{self, Write};
use std::net::TcpStream;

struct IPv4 {
    ip: String,
    port: u16,
}

pub fn connect() -> TcpStream {
    let stream = match TcpStream::connect("127.0.0.1:7878") {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            std::process::exit(1);
        }
    };

    stream
}

pub fn send_device_spec(stream: &mut TcpStream) -> io::Result<()> {
    let mut device_info = spec::get_system_info();

    let local_l3_info = IPv4 {
        ip: stream.local_addr().unwrap().ip().to_string(),
        port: stream.local_addr().unwrap().port(),
    };

    device_info.ip_addr = local_l3_info.ip;
    device_info.port = local_l3_info.port;

    let serialized_data = to_vec(&device_info).unwrap();

    let data_length = serialized_data.len() as u32;
    let length_bytes = data_length.to_be_bytes();

    stream.write_all(&[0u8])?;
    stream.write_all(&length_bytes)?;
    stream.write_all(&serialized_data)?;

    Ok(())
}
