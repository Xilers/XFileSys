use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Copies a part of a file from the source path to the destination file.
///
/// # Arguments
///
/// * `src_path` - The path to the source file.
/// * `dest_file` - The destination file wrapped in an `Arc<Mutex<File>>`.
/// * `start` - The starting position in the source file bytes to copy from.
/// * `length` - The length of the part in bytes to copy.
///
/// # Errors
///
/// Returns an `std::io::Result` indicating the success or failure of the operation.
///
pub fn copy_part(
    src_path: &Path,
    dest_file: Arc<Mutex<File>>,
    start: u64,
    length: u64,
) -> std::io::Result<()> {
    let mut src_file = File::open(src_path)?;
    src_file.seek(SeekFrom::Start(start))?;

    let mut buffer = vec![0; length as usize];
    src_file.read_exact(&mut buffer)?;

    let mut dest_file = dest_file.lock().unwrap();
    dest_file.seek(SeekFrom::Start(start))?;
    dest_file.write_all(&buffer)?;

    Ok(())
}

/// Creates a file with the specified name and size. written with 's'(1B)
///
/// # Arguments
///
/// * `filename` - The name of the file to create.
/// * `size` - The size of the file to create in bytes.
///
/// # Errors
///
/// Returns an `std::io::Result` indicating the success or failure of the operation.
///
pub fn create_file(filename: &str, size: u64) -> Result<(), Error> {
    let mut file = match File::create(filename) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    let dummy = vec![0x73; size as usize];

    let _ = match file.write_all(&dummy) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

/// Reads the contents of a file into a vector of bytes.
///
/// # Arguments
///
/// * `filename` - The name of the file to read.
///
/// # Errors
///
/// Returns an `std::io::Result` indicating the success or failure of the operation.
///
pub fn read_file(filename: &str) -> Result<Vec<u8>, Error> {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    let mut buffer: Vec<u8> = Vec::<u8>::new();

    let _ = match file.read_to_end(&mut buffer) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    return Ok(buffer);
}
