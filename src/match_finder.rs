struct MatchGen {
    sa_index: u32,
    len: u32
}

impl MatchGen {
    fn new(sa_index: usize, len: i32) -> Self { Self { sa_index: sa_index as u32, len: len as u32 } }
}

// make this into 3 arrays?
pub struct Match {
    pub sa_index: u32,
    pub sa_count: u32,
    pub len: u32
}

impl Match {
    fn new(sa_count: u32, mg: MatchGen) -> Self { Self { sa_count, sa_index: mg.sa_index, len: mg.len } }
    pub fn get_range(&self) -> std::ops::Range<usize> {
        return self.sa_index as usize .. (self.sa_index + self.sa_count) as usize;
    }
}

impl Clone for Match {
    fn clone(&self) -> Self {
        Self { sa_index: self.sa_index.clone(), sa_count: self.sa_count.clone(), len: self.len.clone() }
    }
}

pub fn generate(lcp_array: &Vec<i32>) -> Vec<Match> {
    let mut matches = Vec::with_capacity((lcp_array.len() as f64 * 0.75) as usize);
    let mut stack = vec![];

    for index in 0..lcp_array.len() {
        let lcp = lcp_array[index];
        if stack.is_empty() {
            for len in 2..=lcp {
                stack.push(MatchGen::new(index, len))
            }
        }

        while matches!(stack.last(), Some(m) if lcp < m.len as i32) {
            let m_top = stack.pop().unwrap();
            matches.push(Match::new(index as u32 - m_top.sa_index + 1, m_top));
        }

        while matches!(stack.last(), Some(m) if lcp > m.len as i32) {
            let m_top = stack.last().unwrap();
            let mg = MatchGen::new(index, m_top.len as i32 + 1);
            stack.push(mg);
        }
    }

    assert!(stack.is_empty());
    return matches;
}