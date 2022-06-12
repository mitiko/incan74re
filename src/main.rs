use std::path::PathBuf;
use std::{time::Instant, fs};

mod incan74re;
mod bindings;
mod match_finder;
mod entropy_ranking;
mod counting;
mod splitting;
mod parser;

fn main() {
    let file = PathBuf::from("/data/calgary/book1");
    let file_name = file.file_name().expect("Couldn't deduce filename").to_os_string();
    let file_name =  file_name.to_str().expect("Invalid utf8 filename");

    println!("Building dict for: {:?}", file_name);
    let buf = fs::read(file).expect("Couldn't read file into memory");
    let mut index = incan74re::initialize(buf);
    let timer = Instant::now();
    let dict = incan74re::build_dictionary(&mut index);
    println!("Building dict took: {:?}", timer.elapsed());

    // TODO: Move encode dict and decode dict to a new file
    let bits_per_token = (dict.len() as f64).log2().ceil() as u32;
    println!("Bits per token: {bits_per_token}");
    parser::encode_dict(&dict, &index, &format!("dict-{}.bin", file_name));
    parser::parse(&dict, &mut index, &format!("p-{}-{bits_per_token}.bin", file_name));

    dbg!(dict.len());
    if !dict.is_empty() {
        dbg!(dict[0].location);
        dbg!(dict[0].len);
    }
}
