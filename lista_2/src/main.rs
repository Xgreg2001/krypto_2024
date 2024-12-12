use clap::{Parser, Subcommand};
use diffie_hellman::field::f2_poly::F2PolynomialElement;
use diffie_hellman::field::fp::FpElement;
use diffie_hellman::field::fp_poly::FpPolynomialElement;
use diffie_hellman::{FieldContext, FieldElement};
use num::bigint::{RandBigInt, ToBigInt, ToBigUint};
use num::{BigInt, BigUint, One};

// TODO: fix exponentiation to ensure constant time operation

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// demo of the security of the implemented field elements
    Security,
    /// Diffie-Hellman key exchange demo using Fp field
    Fp,
    /// Diffie-Hellman key exchange demo using F2Poly field
    F2Poly,
    /// Diffie-Hellman key exchange demo using FpPoly field
    FpPoly,
}

fn calcualte_average(times: &[std::time::Duration]) -> f64 {
    times.iter().map(|x| x.as_nanos() as f64).sum::<f64>() / times.len() as f64
}

fn calculate_stddev(times: &[std::time::Duration], avg: f64) -> f64 {
    times
        .iter()
        .map(|x| (x.as_nanos() as f64 - avg).powi(2))
        .sum::<f64>()
        .sqrt()
        / times.len() as f64
}

fn showcase_security<'a, F: FieldElement<'a>>(a: F, order: &BigUint, title: &str) {
    let exp_ones = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", 16).unwrap();
    let exp_zeros = BigUint::parse_bytes(b"800000000000000000000000000000", 16).unwrap();
    let exp_half = BigUint::parse_bytes(b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", 16).unwrap();
    let exp_quarter = BigUint::parse_bytes(b"888888888888888888888888888888", 16).unwrap();
    let exp_short = BigUint::parse_bytes(b"1", 16).unwrap();
    let exp_long = order.clone() - BigUint::one();

    const RUNS: u32 = 1000;

    let mut times_ones = Vec::with_capacity(RUNS as usize);
    let mut times_zeros = Vec::with_capacity(RUNS as usize);
    let mut times_half = Vec::with_capacity(RUNS as usize);
    let mut times_quarter = Vec::with_capacity(RUNS as usize);
    let mut times_short = Vec::with_capacity(RUNS as usize);
    let mut times_long = Vec::with_capacity(RUNS as usize);

    for _ in 0..RUNS {
        let now = std::time::Instant::now();
        let _ = a.pow_secure(&exp_ones, order);
        times_ones.push(now.elapsed());

        let now = std::time::Instant::now();
        let _ = a.pow_secure(&exp_zeros, order);
        times_zeros.push(now.elapsed());

        let now = std::time::Instant::now();
        let _ = a.pow_secure(&exp_half, order);
        times_half.push(now.elapsed());

        let now = std::time::Instant::now();
        let _ = a.pow_secure(&exp_quarter, order);
        times_quarter.push(now.elapsed());

        let now = std::time::Instant::now();
        let _ = a.pow_secure(&exp_short, order);
        times_short.push(now.elapsed());

        let now = std::time::Instant::now();
        let _ = a.pow_secure(&exp_long, order);
        times_long.push(now.elapsed());
    }

    let ones_nanos_avg = calcualte_average(&times_ones);
    let zeros_nanos_avg = calcualte_average(&times_zeros);
    let half_nanos_avg = calcualte_average(&times_half);
    let quarter_nanos_avg = calcualte_average(&times_quarter);
    let short_nanos_avg = calcualte_average(&times_short);
    let long_nanos_avg = calcualte_average(&times_long);

    let ones_nanos_stddev = calculate_stddev(&times_ones, ones_nanos_avg);
    let zeros_nanos_stddev = calculate_stddev(&times_zeros, zeros_nanos_avg);
    let half_nanos_stddev = calculate_stddev(&times_half, half_nanos_avg);
    let quarter_nanos_stddev = calculate_stddev(&times_quarter, quarter_nanos_avg);
    let short_nanos_stddev = calculate_stddev(&times_short, short_nanos_avg);
    let long_nanos_stddev = calculate_stddev(&times_long, long_nanos_avg);

    println!("{title}");

    println!("Ones: {}ns ± {}ns", ones_nanos_avg, ones_nanos_stddev);
    println!("Zeros: {}ns ± {}ns", zeros_nanos_avg, zeros_nanos_stddev);
    println!("Half: {}ns ± {}ns", half_nanos_avg, half_nanos_stddev);
    println!(
        "Quarter: {}ns ± {}ns",
        quarter_nanos_avg, quarter_nanos_stddev
    );
    println!("Short: {}ns ± {}ns", short_nanos_avg, short_nanos_stddev);
    println!("Long: {}ns ± {}ns", long_nanos_avg, long_nanos_stddev);
}

fn security_demo() {
    let p = 17.to_bigint().unwrap();
    let ctx = FieldContext::new_prime(p);

    let a = FpElement::new(&ctx, 7.to_bigint().unwrap());
    let order = 16.to_biguint().unwrap();

    showcase_security(a, &order, "FP");

    let p = 11.to_bigint().unwrap();
    let irreducible_poly = vec![
        1.to_bigint().unwrap(),
        0.to_bigint().unwrap(),
        5.to_bigint().unwrap(),
        3.to_bigint().unwrap(),
        1.to_bigint().unwrap(),
        4.to_bigint().unwrap(),
        4.to_bigint().unwrap(),
        1.to_bigint().unwrap(),
    ];
    let ctx = FieldContext::new_poly(p, irreducible_poly);

    let a = FpPolynomialElement::from_vec(&ctx, vec![8, 6, 7, 7, 3, 9, 1]);

    let order = BigUint::from(19487170u32);

    showcase_security(a, &order, "FP POLY");

    let irreducible_poly = BigUint::from(0b11111101111101001u64);
    let ctx = FieldContext::new_binary(irreducible_poly);

    let a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));

    let order = BigUint::from(65535u32);

    showcase_security(a, &order, "F2 POLY");
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Security) => security_demo(),
        Some(Commands::Fp) => diffie_hellman_fp(),
        Some(Commands::F2Poly) => diffie_hellman_f2_poly(),
        Some(Commands::FpPoly) => diffie_hellman_fp_poly(),
        None => {
            println!("usege: diffie-hellman [SUBCOMMAND]");
        }
    }
}

