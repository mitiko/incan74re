use std::ops::Neg;
use crate::match_finder::Match;
use crate::incan74re::DictIndex;

pub fn count(m: &mut Match, dict_index: &DictIndex) -> (u32, usize) {
    if m.self_ref { count_slow(m, dict_index) }
    else          { count_fast(m, dict_index) }
}

// Casts here are safe just unproven because libsais uses i32-s for the SA
fn count_fast(m: &mut Match, dict_index: &DictIndex) -> (u32, usize) {
    let mut count = 0;
    let effective_len = i32::from(m.len) - 1;

    let last_match = dict_index.sa[m.sa_index as usize] as usize;
    let range = m.get_range();

    // TODO: Try unroll?
    // TODO: Prefetch?
    for &loc in dict_index.sa[range].iter() {
        if dict_index.offsets[loc as usize] >= effective_len { count += 1; }
    }

    (count, last_match)
}

// Casts here are safe just unproven because libsais uses i32-s for the SA
fn count_slow(m: &mut Match, dict_index: &DictIndex) -> (u32, usize) {
    let range = m.get_range();
    let mut locations = vec![0; range.len()];
    locations.copy_from_slice(&dict_index.sa[range]);
    locations.sort_unstable();

    let effective_len = i32::from(m.len) - 1;
    let mut count = 0;
    let mut flag = false;
    let mut last_match = i32::from(m.len).neg(); // 0-len

    for loc in locations {
        // TODO: Optimize branching? -> there're no branches in the loop,
        // but the compiler can't (won't) unroll because of the dependency on last_match
        // It's not clear how to unroll either, there's a bunch of ways matches may intertwine
        // if the outer ends of 2 matches are far apart enough to fit 2 matches we can consider 2 matches
        // but perhaps the branch predictor is fine as it is and can even speculatively prefetch offsets[loc]
        if loc <= last_match + effective_len { flag = true; continue; }

        if dict_index.offsets[loc as usize] >= effective_len {
            count += 1;
            last_match = loc;
        }
    }

    m.self_ref = flag;
    (count, last_match.try_into().unwrap_or(usize::MAX))
}
