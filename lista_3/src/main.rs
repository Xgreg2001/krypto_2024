use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use clap::{Parser, Subcommand};
use diffie_hellman::field::ec::{EllipticCurve, Point};
use diffie_hellman::field::ec_binary::{BinaryEllipticCurve, BinaryPoint};
use diffie_hellman::field::f2_poly::F2PolynomialElement;
use diffie_hellman::field::fp::FpElement;
use diffie_hellman::field::fp_poly::FpPolynomialElement;
use diffie_hellman::{FieldContext, FieldElement};
use num::bigint::{RandBigInt, Sign, ToBigInt};
use num::traits::sign;
use num::{BigInt, BigUint, One};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate solution with service provided by the university
    Validate,
    /// Submit solution to the service provided by the university
    Solution,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate) => validate_solution(),
        // Some(Commands::Solution) => submit_solution(),
        Some(Commands::Solution) => todo!(),
        None => {
            println!("usege: diffie-hellman [SUBCOMMAND]");
        }
    }
}

// fn submit_solution() {
//     let base_url = "https://crypto24.random-oracle.xyz/";
//     let student_id = "10000000000000000000000000000033";
//     let url = format!("{}submit/list3/{}/solution", base_url, student_id);
//
//     let client = Client::new();
//
//     let response = client.get(&url).send().unwrap();
//
//     if !response.status().is_success() {
//         println!("Failed to fetch challenge");
//         return;
//     } else {
//         println!("Fetched challenge");
//     }
//
//     let response: SubmissionChallengeResponse = response.json().unwrap();
//
//     let session_id = response.session_id;
//
//     println!("Calcualting for modp");
//     let (modp_public, modp_shared) = solution_fp(&response.modp_params, &response.modp_challenge);
//     println!("Calcualting for fpk");
//     let (fpk_public, fpk_shared) = solution_fp_poly(&response.fpk_params, &response.fpk_challenge);
//     println!("Calcualting for f2m");
//     let (f2m_public, f2m_shared) = solution_f2_poly(&response.f2m_params, &response.f2m_challenge);
//
//     let request = SubmissionResponseRequest {
//         session_id: session_id.clone(),
//         modp: ChallangeResponse {
//             status: "success".to_string(),
//             public: modp_public,
//             shared: modp_shared,
//         },
//         fpk: ChallangeFpkResponse {
//             status: "success".to_string(),
//             public: fpk_public,
//             shared: fpk_shared,
//         },
//         f2m: ChallangeResponse {
//             status: "success".to_string(),
//             public: f2m_public,
//             shared: f2m_shared,
//         },
//     };
//
//     println!("Submitting solution");
//
//     let response = client.post(&url).json(&request).send().unwrap();
//
//     println!("Got response");
//
//     if !response.status().is_success() {
//         println!("Failed to submit solution");
//         return;
//     }
//
//     let response: SubmissionResponseResponse = response.json().unwrap();
//
//     if response.modp.status == "success" {
//         println!("Modp solution is correct");
//     } else {
//         println!("Failed modp verification");
//     }
//
//     if response.fpk.status == "success" {
//         println!("Fpk solution is correct");
//     } else {
//         println!("Failed fpk verification");
//     }
//
//     if response.f2m.status == "success" {
//         println!("F2m solution is correct");
//     } else {
//         println!("Failed f2m verification");
//     }
// }

// fn solution_f2_poly(params: &F2mParams, challenge: &ChallageRequest) -> (String, String) {
//     let extension = params.extension;
//     let irreducible_poly = decode_base64_biguint_le(&params.modulus);
//
//     // add most significant bit
//     let irreducible_poly = irreducible_poly | (BigUint::one() << extension);
//
//     let g = decode_base64_biguint_le(&params.generator);
//     let order = decode_base64_biguint(&params.order);
//
//     let ctx = FieldContext::new_binary(irreducible_poly);
//     let g = F2PolynomialElement::new(&ctx, g);
//
//     let mut rng = rand::thread_rng();
//     let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
//
//     let a_pub = g.pow(&a);
//
//     let b_pub = decode_base64_biguint_le(&challenge.public);
//     let b_pub = F2PolynomialElement::new(&ctx, b_pub);
//
//     let a_shared = b_pub.pow(&a);
//
//     (
//         encode_base64_biguint_le(&a_pub.coeffs),
//         encode_base64_biguint_le(&a_shared.coeffs),
//     )
// }

