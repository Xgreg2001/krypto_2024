use std::fmt::Display;

use num::{BigUint, One, Zero};

use crate::field::fp::FpElement;
use crate::field::fp_poly::FpPolynomialElement;
use crate::{FieldContext, FieldElement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EllipticCurve<'a> {
    // y^2 = x^3 + ax + b
    a: FpPolynomialElement<'a>,
    b: FpPolynomialElement<'a>,
    ctx: &'a FieldContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Point<'a> {
    Infinity,
    Affine {
        x: FpPolynomialElement<'a>,
        y: FpPolynomialElement<'a>,
    },
}

impl<'a> EllipticCurve<'a> {
    pub fn new(
        a: FpPolynomialElement<'a>,
        b: FpPolynomialElement<'a>,
        ctx: &'a FieldContext,
    ) -> Self {
        // Verify that 4a^3 + 27b^2 ≠ 0 (curve is non-singular)
        let four = FpElement::new(ctx, 4.into());
        let twentyseven = FpElement::new(ctx, 27.into());

        let a3 = a.pow(&3u32.into());
        let b2 = b.pow(&2u32.into());

        let lhs = &a3 * &FpPolynomialElement::from_fp(ctx, four);
        let rhs = &b2 * &FpPolynomialElement::from_fp(ctx, twentyseven);

        assert!(!(&lhs + &rhs).is_zero(), "Curve is singular");

        Self { a, b, ctx }
    }

    pub fn contains_point(&self, point: &Point<'a>) -> bool {
        match point {
            Point::Infinity => true,
            Point::Affine { x, y } => {
                // Check if point satisfies y^2 = x^3 + ax + b
                let y2 = y.pow(&2u32.into());
                let x3 = x.pow(&3u32.into());
                let ax = &self.a * x;
                y2 == x3 + ax + self.b.clone()
            }
        }
    }

    pub fn double(&self, point: &Point<'a>) -> Point<'a> {
        match point {
            Point::Infinity => Point::Infinity,
            Point::Affine { x, y } => {
                if y.is_zero() {
                    return Point::Infinity;
                }

                // λ = (3x^2 + a)/(2y)
                // let ctx = self.ctx;
                // let two = FpPolynomialElement::from_fp(ctx, FpElement::new(ctx, 2.into()));
                // let three = FpPolynomialElement::from_fp(ctx, FpElement::new(ctx, 3.into()));

                let x2 = x * x;
                let numerator = &(&x2 + &x2 + x2) + &self.a;
                let denominator = y + y;

                println!("numerator: {:?}", numerator);
                println!("denominator: {:?}", denominator);

                let lambda = &numerator / &denominator;

                // x_r = λ^2 - x - x
                let x_r = &(&lambda * &lambda) - &(x + x);

                // y_r = λ(x - x_r) - y
                let y_r = &(&lambda * &(x - &x_r)) - y;

                Point::Affine { x: x_r, y: y_r }
            }
        }
    }

    pub fn add(&self, p1: &Point<'a>, p2: &Point<'a>) -> Point<'a> {
        match (p1, p2) {
            (Point::Infinity, _) => p2.clone(),
            (_, Point::Infinity) => p1.clone(),
            (Point::Affine { x: x1, y: y1 }, Point::Affine { x: x2, y: y2 }) => {
                if x1 == x2 {
                    if y1 == y2 {
                        return self.double(p1);
                    } else {
                        return Point::Infinity;
                    }
                }

                // λ = (y2-y1)/(x2-x1)
                let numerator = y2 - y1;
                let denominator = x2 - x1;
                let lambda = numerator / denominator;

                // x_r = λ^2 - x1 - x2
                let x_r = (&lambda * &lambda) - (x1 + x2);

                // y_r = λ(x1 - x_r) - y1
                let y_r = &(&lambda * &(x1 - &x_r)) - y1;

                Point::Affine { x: x_r, y: y_r }
            }
        }
    }

    pub fn mul(&self, k: &BigUint, point: &Point<'a>) -> Point<'a> {
        let mut result = Point::Infinity;
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

    pub fn mul_secure(&self, k: &BigUint, point: &Point<'a>, order: &BigUint) -> Point<'a> {
        let mut result = Point::Infinity;
        let mut dummy = Point::Infinity;
        let mut temp = point.clone();
        let k = k % order;

        for i in 0..order.bits() {
            if ((&k >> i) & BigUint::one()) == BigUint::one() {
                result = self.add(&result, &temp);
            } else {
                dummy = self.add(&dummy, &temp);
            }
            temp = self.double(&temp);
        }
        result
    }
}

impl<'a> Display for Point<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Point::Infinity => write!(f, "∞"),
            Point::Affine { x, y } => write!(f, "({}, {})", x, y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FieldContext;
    use num::bigint::ToBigInt;

    #[test]
    fn test_curve_f23_poly() {
        // F_23[x]/(x^2 + 1)
        let p = 23.to_bigint().unwrap();
        let irreducible_poly = vec![
            1.to_bigint().unwrap(),
            0.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);

        // y^2 = x^3 + 2x + 3
        let a = FpPolynomialElement::from_vec(&ctx, vec![2]);
        let b = FpPolynomialElement::from_vec(&ctx, vec![3]);
        let curve = EllipticCurve::new(a, b, &ctx);

        // Points from SageMath output
        let p1 = Point::Affine {
            x: FpPolynomialElement::from_vec(&ctx, vec![0]),
            y: FpPolynomialElement::from_vec(&ctx, vec![7]),
        };
        let p2 = Point::Affine {
            x: FpPolynomialElement::from_vec(&ctx, vec![0]),
            y: FpPolynomialElement::from_vec(&ctx, vec![16]),
        };

        // Verify operations
        let double_p1 = curve.double(&p1);
        match &double_p1 {
            Point::Affine { x, y } => {
                assert_eq!(x, &FpPolynomialElement::from_vec(&ctx, vec![8]));
                assert_eq!(y, &FpPolynomialElement::from_vec(&ctx, vec![5]));
            }
            Point::Infinity => panic!("Expected affine point"),
        }

        let sum = curve.add(&p1, &p2);
        assert!(matches!(sum, Point::Infinity));

        let triple_p1 = curve.mul(&BigUint::from(3u32), &p1);
        match triple_p1 {
            Point::Affine { x, y } => {
                assert_eq!(x, FpPolynomialElement::from_vec(&ctx, vec![5]));
                assert_eq!(y, FpPolynomialElement::from_vec(&ctx, vec![0]));
            }
            Point::Infinity => panic!("Expected affine point"),
        }
    }

    #[test]
    fn test_curve_f11_poly() {
        // F_11[x]/(x^3 + 2x + 7)
        let p = 11.to_bigint().unwrap();
        let irreducible_poly = vec![
            7.to_bigint().unwrap(),
            2.to_bigint().unwrap(),
            0.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);

        // y^2 = x^3 + 5x + 7
        let a = FpPolynomialElement::from_vec(&ctx, vec![5]);
        let b = FpPolynomialElement::from_vec(&ctx, vec![7]);
        let curve = EllipticCurve::new(a, b, &ctx);

        // Points from SageMath output
        let p1 = Point::Affine {
            x: FpPolynomialElement::from_vec(&ctx, vec![2]),
            y: FpPolynomialElement::from_vec(&ctx, vec![5]),
        };
        let p2 = Point::Affine {
            x: FpPolynomialElement::from_vec(&ctx, vec![2]),
            y: FpPolynomialElement::from_vec(&ctx, vec![6]),
        };

        // Verify operations
        let double_p1 = curve.double(&p1);
        match &double_p1 {
            Point::Affine { x, y } => {
                assert_eq!(x, &FpPolynomialElement::from_vec(&ctx, vec![10]));
                assert_eq!(y, &FpPolynomialElement::from_vec(&ctx, vec![10]));
            }
            Point::Infinity => panic!("Expected affine point"),
        }

        let sum = curve.add(&p1, &p2);
        assert!(matches!(sum, Point::Infinity));

        let triple_p1 = curve.mul(&BigUint::from(3u32), &p1);
        match triple_p1 {
            Point::Affine { x, y } => {
                assert_eq!(x, FpPolynomialElement::from_vec(&ctx, vec![3]));
                assert_eq!(y, FpPolynomialElement::from_vec(&ctx, vec![4]));
            }
            Point::Infinity => panic!("Expected affine point"),
        }
    }

    #[test]
    fn test_curve_f7_poly() {
        // F_7[x]/(x^2 + x + 3)
        let p = 7.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);

        // y^2 = x^3 + x + 1
        let a = FpPolynomialElement::from_vec(&ctx, vec![1]);
        let b = FpPolynomialElement::from_vec(&ctx, vec![1]);
        let curve = EllipticCurve::new(a, b, &ctx);

        // Points from SageMath output
        let p1 = Point::Affine {
            x: FpPolynomialElement::from_vec(&ctx, vec![0]),
            y: FpPolynomialElement::from_vec(&ctx, vec![1]),
        };
        let p2 = Point::Affine {
            x: FpPolynomialElement::from_vec(&ctx, vec![0]),
            y: FpPolynomialElement::from_vec(&ctx, vec![6]),
        };
        let p3 = p2.clone();

        // Verify operations
        let sum_p1_neg_p1 = curve.add(&p1, &p2);
        assert!(matches!(sum_p1_neg_p1, Point::Infinity));

        let double_p3 = curve.double(&p3);
        match &double_p3 {
            Point::Affine { x, y } => {
                assert_eq!(x, &FpPolynomialElement::from_vec(&ctx, vec![2]));
                assert_eq!(y, &FpPolynomialElement::from_vec(&ctx, vec![2]));
            }
            Point::Infinity => panic!("Expected affine point"),
        }

        let sum_p1_p3 = curve.add(&p1, &p3);
        assert!(matches!(sum_p1_p3, Point::Infinity));
    }
}
