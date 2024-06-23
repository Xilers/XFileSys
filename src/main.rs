mod file_io;
mod threadpool;

use file_io::{copy_part, create_file, read_file};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time;

/// @@ TODO:
///  - Add/Impl threadpool for preallocating threads
///
fn main() {
    benchmark_file_io_perf();
}

fn benchmark_file_io_perf() {
    const SRC_NAME: &str = "large_file_src.txt";
    const DEST_NAME: &str = "large_file_destination.txt";
    const SRC_SIZE: u64 = 1024 * 1024 * 1024 * 4; // 4GB (in bytes)
    const THREAD_NUM: usize = 1;
    let threadpool = threadpool::ThreadPool::new(THREAD_NUM);

    println!(
        "Writing file of size {} MB with {} Threads...",
        SRC_SIZE / (1024 * 1024),
        THREAD_NUM
    );

    let _ = create_file(SRC_NAME, SRC_SIZE);
    let current_time = time::Instant::now();

    let src_path = Path::new(SRC_NAME);
    let dest_path = Path::new(DEST_NAME);

    let metadata = std::fs::metadata(&src_path).unwrap();
    let file_size = metadata.len();

    let dest_file = File::create(&dest_path).unwrap();
    let dest_file = Arc::new(Mutex::new(dest_file));

    for i in 0..THREAD_NUM {
        let offset = SRC_SIZE / THREAD_NUM as u64;
        let dest_file_clone = Arc::clone(&dest_file);
        let start = i as u64 * offset;
        let length = if i == THREAD_NUM - 1 {
            file_size - start as u64
        } else {
            offset
        };
        threadpool.execute(move || {
            copy_part(&src_path, dest_file_clone, start, length).expect("Failed to copy part");
        });
    }
    drop(threadpool);
    println!("Elapsed time: {} msec", current_time.elapsed().as_millis());

    // let src = read_file(SRC_NAME).unwrap();
    // let dest = read_file(DEST_NAME).unwrap();
    // println!("src length: {} B\ndest length: {} B", src.len(), dest.len());
}
