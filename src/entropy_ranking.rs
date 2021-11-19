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
        mdma_index.sym_counts[sym as usize] += 1f64;
    }

    let mut rank = 0f64;
    let count_prec = count as f64;
    let n_prec = *mdma_index.n as f64;
    let len_prec = len as f64;
    let n1 = n_prec - count_prec * (len_prec - 1f64);

    for sym in slice {
        let sym_index = *sym as usize;
        let sym_count = mdma_index.sym_counts[sym_index];
        // TODO: Try to find a branchless solution to this?
        // Maybe zero out some stuff, would that be faster? (cx:=0, cxw:=0 would make rank_d=0)
        // but log2(0.0) = NaN and 0*NaN -> NaN
        if sym_count == 0f64 { continue; }
        mdma_index.sym_counts[sym_index] = 0f64;

        let cx = mdma_index.model[sym_index];
        let cxw = cx - sym_count * count_prec;
        rank += cxw * cxw.log2() - cx * cx.log2();
    }

    rank -= (8 * (len + 1)) as f64; // Dictionary overhead
    rank += count_prec * count_prec.log2();
    rank -= n1 * n1.log2();
    rank += n_prec * n_prec.log2();

    match rank > 0f64 {
        true => Some(RankedWord {
            word: Word { location: loc, len },
            count, rank
        }),
        false => None
    }
}

// Must guarantee location is an unused location
// Does it tho? -> It does not, indeed
fn count(m: &mut Match, mdma_index: &MdmaIndex) -> (i32, usize) {
    match m.self_ref {
        false => count_fast(m, mdma_index),
        true => count_slow(m, mdma_index)
    }
}

fn count_fast(m: &mut Match, mdma_index: &MdmaIndex) -> (i32, usize) {
    let effective_len = m.get_len() - 1;
    let range = m.get_range();
    let mut count = 0;

    let last_match = mdma_index.sa[range.start] as usize;
    for loc in &mdma_index.sa[range] {
        count += (mdma_index.spots[*loc as usize] >= effective_len) as i32;
    }

    (count, last_match)
}

fn count_slow(m: &mut Match, mdma_index: &MdmaIndex) -> (i32, usize) {
    let range = m.get_range();
    let mut locations = vec![0; range.len()];
    locations.copy_from_slice(&mdma_index.sa[range]);
    locations.sort_unstable();

    let effective_len = m.get_len() - 1;
    let mut count = 0;
    let mut flag = false;
    let mut last_match = - m.get_len();

    for loc in locations {
        // TODO: Optimize branching?
        if loc <= last_match + effective_len { flag = true; continue; }
        let a = mdma_index.spots[loc as usize];

        if a >= effective_len {
            count += 1;
            last_match = loc;
        }
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
    let len = best_match.get_len();
    let effective_len = len - 1;
    let mut last_match = - len;

    // Parse
    for loc in locations {
        if loc <= last_match + effective_len { continue; }

        let range = loc as usize ..= (loc + effective_len) as usize;
        let a = mdma_index.spots[loc as usize];

        if a >= effective_len {
            last_match = loc;
            for i in &mut mdma_index.spots[range] { *i = -1; }
        }
    }
}

pub fn split(best_match: &Match, mdma_index: &mut MdmaIndex) {
    parse(best_match, mdma_index);

    let mut last = -1;
    // TODO: Align the spots array to the suffix array using the inverseSA and lower L3 cache misses on ranking
    // TODO: We can optimize this directly in the parsing as well!
    // TODO: Unroll this?
    // This is a crucial loop, even tho it gets executed only once per iteration, it's O(n)
    for elem in mdma_index.spots.iter_mut().rev() {
        let eq = (*elem != -1) as i32;
        last = -1 + eq * (last + 2);
        *elem = last;
    }
}

pub fn update_model(ranked_word: &RankedWord, mdma_index: &mut MdmaIndex) {
    let slice = &mdma_index.buf[ranked_word.word.get_range()];
    let count = ranked_word.count as f64;
    for &sym in slice {
        mdma_index.model[sym as usize] -= count;
    }

    *mdma_index.n -= ranked_word.count * (ranked_word.word.len - 1) as i32;
}
