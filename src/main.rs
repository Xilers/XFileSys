mod file_io;
mod threadpool;

use file_io::{copy_part, create_file, read_file};
use std::fs::{remove_file, File};
use std::path::Path;
use std::time;

/// @@ TODO:
/// -
fn main() {
    benchmark_file_io_perf();
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
