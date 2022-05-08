use crate::counting::count;
use crate::mdma::{MdmaIndex, Word};
use crate::match_finder::Match;

// TODO: Add get_entropy?

pub fn rank(m: &mut Match, mdma_index: &mut MdmaIndex) -> Option<Word> {
    // From match_finder we know len >= 2 and sa_count >= 2 (if m is valid)
    let (count, loc) = count(m, mdma_index);
    if count < 2 {
        m.is_valid = false;
        return None;
    }

    let len = m.len;
    let slice = &mdma_index.buf[loc..(loc + len as usize)];
    for &sym in slice {
        mdma_index.sym_counts[sym as usize] += 1f64;
    }

    let mut rank = 0f64;
    let count_prec = f64::from(count);
    let n_prec = f64::from(mdma_index.n);
    let len_prec = f64::from(len);
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

    if rank <= 0f64 || rank.is_nan() {
        m.is_valid = false;
        return None;
    }

    Some(Word {
        location: loc, len,
        sa_index: m.sa_index, sa_count: m.sa_count,
        count, rank
    })
}

pub fn update_model(word: &Word, mdma_index: &mut MdmaIndex) {
    let count = f64::from(word.count);
    let slice = &mdma_index.buf[word.get_range()];
    for &sym in slice {
        mdma_index.model[usize::from(sym)] -= count;
    }

    mdma_index.n -= word.count * (u32::from(word.len) - 1);
}
