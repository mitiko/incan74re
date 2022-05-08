use std::time::Instant;

use crate::bindings;
use crate::entropy_ranking::{rank, update_model};
use crate::splitting::split;
use crate::match_finder;

pub struct MdmaIndex {
    pub buf:        Vec<u8>,
    pub sa:         Vec<i32>,
    pub offsets:    Vec<i32>,
    pub model:      [f64; 256],
    pub sym_counts: [f64; 256],
    pub n: u32,
    pub replacement_token: i32
}

pub fn initialize(buf: Vec<u8>) -> MdmaIndex {
    let len: u32 = buf.len().try_into().expect("Buffer must fit into u32 type!");
    let sa = build_suffix_array(&buf);
    let model = build_model(&buf);
    let offsets = build_offsets_array(buf.len());

    MdmaIndex { n: len, buf, sa, offsets, model, sym_counts: [0f64; 256], replacement_token: -256 }
}

pub fn build_dictionary(mdma_index: &mut MdmaIndex) -> Vec<Word> {
    // The cast here is ok, because it's just an approximation we're making and the value may never become negative
    let mut curr_matches = Vec::with_capacity((mdma_index.buf.len() as f64 * 2.3) as usize);
    let mut dict = Vec::with_capacity(128);

    // Initialize with all the macthes
    // match_finder::_static_analyze(lcp_array);
    let lcp_array = build_lcp_array(&mdma_index.buf, &mdma_index.sa);
    match_finder::generate(&mut curr_matches, lcp_array);

    loop {
        let best_word = curr_matches.iter_mut()
            .filter(|m| m.is_valid)
            .filter_map(|m| rank(m, mdma_index))
            .reduce(|best_word, word|
                if word.rank > best_word.rank { word }
                else { best_word }
            );

        if best_word.is_none() { break; }
        let best_word = best_word.unwrap();

        // best_word._print();
        dict.push(best_word.clone());
        split(&best_word, mdma_index);
        update_model(&best_word, mdma_index);
    }

    dict
}

fn build_suffix_array(buf: &[u8]) -> Vec<i32> {
    let timer = Instant::now();
    let len: i32 = buf.len().try_into().expect("Buffer must fit into i32 type to use libsais!");
    let mut sa = vec![0; buf.len()];

    let code = unsafe { bindings::libsais(buf.as_ptr(), sa.as_mut_ptr(), len, 0, std::ptr::null_mut::<i32>()) };
    assert!(code == 0);
    assert!(sa.len() == buf.len());
    println!("Built SA in {:?}", timer.elapsed());

    sa
}

fn build_lcp_array(buf: &[u8], sa: &[i32]) -> Vec<i32> {
    let timer = Instant::now();
    let len: i32 = buf.len().try_into().expect("Buffer must fit into i32 type to use libsais!");
    let mut plcp = vec![0; buf.len()];
    let mut lcp = vec![0; buf.len()+1];

    let code = unsafe { bindings::libsais_plcp(buf.as_ptr(), sa.as_ptr(), plcp.as_mut_ptr(), len) };
    assert!(code == 0);
    assert!(plcp.len() == buf.len());

    let code = unsafe { bindings::libsais_lcp(plcp.as_ptr(), sa.as_ptr(), lcp.as_mut_ptr(), len) };
    assert!(code == 0);
    // This is a bit of hacky magic because the previous implementation of an LCP array (using kasai's alg)
    // produced an array ending with 0, while the libsais version has the extra 0 in the beginning
    // This is ultimately based on where you assume the sentinel token to be placed
    // As it is purely an implementation choice, I found this to be easier to adapt,
    // rather than rewriting the matchfinder
    lcp.remove(0);
    assert!(lcp.len() == buf.len());
    println!("Built LCP in {:?}", timer.elapsed());

    lcp
}

fn build_offsets_array(len: usize) -> Vec<i32> {
    let mut vec = vec![0; len];
    let max = i32::try_from(len).unwrap() - 1;

    // TODO: Does this get unrolled?
    vec.iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = max - i32::try_from(i).unwrap());

    vec
}

fn build_model(buf: &[u8]) -> [f64; 256] {
    let mut model = [0f64; 256];

    for &sym in buf {
        model[sym as usize] += 1f64;
    }

    model
}

#[derive(Clone)]
pub struct Word {
    pub rank: f64,
    pub location: usize,
    pub sa_index: u32,
    pub sa_count: u32,
    pub count: u32,
    pub len: u16,
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
}
