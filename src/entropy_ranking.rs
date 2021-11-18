use crate::fast_log2;

use super::mdma::MdmaIndex;
use super::match_finder::Match;
use super::mdma::Word;

pub struct RankedWord {
    pub word: Word,
    pub count: i32,
    pub rank: f64
}

impl RankedWord {
    pub fn _print(&self) {
        println!("word -> ({}, {}); c={}, r={}", self.word.location, self.word.len, self.count, self.rank);
    }

    pub fn empty() -> Self {
        Self {
            word: Word { location: 0, len: 0 },
            count: -1, rank: 0f64
        }
    }
}

impl Clone for RankedWord {
    fn clone(&self) -> Self {
        Self { word: self.word.clone(), count: self.count, rank: self.rank }
    }
}

// TODO: Add get_entropy?

pub fn rank(m: &mut Match, mdma_index: &mut MdmaIndex<'_>) -> Option<RankedWord> {
    // From match_finder we know len >= 2 and sa_count >= 2
    let len = m.get_len() as usize;
    let (count, loc) = count(m, mdma_index);
    if count < 2 { return None; }

    let slice = &mdma_index.buf[loc..(loc + len)];
    for &sym in slice {
        mdma_index.sym_counts[sym as usize] += 1;
    }

    let mut rank = 0f64;
    let n = *mdma_index.n;
    let n1 = n - count * (len as i32 - 1);

    let mut rank_a = 0.0;
    let mut rank_b = 0.0;
    // Unrolled loop
    for i in 0..(slice.len()/2) {
        let idx = i << 1;
        let sym_index_a = slice[idx] as usize;
        let sym_index_b = slice[idx+1] as usize;

        let cx_a = mdma_index.model[sym_index_a];
        let cx_b = mdma_index.model[sym_index_b];
        let cxw_a = cx_a - mdma_index.sym_counts[sym_index_a] * count;
        let cxw_b = cx_b - mdma_index.sym_counts[sym_index_b] * count;

        // rank_a += (cxw_a * cxw_a.log2()) - (cx_a * cx_a.log2());
        // rank_b += (cxw_b * cxw_b.log2()) - (cx_b * cx_b.log2());
        let res = fast_log2::entropy(cx_a, cxw_a, cx_b, cxw_b, mdma_index.g_log2);
        rank_a += res[1] - res[0];
        rank_b += res[3] - res[2];
        mdma_index.sym_counts[sym_index_a] = 0;
        mdma_index.sym_counts[sym_index_b] = 0;
    }
    rank += rank_a + rank_b;

    if slice.len() & 1 != 0 {
        let sym_index = slice[slice.len() - 1] as usize;

        let cx = mdma_index.model[sym_index];
        let cxw = cx - mdma_index.sym_counts[sym_index] * count;

        // rank += cxw * cxw.log2() - cx * cx.log2();
        let res = fast_log2::entropy(cx, cxw, 0, 0, mdma_index.g_log2);
        rank += res[1] - res[0];
        mdma_index.sym_counts[sym_index] = 0;
    }

    // let a = count * count.log2();
    // let b = n * n.log2();
    // let c = n1 * n1.log2();
    let res = fast_log2::entropy(count, n, n1, 0, mdma_index.g_log2);
    let a = res[0] + res[1];
    let b = (8 * (len + 1)) as f64; /* dictionary overhead */
    let c = res[2] + b;
    rank += a - c;

    match rank > 0f64 {
        true => Some(RankedWord {
            word: Word { location: loc, len },
            count, rank
        }),
        false => None
    }
}

// Must guarantee location is an unused location
fn count(m: &mut Match, mdma_index: &MdmaIndex) -> (i32, usize) {
    match m.self_ref {
        false => count_fast(m, mdma_index),
        true => count_slow(m, mdma_index)
    }
}

fn count_fast(m: &mut Match, mdma_index: &MdmaIndex) -> (i32, usize) {
    let range = m.get_range();
    let len = m.get_len() as i32;
    let mut count = 0;
    let mut last_match = - len;

    for loc in &mdma_index.sa[range] {
        let a = mdma_index.spots[*loc as usize];
        let b = mdma_index.spots[(*loc + len - 1) as usize];

        // Branchless counting
        // TODO: Perhaps setting holes as different negative numbers would speed up this?
        let condition = (a == b && a != -1) as i32;
        count += condition;
        last_match = last_match + condition * (*loc - last_match);
    }

    (count, last_match as usize)
}

fn count_slow(m: &mut Match, mdma_index: &MdmaIndex) -> (i32, usize) {
    let range = m.get_range();
    let mut locations = vec![0; range.len()];
    locations.copy_from_slice(&mdma_index.sa[range]);
    locations.sort_unstable();

    let len = m.get_len() as i32;
    let mut count = 0;
    let mut flag = false;
    let mut last_match = - len;

    for loc in locations {
        if loc < last_match + len { flag = true; continue; }

        let a = mdma_index.spots[loc as usize];
        let b = mdma_index.spots[(loc + len - 1) as usize];
        let condition = (a == b && a != -1) as i32;

        count += condition;
        last_match = last_match + condition * (loc - last_match);
    }

    m.self_ref = flag;
    (count, last_match as usize)
}

pub fn parse(best_match: &Match, mdma_index: &mut MdmaIndex) {
    // Find word from SA
    let sa_range = best_match.get_range();
    let mut locations = vec![0; sa_range.len()];
    locations.copy_from_slice(&mdma_index.sa[sa_range]);
    locations.sort_unstable();

    // Initialize parsing variables
    let len = best_match.get_len() as i32;
    let mut last_match = - len;

    // Parse
    for loc in locations {
        if loc < last_match + len { continue; }

        let range = loc as usize .. (loc + len) as usize;
        let a = mdma_index.spots[range.start];
        let b = mdma_index.spots[range.end-1];

        if a == b && a != -1 {
            last_match = loc;
            for i in &mut mdma_index.spots[range] { *i = -1; }
        }
    }
}

pub fn split(best_match: &Match, mdma_index: &mut MdmaIndex) {
    parse(best_match, mdma_index);

    // Compute spots vector branchless
    let mut spot = 0;
    let mut last = -1;
    // TODO: Align the spots array to the suffix array using the inverseSA and lower L3 cache misses on ranking
    for i in &mut mdma_index.spots[..] {
        let i_eq = (*i != -1) as i32;
        let last_eq = (last == -1) as i32;

        // Stays -1, or either becomes same spot as last, or new spot
        *i = -1 + i_eq * (last + last_eq * (spot - last) + 1);
        spot += i_eq * last_eq;

        last = *i;
    }
}

pub fn update_model(ranked_word: &RankedWord, mdma_index: &mut MdmaIndex) {
    let slice = &mdma_index.buf[ranked_word.word.get_range()];
    let count = ranked_word.count;
    for &sym in slice {
        mdma_index.model[sym as usize] -= count;
    }

    *mdma_index.n -= ranked_word.count * (ranked_word.word.len - 1) as i32;
}