fn diffie_hellman_fp() {
    let p = BigInt::parse_bytes(b"7441601072810284702464629351659524507907489347397523425173826419365612833915210446029303962789322887", 10).unwrap();
    let ctx = FieldContext::new_prime(p.clone());

    let g = FpElement::new(&ctx, 5.to_bigint().unwrap());
    let order = BigUint::parse_bytes(b"7441601072810284702464629351659524507907489347397523425173826419365612833915210446029303962789322886", 10).unwrap();

    println!("p: {}", p);
    println!("g: {}", g);
    println!("order: {}", order);

    assert_eq!(g.pow(&order), FpElement::new(&ctx, BigInt::one()));

    // random BigUint number from 2..order
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
    let b = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    println!("a: {}", a);
    println!("b: {}", b);

    let a_pub = g.pow(&a);
    let b_pub = g.pow(&b);

    println!("A pub (g^a): {}", a_pub);
    println!("B pub (g^b): {}", b_pub);

    let a_secret = b_pub.pow(&a);
    let b_secret = a_pub.pow(&b);

    println!("A (g^(a+b)): {}", a_secret);
    println!("B (g^(a+b)): {}", b_secret);

    assert_eq!(a_secret, b_secret);
}

fn diffie_hellman_fp_poly() {
    let p = 956440951.to_bigint().unwrap();
    let irreducible_poly = vec![
        870405164, 849669533, 857776505, 194197415, 698283868, 56845073, 580160882, 410944103,
        277136257, 598011485, 92018354, 454300110, 25548110, 555642896, 905674278, 252167097,
        645335607, 247316570, 117561546, 45003846, 61805098, 924267459, 622128058, 813265651,
        713198838, 463218131, 780254748, 718481820, 191783787, 713868974, 224922610, 571695487,
        318905839, 724882771, 682755204, 719382678, 378874868, 463947519, 405051462, 74997978, 1,
    ]
    .iter()
    .map(|x| x.to_bigint().unwrap())
    .collect();
    let ctx = FieldContext::new_poly(p.clone(), irreducible_poly);

    let g_poly = vec![
        605205470, 603884639, 622982662, 865041543, 924786951, 162565113, 647335234, 475891684,
        140476253, 553939455, 617185216, 541859025, 375619309, 520559687, 319033389, 450196860,
        314686575, 264428075, 756589791, 439582919, 429462603, 520106441, 453937294, 317104706,
        518717773, 414086629, 376055902, 723505750, 178387900, 511637819, 681079010, 473370811,
        16171561, 787899355, 37368526, 625147772, 673373521, 604110264, 324889700, 841618332,
    ];

    let g = FpPolynomialElement::from_vec(&ctx, g_poly);

    let order = BigUint::parse_bytes(b"56131319245648513625028142239596235653949457152504987191810969803633910917513186014554919308448870096832257251256974496525942506764214117222068725355815424564125623695282360199268465508655562031507274967433400673896220498677433514219562456989260331229162896634610953101402415071717075091876248188718857548300952774028191265801870108156642616682308534864696000", 10).unwrap();

    println!("p: {}", p);
    println!("g: {}", g);
    println!("order: {}", order);

    assert_eq!(g.pow(&order), FpPolynomialElement::one(&ctx));

    // random BigUint number from 2..order
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
    let b = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    println!("a: {}", a);
    println!("b: {}", b);

    let a_pub = g.pow(&a);
    let b_pub = g.pow(&b);

    println!("A pub (g^a): {}", a_pub);
    println!("B pub (g^b): {}", b_pub);

    let a_secret = b_pub.pow(&a);
    let b_secret = a_pub.pow(&b);

    println!("A (g^(a+b)): {}", a_secret);
    println!("B (g^(a+b)): {}", b_secret);

    assert_eq!(a_secret, b_secret);
}

fn diffie_hellman_f2_poly() {
    let irreducible_poly = BigUint::from(0b11111101111101001u64);
    let ctx = FieldContext::new_binary(irreducible_poly);

    let g = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));

    let order = BigUint::from(65535u32);

    println!("p: {}", 2);
    println!("g: {}", g);
    println!("order: {}", order);

    assert_eq!(g.pow(&order), F2PolynomialElement::one(&ctx));

    // random BigUint number from 2..order
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
    let b = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    println!("a: {}", a);
    println!("b: {}", b);

    let a_pub = g.pow(&a);
    let b_pub = g.pow(&b);

    println!("A pub (g^a): {}", a_pub);
    println!("B pub (g^b): {}", b_pub);

    let a_secret = b_pub.pow(&a);
    let b_secret = a_pub.pow(&b);

    println!("A (g^(a+b)): {}", a_secret);
    println!("B (g^(a+b)): {}", b_secret);

    assert_eq!(a_secret, b_secret);
}
