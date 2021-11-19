use super::mdma::MdmaIndex;

pub fn generate(mdma_index: &MdmaIndex, matches: &mut Vec<Match>) {
    let lcp_array = build_lcp_array(mdma_index.sa, mdma_index.buf);
    let mut stack = Vec::with_capacity(256);

    for index in 0..lcp_array.len() {
        let lcp_real = lcp_array[index];
        // todo: optimize this statement
        // Branchless clamp to [0-255] (u8 range)
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
}

pub fn build_lcp_array(sa: &Vec<i32>, buf: &Vec<u8>) -> Vec<i32> {
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

    println!("Built LCP");
    return lcp;
}

// MatchGen is a more lightweight struct that only holds the len and sa_index
// It's used to  minimize the allocations of the match_finder
// Since we always compute the sa_count field, we can not store it and save a couple of bytes
// NOTE: range of len is [2-], but range of u8 is [0-255], so we *could* offset the len by 2
// while storing in match to utilize a bigger range [2-257] if we needed to
// (but I think the bigger range can't compensate for the extra complexity per access)
// - same with sa_count
#[repr(packed(1))]
struct MatchGen {
    sa_index: u32,
    len: u8
}

impl MatchGen {
    fn new(sa_index: usize, len: u8) -> Self { Self { sa_index: sa_index as u32, len } }
}

// Instead of 3*u32=12bytes for sa_index, sa_count, len we can do with just 8
// The trick is to share one u32 for len and sa_count
// We need this as a memory optimization but it imposes a limit on the sa_count and len
// len is (low) 8 bits and sa_count is (high) 24 bits
// This seems to be enough for regular (up to 2GB) text files
// Matches with len > 256 bytes can be

// TODO: Make the sa_count_len also contain 1 bit for self_ref -> it can be the last bit for fast access (with mask)
#[repr(packed(1))]
pub struct Match {
    pub self_ref: bool,
    pub sa_index: u32,
    sa_count_len: u32
}

impl Match {
    fn new(sa_count: u32, mg: MatchGen) -> Self {
        // Branchless clamp to [0-16777215] (u24 range)
        let sa_count_clamped = sa_count + (sa_count > 0xff_ff_ff) as u32 * (0xff_ff_ff - sa_count);
        let sa_count_len = (sa_count_clamped << 8) | mg.len as u32;
        Self { sa_index: mg.sa_index, sa_count_len, self_ref: true }
    }

    pub fn empty() -> Self {
        Self { sa_index: 0, sa_count_len: 0, self_ref: true }
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        return self.sa_index as usize .. (self.sa_index + self.get_count()) as usize;
    }

    pub fn get_count(&self) -> u32 {
        self.sa_count_len >> 8
    }

    pub fn get_len(&self) -> i32 {
        (self.sa_count_len & 0xff) as i32
    }
}

impl Clone for Match {
    fn clone(&self) -> Self {
        Self { sa_index: self.sa_index, sa_count_len: self.sa_count_len, self_ref: self.self_ref }
    }
}

pub fn _static_analyze(lcp_array: &Vec<i32>) {
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