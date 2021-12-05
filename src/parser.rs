use std::io::Write;

use crate::file_operations;
use crate::mdma::{MdmaIndex, Word};

// The format for the dictionary (of size n) (currently) is:
// 2 bytes for dictionary.len() to encode n
// n words with 1+word.len() bytes -> 1 byte for len and x bytes for the word
// Note: The compressed version of the dicitonary may be sorted as the order doesn't matter for decoding
// The len of each word can thus be better predicted using a model of its own
// Sorting can also be done lexiographically
pub fn encode_dict(dict: &Vec<Word>, mdma_index: &MdmaIndex, file_name: &str) {
    let mut writer = file_operations::get_writer(file_name).unwrap();
    writer.write_all(&(dict.len() as u32).to_be_bytes()).unwrap();

    let buf: Vec<u8> = dict.iter()
        .map(|word| {
            let mut data = vec![0u8; word.len + 1];
            data[1..].copy_from_slice(&mdma_index.buf[word.get_range()]);
            data[0] = word.len as u8;
            return data;
        })
        .flatten()
        .collect();

    writer.write_all(&buf).unwrap();
    writer.flush().unwrap();
}
