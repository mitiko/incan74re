use crate::counting::count;
use crate::mdma::MdmaIndex;
use crate::match_finder::Match;
use crate::mdma::Word;

// TODO: Add get_entropy?

pub fn rank(m: &mut Match, mdma_index: &mut MdmaIndex) -> Option<RankedWord> {
    // From match_finder we know len >= 2 and sa_count >= 2
    let len = m.len as usize;
    let (count, loc) = count(m, mdma_index);
    if count < 2 { return None; }

    let slice = &mdma_index.buf[loc..(loc + len)];
    for &sym in slice {
        mdma_index.sym_counts[sym as usize] += 1f64;
    }

    let mut rank = 0f64;
    let count_prec = count as f64;
    let n_prec = mdma_index.n as f64;
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

pub fn update_model(ranked_word: &RankedWord, mdma_index: &mut MdmaIndex) {
    let slice = &mdma_index.buf[ranked_word.word.get_range()];
    let count = ranked_word.count as f64;
    for &sym in slice {
        mdma_index.model[sym as usize] -= count;
    }

    mdma_index.n -= ranked_word.count * (ranked_word.word.len - 1) as i32;
}

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
