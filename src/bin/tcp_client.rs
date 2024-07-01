#[path = "../file/file_io.rs"]
mod file_io;
#[path = "../packet.rs"]
mod packet;
#[path = "../threadpool.rs"]
mod threadpool;
#[path = "../utils.rs"]
mod utils;

use ::serde::de::{DeserializeOwned, DeserializeSeed};
use ::serde::{Deserialize, Serialize};
use packet::MsgPacket;
use std::any::{type_name, type_name_of_val};
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{string, thread};
use threadpool::ThreadPool;
use utils::register_sig_handler;
use uuid::serde;

fn main() -> () {
    const ADDRESS: &str = "127.0.0.1:8080";

    let pool = ThreadPool::new(2);
    let stream = TcpStream::connect(ADDRESS).unwrap_or_else(|e| {
        eprintln!("Failed to connect to server: {}", e);
        std::process::exit(1);
    });
    let local_addr = stream
        .local_addr()
        .expect("Failed to get local address")
        .to_string();
    let stream = Arc::new(stream);
    let stream_clone = Arc::clone(&stream);

    println!("Connected to server: {}", ADDRESS);
    println!("Chat Type [1 for msg_packet  / 2 for string]: ");
    let msg_type: bool;
    let mut client_id = String::new();
    {
        let mut tmp_buf = String::new();
        io::stdin().read_line(&mut tmp_buf).unwrap();
        let tmp_input = &tmp_buf[..tmp_buf.len() - 1];
        msg_type = match tmp_input {
            "1" => {
                print!("Enter client id: ");
                io::stdout().flush().unwrap_or_else(|e| {
                    eprintln!("Failed to flush stdout: {}", e);
                });
                let _ = io::stdin().read_line(&mut client_id).unwrap_or_else(|e| {
                    eprintln!("Invalid id: {}", e);
                    client_id = String::from(local_addr);
                    0
                });
                client_id.pop();
                println!("Logined as {}!", client_id);
                true
            }
            _ => false,
        }
    }
    println!("msg type: {}", msg_type);
    register_sig_handler(move || {
        println!("Exiting....");
        stream_clone
            .shutdown(std::net::Shutdown::Both)
            .unwrap_or_else(|e| {
                eprintln!("Failed to shutdown stream: {}", e);
            });
        std::process::exit(0);
    });

    if msg_type {
        let (tx, rx) = mpsc::channel();
        pool.execute(move || {
            handle_connection2::<MsgPacket>(&stream, rx, MsgPacket::dummy());
        });
        let mut packet_base = MsgPacket::dummy();
        packet_base.id = client_id;
        send_loop(tx, pool, packet_base, MsgPacket::parse);
    } else {
        let (tx, rx) = mpsc::channel();
        pool.execute(move || {
            handle_connection2::<String>(&stream, rx, "".to_string());
        });
        send_loop(tx, pool, "".to_string(), |_, x| x)
    }
}

fn send_loop<T, F>(tx: Sender<T>, mut pool: ThreadPool, parse_base: T, parse: F) -> ()
where
    T: std::fmt::Debug + DeserializeOwned + Serialize + Clone,
    F: Fn(T, String) -> T,
{
    println!("\"q\" : for exit");
    println!("Enter message to send: ");
    loop {
        {
            // let mut stdin = io::stdin();
            let mut stdin = io::stdin().lock();
            let mut input: String = String::new();
            let _ = stdin.read_line(&mut input);
            let msg = &input[..input.len() - 1];
            match msg {
                "q" => {
                    println!("Exiting....");
                    pool.join();
                    std::process::exit(1);
                }
                _ => {
                    tx.send(parse(parse_base.clone(), msg.to_string()))
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to send: {}", e);
                        });
                }
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn handle_connection2<T>(mut stream: &TcpStream, send_rx: mpsc::Receiver<T>, dummy: T) -> ()
where
    T: std::fmt::Debug + DeserializeOwned + Serialize + Clone,
{
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap_or_else(|e| {
            eprintln!("Failed to set read timeout: {}", e);
            std::process::exit(1);
        });
    loop {
        {
            let mut buf = vec![0; 1024];
            let recv_len = stream.read(&mut buf).unwrap_or(0);
            if recv_len > 0 {
                let recv_packet =
                    serde_json::from_slice::<T>(&buf[..recv_len]).unwrap_or_else(|e| {
                        // println!("Bytes: {:?}", &buf[..recv_len]);
                        eprintln!("Error decoding message len: {} Err: {}", recv_len, e);
                        dummy.clone()
                    });
                println!(">> {:?}", recv_packet);
            }
        }
        match send_rx.try_recv() {
            Ok(msg) => {
                let buf = serde_json::to_vec(&msg).unwrap();
                let send_len = stream.write(&buf).unwrap_or_else(|e| {
                    eprintln!("Failed to send: {:#?} Err: {}", msg, e);
                    0
                });
                if send_len > 0 {
                    stream.flush().unwrap_or_else(|e| {
                        eprintln!("Failed to flush buffer for : {:#?} Err: {}", msg, e);
                    });
                }
                ()
            }
            Err(_) => (),
        }
    }
}

fn send_msg(mut stream: &TcpStream, msg: &str) -> () {
    let msg_bytes = msg.as_bytes();
    let mut send_succ = true;
    let _ = stream.write(msg_bytes).unwrap_or_else(|e| {
        println!("Failed to send: \"{}\" Err: {}", msg, e);
        send_succ = false;
        0
    });
    stream.flush().unwrap_or_else(|e| {
        println!("Failed to flush: {}", msg);
    });
    ()
}

fn log_to_file(mut file: &File, msg: &str) {
    file.write_all(msg.as_bytes()).unwrap_or_else(|e| {
        eprintln!("Failed to write to file: {}", e);
        std::process::exit(1);
    });
}
