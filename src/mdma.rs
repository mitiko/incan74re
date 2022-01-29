use std::time::Instant;

use crate::bindings;
use crate::match_finder;
use crate::entropy_ranking;
use crate::match_finder::Match;
use crate::splitting;

pub struct MdmaIndex {
    pub buf:        Vec<u8>,
    pub sa:         Vec<i32>,
    pub offsets:    Vec<i32>,
    pub model:      [f64; 256],
    pub sym_counts: [f64; 256],
    pub n:          i32,
    pub dict_len:   i32
}

// TODO:
// [X] Build SA
// [X] Build LCP
// [X] Create generator for matchfinder
// [X] Use a spots array as bitvector -> renamed to offsets
// [X] Add a model for entropy ranking
// [X] Entropy ranking function
// [X] Parser

pub fn build_dictionary(mdma_index: &mut MdmaIndex) -> Vec<Word> {
    let mut dict = Vec::with_capacity(128);
    // match_finder::_static_analyze(mdma_index);

    // Initialize the match-holding structure
    let mut curr_matches = Vec::with_capacity((mdma_index.buf.len() as f64 * 2.3) as usize);
    match_finder::generate(mdma_index, &mut curr_matches);
    let mut matches = Vec::<Match>::with_capacity(curr_matches.len());
    println!("Matches vec holds: {} matches", curr_matches.len());

    loop {
        std::mem::swap(&mut matches, &mut curr_matches);
        curr_matches.clear();
        curr_matches.shrink_to(matches.len());

        let mut best_word = Word::empty();
        if matches.is_empty() { break; }

        for m in &mut matches {
            if let Some(ranked_word) = entropy_ranking::rank(m, mdma_index) {
                curr_matches.push(m.clone());
                if ranked_word.rank > best_word.rank { best_word = ranked_word.clone(); }
            }
        }

        if best_word.count == -1 { break; }
        // best_word._print();
        dict.push(best_word.clone());
        splitting::split(&best_word, mdma_index);
        entropy_ranking::update_model(&best_word, mdma_index);
    }

    return dict;
}

pub fn initialize(buf: Vec<u8>) -> MdmaIndex {
    let sa = build_suffix_array(&buf);
    let model = build_model(&buf);
    let offsets = build_offsets_array(buf.len());
    MdmaIndex { n: (buf.len() as i32), buf, sa, offsets, model, sym_counts: [0f64; 256], dict_len: 1 }
}

fn build_suffix_array(buf: &Vec<u8>) -> Vec<i32> {
    let timer = Instant::now();
    let mut sa = vec![0; buf.len()];
    let code = unsafe { bindings::libsais(buf.as_ptr(), sa.as_mut_ptr(), buf.len() as i32) };
    assert!(code == 0);
    assert!(sa.len() == buf.len());
    println!("Built SA in {:?}", timer.elapsed());
    return sa;
}

fn build_offsets_array(len: usize) -> Vec<i32> {
    let mut vec = vec![0; len];
    let max = len - 1;
    // Does this get unrolled?
    for i in 0..len {
        vec[i] = (max - i) as i32;
    }
    return vec;
}

fn build_model(buf: &Vec<u8>) -> [f64; 256] {
    let mut model = [0f64; 256];

    for &sym in buf {
        model[sym as usize] += 1f64;
    }

    model
}

#[derive(Clone)]
pub struct Word {
    pub rank: f64,
    pub location: u32,
    pub sa_index: u32,
    pub sa_count: u32,
    pub count: i32,
    pub len: i32,
}

impl Word {
    pub fn _print(&self) {
        println!("word -> ({}, {}); c={}, r={}", self.location, self.len, self.count, self.rank);
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        (self.location as usize)..(self.location as usize + self.len as usize)
    }

    pub fn get_sa_range(&self) -> std::ops::Range<usize> {
        (self.sa_index as usize)..(self.sa_index as usize + self.sa_count as usize)
    }

    pub fn empty() -> Self {
        Self {
            location: 0, sa_index: 0, sa_count: 0, count: -1, len: -1, rank: f64::MIN
        }
    }
}
