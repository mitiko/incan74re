use super::bindings;
use super::match_finder;
use super::entropy_ranking;
use super::entropy_ranking::RankedWord;

pub struct Word {
    pub location: usize,
    pub len: usize
}

impl Word {
    pub fn get_range(&self) -> std::ops::Range<usize> {
        self.location .. (self.location + self.len)
    }
}

impl Clone for Word {
    fn clone(&self) -> Self {
        Self { location: self.location.clone(), len: self.len.clone() }
    }
}

pub struct MdmaIndex<'a> {
    pub buf:   &'a Vec<u8>,
    pub sa:    &'a Vec<i32>,
    pub lcp:   &'a Vec<i32>,
    pub spots: &'a mut Vec<i32>,
    pub model:      &'a mut [f64; 256],
    pub sym_counts: &'a mut [f64; 256],
    pub n: &'a mut i32
}

// TODO:
// [X] Build SA
// [X] Build LCP
// [X] Create generator for matchfinder
// [X] Use a spots array as bitvector
// [X] Add a model for entropy ranking
// [X] Entropy ranking function
// [ ] Parser

// println!("Size of match: {}", std::mem::size_of::<Match>());
// println!("Size of word: {}", std::mem::size_of::<Word>());
// println!("Size of u64: {}", std::mem::size_of::<u64>());

pub fn build_dictionary(buf: &Vec<u8>) -> Vec<Word> {
    // Build bwd_index
    let sa = &build_suffix_array(buf);
    let lcp = &build_lcp_array(sa, buf);
    let model = &mut build_model(buf);
    let spots = &mut vec![0; buf.len()];
    let mdma_index = &mut MdmaIndex { buf, sa, lcp, spots, model, sym_counts: &mut [0f64; 256], n: &mut (buf.len() as i32) };
    let mut dict = vec![];

    // Initialize the match-holding structure
    let mut curr_matches = match_finder::generate(&lcp);
    let mut matches = Vec::<match_finder::Match>::with_capacity(curr_matches.len());
    println!("Matches vec holds: {} matches", curr_matches.len());

    loop {
        std::mem::swap(&mut matches, &mut curr_matches);
        curr_matches.clear();
        curr_matches.shrink_to(matches.len());
        let mut best_match = match_finder::Match { sa_index: 0, sa_count: 0, len: 0 };
        let mut best_word = RankedWord {
            word: Word { location: 0, len: 0 },
            count: -1, rank: 0f64
        };

        if matches.is_empty() { break; }

        for m in &matches {
            if let Some(ranked_word) = entropy_ranking::rank(m, mdma_index) {
                curr_matches.push(m.clone());
                if ranked_word.rank > best_word.rank {
                    best_word = ranked_word.clone();
                    best_match = m.clone();
                }
            }
        }

        best_word.print();
        dict.push(best_word.word.clone());
        entropy_ranking::split(&best_match, mdma_index);
        entropy_ranking::update_model(&best_word, mdma_index);
    }

    return dict;
}

fn build_suffix_array(buf: &Vec<u8>) -> Vec<i32> {
    let mut sa = vec![0; buf.len()];
    let code = unsafe { bindings::libsais(buf.as_ptr(), sa.as_mut_ptr(), buf.len() as i32) };
    assert!(code == 0);
    assert!(sa.len() == buf.len());
    println!("Build SA");
    return sa;
}

fn build_lcp_array(sa: &Vec<i32>, buf: &Vec<u8>) -> Vec<i32> {
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
            if buf[i+k] != buf[j + k] {
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

fn build_model(buf: &Vec<u8>) -> [f64; 256] {
    let mut model = [0f64; 256];

    for &sym in buf {
        model[sym as usize] += 1f64;
    }

    model
}
