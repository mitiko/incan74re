use std::time::Instant;

use crate::mdma::MdmaIndex;

// TODO: Use a range based MatchGen
pub fn generate(mdma_index: &MdmaIndex, matches: &mut Vec<Match>) {
    let lcp_array = build_lcp_array(&mdma_index.sa, &mdma_index.buf);
    let timer = Instant::now();
    let mut stack = Vec::with_capacity(256);

    for index in 0..lcp_array.len() {
        let lcp_real = lcp_array[index];
        // Branchless clamp to [0-255] (u8 range)
        // TODO: Optimize this statement
        // TODO: Use u16 for the lz matches
        let lcp = lcp_real + (lcp_real > 255) as i32 * (255 - lcp_real);
        if stack.is_empty() {
            for len in 2..=lcp {
                stack.push(MatchGen::new(index, len as u8))
            }
        }

        // Pop old matches
        while matches!(stack.last(), Some(m) if lcp < m.len as i32) {
            let m_top = stack.pop().unwrap();
            matches.push(Match::new(index as u32 - m_top.sa_index + 1, m_top));
        }

        // Push new matches
        while matches!(stack.last(), Some(m) if lcp > m.len as i32) {
            let m_top = stack.last().unwrap();
            let mg = MatchGen::new(index, m_top.len + 1);
            stack.push(mg);
        }
    }

    assert!(stack.is_empty());
    println!("Generated matches in: {:?}", timer.elapsed());
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
    fn new(sa_count: u32, mg: MatchGen) -> Self {
        Self { sa_index: mg.sa_index, len: mg.len, sa_count, self_ref: true }
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        return self.sa_index as usize .. (self.sa_index + self.sa_count) as usize;
    }
}

pub fn _static_analyze(mdma_index: &MdmaIndex) {
    let lcp_array = build_lcp_array(&mdma_index.sa, &mdma_index.buf);
    let mut stack = Vec::with_capacity(256);
    let mut max_sa_count = 0;
    let mut max_len = 0;
    let mut count = 0;
    let mut counts = [0; 6];

    for index in 0..lcp_array.len() {
        let lcp_real = lcp_array[index];
        if lcp_real > max_len {
            max_len = lcp_real;
        }
        let lcp = lcp_real + (lcp_real > 255) as i32 * (255 - lcp_real);
        if stack.is_empty() {
            for len in 2..=lcp {
                stack.push(MatchGen::new(index, len as u8))
            }
        }

        // Pop old matches
        while matches!(stack.last(), Some(m) if lcp < m.len as i32) {
            let m_top = stack.pop().unwrap();
            let sa_count = index as u32 - m_top.sa_index + 1;
            if sa_count > max_sa_count { max_sa_count = sa_count; }
            count += 1;
            if m_top.len < 8 {
                counts[(m_top.len - 2) as usize] += 1;
            }
            // matches.push(Match::new(index as u32 - m_top.sa_index + 1, m_top));
        }

        // Push new matches
        while matches!(stack.last(), Some(m) if lcp > m.len as i32) {
            let m_top = stack.last().unwrap();
            let mg = MatchGen::new(index, m_top.len + 1);
            stack.push(mg);
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