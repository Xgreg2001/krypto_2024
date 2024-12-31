use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use clap::{Parser, Subcommand};
use diffie_hellman::field::f2_poly::F2PolynomialElement;
use diffie_hellman::field::fp::FpElement;
use diffie_hellman::field::fp_poly::FpPolynomialElement;
use diffie_hellman::{FieldContext, FieldElement};
use num::bigint::{RandBigInt, Sign, ToBigInt, ToBigUint};
use num::{BigInt, BigUint, One};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

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
    /// Validate solution with service provided by the university
    Validate,
    /// Submit solution to the service provided by the university
    Solution,
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
        Some(Commands::Validate) => validate_solution(),
        Some(Commands::Solution) => submit_solution(),
        None => {
            println!("usege: diffie-hellman [SUBCOMMAND]");
        }
    }
}

fn submit_solution() {
    let base_url = "https://crypto24.random-oracle.xyz/";
    let student_id = "10000000000000000000000000000033";
    let url = format!("{}submit/list2/{}/solution", base_url, student_id);

    let client = Client::new();

    let response = client.get(&url).send().unwrap();

    if !response.status().is_success() {
        println!("Failed to fetch challenge");
        return;
    } else {
        println!("Fetched challenge");
    }

    let response: SubmissionChallengeResponse = response.json().unwrap();

    let session_id = response.session_id;

    println!("Calcualting for modp");
    let (modp_public, modp_shared) = solution_fp(&response.modp_params, &response.modp_challenge);
    println!("Calcualting for fpk");
    let (fpk_public, fpk_shared) = solution_fp_poly(&response.fpk_params, &response.fpk_challenge);
    println!("Calcualting for f2m");
    let (f2m_public, f2m_shared) = solution_f2_poly(&response.f2m_params, &response.f2m_challenge);

    let request = SubmissionResponseRequest {
        session_id: session_id.clone(),
        modp: ChallangeResponse {
            status: "success".to_string(),
            public: modp_public,
            shared: modp_shared,
        },
        fpk: ChallangeFpkResponse {
            status: "success".to_string(),
            public: fpk_public,
            shared: fpk_shared,
        },
        f2m: ChallangeResponse {
            status: "success".to_string(),
            public: f2m_public,
            shared: f2m_shared,
        },
    };

    println!("Submitting solution");

    let response = client.post(&url).json(&request).send().unwrap();

    println!("Got response");

    if !response.status().is_success() {
        println!("Failed to submit solution");
        return;
    }

    let response: SubmissionResponseResponse = response.json().unwrap();

    if response.modp.status == "success" {
        println!("Modp solution is correct");
    } else {
        println!("Failed modp verification");
    }

    if response.fpk.status == "success" {
        println!("Fpk solution is correct");
    } else {
        println!("Failed fpk verification");
    }

    if response.f2m.status == "success" {
        println!("F2m solution is correct");
    } else {
        println!("Failed f2m verification");
    }
}

fn solution_f2_poly(params: &F2mParams, challenge: &ChallageRequest) -> (String, String) {
    let extension = params.extension;
    let irreducible_poly = decode_base64_biguint_le(&params.modulus);

    // add most significant bit
    let irreducible_poly = irreducible_poly | (BigUint::one() << extension);

    let g = decode_base64_biguint_le(&params.generator);
    let order = decode_base64_biguint(&params.order);

    let ctx = FieldContext::new_binary(irreducible_poly);
    let g = F2PolynomialElement::new(&ctx, g);

    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = g.pow(&a);

    let b_pub = decode_base64_biguint_le(&challenge.public);
    let b_pub = F2PolynomialElement::new(&ctx, b_pub);

    let a_shared = b_pub.pow(&a);

    (
        encode_base64_biguint_le(&a_pub.coeffs),
        encode_base64_biguint_le(&a_shared.coeffs),
    )
}

