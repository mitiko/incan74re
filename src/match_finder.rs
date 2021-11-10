pub fn generate(lcp_array: &Vec<i32>) -> Vec<Match> {
    let mut matches = Vec::with_capacity((lcp_array.len() as f64 * 0.75) as usize);
    let mut stack = vec![];

    for index in 0..lcp_array.len() {
        let lcp = lcp_array[index];
        if stack.is_empty() {
            for len in 2..=lcp {
                // Branchless clamp to [0-255] (u8 range)
                let mg_len = (len + (len > 255) as i32 * (255 - len)) as u8;
                stack.push(MatchGen::new(index, mg_len))
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
            let len = m_top.len as i32;
            // Branchless clamp to [0-255] (u8 range)
            let mg_len = (len + (len > 254) as i32 * (254 - len)) as u8 + 1;
            let mg = MatchGen::new(index, mg_len);
            stack.push(mg);
        }
    }

    assert!(stack.is_empty());
    return matches;
}

// MatchGen is a more lightweight struct that only holds the len and sa_index
// It's used to  minimize the allocations of the match_finder
// Since we always compute the sa_count field, we can not store it and save a couple of bytes
// NOTE: range of len is [2-], but range of u8 is [0-255], so we *could* offset the len by 2
// while storing in match to utilize a bigger range [2-257] if we needed to
// (but I think the bigger range can't compensate for the extra complexity per access)
// - same with sa_count
#[repr(packed(1))]
pub struct MatchGen {
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
pub struct Match {
    pub sa_index: u32,
    sa_count_len: u32
}

impl Match {
    fn new(sa_count: u32, mg: MatchGen) -> Self {
        // Branchless clamp to [0-16777215] (u24 range)
        let sa_count_clamped = sa_count + (sa_count > 0xff_ff_ff) as u32 * (0xff_ff_ff - sa_count);
        let sa_count_len = (sa_count_clamped << 8) | mg.len as u32;
        Self { sa_index: mg.sa_index, sa_count_len }
    }

    pub fn empty() -> Self {
        Self { sa_index: 0, sa_count_len: 0 }
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        return self.sa_index as usize .. (self.sa_index + self.get_count()) as usize;
    }

    pub fn get_count(&self) -> u32 {
        self.sa_count_len >> 8
    }

    pub fn get_len(&self) -> u32 {
        self.sa_count_len & 0xff
    }
}

impl Clone for Match {
    fn clone(&self) -> Self {
        Self { sa_index: self.sa_index.clone(), sa_count_len: self.sa_count_len.clone() }
    }
}