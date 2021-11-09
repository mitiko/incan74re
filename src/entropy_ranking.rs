use super::bwd::BwdIndex;
use super::match_finder::Match;
use super::bwd::Word;

pub struct RankedWord {
    pub word: Word,
    pub count: i32,
    pub rank: f64
}

impl RankedWord {
    pub fn print(&self) {
        println!("word -> ({}, {}); c={}, r={}", self.word.location, self.word.len, self.count, self.rank);
    }
}

impl Clone for RankedWord {
    fn clone(&self) -> Self {
        Self { word: self.word.clone(), count: self.count.clone(), rank: self.rank.clone() }
    }
}

// TODO: Add get_entropy

pub fn rank(m: &Match, bwd_index: &mut BwdIndex<'_>) -> Option<RankedWord> {
    // From match_finder we know len >= 2 and sa_count >= 2
    let len = m.len as usize;
    let (count, loc) = count(m, bwd_index);
    if count < 2 { return None; }

    let slice = &bwd_index.buf[loc..(loc + len)];
    for &sym in slice {
        bwd_index.sym_counts[sym as usize] += 1f64;
    }

    let mut rank = 0f64;
    let count_prec = count as f64;
    let n_prec = *bwd_index.n as f64;
    let len_prec = len as f64;
    let n1 = n_prec - count_prec * (len_prec - 1f64);

    for sym in slice {
        let sym_index = *sym as usize;
        let sym_count = bwd_index.sym_counts[sym_index];
        if sym_count == 0f64 { continue; }
        bwd_index.sym_counts[sym_index] = 0f64;

        let cx = bwd_index.model[sym_index];
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
fn count(m: &Match, bwd_index: &BwdIndex) -> (i32, usize) {
    let range = m.get_range();
    let mut locations = vec![0; range.len()];
    locations.copy_from_slice(&bwd_index.sa[range]);
    locations.sort_unstable();

    let len = m.len as i32;
    let mut count = 0;
    let mut last_match = - len;

    for loc in locations {
        if loc < last_match + len { continue; }

        let a = bwd_index.spots[loc as usize];
        let b = bwd_index.spots[(loc + len - 1) as usize];

        if a == b && a != -1 {
            last_match = loc;
            count += 1;
        }
    }

    return (count, last_match as usize);
}

pub fn split(best_match: &Match, bwd_index: &mut BwdIndex) {
    // Find word from SA
    let range = best_match.get_range();
    let mut locations = vec![0; range.len()];
    locations.copy_from_slice(&bwd_index.sa[range]);
    locations.sort_unstable();

    // Initialize parsing variables
    let len = best_match.len as i32;
    let mut parsed_locs = Vec::with_capacity(locations.len());
    let mut last_match = - len;

    // Parse
    for loc in locations {
        if loc < last_match + len { continue; }

        let a = bwd_index.spots[loc as usize];
        let b = bwd_index.spots[(loc + len - 1) as usize];

        if a == b && a != -1 {
            last_match = loc;
            parsed_locs.push(loc);
        }
    }

    // Slice by word
    for loc in parsed_locs {
        for i in 0..len {
            bwd_index.spots[(loc+i) as usize] = -1;
        }
    }

    // Compute spots vector
    let mut spot = 0;
    let mut last = -1;
    for i in 0..bwd_index.spots.len() {
        if bwd_index.spots[i] != -1 {
            if last == -1 {
                bwd_index.spots[i] = spot;
                spot += 1;
            }
            else {
                bwd_index.spots[i] = last;
            }
        }

        last = bwd_index.spots[i];
    }
}

pub fn update_model(ranked_word: &RankedWord, bwd_index: &mut BwdIndex) {
    let slice = &bwd_index.buf[ranked_word.word.get_range()];
    let count = ranked_word.count as f64;
    for &sym in slice {
        bwd_index.model[sym as usize] -= count;
    }

    *bwd_index.n -= ranked_word.count * (ranked_word.word.len - 1) as i32;
}
