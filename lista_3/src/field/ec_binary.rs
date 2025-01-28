use std::fmt::Display;

use num::{BigUint, One, Zero};

use crate::field::f2_poly::F2PolynomialElement;
use crate::FieldContext;
use crate::FieldElement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryEllipticCurve<'a> {
    // y^2 + xy = x^3 + ax^2 + b
    a: F2PolynomialElement<'a>,
    b: F2PolynomialElement<'a>,
    ctx: &'a FieldContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryPoint<'a> {
    Infinity,
    Affine {
        x: F2PolynomialElement<'a>,
        y: F2PolynomialElement<'a>,
    },
}

impl<'a> BinaryEllipticCurve<'a> {
    pub fn new(
        a: F2PolynomialElement<'a>,
        b: F2PolynomialElement<'a>,
        ctx: &'a FieldContext,
    ) -> Self {
        assert!(ctx.is_binary(), "Context must be for binary field");
        // For binary fields, the curve is non-singular if b ≠ 0
        assert!(!b.is_zero(), "Curve is singular (b = 0)");

        Self { a, b, ctx }
    }

    pub fn contains_point(&self, point: &BinaryPoint<'a>) -> bool {
        match point {
            BinaryPoint::Infinity => true,
            BinaryPoint::Affine { x, y } => {
                // Check if point satisfies y^2 + xy = x^3 + ax^2 + b
                let y2 = y * y;
                let xy = x * y;
                let x3 = &(x * x) * x;
                let ax2 = &self.a * &(x * x);
                y2 + xy == x3 + ax2 + self.b.clone()
            }
        }
    }

    pub fn double(&self, point: &BinaryPoint<'a>) -> BinaryPoint<'a> {
        match point {
            BinaryPoint::Infinity => BinaryPoint::Infinity,
            BinaryPoint::Affine { x, y } => {
                if x.is_zero() {
                    return BinaryPoint::Infinity;
                }

                // For binary fields:
                // λ = x + y/x
                let lambda = x + &(y / x);

                // x_r = λ^2 + λ + a
                let lambda2 = &lambda * &lambda;
                let x_r = &(&lambda2 + &lambda) + &self.a;

                // y_r = x^2 + λx_r + x_r
                let x2 = x * x;
                let y_r = &(&x2 + &(&lambda * &x_r)) + &x_r;

                BinaryPoint::Affine { x: x_r, y: y_r }
            }
        }
    }

    pub fn add(&self, p1: &BinaryPoint<'a>, p2: &BinaryPoint<'a>) -> BinaryPoint<'a> {
        match (p1, p2) {
            (BinaryPoint::Infinity, _) => p2.clone(),
            (_, BinaryPoint::Infinity) => p1.clone(),
            (BinaryPoint::Affine { x: x1, y: y1 }, BinaryPoint::Affine { x: x2, y: y2 }) => {
                if x1 == x2 {
                    if y1 == y2 {
                        return self.double(p1);
                    } else {
                        return BinaryPoint::Infinity;
                    }
                }

                // For binary fields:
                // λ = (y1 + y2)/(x1 + x2)
                let lambda = &(y1 + y2) / &(x1 + x2);

                // x_r = λ^2 + λ + x1 + x2 + a
                let lambda2 = &lambda * &lambda;
                let x_r = &((&lambda2 + &lambda) + (x1 + x2)) + &self.a;

                // y_r = λ(x1 + x_r) + x_r + y1
                let y_r = &((lambda * (x1 + &x_r)) + x_r.clone()) + y1;

                BinaryPoint::Affine { x: x_r, y: y_r }
            }
        }
    }

