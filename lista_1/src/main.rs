mod md5;

fn main() {
    let a: u32 = 0x67452301;
    let b: u32 = 0xefcdab89;
    let c: u32 = 0x98badcfe;
    let d: u32 = 0x10325476;

    let initial_state: [u32; 4] = [a, b, c, d];

    let m0: &[u32] = &[
        0x2dd31d1, 0xc4eee6c5, 0x69a3d69, 0x5cf9af98, 0x87b5ca2f, 0xab7e4612, 0x3e580440,
        0x897ffbb8, 0x634ad55, 0x2b3f409, 0x8388e483, 0x5a417125, 0xe8255108, 0x9fc9cdf7,
        0xf2bd1dd9, 0x5b3c3780,
    ];

    let m1: &[u32] = &[
        0xd11d0b96, 0x9c7b41dc, 0xf497d8e4, 0xd555655a, 0xc79a7335, 0xcfdebf0, 0x66f12930,
        0x8fb109d1, 0x797f2775, 0xeb5cd530, 0xbaade822, 0x5c15cc79, 0xddcb74ed, 0x6dd3c55f,
        0xd80a9bb1, 0xe3a7cc35,
    ];

    let result = md5::hash(md5::hash(initial_state, m0), m1);

    println!(
        "{:#x} {:#x} {:#x} {:#x}",
        result[0], result[1], result[2], result[3]
    );

    let m0_prim: &[u32] = &[
        0x2dd31d1, 0xc4eee6c5, 0x69a3d69, 0x5cf9af98, 0x7b5ca2f, 0xab7e4612, 0x3e580440,
        0x897ffbb8, 0x634ad55, 0x2b3f409, 0x8388e483, 0x5a41f125, 0xe8255108, 0x9fc9cdf7,
        0x72bd1dd9, 0x5b3c3780,
    ];

    let m1_prim: &[u32] = &[
        0xd11d0b96, 0x9c7b41dc, 0xf497d8e4, 0xd555655a, 0x479a7335, 0xcfdebf0, 0x66f12930,
        0x8fb109d1, 0x797f2775, 0xeb5cd530, 0xbaade822, 0x5c154c79, 0xddcb74ed, 0x6dd3c55f,
        0x580a9bb1, 0xe3a7cc35,
    ];

    let result_prim = md5::hash(md5::hash(initial_state, m0_prim), m1_prim);

    println!(
        "{:#x} {:#x} {:#x} {:#x}",
        result_prim[0], result_prim[1], result_prim[2], result_prim[3]
    );
}
