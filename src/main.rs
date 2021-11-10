mod mdma;
mod bindings;
mod bit_operations;
mod file_operations;
mod match_finder;
mod entropy_ranking;

fn main() -> std::io::Result<()> {
    let buf = file_operations::read_file_into_buffer("../rrans/data/test1_demo")?;
    let _dict = mdma::build_dictionary(&buf);
    let _word = &_dict[0];
    dbg!(_word.location);
    dbg!(_word.len);

    return Ok(());
}
