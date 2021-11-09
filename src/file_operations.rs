// use std::fs;
use std::fs::File;
use std::io::prelude::*;
// use std::io::BufWriter;

pub fn read_file_into_buffer(input_file_name: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(input_file_name)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    return Ok(buf);
}

// pub fn get_writer(output_file_name: &str) -> std::io::Result<BufWriter<File>> {
//     let _ = fs::remove_file(output_file_name);
//     let file = File::create(output_file_name)?;

//     return Ok(BufWriter::with_capacity(1 << 16, file)); // 64 KiB buffer
// }
