mod connect;
mod device;
mod file;
mod packet;
mod threadpool;
mod utils;

use file::file_io::{copy_part, create_file, read_file};
use threadpool::Message;

use packet::MsgPacket;
use std::fs::{remove_file, File};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{thread, time};
use utils::register_sig_handler;

fn main() {
    run_loopback_server();
}

/// Handle connection for both send and receive over TcpStream
///
/// # Arguments
///
/// * `stream` - TcpStream for connection
/// * `rx` - Receiver for shutdown signal
///
fn handle_connection(mut stream: TcpStream, rx: Receiver<threadpool::Message>) {
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("Failed to set read timeout");
    println!("Source Address: {}", (&stream).peer_addr().unwrap());
    let client_id = (&stream).peer_addr().unwrap().port();
    loop {
        match rx.try_recv() {
            Ok(msg) => match msg {
                threadpool::Message::Terminate => {
                    println!("Terminate received.");
                    stream
                        .shutdown(std::net::Shutdown::Both)
                        .unwrap_or_else(|_| eprintln!("Failed to shutdown stream."));
                    break;
                }
                _ => (),
            },
            Err(_) => {}
        }
        let mut buf: Vec<u8> = vec![0; 1024];
        let recv_len = stream.read(&mut buf).unwrap_or_else(|_| 0);
        if recv_len == 0 {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        let mut msg = String::new();
        match serde_json::from_slice::<MsgPacket>(&buf) {
            Ok(packet) => {
                println!("#{:5}(msg): {}", client_id, packet.data);
                msg = packet.data;
            }
            Err(_) => {
                msg = String::from_utf8_lossy(&buf[..(recv_len)]).to_string();
                println!("#{:5}(str): {}", client_id, msg);
            }
        }
        stream.write(msg.as_bytes()).unwrap();
        stream.flush().unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}

fn run_loopback_server() {
    const ADDRESS: &str = "127.0.0.1:8080";
    let listener = TcpListener::bind(ADDRESS).unwrap();
    println!("Server listening on {}", ADDRESS);
    let pool = threadpool::ThreadPool::new(4);
    let pool = Arc::new(Mutex::new(pool));
    let pool_clone = Arc::clone(&pool);
    let term_tx_list = Arc::new(Mutex::new(Vec::<Sender<Message>>::new()));
    let term_tx_list_clone: Arc<Mutex<Vec<Sender<Message>>>> = Arc::clone(&term_tx_list);
    register_sig_handler(move || {
        println!("Shutting down Threadpool on signal.");
        for tx in term_tx_list_clone.lock().unwrap().iter() {
            tx.send(threadpool::Message::Terminate).unwrap();
        }
        match pool_clone.try_lock() {
            Ok(mut pool) => {
                println!("Threadpool Joining....");
                pool.join();
                println!("Threadpool Joined!");
            }
            Err(_) => {
                println!("Failed to aquire lock for threadpool. Unsafe Exiting..");
            }
        };
        std::process::exit(0);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let (tx, rx) = mpsc::channel();
                term_tx_list.lock().unwrap().push(tx);
                pool.lock().unwrap().execute(|| {
                    handle_connection(stream, rx);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    println!("Shutting down.");
}

fn benchmark_file_io_perf() {
    const SRC_NAME: &str = "large_file_src.txt";
    const DEST_NAME_PREFIX: &str = "large_file_destination";
    const SRC_SIZE: u64 = 1024 * 1024 * 1024 * 4; // 4GB (in bytes)
    const THREAD_NUM: usize = 1;

    let threadpool = threadpool::ThreadPool::new(THREAD_NUM);

    for i in 0..THREAD_NUM {
        let dest_name = format!("{DEST_NAME_PREFIX}_{i}.txt");
        let dest_path = Path::new(dest_name.as_str());
        if dest_path.exists() {
            remove_file(dest_path).expect(format!("Failed to remove file: {dest_name}").as_str());
        }
    }
    println!(
        "Writing file of size {} MB with {} Threads...",
        SRC_SIZE / (1024 * 1024),
        THREAD_NUM
    );

    let _ = create_file(SRC_NAME, SRC_SIZE);
    let current_time = time::Instant::now();

    let src_path = Path::new(SRC_NAME);

    let metadata = std::fs::metadata(&src_path).unwrap();
    let file_size = metadata.len();

    for i in 0..THREAD_NUM {
        let offset = SRC_SIZE / THREAD_NUM as u64;
        let dest_name = format!("{DEST_NAME_PREFIX}_{i}.txt");
        let dest_path = Path::new(dest_name.as_str());
        let mut dest_file = File::create(&dest_path).unwrap();
        let start = i as u64 * offset;
        let length = if i == THREAD_NUM - 1 {
            file_size - start as u64
        } else {
            offset
        };
        // println!("Copying part from {} at length {}...", start, length);
        threadpool.execute(move || {
            copy_part(&src_path, &mut dest_file, start, length).expect("Failed to copy part");
        });
    }
    drop(threadpool);
    println!("Elapsed time: {} msec", current_time.elapsed().as_millis());

    let mut dest_len_sum = 0;
    let mut dest_lens = Vec::<usize>::new();
    let src = read_file(SRC_NAME).unwrap();
    let src_len = src.len();
    for i in 0..THREAD_NUM {
        let dest_name = format!("{DEST_NAME_PREFIX}_{i}.txt");
        let dest = read_file(dest_name.as_str()).unwrap();
        dest_lens.push(dest.len());
        dest_len_sum += dest.len();
    }
    if src_len == dest_len_sum {
        println!("File copied successfully");
    } else {
        println!(
            "File copy failed : src_len: {}, dest_len_sum: {}",
            src_len, dest_len_sum
        );
        for i in 0..THREAD_NUM {
            println!("dest[{}]: {}", i, dest_lens[i]);
        }
        println!("File difference: {}B", src_len - dest_len_sum);
    }

    remove_file(src_path).expect("Failed to remove file: SRC_NAME");
    for i in 0..THREAD_NUM {
        let dest_name = format!("{DEST_NAME_PREFIX}_{i}.txt");
        let dest_path = Path::new(dest_name.as_str());
        remove_file(dest_path).expect(format!("Failed to remove file: {dest_name}").as_str());
    }
}
