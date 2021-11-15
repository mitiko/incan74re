use super::bindings;
use super::match_finder;
use super::entropy_ranking;
use super::entropy_ranking::RankedWord;
use super::match_finder::Match;

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
    pub spots: &'a mut Vec<i32>,
    pub model:      &'a mut [f64; 256],
    pub sym_counts: &'a mut [f64; 256],
    pub n:          &'a mut i32
}

// TODO:
// [X] Build SA
// [X] Build LCP
// [X] Create generator for matchfinder
// [X] Use a spots array as bitvector
// [X] Add a model for entropy ranking
// [X] Entropy ranking function
// [ ] Parser

pub fn build_dictionary(buf: &Vec<u8>) -> Vec<Word> {
    // Build bwd_index
    let sa = &build_suffix_array(buf);
    let model = &mut build_model(buf);
    let spots = &mut vec![0; buf.len()];
    let mdma_index = &mut MdmaIndex { buf, sa, spots, model, sym_counts: &mut [0f64; 256], n: &mut (buf.len() as i32) };
    let mut dict = vec![];
    // match_finder::static_analyze(lcp_array);

    // Initialize the match-holding structure
    let mut curr_matches = Vec::with_capacity((mdma_index.buf.len() as f64 * 2.3) as usize);
    match_finder::generate(mdma_index, &mut curr_matches);
    let mut matches = Vec::<Match>::with_capacity(curr_matches.len());
    println!("Matches vec holds: {} matches", curr_matches.len());
    loop {
        std::mem::swap(&mut matches, &mut curr_matches);
        curr_matches.clear();
        curr_matches.shrink_to(matches.len());
        let mut best_match = Match::empty();
        let mut best_word = RankedWord::empty();

        if matches.is_empty() { break; }

        for m in &mut matches {
            if let Some(ranked_word) = entropy_ranking::rank(m, mdma_index) {
                curr_matches.push(m.clone());
                if ranked_word.rank > best_word.rank {
                    best_word = ranked_word.clone();
                    best_match = m.clone();
                }
            }
        }

        if best_word.count == -1 { break; }
        // best_word._print();
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

fn build_model(buf: &Vec<u8>) -> [f64; 256] {
    let mut model = [0f64; 256];

    for &sym in buf {
        model[sym as usize] += 1f64;
    }

    model
}