    pub fn mul(&self, k: &BigUint, point: &BinaryPoint<'a>) -> BinaryPoint<'a> {
        let mut result = BinaryPoint::Infinity;
        let mut temp = point.clone();
        let mut k = k.clone();

        while k > BigUint::zero() {
            if (&k & BigUint::one()) == BigUint::one() {
                result = self.add(&result, &temp);
            }
            temp = self.double(&temp);
            k >>= 1;
        }
        result
    }
}

impl<'a> Display for BinaryPoint<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryPoint::Infinity => write!(f, "∞"),
            BinaryPoint::Affine { x, y } => write!(f, "({}, {})", x, y),
        }
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use num::Num;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    use crate::get_binary_poly_degree;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct PointJson {
        x: String,
        y: String,
    }

    #[test]
    fn test_schnorr_sign() {
        let irreducible = BigUint::from_str_radix(
            "13803492693581127574869511724554050904902217944359662576256527028453377",
            10,
        )
        .unwrap();

        {
            let mut has_printed_term = false;
            let degree = get_binary_poly_degree(&irreducible);

            for i in (0..=degree).rev() {
                if (&irreducible >> i) & BigUint::one() == BigUint::one() {
                    if has_printed_term {
                        print!(" + ");
                    }
                    if i == 0 {
                        print!("1");
                    } else if i == 1 {
                        print!("x");
                    } else {
                        print!("x^{}", i);
                    }
                    has_printed_term = true;
                }
            }

            if !has_printed_term {
                print!("0");
            }
        }

        let ctx = FieldContext::new_binary(irreducible);

        let a = F2PolynomialElement::new(&ctx, BigUint::one());
        let b = F2PolynomialElement::new(
            &ctx,
            BigUint::from_str_radix(
                "2760497980029204187078845502377898520307707256259003964398570147123373",
                10,
            )
            .unwrap(),
        );

        let curve = BinaryEllipticCurve::new(a, b, &ctx);

        let g_x = F2PolynomialElement::new(
            &ctx,
            BigUint::from_str_radix(
                "6761246501583409083997096882159824046681246465812468867444643442021771",
                10,
            )
            .unwrap(),
        );

        let g_y = F2PolynomialElement::new(
            &ctx,
            BigUint::from_str_radix(
                "6912913004411390932094889411904587007871508723951293564567204383952978",
                10,
            )
            .unwrap(),
        );

        let pub_x = F2PolynomialElement::new(
            &ctx,
            BigUint::from_str_radix(
                "12402295794884592605523544085172117365641327725166099250818921853177136",
                10,
            )
            .unwrap(),
        );

        let pub_y = F2PolynomialElement::new(
            &ctx,
            BigUint::from_str_radix(
                "5768410990405248830209118733282437954417583603301117360400494962850628",
                10,
            )
            .unwrap(),
        );

        let s = BigUint::from_str_radix(
            "2520234969816925095800867241965673325266602350141567129250207801352592",
            10,
        )
        .unwrap();

        let e = BigUint::from_str_radix(
            "75318459842236511478849343242161054881755347063699594280921693336154548583431",
            10,
        )
        .unwrap();

        let r_v_message = "{\"x\":\"h9G2DWCVjGlgJ1Ue11BDkETBRFuleJe2mtXsXsc\",\"y\":\"AHdBlgzbAy5Tre7ENMvnIV_PVurnhf1J-bkol2c\"}string";

        let e_v = BigUint::from_str_radix(
            "9199230757432990032429506297410042713924164791025350816076345699126147556838",
            10,
        )
        .unwrap();

        let g = BinaryPoint::Affine { x: g_x, y: g_y };

        let pub_key = BinaryPoint::Affine { x: pub_x, y: pub_y };

        let g_s = curve.mul(&s, &g);
        let y_e = curve.mul(&e, &pub_key);

        let r = curve.add(&g_s, &y_e);

        fn encode_base64_biguint_le(n: &BigUint) -> String {
            let bytes = n.to_bytes_le();
            URL_SAFE_NO_PAD.encode(&bytes)
        }

        match r {
            BinaryPoint::Affine { x, y } => {
                let r_v = PointJson {
                    x: encode_base64_biguint_le(&x.coeffs),
                    y: encode_base64_biguint_le(&y.coeffs),
                };

                let r_v_str = serde_json::to_string(&r_v).unwrap();

                let r_v_message_test = format!("{}{}", r_v_str, "string");

                assert_eq!(r_v_message_test, r_v_message);

                let e_v_test = Sha256::digest(r_v_message_test.as_bytes()).to_vec();
                let e_v_test = BigUint::from_bytes_be(&e_v_test);

                assert_eq!(e, e_v_test);
            }
            _ => panic!("Expected affine point"),
        }
    }

    #[test]
    fn test_binary_curve_f2_4() {
        // Using F_2[x]/(x^4 + x + 1)
        let irreducible = BigUint::from(0b10011u32); // x^4 + x + 1
        let ctx = FieldContext::new_binary(irreducible);

        // Create curve y^2 + xy = x^3 + x^2 + 1
        let a = F2PolynomialElement::new(&ctx, BigUint::one());
        let b = F2PolynomialElement::new(&ctx, BigUint::one());
        let curve = BinaryEllipticCurve::new(a, b, &ctx);

        // Point P1 = (0, 1)
        let p1 = BinaryPoint::Affine {
            x: F2PolynomialElement::new(&ctx, BigUint::zero()),
            y: F2PolynomialElement::new(&ctx, BigUint::one()),
        };

        // Point P2 = (1, a^2 + a)
        let p2 = BinaryPoint::Affine {
            x: F2PolynomialElement::new(&ctx, BigUint::one()),
            y: F2PolynomialElement::new(&ctx, BigUint::from(0b0110u32)), // a^2 + a = x^2 + x
        };

        assert!(curve.contains_point(&p1));
        assert!(curve.contains_point(&p2));

        // Test 2*P1 = infinity
        let double_p1 = curve.double(&p1);
        assert_eq!(double_p1, BinaryPoint::Infinity);

        // Test P1 + P2 = (1, a^2 + a + 1)
        let sum = curve.add(&p1, &p2);
        match &sum {
            BinaryPoint::Affine { x, y } => {
                assert_eq!(x.coeffs, BigUint::one());
                assert_eq!(y.coeffs, BigUint::from(0b0111u32)); // a^2 + a + 1 = x^2 + x + 1
            }
            BinaryPoint::Infinity => panic!("Expected affine point"),
        }

        // Test 3*P1 = (0, 1)
        let triple_p1 = curve.mul(&BigUint::from(3u32), &p1);
        match &triple_p1 {
            BinaryPoint::Affine { x, y } => {
                assert_eq!(x.coeffs, BigUint::zero());
                assert_eq!(y.coeffs, BigUint::one());
            }
            BinaryPoint::Infinity => panic!("Expected affine point"),
        }

        // Verify curve order = 16
        let order = BigUint::from(16u32);
        let order_p1 = curve.mul(&order, &p1);
        assert_eq!(order_p1, BinaryPoint::Infinity);
    }

    #[test]
    fn test_binary_curve_f2_3() {
        // Using F_2[x]/(x^3 + x + 1)
        let irreducible = BigUint::from(0b1011u32); // x^3 + x + 1
        let ctx = FieldContext::new_binary(irreducible);

        // Create curve y^2 + xy = x^3 + 1
        let a = F2PolynomialElement::new(&ctx, BigUint::zero());
        let b = F2PolynomialElement::new(&ctx, BigUint::one());
        let curve = BinaryEllipticCurve::new(a, b, &ctx);

        // Point P1 = (0, 1)
        let p1 = BinaryPoint::Affine {
            x: F2PolynomialElement::new(&ctx, BigUint::zero()),
            y: F2PolynomialElement::new(&ctx, BigUint::one()),
        };

        assert!(curve.contains_point(&p1));

        // Test 2*P1 = infinity
        let double_p1 = curve.double(&p1);
        assert_eq!(double_p1, BinaryPoint::Infinity);

        // Test 3*P1 = (0, 1)
        let triple_p1 = curve.mul(&BigUint::from(3u32), &p1);
        match &triple_p1 {
            BinaryPoint::Affine { x, y } => {
                assert_eq!(x.coeffs, BigUint::zero());
                assert_eq!(y.coeffs, BigUint::one());
            }
            BinaryPoint::Infinity => panic!("Expected affine point"),
        }

        // Test 7*P1 = (0, 1)
        let seven_p1 = curve.mul(&BigUint::from(7u32), &p1);
        match &seven_p1 {
            BinaryPoint::Affine { x, y } => {
                assert_eq!(x.coeffs, BigUint::zero());
                assert_eq!(y.coeffs, BigUint::one());
            }
            BinaryPoint::Infinity => panic!("Expected affine point"),
        }

        // Verify curve order = 4
        let order = BigUint::from(4u32);
        let order_p1 = curve.mul(&order, &p1);
        assert_eq!(order_p1, BinaryPoint::Infinity);
    }

    #[test]
    fn test_binary_curve_point_doubling() {
        // Using F_2[x]/(x^4 + x + 1)
        let irreducible = BigUint::from(0b10011u32); // x^4 + x + 1
        let ctx = FieldContext::new_binary(irreducible);

        // Create curve y^2 + xy = x^3 + x^2 + 1
        let a = F2PolynomialElement::new(&ctx, BigUint::one());
        let b = F2PolynomialElement::new(&ctx, BigUint::one());
        let curve = BinaryEllipticCurve::new(a, b, &ctx);

        // Point P = (a^2 + a, 1) = (0110, 0001)
        let p = BinaryPoint::Affine {
            x: F2PolynomialElement::new(&ctx, BigUint::from(0b0110u32)),
            y: F2PolynomialElement::new(&ctx, BigUint::from(0b0001u32)),
        };

        // Verify P is on the curve
        assert!(curve.contains_point(&p));

        // Calculate 2P
        let p2 = curve.double(&p);
        match &p2 {
            BinaryPoint::Affine { x, y } => {
                assert_eq!(x.coeffs, BigUint::from(0b0001u32));
                assert_eq!(y.coeffs, BigUint::from(0b0111u32));
            }
            BinaryPoint::Infinity => panic!("2P should not be point at infinity"),
        }

        // Calculate 4P = 2(2P)
        let p4 = curve.double(&p2);
        match &p4 {
            BinaryPoint::Affine { x, y } => {
                assert_eq!(x.coeffs, BigUint::from(0b0000u32));
                assert_eq!(y.coeffs, BigUint::from(0b0001u32));
            }
            BinaryPoint::Infinity => panic!("4P should not be point at infinity"),
        }

        // Verify using scalar multiplication
        let p4_mul = curve.mul(&BigUint::from(4u32), &p);
        assert_eq!(p4, p4_mul);

        // Verify all points are on the curve
        assert!(curve.contains_point(&p2));
        assert!(curve.contains_point(&p4));
    }
}
