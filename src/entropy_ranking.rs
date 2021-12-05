use crate::counting::count;
use crate::mdma::{MdmaIndex, Word};
use crate::match_finder::Match;

// TODO: Add get_entropy?

pub fn rank(m: &mut Match, mdma_index: &mut MdmaIndex) -> Option<Word> {
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

    rank -= 8f64 * (len_prec + 1f64); // Dictionary overhead
    rank += count_prec * count_prec.log2();
    rank -= n1 * n1.log2();
    rank += n_prec * n_prec.log2();

    match rank > 0f64 {
        true => Some(Word {
            location: loc as u32, len: len as i32,
            sa_index: m.sa_index, sa_count: m.sa_count,
            count, rank
        }),
        false => None
    }
}

pub fn update_model(word: &Word, mdma_index: &mut MdmaIndex) {
    let slice = &mdma_index.buf[word.get_range()];
    let count = word.count as f64;
    for &sym in slice {
        mdma_index.model[sym as usize] -= count;
    }

    mdma_index.n -= word.count * (word.len - 1);
}
