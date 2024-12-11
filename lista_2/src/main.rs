use diffie_hellman::field::fp::FpElement;
use diffie_hellman::{FieldContext, FieldElement};
use num::bigint::ToBigInt;
use num::BigUint;

// TODO: DIFFIE-HELLMAN
// TODO: special implementation for p = 2
// TODO: fix exponentiation to ensure constant time operation
// TODO: check with bigger numbers

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

fn main() {
    let p = 17.to_bigint().unwrap();
    let ctx = FieldContext::new_prime(p);

    let a = FpElement::new(&ctx, 7.to_bigint().unwrap());
    let exp_ones = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", 16).unwrap();
    let exp_zeros = BigUint::parse_bytes(b"800000000000000000000000000000", 16).unwrap();
    let exp_half = BigUint::parse_bytes(b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", 16).unwrap();
    let exp_quarter = BigUint::parse_bytes(b"888888888888888888888888888888", 16).unwrap();

    const RUNS: u32 = 100000;

    let mut times_ones = Vec::with_capacity(RUNS as usize);
    let mut times_zeros = Vec::with_capacity(RUNS as usize);
    let mut times_half = Vec::with_capacity(RUNS as usize);
    let mut times_quarter = Vec::with_capacity(RUNS as usize);

    for _ in 0..RUNS {
        let now = std::time::Instant::now();
        let _ = a.pow(&exp_ones);
        times_ones.push(now.elapsed());

        let now = std::time::Instant::now();
        let _ = a.pow(&exp_zeros);
        times_zeros.push(now.elapsed());

        let now = std::time::Instant::now();
        let _ = a.pow(&exp_half);
        times_half.push(now.elapsed());

        let now = std::time::Instant::now();
        let _ = a.pow(&exp_quarter);
        times_quarter.push(now.elapsed());
    }

    let ones_nanos_avg = calcualte_average(&times_ones);
    let zeros_nanos_avg = calcualte_average(&times_zeros);
    let half_nanos_avg = calcualte_average(&times_half);
    let quarter_nanos_avg = calcualte_average(&times_quarter);

    let ones_nanos_stddev = calculate_stddev(&times_ones, ones_nanos_avg);
    let zeros_nanos_stddev = calculate_stddev(&times_zeros, zeros_nanos_avg);
    let half_nanos_stddev = calculate_stddev(&times_half, half_nanos_avg);
    let quarter_nanos_stddev = calculate_stddev(&times_quarter, quarter_nanos_avg);

    println!("Ones: {}ns ± {}ns", ones_nanos_avg, ones_nanos_stddev);
    println!("Zeros: {}ns ± {}ns", zeros_nanos_avg, zeros_nanos_stddev);
    println!("Half: {}ns ± {}ns", half_nanos_avg, half_nanos_stddev);
    println!(
        "Quarter: {}ns ± {}ns",
        quarter_nanos_avg, quarter_nanos_stddev
    );
}
