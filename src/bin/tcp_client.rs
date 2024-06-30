#[path = "../file/file_io.rs"]
mod file_io;
#[path = "../threadpool.rs"]
mod threadpool;
#[path = "../utils.rs"]
mod utils;

use std::fs::File;
use std::io::prelude::*;
use std::io::{self};
use std::net::TcpStream;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;
use utils::register_sig_handler;

fn main() -> () {
    const ADDRESS: &str = "127.0.0.1:8080";

    let mut pool = ThreadPool::new(2);
    let stream = TcpStream::connect(ADDRESS).unwrap_or_else(|e| {
        eprintln!("Failed to connect to server: {}", e);
        std::process::exit(1);
    });
    let stream = Arc::new(stream);
    let stream_clone = Arc::clone(&stream);
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(tx);
    register_sig_handler(move || {
        println!("Exiting....");
        stream_clone
            .shutdown(std::net::Shutdown::Both)
            .unwrap_or_else(|e| {
                eprintln!("Failed to shutdown stream: {}", e);
            });
        std::process::exit(0);
    });
    pool.execute(move || {
        handle_connection(&stream, rx);
    });

    println!("Connected to server: {}", ADDRESS);
    println!("\"q\" : for exit");
    println!("Enter message to send: ");
    loop {
        {
            // let mut stdin = io::stdin();
            let mut stdin = io::stdin().lock();
            let mut input: String = String::new();
            let _ = stdin.read_line(&mut input);
            let msg = input.as_str();
            match msg {
                "q" => {
                    println!("Exiting....");
                    pool.join();
                    std::process::exit(1);
                }
                _ => {
                    tx.send(msg.to_string()).unwrap_or_else(|e| {
                        eprintln!("Failed to send: {}", e);
                    });
                }
            }
        }
        thread::sleep(Duration::from_millis(20));
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

fn handle_connection(mut stream: &TcpStream, rx: mpsc::Receiver<String>) -> () {
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap_or_else(|e| {
            eprintln!("Failed to set read timeout: {}", e);
            std::process::exit(1);
        });
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        let recv_len = stream.read(&mut buf).unwrap_or(0);
        if recv_len > 0 {
            let buf_str = String::from_utf8_lossy(&buf[..recv_len]).to_string();
            println!(">> {}", buf_str);
        }
        match rx.try_recv() {
            Ok(msg) => {
                let trans_len = stream.write(msg.as_bytes()).unwrap_or_else(|e| {
                    eprintln!("Failed to send: {} Err: {}", msg, e);
                    0
                });
                if trans_len > 0 {
                    stream.flush().unwrap_or_else(|e| {
                        eprintln!("Failed to flush buffer: {}", msg);
                    });
                }
                ()
            }
            Err(_) => (),
        }
    }
}

fn log_to_file(mut file: &File, msg: &str) {
    file.write_all(msg.as_bytes()).unwrap_or_else(|e| {
        eprintln!("Failed to write to file: {}", e);
        std::process::exit(1);
    });
}
