mod mdma;
mod bindings;
mod bit_operations;
mod file_operations;
mod match_finder;
mod entropy_ranking;

fn main() -> std::io::Result<()> {
    let file_name = "../rrans/data/book1";
    println!("Building dict for: {}", file_name);
    let buf = file_operations::read_file_into_buffer(file_name)?;
    let _dict = mdma::build_dictionary(&buf);
    let _word = &_dict[0];
    dbg!(_dict.len());
    dbg!(_word.location);
    dbg!(_word.len);

    return Ok(());
}
