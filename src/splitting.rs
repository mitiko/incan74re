use crate::mdma::{MdmaIndex, Word};

// TODO: Align the offsets array to the suffix array using the inverseSA and lower L3 cache misses on ranking
// This is a crucial loop, even tho it gets executed only once per iteration, it's O(n)
// Note that we get a small speedup in doing parsing and offsets reseting together because
// 1) It's only O(n), we're not doing 2 passes
// 2) We're doing them in different directions -> gives us an exra speedup
pub fn split(word: &Word, mdma_index: &mut MdmaIndex) {
    // Find word from SA
    let mut locations = vec![0; word.sa_count as usize];
    locations.copy_from_slice(&mdma_index.sa[word.get_sa_range()]);
    locations.sort_unstable();

    let effective_len = i32::from(word.len) - 1;
    let word_len      = usize::from(word.len);
    let rt = mdma_index.replacement_token; // used for parsing later
    mdma_index.replacement_token -= 1;

    // Parse this word
    for loc in locations {
        let loc = loc as usize;
        if mdma_index.offsets[loc] < effective_len { continue; }

        // Replace locations of the word with a token for parsing
        mdma_index.offsets[loc..(loc + word_len)]
            .iter_mut()
            .rev()
            .for_each(|x| *x = rt);

        // TODO: Unroll?
        // Calculate offsets, traversing the vec backwards
        for (last, offset) in mdma_index.offsets[..loc].iter_mut().rev().enumerate() {
            if *offset < 0 { break; }
            *offset = last.try_into().unwrap();
        }
    }
}
