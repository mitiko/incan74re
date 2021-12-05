use std::time::Instant;

mod mdma;
mod bindings;
mod file_operations;
mod match_finder;
mod entropy_ranking;
mod counting;
mod splitting;
mod parser;

fn main() -> std::io::Result<()> {
    let file_name = "/data/enwik7";
    println!("Building dict for: {}", file_name);
    let timer = Instant::now();
    let buf = file_operations::read_file_into_buffer(file_name)?;
    let mut index = mdma::initialize(buf);
    let dict = mdma::build_dictionary(&mut index);
    println!("Dict took: {:?}", timer.elapsed());
    // parser::encode_dict(&dict, &index, "dict.bin");
    // TODO: Encode output as u16? (parsing)
    let first_word = &dict[0];
    dbg!(dict.len());
    dbg!(first_word.location);
    dbg!(first_word.len);

    return Ok(());
}
