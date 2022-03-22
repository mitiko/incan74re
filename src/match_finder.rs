use std::time::Instant;

use crate::mdma::MdmaIndex;

pub fn generate(mdma_index: &MdmaIndex, matches: &mut Vec<Match>) {
    let lcp_array = build_lcp_array(&mdma_index.sa, &mdma_index.buf);
    let timer = Instant::now();
    let mut stack: Vec<MatchGen> = Vec::with_capacity(256);

    for (index, lcp) in lcp_array.into_iter().enumerate() {
        let lcp = u16::try_from(lcp).unwrap_or(u16::MAX);

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

// TODO: Prefetch
// Casts here are safe just unproven because libsais uses i32-s for the SA
pub fn build_lcp_array(sa: &[i32], buf: &[u8]) -> Vec<u32> {
    let timer = Instant::now();
    let mut lcp = vec![0; sa.len()];
    let mut sa_inv = vec![0; sa.len()];

    for (i, &sa_idx) in sa.iter().enumerate() {
        // prefetch here
        sa_inv[sa_idx as usize] = i;
    }

    let mut k: u32 = 0;
    let n = sa.len();
    for (i, &sa_inv_idx) in sa_inv.iter().enumerate() {
        if sa_inv_idx == n - 1 {
            k = 0;
            continue;
        }

        // prefetch here
        let j = sa[sa_inv_idx + 1] as usize;
        loop {
            match (buf.get(i+k as usize), buf.get(j+k as usize)) {
                (Some(loc1), Some(loc2)) if loc1 != loc2 => break,
                (None, _) | (_, None) => break,
                _ => k += 1
            };
        }

        // prefetch here
        lcp[sa_inv_idx] = k;
        k = k.saturating_sub(1);
    }

    println!("Built LCP in {:?}", timer.elapsed());
    lcp
}

// MatchGen is a more lightweight struct that only holds the len and sa_index
// It's used to  minimize the allocations of the match_finder
// Since we always compute the sa_count field, we can not store it and save a couple of bytes

// #[repr(packed(1))]
struct MatchGen {
    sa_index: u32,
    len: u16
}

// Cast is safe because SA.len() < u32::MAX
impl MatchGen {
    fn new(sa_index: usize, len: u16) -> Self { Self { sa_index: sa_index as u32, len } }
}

#[derive(Clone)]
pub struct Match {
    pub self_ref: bool,
    pub is_valid: bool,
    pub sa_index: u32,
    pub sa_count: u32,
    pub len:      u16
}

// Cast is safe because SA.len() < u32::MAX
impl Match {
    fn new(index: usize, mg: &MatchGen) -> Self {
        Self { self_ref: true, sa_index: mg.sa_index, sa_count: index as u32 - mg.sa_index + 1, len: mg.len, is_valid: true }
    }

    fn with_len(m: &Match, len: u16) -> Self {
        let mut clone = m.clone();
        clone.len = len;
        clone
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        self.sa_index as usize .. (self.sa_index + self.sa_count) as usize
    }
}

pub fn _static_analyze(mdma_index: &MdmaIndex) {
    let lcp_array = build_lcp_array(&mdma_index.sa, &mdma_index.buf);
    let mut stack: Vec<MatchGen> = Vec::with_capacity(256);
    let mut max_sa_count = 0;
    let mut max_len = 0;
    let mut total_count = 0;
    let mut counts = [0; 6];

    for (index, lcp) in lcp_array.into_iter().enumerate() {
        if lcp > max_len { max_len = lcp; }
        let lcp = u16::try_from(lcp).unwrap_or(u16::MAX);

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
            total_count += i32::from(mx.len - min_len + 1);
            if mx.sa_count > max_sa_count { max_sa_count = mx.sa_count; }
        }
    }

    assert!(stack.is_empty());
    dbg!(total_count);
    dbg!(max_sa_count);
    dbg!(max_len);
    for (i, count) in counts.iter().enumerate() {
        println!("counts for len={} -> {count}", i+2);
    }
    let sum: i32 = counts.iter().sum();
    println!("counts for len>7 -> {}", total_count - sum);
    std::process::exit(1);
}