fn solution_fp_poly(
    params: &FpkParams,
    challenge: &ChallageFpkRequest,
) -> (Vec<String>, Vec<String>) {
    let p = decode_base64(&params.prime_base);
    let mut irreducible_poly = params
        .modulus
        .iter()
        .map(|x| decode_base64(x))
        .collect::<Vec<BigInt>>();

    // add 1 at the end of vector
    irreducible_poly.push(BigInt::one());

    let ctx = FieldContext::new_poly(p, irreducible_poly);

    let g = params
        .generator
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();

    let order = decode_base64_biguint(&params.order);

    let g = FpPolynomialElement::new(&ctx, g);

    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = g.pow(&a);

    let b_pub = challenge
        .public
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let b_pub = FpPolynomialElement::new(&ctx, b_pub);

    let a_shared = b_pub.pow(&a);

    (
        a_pub.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
        a_shared
            .coeffs
            .iter()
            .map(|x| encode_base64(&x.val))
            .collect(),
    )
}

fn solution_fp(params: &FpParams, challenge: &ChallageRequest) -> (String, String) {
    let p = decode_base64(&params.modulus);
    let g = decode_base64(&params.generator);
    let order = decode_base64_biguint(&params.order);

    let ctx = FieldContext::new_prime(p.clone());
    let g = FpElement::new(&ctx, g);

    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = g.pow(&a);

    let b_pub = decode_base64(&challenge.public);
    let b_pub = FpElement::new(&ctx, b_pub);

    let a_shared = b_pub.pow(&a);

    (encode_base64(&a_pub.val), encode_base64(&a_shared.val))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionChallengeResponse {
    status: String,
    session_id: String,
    timeout: String,
    modp_params: FpParams,
    modp_challenge: ChallageRequest,
    fpk_params: FpkParams,
    fpk_challenge: ChallageFpkRequest,
    f2m_params: F2mParams,
    f2m_challenge: ChallageRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionResponseRequest {
    session_id: String,
    modp: ChallangeResponse,
    fpk: ChallangeFpkResponse,
    f2m: ChallangeResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Status {
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionResponseResponse {
    status: String,
    modp: Status,
    fpk: Status,
    f2m: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamFpResponse {
    status: String,
    r#type: String,
    params: FpParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamFpkREsponse {
    status: String,
    r#type: String,
    params: FpkParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamF2mResponse {
    status: String,
    r#type: String,
    params: F2mParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FpkParams {
    name: String,
    prime_base: String,
    extension: i32,
    modulus: Vec<String>,
    generator: Vec<String>,
    order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct F2mParams {
    name: String,
    modulus: String,
    generator: String,
    order: String,
    extension: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FpParams {
    name: String,
    modulus: String,
    generator: String,
    order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChallageRequest {
    public: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChallangeResponse {
    status: String,
    public: String,
    shared: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChallageFpkRequest {
    public: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChallangeFpkResponse {
    status: String,
    public: Vec<String>,
    shared: Vec<String>,
}

fn validate_solution_fp(base_url: &str, client: &Client) {
    let params = client
        .get(format!("{}/validate/list2/modp/param", base_url))
        .send()
        .unwrap();

    if !params.status().is_success() {
        println!("Failed to fetch parameters for F_p");
        return;
    }

    let params: ParamFpResponse = params.json().unwrap();

    if params.r#type != "modp" {
        println!("Invalid type of parameters");
        return;
    }

    let p = decode_base64(&params.params.modulus);
    let g = decode_base64(&params.params.generator);
    let order = decode_base64_biguint(&params.params.order);

    let ctx = FieldContext::new_prime(p.clone());
    let g = FpElement::new(&ctx, g);

    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = g.pow(&a);

    let challange = ChallageRequest {
        public: encode_base64(&a_pub.val),
    };

    let challange = client
        .post(format!("{}/validate/list2/modp/challenge", base_url))
        .json(&challange)
        .send()
        .unwrap();

    if !challange.status().is_success() {
        println!("Failed to send challange");
        return;
    }

    let challange: ChallangeResponse = challange.json().unwrap();

    let b_pub = decode_base64(&challange.public);
    let b_pub = FpElement::new(&ctx, b_pub);
    let b_shared = decode_base64(&challange.shared);
    let b_shared = FpElement::new(&ctx, b_shared);

    let a_shared = b_pub.pow(&a);

    assert_eq!(a_shared, b_shared);

    println!("Fp Solution is correct");
}

fn validate_solution() {
    let base_url = "https://crypto24.random-oracle.xyz/";
    let client = Client::new();

    validate_solution_fp(base_url, &client);
    validate_solution_f2_poly(base_url, &client);
    validate_solution_fp_poly(base_url, &client);
}

fn validate_solution_fp_poly(base_url: &str, client: &Client) {
    let params = client
        .get(format!("{}/validate/list2/fpk/param", base_url))
        .send()
        .unwrap();

    if !params.status().is_success() {
        println!("Failed to fetch parameters for F_p^k");
        return;
    }

    let params: ParamFpkREsponse = params.json().unwrap();

    if params.r#type != "fpk" {
        println!("Invalid type of parameters");
        return;
    }

    // let extension = params.params.extension;

    let p = decode_base64(&params.params.prime_base);
    let mut irreducible_poly = params
        .params
        .modulus
        .iter()
        .map(|x| decode_base64(x))
        .collect::<Vec<BigInt>>();

    // add 1 at the end of vector
    irreducible_poly.push(BigInt::one());

    let ctx = FieldContext::new_poly(p, irreducible_poly);

    let g = params
        .params
        .generator
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();

    let order = decode_base64_biguint(&params.params.order);

    let g = FpPolynomialElement::new(&ctx, g);

    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = g.pow(&a);

    let challange = ChallageFpkRequest {
        public: a_pub.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
    };

    let challange = client
        .post(format!("{}/validate/list2/fpk/challenge", base_url))
        .json(&challange)
        .send()
        .unwrap();

    if !challange.status().is_success() {
        println!("Failed to send challange");
        println!("{:?}", challange.text());
        return;
    }

    let challange: ChallangeFpkResponse = challange.json().unwrap();

    let b_pub = challange
        .public
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let b_pub = FpPolynomialElement::new(&ctx, b_pub);
    let b_shared = challange
        .shared
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let b_shared = FpPolynomialElement::new(&ctx, b_shared);

    let a_shared = b_pub.pow(&a);

    assert_eq!(a_shared, b_shared);

    println!("Fpk Solution is correct");
}

fn validate_solution_f2_poly(base_url: &str, client: &Client) {
    let params = client
        .get(format!("{}/validate/list2/f2m/param", base_url))
        .send()
        .unwrap();

    if !params.status().is_success() {
        println!("Failed to fetch parameters for F_2^m");
        return;
    }

    let params: ParamF2mResponse = params.json().unwrap();

    if params.r#type != "f2m" {
        println!("Invalid type of parameters");
        return;
    }

    let extension = params.params.extension;
    let irreducible_poly = decode_base64_biguint_le(&params.params.modulus);

    // add most significant bit
    let irreducible_poly = irreducible_poly | (BigUint::one() << extension);

    let g = decode_base64_biguint_le(&params.params.generator);
    let order = decode_base64_biguint(&params.params.order);

    let ctx = FieldContext::new_binary(irreducible_poly);
    let g = F2PolynomialElement::new(&ctx, g);

    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = g.pow(&a);

    let challange = ChallageRequest {
        public: encode_base64_biguint_le(&a_pub.coeffs),
    };

    let challange = client
        .post(format!("{}/validate/list2/f2m/challenge", base_url))
        .json(&challange)
        .send()
        .unwrap();

    if !challange.status().is_success() {
        println!("Failed to send challange");
        println!("{:?}", challange.text());
        return;
    }

    let challange: ChallangeResponse = challange.json().unwrap();

    let b_pub = decode_base64_biguint_le(&challange.public);
    let b_pub = F2PolynomialElement::new(&ctx, b_pub);
    let b_shared = decode_base64_biguint_le(&challange.shared);
    let b_shared = F2PolynomialElement::new(&ctx, b_shared);

    let a_shared = b_pub.pow(&a);

    assert_eq!(a_shared, b_shared);

    println!("F2m Solution is correct");
}

fn decode_base64(s: &str) -> BigInt {
    let s = URL_SAFE_NO_PAD.decode(s.as_bytes()).unwrap();
    BigInt::from_bytes_be(Sign::Plus, &s)
}

fn decode_base64_biguint(s: &str) -> BigUint {
    let s = URL_SAFE_NO_PAD.decode(s.as_bytes()).unwrap();
    BigUint::from_bytes_be(&s)
}

fn decode_base64_biguint_le(s: &str) -> BigUint {
    let s = URL_SAFE_NO_PAD.decode(s.as_bytes()).unwrap();
    BigUint::from_bytes_le(&s)
}

fn encode_base64(n: &BigInt) -> String {
    let (_, bytes) = n.to_bytes_be();
    URL_SAFE_NO_PAD.encode(&bytes)
}

// fn encode_base64_le(n: &BigInt) -> String {
//     let (_, bytes) = n.to_bytes_le();
//     URL_SAFE_NO_PAD.encode(&bytes)
// }

// fn encode_base64_biguint(n: &BigUint) -> String {
//     let bytes = n.to_bytes_be();
//     URL_SAFE_NO_PAD.encode(&bytes)
// }

fn encode_base64_biguint_le(n: &BigUint) -> String {
    let bytes = n.to_bytes_le();
    URL_SAFE_NO_PAD.encode(&bytes)
}

fn diffie_hellman_fp() {
    let p = BigInt::parse_bytes(b"7441601072810284702464629351659524507907489347397523425173826419365612833915210446029303962789322887", 10).unwrap();
    let ctx = FieldContext::new_prime(p.clone());

    let g = FpElement::new(&ctx, 5.to_bigint().unwrap());
    let order = BigUint::parse_bytes(b"7441601072810284702464629351659524507907489347397523425173826419365612833915210446029303962789322886", 10).unwrap();

    println!("p: {}", p);
    println!("g: {}", g);
    println!("order: {}", order);

    assert_eq!(
        g.pow_secure(&order, &order),
        FpElement::new(&ctx, BigInt::one())
    );

    // random BigUint number from 2..order
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
    let b = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    println!("a: {}", a);
    println!("b: {}", b);

    let a_pub = g.pow_secure(&a, &order);
    let b_pub = g.pow_secure(&b, &order);

    println!("A pub (g^a): {}", a_pub);
    println!("B pub (g^b): {}", b_pub);

    let a_secret = b_pub.pow_secure(&a, &order);
    let b_secret = a_pub.pow_secure(&b, &order);

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

    assert_eq!(g.pow_secure(&order, &order), FpPolynomialElement::one(&ctx));

    // random BigUint number from 2..order
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
    let b = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    println!("a: {}", a);
    println!("b: {}", b);

    let a_pub = g.pow_secure(&a, &order);
    let b_pub = g.pow_secure(&b, &order);

    println!("A pub (g^a): {}", a_pub);
    println!("B pub (g^b): {}", b_pub);

    let a_secret = b_pub.pow_secure(&a, &order);
    let b_secret = a_pub.pow_secure(&b, &order);

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

    assert_eq!(g.pow_secure(&order, &order), F2PolynomialElement::one(&ctx));

    // random BigUint number from 2..order
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
    let b = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    println!("a: {}", a);
    println!("b: {}", b);

    let a_pub = g.pow_secure(&a, &order);
    let b_pub = g.pow_secure(&b, &order);

    println!("A pub (g^a): {}", a_pub);
    println!("B pub (g^b): {}", b_pub);

    let a_secret = b_pub.pow_secure(&a, &order);
    let b_secret = a_pub.pow_secure(&b, &order);

    println!("A (g^(a+b)): {}", a_secret);
    println!("B (g^(a+b)): {}", b_secret);

    assert_eq!(a_secret, b_secret);
}
