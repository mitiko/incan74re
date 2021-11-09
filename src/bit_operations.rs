// use std::collections::VecDeque;

// pub fn to_vec_rev(number: u32, vec: &mut Vec<u8>) {
//     vec.push(( number        & 0xff) as u8);
//     vec.push(((number >> 8 ) & 0xff) as u8);
//     vec.push(((number >> 16) & 0xff) as u8);
//     vec.push(((number >> 24) & 0xff) as u8);
// }

// pub fn to_vec_deque(number: u32, vec: &mut VecDeque<u8>) {
//     vec.push_back(((number >> 24) & 0xff) as u8);
//     vec.push_back(((number >> 16) & 0xff) as u8);
//     vec.push_back(((number >> 8 ) & 0xff) as u8);
//     vec.push_back(( number        & 0xff) as u8);
// }

// pub fn to_vec_deque_rev(number: u16, vec: &mut VecDeque<u8>) {
//     vec.push_front(( number        & 0xff) as u8);
//     vec.push_front(((number >> 8 ) & 0xff) as u8);
// }

// pub fn from_vec(index: &mut usize, vec: &Vec<u8>) -> u32 {
//         *index += 4;
//         ((vec[*index-4] as u32) << 24) +
//         ((vec[*index-3] as u32) << 16) +
//         ((vec[*index-2] as u32) <<  8) +
//         ( vec[*index-1] as u32)
// }

// pub fn from_vec_u16(index: &mut usize, vec: &Vec<u8>) -> u16 {
//         *index += 2;
//         ((vec[*index-2] as u16) << 8) +
//         ( vec[*index-1] as u16)
// }