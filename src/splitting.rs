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

    // Initialize parsing variables
    let effective_len = word.len - 1;
    let mut last_match = 0 - word.len;

    // Parse
    for loc in locations {
        if loc <= last_match + effective_len { continue; }
        let range = loc as usize ..= (loc + effective_len) as usize;

        if mdma_index.offsets[loc as usize] >= effective_len {
            last_match = loc;
            for x in range.rev() { mdma_index.offsets[x] = -1; }

            // TODO: Manually unroll?
            let mut idx = loc as usize;
            let mut last = -1;
            while idx > 0 {
                idx -= 1; last += 1;
                if mdma_index.offsets[idx] == -1 { break; }
                mdma_index.offsets[idx] = last;
            }
        }
    }
}