// OF; <p.oscar.franzen@gmail.com>
// https://github.com/oscar-franzen/md5inrust

// A vanilla md5 implementation I used as a way to learn the Rust
// programming language. See README.md for more info on how to
// compile.

use std::fmt::Display;
use std::io::BufReader;
use std::io::Read;
use std::num::Wrapping;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;
use std::usize;
use std::vec::Vec;

use rand::random;

const BLOCK_SIZE: usize = 512;
pub const INITIAL_STATE: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
const M0_DELTA: [u32; 16] = [
    0,
    0,
    0,
    0,
    2_u32.pow(31),
    0,
    0,
    0,
    0,
    0,
    0,
    2_u32.pow(15),
    0,
    0,
    2_u32.pow(31),
    0,
];

const A1_ONE_BITS: u32 = 0x84200000;
const A1_ZERO_BITS: u32 = 0x0A000820;
const D1_ONE_BITS: u32 = 0x8C000800;
const D1_ZERO_BITS: u32 = 0x02208026;
const D1_A1_SAME_BITS: u32 = 0x701F10C0;
const C1_ONE_BITS: u32 = 0xBE1F0966;
const C1_ZERO_BITS: u32 = 0x40201080;
const C1_D1_SAME_BITS: u32 = 0x00000018;
const B1_ONE_BITS: u32 = 0xBA040010;
const B1_ZERO_BITS: u32 = 0x443B19EE;
const B1_C1_SAME_BITS: u32 = 0x00000601;
const A2_ONE_BITS: u32 = 0x482F0E50;
const A2_ZERO_BITS: u32 = 0xB41011AF;
const D2_ONE_BITS: u32 = 0x04220C56;
const D2_ZERO_BITS: u32 = 0x9A1113A9;
const C2_ONE_BITS: u32 = 0x96011E01;
const C2_ZERO_BITS: u32 = 0x083201C0;
const C2_D2_SAME_BITS: u32 = 0x01808000;
const B2_ONE_BITS: u32 = 0x843283C0;
const B2_ZERO_BITS: u32 = 0x1B810001;
const B2_C2_SAME_BITS: u32 = 0x00000002;
const A3_ONE_BITS: u32 = 0x9C0101C1;
const A3_ZERO_BITS: u32 = 0x03828202;
const A3_B2_SAME_BITS: u32 = 0x00001000;
const D3_ONE_BITS: u32 = 0x878383C0;
const D3_ZERO_BITS: u32 = 0x00041003;
const C3_ONE_BITS: u32 = 0x800583C3;
const C3_ZERO_BITS: u32 = 0x00021000;
const C3_D3_SAME_BITS: u32 = 0x00086000;
const B3_ONE_BITS: u32 = 0x80081080;
const B3_ZERO_BITS: u32 = 0x0007E000;
const B3_C3_SAME_BITS: u32 = 0x7F000000;
const A4_ONE_BITS: u32 = 0x3F0FE008;
const A4_ZERO_BITS: u32 = 0xC0000080;
const D4_ONE_BITS: u32 = 0x400BE088;
const D4_ZERO_BITS: u32 = 0xBF040000;
const C4_ONE_BITS: u32 = 0x7D000000;
const C4_ZERO_BITS: u32 = 0x82008008;
const B4_ONE_BITS: u32 = 0x20000000;
const B4_ZERO_BITS: u32 = 0x80000000;
const A5_ZERO_BITS: u32 = 0x80020000;
const A5_B4_SAME_BITS: u32 = 0x00008008;
const D5_ONE_BITS: u32 = 0x00020000;
const D5_ZERO_BITS: u32 = 0x80000000;
const D5_A5_SAME_BITS: u32 = 0x20000000;
const C5_ZERO_BITS: u32 = 0x80020000;
const B5_ZERO_BITS: u32 = 0x80000000;
const A6_ZERO_BITS: u32 = 0x80000000;
const A6_B5_SAME_BITS: u32 = 0x00020000;
const D6_ZERO_BITS: u32 = 0x80000000;
const C6_ZERO_BITS: u32 = 0x80000000;
const B6_C6_DIFFERENT_BITS: u32 = 0x80000000;
// const PHI34_ONE_BITS: u32 = 0x80000000;
const B12_D12_SAME_BITS: u32 = 0x80000000;
const A13_C12_SAME_BITS: u32 = 0x80000000;
const D13_B12_DIFFERENT_BITS: u32 = 0x80000000;
const C13_A13_SAME_BITS: u32 = 0x80000000;
const B13_D13_SAME_BITS: u32 = 0x80000000;
const A14_C13_SAME_BITS: u32 = 0x80000000;
const D14_B13_SAME_BITS: u32 = 0x80000000;
const C14_A14_SAME_BITS: u32 = 0x80000000;
const B14_D14_SAME_BITS: u32 = 0x80000000;
const A15_C14_SAME_BITS: u32 = 0x80000000;
const D15_B14_SAME_BITS: u32 = 0x80000000;
const C15_A15_SAME_BITS: u32 = 0x80000000;
const B15_D15_DIFFERENT_BITS: u32 = 0x80000000;
const A16_ONE_BITS: u32 = 0x02000000;
const A16_C15_SAME_BITS: u32 = 0x80000000;
const D16_ONE_BITS: u32 = 0x02000000;
const D16_B15_SAME_BITS: u32 = 0x80000000;
// const C16_ONE_BITS: u32 = 0x02000000;
// const C16_A16_SAME_BITS: u32 = 0x80000000;
// const B16_ONE_BITS: u32 = 0x02000000;

