use std::time::Instant;

use crate::mdma::MdmaIndex;

// TODO: Use a range based MatchGen
pub fn generate(mdma_index: &MdmaIndex, matches: &mut Vec<Match>) {
    let lcp_array = build_lcp_array(&mdma_index.sa, &mdma_index.buf);
    let timer = Instant::now();
    let mut stack: Vec<MatchGen> = Vec::with_capacity(256);

    for index in 0..lcp_array.len() {
        // TODO: Use u16 for the lz matches
        let lcp = lcp_array[index];
        let lcp = if lcp > u8::MAX as i32 { u8::MAX } else { lcp as u8 };

        // Push new matches
        if lcp > stack.last().map_or(1, |m| m.len) {
            stack.push(MatchGen::new(index, lcp));
        }

        // Pop old matches
        while matches!(stack.last(), Some(m) if lcp < m.len) {
            let mut min_len = stack.get(stack.len() - 2).map_or(2, |m| m.len + 1);
            let mx = Match::new(index, stack.last().unwrap());

            if lcp >= min_len { stack.last_mut().unwrap().len = lcp; min_len = lcp + 1; }
            else              { stack.pop().unwrap(); }

            for len in min_len..=mx.len { matches.push(Match::with_len(&mx, len)); }
        }
    }

    assert!(stack.is_empty());
    println!("Generated {} matches in: {:?}", matches.len(), timer.elapsed());
}

pub fn build_lcp_array(sa: &Vec<i32>, buf: &Vec<u8>) -> Vec<i32> {
    let timer = Instant::now();
    let n = sa.len();
    let mut lcp = vec![0; n];

    let mut sa_inv = vec![0; n];
    for i in 0..n {
        sa_inv[sa[i] as usize] = i;
    }

    let mut k = 0;
    for i in 0..n {
        if sa_inv[i] == n - 1 {
            k = 0;
            continue;
        }

        let j = sa[sa_inv[i] + 1] as usize;
        loop {
            if i+k >= n || j+k >= n {
                break;
            }
            if buf[i+k] != buf[j+k] {
                break;
            }
            k += 1;
        }

        lcp[sa_inv[i]] = k as i32;
        if k > 0 {
            k -= 1;
        }
    }

    println!("Built LCP in {:?}", timer.elapsed());
    return lcp;
}

// TODO: Maybe find a way to align this?
// [ ] Check how big the stack can get
// [ ] Check hwo big the stack can get with len < 256
// [ ] See if we can group multiple matches together?
// Elaborating on that last one: When we have matches with the same count and same sa_index, but different lenghts,
// maybe we don't need to rank all of them (just the longest one?)
// Even if we can't get around ranking all of them, perhaps we can at least store them in stack more efficiently?
// ---------------------------
// MatchGen is a more lightweight struct that only holds the len and sa_index
// It's used to  minimize the allocations of the match_finder
// Since we always compute the sa_count field, we can not store it and save a couple of bytes
// NOTE: range of len is [2-], but range of u8 is [0-255], so we *could* offset the len by 2
// while storing in match to utilize a bigger range [2-257] if we needed to
// (but I think the bigger range can't compensate for the extra complexity per access)
// - same with sa_count

// #[repr(packed(1))]
struct MatchGen {
    sa_index: u32,
    len: u8
}

impl MatchGen {
    fn new(sa_index: usize, len: u8) -> Self { Self { sa_index: sa_index as u32, len } }
}

#[derive(Clone)]
pub struct Match {
    pub self_ref: bool,
    pub sa_index: u32,
    pub sa_count: u32,
    pub len:      u8
}

impl Match {
    fn new(index: usize, mg: &MatchGen) -> Self {
        Self { self_ref: true, sa_index: mg.sa_index, sa_count: index as u32 - mg.sa_index + 1, len: mg.len }
    }

    fn with_len(m: &Match, len: u8) -> Self {
        let mut clone = m.clone();
        clone.len = len;
        return clone;
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        return self.sa_index as usize .. (self.sa_index + self.sa_count) as usize;
    }
}

pub fn _static_analyze(mdma_index: &MdmaIndex) {
    let lcp_array = build_lcp_array(&mdma_index.sa, &mdma_index.buf);
    let mut stack: Vec<MatchGen> = Vec::with_capacity(256);
    let mut max_sa_count = 0;
    let mut max_len = 0;
    let mut count = 0;
    let mut counts = [0; 6];

    for index in 0..lcp_array.len() {
        let lcp = lcp_array[index];
        if  lcp > max_len { max_len = lcp; }
        let lcp = if lcp > u8::MAX as i32 { u8::MAX } else { lcp as u8 };

        // Push new matches
        if lcp > stack.last().map_or(1, |m| m.len) {
            stack.push(MatchGen::new(index, lcp));
        }

        // Pop old matches
        while matches!(stack.last(), Some(m) if lcp < m.len) {
            let mut min_len = stack.get(stack.len() - 2).map_or(2, |m| m.len + 1);
            let mx = Match::new(index, stack.last().unwrap());

            if lcp >= min_len { stack.last_mut().unwrap().len = lcp; min_len = lcp + 1; }
            else              { stack.pop().unwrap(); }

            // Stats
            for len in min_len..8 {
                counts[(len - 2) as usize] += 1;
            }
            count += (mx.len - min_len + 1) as i32;
            if mx.sa_count > max_sa_count { max_sa_count = mx.sa_count; }
        }
    }

    assert!(stack.is_empty());
    dbg!(count);
    dbg!(max_sa_count);
    dbg!(max_len);
    for i in 0..6 {
        println!("counts for len={} -> {}", i+2, counts[i]);
    }
    let sum: i32 = counts.iter().sum();
    println!("counts for len>7 -> {}", count - sum);
    std::process::exit(1);
}
