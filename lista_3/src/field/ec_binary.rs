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
                let y2 = y.pow(&2u32.into());
                let xy = x * y;
                let x3 = x.pow(&3u32.into());
                let ax2 = &self.a * &x.pow(&2u32.into());
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
                let x2 = x.pow(&2u32.into());
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
                let x_r = &(&lambda2 + &lambda + (x1 + x2)) + &self.a;

                // y_r = λ(x1 + x_r) + x_r + y1
                let y_r = &(&(&lambda * &(x1 + &x_r)) + &x_r) + y1;

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

    pub fn mul_secure(
        &self,
        k: &BigUint,
        point: &BinaryPoint<'a>,
        order: &BigUint,
    ) -> BinaryPoint<'a> {
        let mut result = BinaryPoint::Infinity;
        let mut dummy = BinaryPoint::Infinity;
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
    use super::*;

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
}