struct InternalState {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

fn tr_f(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

fn tr_g(x: u32, y: u32, z: u32) -> u32 {
    (x & z) | (y & !z)
}

fn tr_h(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

fn tr_i(x: u32, y: u32, z: u32) -> u32 {
    y ^ (x | !z)
}

fn transform(
    func: impl Fn(u32, u32, u32) -> u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    word: u32,
    k: u32,
    s: u32,
) -> u32 {
    let f = a.wrapping_add(func(b, c, d));

    let mut temp = f.wrapping_add(word).wrapping_add(k);

    temp = temp.rotate_left(s);
    return temp.wrapping_add(b);
}

fn apply_one_bits(v: u32, mask: u32) -> u32 {
    return v | mask;
}

fn apply_zero_bits(v: u32, mask: u32) -> u32 {
    return v & (!mask);
}

fn apply_same_bits(v: u32, u: u32, mask: u32) -> u32 {
    return (v | (u & mask)) & (u | (!mask));
}

fn validate_candiate(state: &[u32; 4], candidate: [u32; 16]) -> Option<[u32; 16]> {
    let mut words: [u32; 16] = candidate;

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];

    //  round 1

    //  a1
    let mut orig = a;
    a = transform(tr_f, a, b, c, d, words[0], 0xD76AA478, 7);
    a = apply_one_bits(a, A1_ONE_BITS);
    a = apply_zero_bits(a, A1_ZERO_BITS);
    words[0] = reverse_transfrom(a, b, c, d, 0xD76AA478, 7, orig);

    // d1
    orig = d;
    d = transform(tr_f, d, a, b, c, words[1], 0xE8C7B756, 12);
    d = apply_one_bits(d, D1_ONE_BITS);
    d = apply_zero_bits(d, D1_ZERO_BITS);
    d = apply_same_bits(d, a, D1_A1_SAME_BITS);
    words[1] = reverse_transfrom(d, a, b, c, 0xE8C7B756, 12, orig);

    // c1
    orig = c;
    c = transform(tr_f, c, d, a, b, words[2], 0x242070DB, 17);
    c = apply_one_bits(c, C1_ONE_BITS);
    c = apply_zero_bits(c, C1_ZERO_BITS);
    c = apply_same_bits(c, d, C1_D1_SAME_BITS);
    words[2] = reverse_transfrom(c, d, a, b, 0x242070DB, 17, orig);

    // b1
    orig = b;
    b = transform(tr_f, b, c, d, a, words[3], 0xC1BDCEEE, 22);
    b = apply_one_bits(b, B1_ONE_BITS);
    b = apply_zero_bits(b, B1_ZERO_BITS);
    b = apply_same_bits(b, c, B1_C1_SAME_BITS);
    words[3] = reverse_transfrom(b, c, d, a, 0xC1BDCEEE, 22, orig);

    // a2
    orig = a;
    a = transform(tr_f, a, b, c, d, words[4], 0xF57C0FAF, 7);
    a = apply_one_bits(a, A2_ONE_BITS);
    a = apply_zero_bits(a, A2_ZERO_BITS);
    words[4] = reverse_transfrom(a, b, c, d, 0xF57C0FAF, 7, orig);

    // d2
    orig = d;
    d = transform(tr_f, d, a, b, c, words[5], 0x4787C62A, 12);
    d = apply_one_bits(d, D2_ONE_BITS);
    d = apply_zero_bits(d, D2_ZERO_BITS);
    words[5] = reverse_transfrom(d, a, b, c, 0x4787C62A, 12, orig);

    // c2
    orig = c;
    c = transform(tr_f, c, d, a, b, words[6], 0xA8304613, 17);
    c = apply_one_bits(c, C2_ONE_BITS);
    c = apply_zero_bits(c, C2_ZERO_BITS);
    c = apply_same_bits(c, d, C2_D2_SAME_BITS);
    words[6] = reverse_transfrom(c, d, a, b, 0xA8304613, 17, orig);

    // b2
    orig = b;
    b = transform(tr_f, b, c, d, a, words[7], 0xFD469501, 22);
    b = apply_one_bits(b, B2_ONE_BITS);
    b = apply_zero_bits(b, B2_ZERO_BITS);
    b = apply_same_bits(b, c, B2_C2_SAME_BITS);
    words[7] = reverse_transfrom(b, c, d, a, 0xFD469501, 22, orig);

    // a3
    orig = a;
    a = transform(tr_f, a, b, c, d, words[8], 0x698098D8, 7);
    a = apply_one_bits(a, A3_ONE_BITS);
    a = apply_zero_bits(a, A3_ZERO_BITS);
    a = apply_same_bits(a, b, A3_B2_SAME_BITS);
    words[8] = reverse_transfrom(a, b, c, d, 0x698098D8, 7, orig);

    // d3
    orig = d;
    d = transform(tr_f, d, a, b, c, words[9], 0x8B44F7AF, 12);
    d = apply_one_bits(d, D3_ONE_BITS);
    d = apply_zero_bits(d, D3_ZERO_BITS);
    words[9] = reverse_transfrom(d, a, b, c, 0x8B44F7AF, 12, orig);

    // c3
    orig = c;
    c = transform(tr_f, c, d, a, b, words[10], 0xFFFF5BB1, 17);
    c = apply_one_bits(c, C3_ONE_BITS);
    c = apply_zero_bits(c, C3_ZERO_BITS);
    c = apply_same_bits(c, d, C3_D3_SAME_BITS);
    words[10] = reverse_transfrom(c, d, a, b, 0xFFFF5BB1, 17, orig);

    // b3
    orig = b;
    b = transform(tr_f, b, c, d, a, words[11], 0x895CD7BE, 22);
    b = apply_one_bits(b, B3_ONE_BITS);
    b = apply_zero_bits(b, B3_ZERO_BITS);
    b = apply_same_bits(b, c, B3_C3_SAME_BITS);
    words[11] = reverse_transfrom(b, c, d, a, 0x895CD7BE, 22, orig);

    // a4
    orig = a;
    a = transform(tr_f, a, b, c, d, words[12], 0x6B901122, 7);
    a = apply_one_bits(a, A4_ONE_BITS);
    a = apply_zero_bits(a, A4_ZERO_BITS);
    words[12] = reverse_transfrom(a, b, c, d, 0x6B901122, 7, orig);

    // d4
    orig = d;
    d = transform(tr_f, d, a, b, c, words[13], 0xFD987193, 12);
    d = apply_one_bits(d, D4_ONE_BITS);
    d = apply_zero_bits(d, D4_ZERO_BITS);
    words[13] = reverse_transfrom(d, a, b, c, 0xFD987193, 12, orig);

    // c4
    orig = c;
    c = transform(tr_f, c, d, a, b, words[14], 0xA679438E, 17);
    c = apply_one_bits(c, C4_ONE_BITS);
    c = apply_zero_bits(c, C4_ZERO_BITS);
    words[14] = reverse_transfrom(c, d, a, b, 0xA679438E, 17, orig);

    // b4
    orig = b;
    b = transform(tr_f, b, c, d, a, words[15], 0x49B40821, 22);
    b = apply_one_bits(b, B4_ONE_BITS);
    b = apply_zero_bits(b, B4_ZERO_BITS);
    words[15] = reverse_transfrom(b, c, d, a, 0x49B40821, 22, orig);

    // round 2

    // a5
    a = transform(tr_g, a, b, c, d, words[1], 0xF61E2562, 5);
    if a & A5_ZERO_BITS != 0 {
        return None;
    }
    if a & A5_B4_SAME_BITS != b & A5_B4_SAME_BITS {
        return None;
    }

    // d5
    d = transform(tr_g, d, a, b, c, words[6], 0xC040B340, 9);
    if d & D5_ZERO_BITS != 0 {
        return None;
    }
    if d & D5_ONE_BITS != D5_ONE_BITS {
        return None;
    }
    if d & D5_A5_SAME_BITS != a & D5_A5_SAME_BITS {
        return None;
    }

    // c5
    c = transform(tr_g, c, d, a, b, words[11], 0x265E5A51, 14);
    if c & C5_ZERO_BITS != 0 {
        return None;
    }

    // b5
    b = transform(tr_g, b, c, d, a, words[0], 0xE9B6C7AA, 20);
    if b & B5_ZERO_BITS != 0 {
        return None;
    }

    // a6
    a = transform(tr_g, a, b, c, d, words[5], 0xD62F105D, 5);
    if a & A6_ZERO_BITS != 0 {
        return None;
    }
    if a & A6_B5_SAME_BITS != b & A6_B5_SAME_BITS {
        return None;
    }

    // d6
    d = transform(tr_g, d, a, b, c, words[10], 0x02441453, 9);
    if d & D6_ZERO_BITS != 0 {
        return None;
    }

    // c6
    c = transform(tr_g, c, d, a, b, words[15], 0xD8A1E681, 14);
    if c & C6_ZERO_BITS != 0 {
        return None;
    }

    // b6
    b = transform(tr_g, b, c, d, a, words[4], 0xE7D3FBC8, 20);
    if b & B6_C6_DIFFERENT_BITS == c & B6_C6_DIFFERENT_BITS {
        return None;
    }

    a = transform(tr_g, a, b, c, d, words[9], 0x21E1CDE6, 5);
    d = transform(tr_g, d, a, b, c, words[14], 0xC33707D6, 9);
    c = transform(tr_g, c, d, a, b, words[3], 0xF4D50D87, 14);
    b = transform(tr_g, b, c, d, a, words[8], 0x455A14ED, 20);

    a = transform(tr_g, a, b, c, d, words[13], 0xA9E3E905, 5);
    d = transform(tr_g, d, a, b, c, words[2], 0xFCEFA3F8, 9);
    c = transform(tr_g, c, d, a, b, words[7], 0x676F02D9, 14);
    b = transform(tr_g, b, c, d, a, words[12], 0x8D2A4C8A, 20);

    // round 3
    a = transform(tr_h, a, b, c, d, words[5], 0xFFFA3942, 4);
    d = transform(tr_h, d, a, b, c, words[8], 0x8771F681, 11);
    c = transform(tr_h, c, d, a, b, words[11], 0x6D9D6122, 16);
    b = transform(tr_h, b, c, d, a, words[14], 0xFDE5380C, 23);

    a = transform(tr_h, a, b, c, d, words[1], 0xA4BEEA44, 4);
    d = transform(tr_h, d, a, b, c, words[4], 0x4BDECFA9, 11);
    c = transform(tr_h, c, d, a, b, words[7], 0xF6BB4B60, 16);
    b = transform(tr_h, b, c, d, a, words[10], 0xBEBFBC70, 23);

    a = transform(tr_h, a, b, c, d, words[13], 0x289B7EC6, 4);
    d = transform(tr_h, d, a, b, c, words[0], 0xEAA127FA, 11);
    c = transform(tr_h, c, d, a, b, words[3], 0xD4EF3085, 16);
    b = transform(tr_h, b, c, d, a, words[6], 0x04881D05, 23);

    a = transform(tr_h, a, b, c, d, words[9], 0xD9D4D039, 4);
    d = transform(tr_h, d, a, b, c, words[12], 0xE6DB99E5, 11);
    c = transform(tr_h, c, d, a, b, words[15], 0x1FA27CF8, 16);
    // b12
    b = transform(tr_h, b, c, d, a, words[2], 0xC4AC5665, 23);
    if d & B12_D12_SAME_BITS != b & B12_D12_SAME_BITS {
        return None;
    }

    // round 4

    // a13
    a = transform(tr_i, a, b, c, d, words[0], 0xF4292244, 6);
    if a & A13_C12_SAME_BITS != c & A13_C12_SAME_BITS {
        return None;
    }

    // d13
    d = transform(tr_i, d, a, b, c, words[7], 0x432AFF97, 10);
    if d & D13_B12_DIFFERENT_BITS == b & D13_B12_DIFFERENT_BITS {
        return None;
    }

    // c13
    c = transform(tr_i, c, d, a, b, words[14], 0xAB9423A7, 15);
    if c & C13_A13_SAME_BITS != a & C13_A13_SAME_BITS {
        return None;
    }

    // b13
    b = transform(tr_i, b, c, d, a, words[5], 0xFC93A039, 21);
    if b & B13_D13_SAME_BITS != d & B13_D13_SAME_BITS {
        return None;
    }

    // a14
    a = transform(tr_i, a, b, c, d, words[12], 0x655B59C3, 6);
    if a & A14_C13_SAME_BITS != c & A14_C13_SAME_BITS {
        return None;
    }

    // d14
    d = transform(tr_i, d, a, b, c, words[3], 0x8F0CCC92, 10);
    if d & D14_B13_SAME_BITS != b & D14_B13_SAME_BITS {
        return None;
    }

    // c14
    c = transform(tr_i, c, d, a, b, words[10], 0xFFEFF47D, 15);
    if c & C14_A14_SAME_BITS != a & C14_A14_SAME_BITS {
        return None;
    }

    // b14
    b = transform(tr_i, b, c, d, a, words[1], 0x85845DD1, 21);
    if b & B14_D14_SAME_BITS != d & B14_D14_SAME_BITS {
        return None;
    }

    // a15
    a = transform(tr_i, a, b, c, d, words[8], 0x6FA87E4F, 6);
    if a & A15_C14_SAME_BITS != c & A15_C14_SAME_BITS {
        return None;
    }

    // d15
    d = transform(tr_i, d, a, b, c, words[15], 0xFE2CE6E0, 10);
    if d & D15_B14_SAME_BITS != b & D15_B14_SAME_BITS {
        return None;
    }

    // c15
    c = transform(tr_i, c, d, a, b, words[6], 0xA3014314, 15);
    if c & C15_A15_SAME_BITS != a & C15_A15_SAME_BITS {
        return None;
    }

    // b15
    b = transform(tr_i, b, c, d, a, words[13], 0x4E0811A1, 21);
    if b & B15_D15_DIFFERENT_BITS == d & B15_D15_DIFFERENT_BITS {
        return None;
    }

    // a16
    a = transform(tr_i, a, b, c, d, words[4], 0xF7537E82, 6);
    if a & A16_ONE_BITS != A16_ONE_BITS {
        return None;
    }
    if a & A16_C15_SAME_BITS != c & A16_C15_SAME_BITS {
        return None;
    }

    // d16
    d = transform(tr_i, d, a, b, c, words[11], 0xBD3AF235, 10);
    if d & D16_ONE_BITS != D16_ONE_BITS {
        return None;
    }
    if d & D16_B15_SAME_BITS != b & D16_B15_SAME_BITS {
        return None;
    }

    // // c16
    // c = transform(tr_i, c, d, a, b, words[2], 0x2AD7D2BB, 15);
    // if c & C16_ONE_BITS != C16_ONE_BITS {
    //     return None;
    // }
    // if c & C16_A16_SAME_BITS != a & C16_A16_SAME_BITS {
    //     return None;
    // }
    //
    // // b16
    // b = transform(tr_i, b, c, d, a, words[9], 0xEB86D391, 21);
    // if b & B16_ONE_BITS != B16_ONE_BITS {
    //     return None;
    // }

    Some(words)
}

fn reverse_transfrom(a: u32, b: u32, c: u32, d: u32, t: u32, s: u32, orig: u32) -> u32 {
    return (a.wrapping_sub(b))
        .rotate_right(s)
        .wrapping_sub(tr_f(b, c, d))
        .wrapping_sub(t)
        .wrapping_sub(orig);
}

fn process_block(buf: &mut [u8], state: InternalState) -> InternalState {
    let mut words: [u32; 16] = [0; 16];

    // break chunk into 16 words (each word being 32 bits)
    for i in 0..16 {
        let start: usize = i * 4;
        let stop: usize = i * 4 + 4;
        let word: u32 = u32::from_le_bytes(buf[start..stop].try_into().unwrap());
        words[i] = word;
    }

    let mut a = state.a;
    let mut b = state.b;
    let mut c = state.c;
    let mut d = state.d;

    //  round 1
    a = transform(tr_f, a, b, c, d, words[0], 0xD76AA478, 7);
    d = transform(tr_f, d, a, b, c, words[1], 0xE8C7B756, 12);
    c = transform(tr_f, c, d, a, b, words[2], 0x242070DB, 17);
    b = transform(tr_f, b, c, d, a, words[3], 0xC1BDCEEE, 22);

    a = transform(tr_f, a, b, c, d, words[4], 0xF57C0FAF, 7);
    d = transform(tr_f, d, a, b, c, words[5], 0x4787C62A, 12);
    c = transform(tr_f, c, d, a, b, words[6], 0xA8304613, 17);
    b = transform(tr_f, b, c, d, a, words[7], 0xFD469501, 22);

    a = transform(tr_f, a, b, c, d, words[8], 0x698098D8, 7);
    d = transform(tr_f, d, a, b, c, words[9], 0x8B44F7AF, 12);
    c = transform(tr_f, c, d, a, b, words[10], 0xFFFF5BB1, 17);
    b = transform(tr_f, b, c, d, a, words[11], 0x895CD7BE, 22);

    a = transform(tr_f, a, b, c, d, words[12], 0x6B901122, 7);
    d = transform(tr_f, d, a, b, c, words[13], 0xFD987193, 12);
    c = transform(tr_f, c, d, a, b, words[14], 0xA679438E, 17);
    b = transform(tr_f, b, c, d, a, words[15], 0x49B40821, 22);

    // round 2
    a = transform(tr_g, a, b, c, d, words[1], 0xF61E2562, 5);
    d = transform(tr_g, d, a, b, c, words[6], 0xC040B340, 9);
    c = transform(tr_g, c, d, a, b, words[11], 0x265E5A51, 14);
    b = transform(tr_g, b, c, d, a, words[0], 0xE9B6C7AA, 20);

    a = transform(tr_g, a, b, c, d, words[5], 0xD62F105D, 5);
    d = transform(tr_g, d, a, b, c, words[10], 0x02441453, 9);
    c = transform(tr_g, c, d, a, b, words[15], 0xD8A1E681, 14);
    b = transform(tr_g, b, c, d, a, words[4], 0xE7D3FBC8, 20);

    a = transform(tr_g, a, b, c, d, words[9], 0x21E1CDE6, 5);
    d = transform(tr_g, d, a, b, c, words[14], 0xC33707D6, 9);
    c = transform(tr_g, c, d, a, b, words[3], 0xF4D50D87, 14);
    b = transform(tr_g, b, c, d, a, words[8], 0x455A14ED, 20);

    a = transform(tr_g, a, b, c, d, words[13], 0xA9E3E905, 5);
    d = transform(tr_g, d, a, b, c, words[2], 0xFCEFA3F8, 9);
    c = transform(tr_g, c, d, a, b, words[7], 0x676F02D9, 14);
    b = transform(tr_g, b, c, d, a, words[12], 0x8D2A4C8A, 20);

    // round 3
    a = transform(tr_h, a, b, c, d, words[5], 0xFFFA3942, 4);
    d = transform(tr_h, d, a, b, c, words[8], 0x8771F681, 11);
    c = transform(tr_h, c, d, a, b, words[11], 0x6D9D6122, 16);
    b = transform(tr_h, b, c, d, a, words[14], 0xFDE5380C, 23);

    a = transform(tr_h, a, b, c, d, words[1], 0xA4BEEA44, 4);
    d = transform(tr_h, d, a, b, c, words[4], 0x4BDECFA9, 11);
    c = transform(tr_h, c, d, a, b, words[7], 0xF6BB4B60, 16);
    b = transform(tr_h, b, c, d, a, words[10], 0xBEBFBC70, 23);

    a = transform(tr_h, a, b, c, d, words[13], 0x289B7EC6, 4);
    d = transform(tr_h, d, a, b, c, words[0], 0xEAA127FA, 11);
    c = transform(tr_h, c, d, a, b, words[3], 0xD4EF3085, 16);
    b = transform(tr_h, b, c, d, a, words[6], 0x04881D05, 23);

    a = transform(tr_h, a, b, c, d, words[9], 0xD9D4D039, 4);
    d = transform(tr_h, d, a, b, c, words[12], 0xE6DB99E5, 11);
    c = transform(tr_h, c, d, a, b, words[15], 0x1FA27CF8, 16);
    b = transform(tr_h, b, c, d, a, words[2], 0xC4AC5665, 23);

    // round 4
    a = transform(tr_i, a, b, c, d, words[0], 0xF4292244, 6);
    d = transform(tr_i, d, a, b, c, words[7], 0x432AFF97, 10);
    c = transform(tr_i, c, d, a, b, words[14], 0xAB9423A7, 15);
    b = transform(tr_i, b, c, d, a, words[5], 0xFC93A039, 21);

    a = transform(tr_i, a, b, c, d, words[12], 0x655B59C3, 6);
    d = transform(tr_i, d, a, b, c, words[3], 0x8F0CCC92, 10);
    c = transform(tr_i, c, d, a, b, words[10], 0xFFEFF47D, 15);
    b = transform(tr_i, b, c, d, a, words[1], 0x85845DD1, 21);

    a = transform(tr_i, a, b, c, d, words[8], 0x6FA87E4F, 6);
    d = transform(tr_i, d, a, b, c, words[15], 0xFE2CE6E0, 10);
    c = transform(tr_i, c, d, a, b, words[6], 0xA3014314, 15);
    b = transform(tr_i, b, c, d, a, words[13], 0x4E0811A1, 21);

    a = transform(tr_i, a, b, c, d, words[4], 0xF7537E82, 6);
    d = transform(tr_i, d, a, b, c, words[11], 0xBD3AF235, 10);
    c = transform(tr_i, c, d, a, b, words[2], 0x2AD7D2BB, 15);
    b = transform(tr_i, b, c, d, a, words[9], 0xEB86D391, 21);

    return InternalState {
        a: state.a.wrapping_add(a),
        b: state.b.wrapping_add(b),
        c: state.c.wrapping_add(c),
        d: state.d.wrapping_add(d),
    };
}

impl From<[u32; 4]> for InternalState {
    fn from(state: [u32; 4]) -> Self {
        InternalState {
            a: state[0],
            b: state[1],
            c: state[2],
            d: state[3],
        }
    }
}

impl Into<[u32; 4]> for InternalState {
    fn into(self) -> [u32; 4] {
        [self.a, self.b, self.c, self.d]
    }
}

#[derive(Debug, Clone)]
pub struct Collision {
    pub m0: [u32; 16],
    pub m1: [u32; 16],
    pub m0_prim: [u32; 16],
    pub m1_prim: [u32; 16],
    pub hash: [u32; 4],
}

impl Display for Collision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m0: [")?;
        for i in 0..16 {
            if i != 15 {
                write!(f, "{:#x}, ", self.m0[i])?;
            } else {
                write!(f, "{:#x}],\n", self.m0[i])?;
            }
        }

        write!(f, "m1: [")?;
        for i in 0..16 {
            if i != 15 {
                write!(f, "{:#x}, ", self.m1[i])?;
            } else {
                write!(f, "{:#x}],\n", self.m1[i])?;
            }
        }

        write!(f, "m0_prim: [")?;
        for i in 0..16 {
            if i != 15 {
                write!(f, "{:#x}, ", self.m0_prim[i])?;
            } else {
                write!(f, "{:#x}],\n", self.m0_prim[i])?;
            }
        }

        write!(f, "m1_prim: [")?;
        for i in 0..16 {
            if i != 15 {
                write!(f, "{:#x}, ", self.m1_prim[i])?;
            } else {
                write!(f, "{:#x}],\n", self.m1_prim[i])?;
            }
        }

        write!(
            f,
            "hash: [{:#x}, {:#x}, {:#x}, {:#x}]",
            self.hash[0], self.hash[1], self.hash[2], self.hash[3]
        )
    }
}

pub fn second_step(m0: [u32; 16], m0_prim: [u32; 16]) -> Collision {
    let mut delta_m0: [u32; 16] = [0; 16];

    for i in 0..16 {
        delta_m0[i] = m0_prim[i] ^ m0[i];
    }

    assert!(delta_m0 == M0_DELTA);

    let state_m0 = hash(&INITIAL_STATE, &m0);
    let state_m0_prim = hash(&INITIAL_STATE, &m0_prim);

    let mut count: usize = 0;
    let start = Instant::now();

    const NTHREADS: usize = 12;

    let mut children = vec![];

    let shoudl_stop = Arc::new(AtomicBool::new(false));

    let result = Arc::new(Mutex::new(Collision {
        m0: [0; 16],
        m1: [0; 16],
        m0_prim: [0; 16],
        m1_prim: [0; 16],
        hash: [0; 4],
    }));

    for i in 0..NTHREADS {
        let result = Arc::clone(&result);
        let shoudl_stop = shoudl_stop.clone();

        children.push(thread::spawn(move || loop {
            let candidate = random();

            if let Some(candidate) = validate_candiate(&state_m0, candidate) {
                print!(
                    "[{:#?}] Candidate found by worker {} after {} iterations:\n[",
                    start.elapsed(),
                    i,
                    count,
                );
                for i in 0..16 {
                    if i != 15 {
                        print!("{:#x}, ", candidate[i]);
                    } else {
                        print!("{:#x}]\n", candidate[i]);
                    }
                }

                let candidate_prim: [u32; 16] = candidate
                    .iter()
                    .zip(delta_m0.iter())
                    .map(|(x, y)| x ^ y)
                    .collect::<Vec<u32>>()
                    .try_into()
                    .unwrap();

                if hash(&state_m0, &candidate) == hash(&state_m0_prim, &candidate_prim) {
                    println!(
                        "[{:#?}] Collision found by worker {} after {} iterations",
                        start.elapsed(),
                        i,
                        count
                    );
                    println!("[{:#?}] Closing all threads...", start.elapsed());
                    let mut result = result.lock().unwrap();
                    *result = Collision {
                        m0,
                        m1: candidate,
                        m0_prim,
                        m1_prim: candidate_prim,
                        hash: state_m0,
                    };
                    shoudl_stop.store(true, Ordering::Relaxed);
                    break;
                }
            }

            count += 1;
            if count % 100000000 == 0 {
                if shoudl_stop.load(Ordering::Relaxed) {
                    break;
                }
            }
        }))
    }

    for child in children {
        child.join().unwrap();
    }

    let result = result.lock().unwrap();

    return result.clone();
}

pub fn hash(state: &[u32; 4], input: &[u32]) -> [u32; 4] {
    let mut state = InternalState::from(*state);

    let binding = input
        .iter()
        .flat_map(|x| x.to_le_bytes().to_vec())
        .collect::<Vec<u8>>();
    binding.as_slice();

    let mut reader = BufReader::new(binding.as_slice());

    let mut buf = [0; BLOCK_SIZE];
    let mut count = 0;

    // https://www.ietf.org/rfc/rfc1321.txt

    loop {
        // read block
        let res = reader.read(&mut buf).unwrap();

        if res == 0 {
            break;
        }

        count += res;

        // this is the last block
        if res != BLOCK_SIZE {
            // how many bytes are missing from a complete 64 byte
            // multiple?
            let size = 64 - (res as i32 % 64);
            let padding_to_add: usize;

            // we do like this b/c: "as many zeros as are required to
            // bring the length of the message up to 64 bits (8
            // bytes) fewer than a multiple of 512 (64 bytes)"

            if (size - 8) < 0 {
                padding_to_add = size as usize + (64 - 8);
            } else {
                padding_to_add = size as usize - 8;
            }

            let mut padbuf = vec![0; padding_to_add];
            padbuf[0] = 0x80;

            // The remaining bits are filled up with 64 bits
            // representing the length of the original message in
            // *BITS* (not bytes), modulo 2^64.

            let bitsize = Wrapping(count as u64 * 8);
            let orig_size: [u8; 8] = bitsize.0.to_le_bytes();
            let mut orig_size = orig_size.to_vec();
            padbuf.append(&mut orig_size);

            let mut buf2: Vec<u8> = Vec::new();

            buf2.append(&mut buf[0..res].to_vec());
            buf2.append(&mut padbuf.to_vec());

            state = process_block(&mut buf2, state);
        } else {
            state = process_block(&mut buf, state);
        }
    }

    return state.into();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_2_test_1() {
        let m0 = [
            0x2dd31d1, 0xc4eee6c5, 0x69a3d69, 0x5cf9af98, 0x87b5ca2f, 0xab7e4612, 0x3e580440,
            0x897ffbb8, 0x634ad55, 0x2b3f409, 0x8388e483, 0x5a417125, 0xe8255108, 0x9fc9cdf7,
            0xf2bd1dd9, 0x5b3c3780,
        ];

        let m1 = [
            0xd11d0b96, 0x9c7b41dc, 0xf497d8e4, 0xd555655a, 0xc79a7335, 0xcfdebf0, 0x66f12930,
            0x8fb109d1, 0x797f2775, 0xeb5cd530, 0xbaade822, 0x5c15cc79, 0xddcb74ed, 0x6dd3c55f,
            0xd80a9bb1, 0xe3a7cc35,
        ];

        let result = hash(&hash(&INITIAL_STATE, &m0), &m1);

        let m0_prim = [
            0x2dd31d1, 0xc4eee6c5, 0x69a3d69, 0x5cf9af98, 0x7b5ca2f, 0xab7e4612, 0x3e580440,
            0x897ffbb8, 0x634ad55, 0x2b3f409, 0x8388e483, 0x5a41f125, 0xe8255108, 0x9fc9cdf7,
            0x72bd1dd9, 0x5b3c3780,
        ];

        let m1_prim = [
            0xd11d0b96, 0x9c7b41dc, 0xf497d8e4, 0xd555655a, 0x479a7335, 0xcfdebf0, 0x66f12930,
            0x8fb109d1, 0x797f2775, 0xeb5cd530, 0xbaade822, 0x5c154c79, 0xddcb74ed, 0x6dd3c55f,
            0x580a9bb1, 0xe3a7cc35,
        ];

        let result_prim = hash(&hash(&INITIAL_STATE, &m0_prim), &m1_prim);

        assert_eq!(result, result_prim);
        assert_eq!(result, [0x9603161f, 0xa30f9dbf, 0x9f65ffbc, 0xf41fc7ef]);
    }

    #[test]
    fn table_2_test_2() {
        let m0 = [
            0x2dd31d1, 0xc4eee6c5, 0x69a3d69, 0x5cf9af98, 0x87b5ca2f, 0xab7e4612, 0x3e580440,
            0x897ffbb8, 0x634ad55, 0x2b3f409, 0x8388e483, 0x5a417125, 0xe8255108, 0x9fc9cdf7,
            0xf2bd1dd9, 0x5b3c3780,
        ];

        let m1 = [
            0x313e82d8, 0x5b8f3456, 0xd4ac6dae, 0xc619c936, 0xb4e253dd, 0xfd03da87, 0x6633902,
            0xa0cd48d2, 0x42339fe9, 0xe87e570f, 0x70b654ce, 0x1e0da880, 0xbc2198c6, 0x9383a8b6,
            0x2b65f996, 0x702af76f,
        ];

        let result = hash(&hash(&INITIAL_STATE, &m0), &m1);

        let m0_prim = [
            0x2dd31d1, 0xc4eee6c5, 0x69a3d69, 0x5cf9af98, 0x7b5ca2f, 0xab7e4612, 0x3e580440,
            0x897ffbb8, 0x634ad55, 0x2b3f409, 0x8388e483, 0x5a41f125, 0xe8255108, 0x9fc9cdf7,
            0x72bd1dd9, 0x5b3c3780,
        ];

        let m1_prim = [
            0x313e82d8, 0x5b8f3456, 0xd4ac6dae, 0xc619c936, 0x34e253dd, 0xfd03da87, 0x6633902,
            0xa0cd48d2, 0x42339fe9, 0xe87e570f, 0x70b654ce, 0x1e0d2880, 0xbc2198c6, 0x9383a8b6,
            0xab65f996, 0x702af76f,
        ];

        let result_prim = hash(&hash(&INITIAL_STATE, &m0_prim), &m1_prim);

        assert_eq!(result, result_prim);
        assert_eq!(result, [0x8d5e7019, 0x61804e08, 0x715d6b58, 0x6324c015]);
    }

    #[test]
    fn validate_candiate_test() {
        let m0 = [
            0x2dd31d1, 0xc4eee6c5, 0x69a3d69, 0x5cf9af98, 0x87b5ca2f, 0xab7e4612, 0x3e580440,
            0x897ffbb8, 0x634ad55, 0x2b3f409, 0x8388e483, 0x5a417125, 0xe8255108, 0x9fc9cdf7,
            0xf2bd1dd9, 0x5b3c3780,
        ];

        let m0_prim = [
            0x2dd31d1, 0xc4eee6c5, 0x69a3d69, 0x5cf9af98, 0x7b5ca2f, 0xab7e4612, 0x3e580440,
            0x897ffbb8, 0x634ad55, 0x2b3f409, 0x8388e483, 0x5a41f125, 0xe8255108, 0x9fc9cdf7,
            0x72bd1dd9, 0x5b3c3780,
        ];

        let m1_prim = [
            0x313e82d8, 0x5b8f3456, 0xd4ac6dae, 0xc619c936, 0x34e253dd, 0xfd03da87, 0x6633902,
            0xa0cd48d2, 0x42339fe9, 0xe87e570f, 0x70b654ce, 0x1e0d2880, 0xbc2198c6, 0x9383a8b6,
            0xab65f996, 0x702af76f,
        ];

        let m1 = [
            0x313e82d8, 0x5b8f3456, 0xd4ac6dae, 0xc619c936, 0xb4e253dd, 0xfd03da87, 0x6633902,
            0xa0cd48d2, 0x42339fe9, 0xe87e570f, 0x70b654ce, 0x1e0da880, 0xbc2198c6, 0x9383a8b6,
            0x2b65f996, 0x702af76f,
        ];

        let state_m0 = hash(&INITIAL_STATE, &m0);

        let state_m0_prim = hash(&INITIAL_STATE, &m0_prim);

        let modified_candidate = validate_candiate(&state_m0, m1).unwrap();

        assert_eq!(modified_candidate, m1);

        let candidate_prim: [u32; 16] = m1
            .iter()
            .zip(M0_DELTA.iter())
            .map(|(x, y)| x ^ y)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();

        assert_eq!(candidate_prim, m1_prim);

        assert_eq!(hash(&state_m0, &m1), hash(&state_m0_prim, &candidate_prim));
    }
}
