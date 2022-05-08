use std::fs::File;
use std::io::{Write, BufWriter};
use std::ops::Neg;

use crate::mdma::{MdmaIndex, Word};

// The format for the dictionary (of size n) (currently) is:
// 2 bytes for dictionary.len() to encode n
// n words with 2+word.len() bytes -> 2 bytes for len and x bytes for the word
// The order of the words in the dictionary is not restrictive and can be changed when further compressing the dict

// TODO: Propagate io errors up
pub fn encode_dict(dict: &[Word], mdma_index: &MdmaIndex, file_name: &str) {
    let mut writer = BufWriter::new(File::create(file_name).unwrap());
    writer.write_all(&u32::try_from(dict.len()).unwrap().to_be_bytes()).unwrap();

    dict.iter()
        .map(|word| {
            let mut data = vec![0u8; word.len as usize + 2];
            data[..2].copy_from_slice(&(word.len as u16).to_be_bytes());
            data[2..].copy_from_slice(&mdma_index.buf[word.get_range()]);

            data
        })
        .for_each(|word_data| writer.write_all(&word_data).unwrap());

    writer.flush().unwrap();
}

// Creates a u16 array (big-endian order) of word indexes and writes it to a file
// Uses the offsets array from the dictionary computing phase for O(n) parsing
// Indexes in the range [0 .. 255] are leftover uncovered raw literals
// Indexes in the range [256 .. 256 + dict.len()] are dictionary words
// Words can be decoded as dict[index-256], while literals as (index as u8)

// Offset array mapping to words, mapping to parsed u16:
// -0     -> lit 0x00    -> 0
// -1     -> lit 0x01    -> 1
// -255   -> lit 0xff    -> 255
// -256   -> dict[0]     -> 256
// -257   -> dict[1]     -> 257
// -65535 -> dict[65279] -> 65535 (u16::MAX)
pub fn parse(dict: &[Word], mdma_index: &mut MdmaIndex, file_name: &str) {
    let mut writer = BufWriter::new(File::create(file_name).unwrap());

    // Cover with raw literals
    for (loc, x) in &mut mdma_index.offsets.iter_mut().enumerate() {
        if *x >= 0 { *x = i32::from(mdma_index.buf[loc]).neg(); }
    }

    let mut idx = 0;
    while idx < mdma_index.offsets.len() {
        let token = mdma_index.offsets[idx].neg() as usize;

        writer.write_all(&(token as u16).to_be_bytes()).unwrap();

        idx += if token >= 256 { usize::from(dict[token-256].len) } else { 1 };
    }

    writer.flush().unwrap();
}
