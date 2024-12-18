use md5::{hash, second_step, INITIAL_STATE};

mod md5;

fn main() {
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

    let collision = second_step(m0, m0_prim);

    println!("Collision found:");
    println!("{}", collision);

    // verify the collision
    assert_eq!(
        hash(&hash(&INITIAL_STATE, &collision.m0), &collision.m1),
        hash(
            &hash(&INITIAL_STATE, &collision.m0_prim),
            &collision.m1_prim
        )
    );
}