// fn solution_fp_poly(
//     params: &ECpkParams,
//     challenge: &ChallageECpkRequest,
// ) -> (Vec<String>, Vec<String>) {
//     let p = decode_base64(&params.prime_base);
//     let mut irreducible_poly = params
//         .modulus
//         .iter()
//         .map(|x| decode_base64(x))
//         .collect::<Vec<BigInt>>();
//
//     // add 1 at the end of vector
//     irreducible_poly.push(BigInt::one());
//
//     let ctx = FieldContext::new_poly(p, irreducible_poly);
//
//     let g = params
//         .generator
//         .iter()
//         .map(|x| FpElement::new(&ctx, decode_base64(x)))
//         .collect::<Vec<FpElement>>();
//
//     let order = decode_base64_biguint(&params.order);
//
//     let g = FpPolynomialElement::new(&ctx, g);
//
//     let mut rng = rand::thread_rng();
//     let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
//
//     let a_pub = g.pow(&a);
//
//     let b_pub = challenge
//         .public
//         .iter()
//         .map(|x| FpElement::new(&ctx, decode_base64(x)))
//         .collect::<Vec<FpElement>>();
//     let b_pub = FpPolynomialElement::new(&ctx, b_pub);
//
//     let a_shared = b_pub.pow(&a);
//
//     (
//         a_pub.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
//         a_shared
//             .coeffs
//             .iter()
//             .map(|x| encode_base64(&x.val))
//             .collect(),
//     )
// }

