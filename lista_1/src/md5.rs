// OF; <p.oscar.franzen@gmail.com>
// https://github.com/oscar-franzen/md5inrust

// A vanilla md5 implementation I used as a way to learn the Rust
// programming language. See README.md for more info on how to
// compile.

use std::io::BufReader;
use std::io::Read;
use std::num::Wrapping;
use std::vec::Vec;

const BLOCK_SIZE: usize = 512;
pub const INITIAL_STATE: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];

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

    //  round 1, apply on all 16 words
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

    // round 2, apply on all 16 words
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

    // round 3, apply on all 16 words
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

    // round 4, apply on all 16 words
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

pub fn hash(state: [u32; 4], input: &[u32]) -> [u32; 4] {
    let mut state = InternalState::from(state);

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

        let result = hash(hash(INITIAL_STATE, &m0), &m1);

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

        let result_prim = hash(hash(INITIAL_STATE, &m0_prim), &m1_prim);

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

        let result = hash(hash(INITIAL_STATE, &m0), &m1);

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

        let result_prim = hash(hash(INITIAL_STATE, &m0_prim), &m1_prim);

        assert_eq!(result, result_prim);
        assert_eq!(result, [0x8d5e7019, 0x61804e08, 0x715d6b58, 0x6324c015]);
    }
}
