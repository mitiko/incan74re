use std::fs::File;
use std::io::{Write, BufWriter};

use crate::mdma::{MdmaIndex, Word};

const LITERAL_RANGE_START: usize = (u16::MAX - 256) as usize;

// The format for the dictionary (of size n) (currently) is:
// 2 bytes for dictionary.len() to encode n
// n words with 1+word.len() bytes -> 1 byte for len and x bytes for the word
// Note: The compressed version of the dicitonary may be sorted as the order doesn't matter for decoding
// The len of each word can thus be better predicted using a model of its own
// Sorting can also be done lexiographically
pub fn encode_dict(dict: &Vec<Word>, mdma_index: &MdmaIndex, file_name: &str) {
    let mut writer = BufWriter::new(File::create(file_name).unwrap());
    writer.write_all(&(dict.len() as u32).to_be_bytes()).unwrap();

    let buf: Vec<u8> = dict.iter()
        .map(|word| {
            let mut data = vec![0u8; word.len as usize + 1];
            data[1..].copy_from_slice(&mdma_index.buf[word.get_range()]);
            data[0] = word.len as u8;
            return data;
        })
        .flatten()
        .collect();

    writer.write_all(&buf).unwrap();
    writer.flush().unwrap();
}

// Creates a u16 array (big-endian order) of word indexes and writes it to a file
// Indexes in the range [0 .. dict.len()] are dictionary words
// Indexes in the range [u16::MAX-256 .. u16::MAX] are leftover uncovered raw literals
// TODO: Don't use top of the range as it's confusing
// Words can be decoded as dict[index], while literals as (index - (u16::MAX-256))
pub fn parse(dict: &Vec<Word>, mdma_index: &mut MdmaIndex, file_name: &str) {
    assert!(dict.len() < LITERAL_RANGE_START);
    let buf_capacity = 256;
    let mut writer = BufWriter::new(File::create(file_name).unwrap());
    let mut buf = Vec::with_capacity(buf_capacity);

    // Cover with raw literals
    for (loc, x) in &mut mdma_index.offsets.iter_mut().enumerate() {
        if *x >= 0 { *x = - (1 + LITERAL_RANGE_START as i32 + mdma_index.buf[loc] as i32); }
    }

    let mut idx = 0;
    while idx < mdma_index.offsets.len() {
        // Parsed words are 1 index based
        let word_idx = (-mdma_index.offsets[idx] - 1) as usize;
        // TODO: Use 10-bit, 12-bit, 14-bit?, 16-bit parsing
        // TODO: Update: Standardize to 16bits and let the entropy coder dismiss the first couple if needed
        buf.extend((word_idx as u16).to_be_bytes());

        if buf.len() >= buf_capacity {
            writer.write_all(&buf).unwrap();
            buf.clear();
        }

        idx += if word_idx < dict.len() { dict[word_idx].len as usize } else { 1 };
    }

    writer.write_all(&buf).unwrap();
    writer.flush().unwrap();
}