// fn solution_fp(params: &EC/* pParams, challeng */e: &ChallageRequest) -> (String, String) {
//     let p = decode_base64(&params.modulus);
//     let g = decode_base64(&params.generator);
//     let order = decode_base64_biguint(&params.order);
//
//     let ctx = FieldContext::new_prime(p.clone());
//     let g = FpElement::new(&ctx, g);
//
//     let mut rng = rand::thread_rng();
//     let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);
//
//     let a_pub = g.pow(&a);
//
//     let b_pub = decode_base64(&challenge.public);
//     let b_pub = FpElement::new(&ctx, b_pub);
//
//     let a_shared = b_pub.pow(&a);
//
//     (encode_base64(&a_pub.val), encode_base64(&a_shared.val))
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionChallengeResponse {
    status: String,
    session_id: String,
    timeout: String,
    modp_params: ECpParams,
    modp_challenge: ChallageRequest,
    fpk_params: ECpkParams,
    fpk_challenge: ChallageECpkRequest,
    f2m_params: EC2mParams,
    f2m_challenge: ChallageRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionResponseRequest {
    session_id: String,
    modp: ChallangeResponse,
    fpk: ChallangeECpkResponse,
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
struct ParamECpResponse {
    status: String,
    r#type: String,
    params: ECpParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamECpkResponse {
    status: String,
    r#type: String,
    params: ECpkParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamEC2mResponse {
    status: String,
    r#type: String,
    params: EC2mParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ECpkParams {
    name: String,
    prime_base: String,
    extension: i32,
    modulus: Vec<String>,
    a: Vec<String>,
    b: Vec<String>,
    generator: PointECpkParams,
    order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EC2mParams {
    name: String,
    modulus: String,
    a: String,
    b: String,
    generator: PointParams,
    order: String,
    extension: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ECpParams {
    name: String,
    modulus: String,
    a: String,
    b: String,
    generator: PointParams,
    order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PointECpkParams {
    x: Vec<String>,
    y: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignatureECpk {
    s: Vec<String>,
    e: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PointParams {
    x: String,
    y: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Signature {
    s: String,
    e: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChallageRequest {
    public: PointParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChallangeResponse {
    status: String,
    public: PointParams,
    shared: PointParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChallageECpkRequest {
    public: PointECpkParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChallangeECpkResponse {
    status: String,
    public: PointECpkParams,
    shared: PointECpkParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignatureRequest {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignatureResponse {
    status: String,
    public: PointParams,
    signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignatureECpkResponse {
    status: String,
    public: PointECpkParams,
    signature: Signature,
}

fn validate_solution() {
    let base_url = "https://crypto24.random-oracle.xyz";
    let client = Client::new();

    validate_solution_ecp(base_url, &client);
    validate_solution_f2_poly(base_url, &client);
    validate_solution_ecpk(base_url, &client);
}

fn validate_solution_ecp(base_url: &str, client: &Client) {
    let params = client
        .get(format!("{}/validate/list3/ecp/param", base_url))
        .send()
        .unwrap();

    if !params.status().is_success() {
        println!("Failed to fetch parameters for EC_p");
        return;
    }

    let params: ParamECpResponse = params.json().unwrap();

    if params.r#type != "ecp" {
        println!("Invalid type of parameters");
        return;
    }

    let p = decode_base64(&params.params.modulus);
    let a_param = decode_base64(&params.params.a);
    let b_param = decode_base64(&params.params.b);
    let order = decode_base64_biguint(&params.params.order);
    let x = decode_base64(&params.params.generator.x);
    let y = decode_base64(&params.params.generator.y);

    let poly = vec![0.to_bigint().unwrap(), 1.to_bigint().unwrap()];
    let ctx = FieldContext::new_poly(p, poly);
    let a = FpPolynomialElement::from_fp(&ctx, FpElement::new(&ctx, a_param));
    let b = FpPolynomialElement::from_fp(&ctx, FpElement::new(&ctx, b_param));
    let curve = EllipticCurve::new(a, b, &ctx);
    let g = curve.point(x, y);

    // DH part
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = curve.mul(&a, &g);

    match a_pub {
        Point::Affine { x, y } => {
            assert_eq!(x.coeffs.len(), 1);
            assert_eq!(y.coeffs.len(), 1);

            let challange = ChallageRequest {
                public: PointParams {
                    x: encode_base64(&x.coeffs[0].val),
                    y: encode_base64(&y.coeffs[0].val),
                },
            };

            let challange = client
                .post(format!("{}/validate/list3/ecp/dh/challenge", base_url))
                .json(&challange)
                .send()
                .unwrap();

            if !challange.status().is_success() {
                println!("Failed to send challange");
                return;
            }

            let challange: ChallangeResponse = challange.json().unwrap();

            let b_x_pub = decode_base64(&challange.public.x);
            let b_y_pub = decode_base64(&challange.public.y);

            let b_pub = curve.point(b_x_pub, b_y_pub);

            let b_x_shared = decode_base64(&challange.shared.x);
            let b_y_shared = decode_base64(&challange.shared.y);

            let b_shared = curve.point(b_x_shared, b_y_shared);

            let a_shared = curve.mul(&a, &b_pub);

            assert_eq!(a_shared, b_shared);

            println!("EC_p DH Solution is correct");
        }
        _ => {
            panic!("Expected affine point");
        }
    }

    // SCHNORR part
    let message = "string";

    let signature_req = SignatureRequest {
        message: message.to_string(),
    };

    let signature_resp = client
        .post(format!("{}/validate/list3/ecp/schnorr/sign", base_url))
        .json(&signature_req)
        .send()
        .unwrap();

    if !signature_resp.status().is_success() {
        println!("Failed to send request for signature");
        return;
    }

    let signature_resp: SignatureResponse = signature_resp.json().unwrap();

    let pub_x = decode_base64(&signature_resp.public.x);
    let pub_y = decode_base64(&signature_resp.public.y);

    let pub_key = curve.point(pub_x, pub_y);

    let s = decode_base64_biguint(&signature_resp.signature.s);
    let e = decode_base64_biguint(&signature_resp.signature.e);

    let g_s = curve.mul(&s, &g);
    let y_e = curve.mul(&e, &pub_key);

    let r_v = curve.add(&g_s, &y_e);

    match r_v {
        Point::Affine { x, y } => {
            assert_eq!(x.coeffs.len(), 1);
            assert_eq!(y.coeffs.len(), 1);

            let r_v = PointParams {
                x: encode_base64(&x.coeffs[0].val),
                y: encode_base64(&y.coeffs[0].val),
            };

            let r_v = serde_json::to_string(&r_v).unwrap();

            let r_v_message = format!("{}{}", r_v, message);

            let e_v = Sha256::digest(r_v_message.as_bytes()).to_vec();

            let e_v = BigUint::from_bytes_be(&e_v);

            if e_v == e {
                println!("EC_p Schnorr Signature is correct");
            } else {
                println!("EC_p Schnorr Signature is incorrect");
            }
        }
        _ => panic!("Expected affine point"),
    }
}

fn validate_solution_ecpk(base_url: &str, client: &Client) {
    let params = client
        .get(format!("{}/validate/list3/ecpk/param", base_url))
        .send()
        .unwrap();

    if !params.status().is_success() {
        println!("Failed to fetch parameters for EC_p^k");
        return;
    }

    let params: ParamECpkResponse = params.json().unwrap();

    if params.r#type != "ecpk" {
        println!("Invalid type of parameters");
        return;
    }

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

    let g_x = params
        .params
        .generator
        .x
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let g_x = FpPolynomialElement::new(&ctx, g_x);

    let g_y = params
        .params
        .generator
        .y
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let g_y = FpPolynomialElement::new(&ctx, g_y);

    let order = decode_base64_biguint(&params.params.order);

    let a_param = params
        .params
        .a
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let a_param = FpPolynomialElement::new(&ctx, a_param);

    let b_param = params
        .params
        .b
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let b_param = FpPolynomialElement::new(&ctx, b_param);

    let curve = EllipticCurve::new(a_param, b_param, &ctx);

    let g = Point::Affine { x: g_x, y: g_y };

    // DH part
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = curve.mul(&a, &g);

    match a_pub {
        Point::Affine { x, y } => {
            let challange = ChallageECpkRequest {
                public: PointECpkParams {
                    x: x.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
                    y: y.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
                },
            };

            let challange = client
                .post(format!("{}/validate/list3/ecpk/dh/challenge", base_url))
                .json(&challange)
                .send()
                .unwrap();

            if !challange.status().is_success() {
                println!("Failed to send challange");
                return;
            }

            let challange: ChallangeECpkResponse = challange.json().unwrap();

            let b_x_pub = challange
                .public
                .x
                .iter()
                .map(|x| FpElement::new(&ctx, decode_base64(x)))
                .collect::<Vec<FpElement>>();
            let b_x_pub = FpPolynomialElement::new(&ctx, b_x_pub);

            let b_y_pub = challange
                .public
                .y
                .iter()
                .map(|x| FpElement::new(&ctx, decode_base64(x)))
                .collect::<Vec<FpElement>>();
            let b_y_pub = FpPolynomialElement::new(&ctx, b_y_pub);

            let b_pub = Point::Affine {
                x: b_x_pub,
                y: b_y_pub,
            };

            let b_x_shared = challange
                .shared
                .x
                .iter()
                .map(|x| FpElement::new(&ctx, decode_base64(x)))
                .collect::<Vec<FpElement>>();
            let b_x_shared = FpPolynomialElement::new(&ctx, b_x_shared);

            let b_y_shared = challange
                .shared
                .y
                .iter()
                .map(|x| FpElement::new(&ctx, decode_base64(x)))
                .collect::<Vec<FpElement>>();
            let b_y_shared = FpPolynomialElement::new(&ctx, b_y_shared);

            let b_shared = Point::Affine {
                x: b_x_shared,
                y: b_y_shared,
            };

            let a_shared = curve.mul(&a, &b_pub);

            assert_eq!(a_shared, b_shared);
            println!("EC_p^k DH Solution is correct");
        }
        _ => {
            panic!("Expected affine point");
        }
    }

    // SCHNORR part
    let message = "string";

    let signature_req = SignatureRequest {
        message: message.to_string(),
    };

    let signature_resp = client
        .post(format!("{}/validate/list3/ecpk/schnorr/sign", base_url))
        .json(&signature_req)
        .send()
        .unwrap();

    if !signature_resp.status().is_success() {
        println!("Failed to send request for signature");
        return;
    }

    let signature_resp: SignatureECpkResponse = signature_resp.json().unwrap();

    let pub_x = signature_resp
        .public
        .x
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let pub_x = FpPolynomialElement::new(&ctx, pub_x);

    let pub_y = signature_resp
        .public
        .y
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let pub_y = FpPolynomialElement::new(&ctx, pub_y);

    let pub_key = Point::Affine { x: pub_x, y: pub_y };

    let s = decode_base64_biguint(&signature_resp.signature.s);
    let e = decode_base64_biguint(&signature_resp.signature.e);

    let g_s = curve.mul(&s, &g);
    let y_e = curve.mul(&e, &pub_key);

    let r_v = curve.add(&g_s, &y_e);

    match r_v {
        Point::Affine { x, y } => {
            let r_v = PointECpkParams {
                x: x.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
                y: y.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
            };

            let r_v = serde_json::to_string(&r_v).unwrap();

            let r_v_message = format!("{}{}", r_v, message);

            let e_v = Sha256::digest(r_v_message.as_bytes()).to_vec();

            let e_v = BigUint::from_bytes_be(&e_v);

            if e_v == e {
                println!("EC_p^k Schnorr Signature is correct");
            } else {
                println!("EC_p^k Schnorr Signature is incorrect");
            }
        }
        _ => panic!("Expected affine point"),
    }
}

fn validate_solution_f2_poly(base_url: &str, client: &Client) {
    let params = client
        .get(format!("{}/validate/list3/ec2m/param", base_url))
        .send()
        .unwrap();

    if !params.status().is_success() {
        println!("Failed to fetch parameters for EC_2^m");
        return;
    }

    let params: ParamEC2mResponse = params.json().unwrap();

    if params.r#type != "ec2m" {
        println!("Invalid type of parameters");
        return;
    }

    let extension = params.params.extension;
    let irreducible_poly = decode_base64_biguint_le(&params.params.modulus);

    // add most significant bit
    let irreducible_poly = irreducible_poly | (BigUint::one() << extension);

    let g_x = decode_base64_biguint_le(&params.params.generator.x);
    let g_y = decode_base64_biguint_le(&params.params.generator.y);
    let a_param = decode_base64_biguint_le(&params.params.a);
    let b_param = decode_base64_biguint_le(&params.params.b);
    let order = decode_base64_biguint(&params.params.order);

    let ctx = FieldContext::new_binary(irreducible_poly.clone());
    let curve = BinaryEllipticCurve::new(
        F2PolynomialElement::new(&ctx, a_param),
        F2PolynomialElement::new(&ctx, b_param),
        &ctx,
    );

    let g = BinaryPoint::Affine {
        x: F2PolynomialElement::new(&ctx, g_x.clone()),
        y: F2PolynomialElement::new(&ctx, g_y.clone()),
    };

    // DH part
    let mut rng = rand::thread_rng();
    let a = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    let a_pub = curve.mul(&a, &g);

    match a_pub {
        BinaryPoint::Affine { x, y } => {
            let challange = ChallageRequest {
                public: PointParams {
                    x: encode_base64_biguint_le(&x.coeffs),
                    y: encode_base64_biguint_le(&y.coeffs),
                },
            };

            let challange = client
                .post(format!("{}/validate/list3/ec2m/dh/challenge", base_url))
                .json(&challange)
                .send()
                .unwrap();

            if !challange.status().is_success() {
                println!("Failed to send challange");
                return;
            }

            let challange: ChallangeResponse = challange.json().unwrap();

            let b_x_pub = decode_base64_biguint_le(&challange.public.x);
            let b_y_pub = decode_base64_biguint_le(&challange.public.y);

            let b_pub = BinaryPoint::Affine {
                x: F2PolynomialElement::new(&ctx, b_x_pub),
                y: F2PolynomialElement::new(&ctx, b_y_pub),
            };

            let b_x_shared = decode_base64_biguint_le(&challange.shared.x);
            let b_y_shared = decode_base64_biguint_le(&challange.shared.y);

            let b_shared = BinaryPoint::Affine {
                x: F2PolynomialElement::new(&ctx, b_x_shared),
                y: F2PolynomialElement::new(&ctx, b_y_shared),
            };

            let a_shared = curve.mul(&a, &b_pub);

            assert_eq!(a_shared, b_shared);
            println!("EC_2^m DH Solution is correct");
        }
        _ => {
            panic!("Expected affine point");
        }
    }

    // SCHNORR part
    let message = "string";

    let signature_req = SignatureRequest {
        message: message.to_string(),
    };

    let signature_resp = client
        .post(format!("{}/validate/list3/ec2m/schnorr/sign", base_url))
        .json(&signature_req)
        .send()
        .unwrap();

    if !signature_resp.status().is_success() {
        println!(
            "Failed to send request for signature. Status: {}",
            signature_resp.status()
        );
        return;
    }

    let signature_resp: SignatureResponse = signature_resp.json().unwrap();

    let pub_x = decode_base64_biguint_le(&signature_resp.public.x);
    let pub_y = decode_base64_biguint_le(&signature_resp.public.y);

    let pub_key = BinaryPoint::Affine {
        x: F2PolynomialElement::new(&ctx, pub_x),
        y: F2PolynomialElement::new(&ctx, pub_y),
    };

    let s = decode_base64_biguint(&signature_resp.signature.s);
    let e = decode_base64_biguint(&signature_resp.signature.e);

    let g_s = curve.mul(&s, &g);
    let y_e = curve.mul(&e, &pub_key);

    let r_v = curve.add(&g_s, &y_e);

    match r_v {
        BinaryPoint::Affine { x, y } => {
            let r_v = PointParams {
                x: encode_base64_biguint_le(&x.coeffs),
                y: encode_base64_biguint_le(&y.coeffs),
            };

            let r_v_str = serde_json::to_string(&r_v).unwrap();

            let r_v_message = format!("{}{}", r_v_str, message);

            let e_v = Sha256::digest(r_v_message.as_bytes()).to_vec();
            let e_v = BigUint::from_bytes_be(&e_v);

            if e_v == e {
                println!("EC_2^m Schnorr Signature is correct");
            } else {
                println!("EC_2^m Schnorr Signature is incorrect");
            }
        }
        _ => panic!("Expected affine point"),
    }
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

fn encode_base64_le(n: &BigInt) -> String {
    let (_, bytes) = n.to_bytes_le();
    URL_SAFE_NO_PAD.encode(&bytes)
}

fn encode_base64_biguint(n: &BigUint) -> String {
    let bytes = n.to_bytes_be();
    URL_SAFE_NO_PAD.encode(&bytes)
}

fn encode_base64_biguint_le(n: &BigUint) -> String {
    let bytes = n.to_bytes_le();
    URL_SAFE_NO_PAD.encode(&bytes)
}
