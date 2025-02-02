use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use clap::{Parser, Subcommand};
use diffie_hellman::field::ec::{EllipticCurve, Point};
use diffie_hellman::field::ec_binary::{BinaryEllipticCurve, BinaryPoint};
use diffie_hellman::field::f2_poly::F2PolynomialElement;
use diffie_hellman::field::fp::FpElement;
use diffie_hellman::field::fp_poly::FpPolynomialElement;
use diffie_hellman::FieldContext;
use num::bigint::{RandBigInt, Sign, ToBigInt};
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
        Some(Commands::Solution) => submit_solution(),
        None => {
            println!("usege: diffie-hellman [SUBCOMMAND]");
        }
    }
}

fn submit_solution() {
    let base_url = "https://crypto24.random-oracle.xyz/";
    let student_id = "10000000000000000000000000000033";
    let url = format!("{}submit/list3/{}/ec/solution", base_url, student_id);

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

    println!("Calculating for EC_p");
    let (ecp_public, ecp_signature) = solution_fp(&response.ecp_params, &response.ecp_challenge);
    println!("Calculating for EC_p^k");
    let (ecpk_public, ecpk_signature) =
        solution_fp_poly(&response.ecpk_params, &response.ecpk_challenge);
    println!("Calculating for EC_2^m");
    let (ec2m_public, ec2m_signature) =
        solution_f2_poly(&response.ec2m_params, &response.ec2m_challenge);

    let request = SubmissionResponseRequest {
        session_id: session_id.clone(),
        ecp: ChallangeParams {
            public: ecp_public,
            signature: ecp_signature,
        },
        ecpk: ChallangeECpkParams {
            public: ecpk_public,
            signature: ecpk_signature,
        },
        ec2m: ChallangeParams {
            public: ec2m_public,
            signature: ec2m_signature,
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

    if response.ecp.status == "success" {
        println!("EC_p solution is correct");
    } else {
        println!("Failed EC_p verification");
    }

    if response.ecpk.status == "success" {
        println!("EC_p^k solution is correct");
    } else {
        println!("Failed EC_p^k verification");
    }

    if response.ec2m.status == "success" {
        println!("EC_2^m solution is correct");
    } else {
        println!("Failed EC_2^m verification");
    }
}

fn solution_f2_poly(params: &EC2mParams, challenge: &ChallageRequest) -> (PointParams, Signature) {
    let extension = params.extension;
    let irreducible_poly = decode_base64_biguint_le(&params.modulus);

    // Add most significant bit to make it irreducible polynomial
    let irreducible_poly = irreducible_poly | (BigUint::one() << extension);

    let g_x = decode_base64_biguint_le(&params.generator.x);
    let g_y = decode_base64_biguint_le(&params.generator.y);
    let a_param = decode_base64_biguint_le(&params.a);
    let b_param = decode_base64_biguint_le(&params.b);
    let order = decode_base64_biguint(&params.order);

    let ctx = FieldContext::new_binary(irreducible_poly.clone());
    let curve = BinaryEllipticCurve::new(
        F2PolynomialElement::new(&ctx, a_param),
        F2PolynomialElement::new(&ctx, b_param),
        &ctx,
    );

    let g = BinaryPoint::Affine {
        x: F2PolynomialElement::new(&ctx, g_x),
        y: F2PolynomialElement::new(&ctx, g_y),
    };

    // Generate private key for both DH and signing
    let mut rng = rand::thread_rng();
    let private_key = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    // DH part - compute public key and shared secret
    let public_key = curve.mul(&private_key, &g);

    match public_key {
        BinaryPoint::Affine { x, y } => {
            let b_x = decode_base64_biguint_le(&challenge.public.x);
            let b_y = decode_base64_biguint_le(&challenge.public.y);

            let b_pub = BinaryPoint::Affine {
                x: F2PolynomialElement::new(&ctx, b_x),
                y: F2PolynomialElement::new(&ctx, b_y),
            };

            let shared_secret = curve.mul(&private_key, &b_pub);

            match shared_secret {
                BinaryPoint::Affine {
                    x: shared_x,
                    y: shared_y,
                } => {
                    // Create shared point representation for signing
                    let shared_point = PointParams {
                        x: encode_base64_biguint_le(&shared_x.coeffs),
                        y: encode_base64_biguint_le(&shared_y.coeffs),
                    };

                    // Generate random k for signing
                    let k = rng.gen_biguint_range(&BigUint::from(2u32), &order);
                    let r = curve.mul(&k, &g);

                    match r {
                        BinaryPoint::Affine { x: r_x, y: r_y } => {
                            // Create message from shared point
                            let message = serde_json::to_string(&shared_point).unwrap();
                            let r_point = PointParams {
                                x: encode_base64_biguint_le(&r_x.coeffs),
                                y: encode_base64_biguint_le(&r_y.coeffs),
                            };
                            let r_json = serde_json::to_string(&r_point).unwrap();

                            // Calculate e = H(r || m)
                            let mut hasher = Sha256::new();
                            hasher.update(r_json.as_bytes());
                            hasher.update(message.as_bytes());
                            let e = BigUint::from_bytes_be(&hasher.finalize());

                            // Calculate s = k - ae
                            let ae = (&private_key * &e) % &order;
                            let s = if k >= ae {
                                (k - ae) % &order
                            } else {
                                (&order - (ae - k) % &order) % &order
                            };

                            // Return public key and signature
                            (
                                PointParams {
                                    x: encode_base64_biguint_le(&x.coeffs),
                                    y: encode_base64_biguint_le(&y.coeffs),
                                },
                                Signature {
                                    s: encode_base64_biguint(&s),
                                    e: encode_base64_biguint(&e),
                                },
                            )
                        }
                        _ => panic!("Expected affine point for signature"),
                    }
                }
                _ => panic!("Expected affine point for shared secret"),
            }
        }
        _ => panic!("Expected affine point for public key"),
    }
}

fn solution_fp_poly(
    params: &ECpkParams,
    challenge: &ChallageECpkRequest,
) -> (PointECpkParams, Signature) {
    let p = decode_base64(&params.prime_base);
    let order = decode_base64_biguint(&params.order);

    // Convert modulus vector to BigInt coefficients
    let mut irreducible_poly = params
        .modulus
        .iter()
        .map(|x| decode_base64(x))
        .collect::<Vec<BigInt>>();

    // Add 1 as highest degree coefficient
    irreducible_poly.push(BigInt::one());

    let ctx = FieldContext::new_poly(p, irreducible_poly);

    // Convert generator coordinates from base64 vectors to polynomial elements
    let g_x = params
        .generator
        .x
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let g_x = FpPolynomialElement::new(&ctx, g_x);

    let g_y = params
        .generator
        .y
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let g_y = FpPolynomialElement::new(&ctx, g_y);

    // Convert curve parameters from base64 vectors to polynomial elements
    let a_param = params
        .a
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let a_param = FpPolynomialElement::new(&ctx, a_param);

    let b_param = params
        .b
        .iter()
        .map(|x| FpElement::new(&ctx, decode_base64(x)))
        .collect::<Vec<FpElement>>();
    let b_param = FpPolynomialElement::new(&ctx, b_param);

    let curve = EllipticCurve::new(a_param, b_param, &ctx);
    let g = Point::Affine { x: g_x, y: g_y };

    // Generate private key for both DH and signing
    let mut rng = rand::thread_rng();
    let private_key = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    // DH part - compute public key and shared secret
    let public_key = curve.mul(&private_key, &g);

    match public_key {
        Point::Affine { x, y } => {
            // Convert challenge public key coordinates to polynomial elements
            let b_x = challenge
                .public
                .x
                .iter()
                .map(|x| FpElement::new(&ctx, decode_base64(x)))
                .collect::<Vec<FpElement>>();
            let b_x = FpPolynomialElement::new(&ctx, b_x);

            let b_y = challenge
                .public
                .y
                .iter()
                .map(|x| FpElement::new(&ctx, decode_base64(x)))
                .collect::<Vec<FpElement>>();
            let b_y = FpPolynomialElement::new(&ctx, b_y);

            let b_pub = Point::Affine { x: b_x, y: b_y };

            let shared_secret = curve.mul(&private_key, &b_pub);

            match shared_secret {
                Point::Affine {
                    x: shared_x,
                    y: shared_y,
                } => {
                    // Create shared point representation for signing
                    let shared_point = PointECpkParams {
                        x: shared_x
                            .coeffs
                            .iter()
                            .map(|x| encode_base64(&x.val))
                            .collect(),
                        y: shared_y
                            .coeffs
                            .iter()
                            .map(|x| encode_base64(&x.val))
                            .collect(),
                    };

                    // Generate random k for signing
                    let k = rng.gen_biguint_range(&BigUint::from(2u32), &order);
                    let r = curve.mul(&k, &g);

                    match r {
                        Point::Affine { x: r_x, y: r_y } => {
                            // Create message from shared point
                            let message = serde_json::to_string(&shared_point).unwrap();
                            let r_json = serde_json::to_string(&PointECpkParams {
                                x: r_x.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
                                y: r_y.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
                            })
                            .unwrap();

                            // Calculate e = H(r || m)
                            let mut hasher = Sha256::new();
                            hasher.update(r_json.as_bytes());
                            hasher.update(message.as_bytes());
                            let e = BigUint::from_bytes_be(&hasher.finalize());

                            // Calculate s = k - ae
                            let ae = (&private_key * &e) % &order;
                            let s = if k >= ae {
                                (k - ae) % &order
                            } else {
                                (&order - (ae - k) % &order) % &order
                            };

                            // Return public key and signature
                            (
                                PointECpkParams {
                                    x: x.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
                                    y: y.coeffs.iter().map(|x| encode_base64(&x.val)).collect(),
                                },
                                Signature {
                                    s: encode_base64_biguint(&s),
                                    e: encode_base64_biguint(&e),
                                },
                            )
                        }
                        _ => panic!("Expected affine point for signature"),
                    }
                }
                _ => panic!("Expected affine point for shared secret"),
            }
        }
        _ => panic!("Expected affine point for public key"),
    }
}

fn solution_fp(params: &ECpParams, challenge: &ChallageRequest) -> (PointParams, Signature) {
    let p = decode_base64(&params.modulus);
    let a_param = decode_base64(&params.a);
    let b_param = decode_base64(&params.b);
    let order = decode_base64_biguint(&params.order);
    let x = decode_base64(&params.generator.x);
    let y = decode_base64(&params.generator.y);

    let poly = vec![0.to_bigint().unwrap(), 1.to_bigint().unwrap()];
    let ctx = FieldContext::new_poly(p, poly);
    let a = FpPolynomialElement::from_fp(&ctx, FpElement::new(&ctx, a_param));
    let b = FpPolynomialElement::from_fp(&ctx, FpElement::new(&ctx, b_param));
    let curve = EllipticCurve::new(a, b, &ctx);
    let g = curve.point(x, y);

    // Generate private key for both DH and signing
    let mut rng = rand::thread_rng();
    let private_key = rng.gen_biguint_range(&BigUint::from(2u32), &order);

    // DH part - compute public key and shared secret
    let public_key = curve.mul(&private_key, &g);

    match public_key {
        Point::Affine { x, y } => {
            assert_eq!(x.coeffs.len(), 1);
            assert_eq!(y.coeffs.len(), 1);

            let b_x = decode_base64(&challenge.public.x);
            let b_y = decode_base64(&challenge.public.y);
            let b_pub = curve.point(b_x, b_y);

            let shared_secret = curve.mul(&private_key, &b_pub);

            match shared_secret {
                Point::Affine {
                    x: shared_x,
                    y: shared_y,
                } => {
                    assert_eq!(shared_x.coeffs.len(), 1);
                    assert_eq!(shared_y.coeffs.len(), 1);

                    // Schnorr signing of the shared secret
                    let shared_point = PointParams {
                        x: encode_base64(&shared_x.coeffs[0].val),
                        y: encode_base64(&shared_y.coeffs[0].val),
                    };

                    // Generate random k for signing
                    let k = rng.gen_biguint_range(&BigUint::from(2u32), &order);
                    let r = curve.mul(&k, &g);

                    match r {
                        Point::Affine { x: r_x, y: r_y } => {
                            assert_eq!(r_x.coeffs.len(), 1);
                            assert_eq!(r_y.coeffs.len(), 1);

                            // Create message as combination of point coordinates
                            let message = serde_json::to_string(&shared_point).unwrap();
                            let r_json = serde_json::to_string(&PointParams {
                                x: encode_base64(&r_x.coeffs[0].val),
                                y: encode_base64(&r_y.coeffs[0].val),
                            })
                            .unwrap();

                            // Calculate e = H(r || m)
                            let mut hasher = Sha256::new();
                            hasher.update(r_json.as_bytes());
                            hasher.update(message.as_bytes());
                            let e = BigUint::from_bytes_be(&hasher.finalize());

                            // Calculate s = k - ae
                            let ae = (&private_key * &e) % &order;
                            let s = if k >= ae {
                                (k - ae) % &order
                            } else {
                                (&order - (ae - k) % &order) % &order
                            };

                            // Return the signed shared secret as the "shared" point
                            (
                                PointParams {
                                    x: encode_base64(&x.coeffs[0].val),
                                    y: encode_base64(&y.coeffs[0].val),
                                },
                                Signature {
                                    s: encode_base64_biguint(&s),
                                    e: encode_base64_biguint(&e),
                                },
                            )
                        }
                        _ => panic!("Expected affine point for signature"),
                    }
                }
                _ => panic!("Expected affine point for shared secret"),
            }
        }
        _ => panic!("Expected affine point for public key"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionChallengeResponse {
    status: String,
    session_id: String,
    timeout: String,
    ecp_params: ECpParams,
    ecp_challenge: ChallageRequest,
    ecpk_params: ECpkParams,
    ecpk_challenge: ChallageECpkRequest,
    ec2m_params: EC2mParams,
    ec2m_challenge: ChallageRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionResponseRequest {
    session_id: String,
    ecp: ChallangeParams,
    ecpk: ChallangeECpkParams,
    ec2m: ChallangeParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Status {
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionResponseResponse {
    status: String,
    ecp: Status,
    ecpk: Status,
    ec2m: Status,
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
struct ChallangeParams {
    public: PointParams,
    signature: Signature,
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
struct ChallangeECpkParams {
    public: PointECpkParams,
    signature: Signature,
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

    println!("a_param: {}", a_param);

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

            let mut hasher = Sha256::new();

            hasher.update(serde_json::to_string(&r_v).unwrap().as_bytes());
            hasher.update(message.as_bytes());

            let e_v = hasher.finalize();
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

// fn encode_base64_le(n: &BigInt) -> String {
//     let (_, bytes) = n.to_bytes_le();
//     URL_SAFE_NO_PAD.encode(&bytes)
// }

fn encode_base64_biguint(n: &BigUint) -> String {
    let bytes = n.to_bytes_be();
    URL_SAFE_NO_PAD.encode(&bytes)
}

fn encode_base64_biguint_le(n: &BigUint) -> String {
    let bytes = n.to_bytes_le();
    URL_SAFE_NO_PAD.encode(&bytes)
}
