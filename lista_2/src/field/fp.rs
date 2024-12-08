use num::bigint::BigInt;
use num::traits::{One, Zero};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::{FieldContext, FieldElement};

/// An element in the prime field Fp, referencing a `FieldContext`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FpElement<'a> {
    context: &'a FieldContext,
    pub val: BigInt,
}

impl<'a> fmt::Display for FpElement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Just display the value field
        write!(f, "{}", self.val)
    }
}

impl<'a> FpElement<'a> {
    pub fn new(context: &'a FieldContext, val: BigInt) -> Self {
        Self {
            context,
            val: val % &context.p,
        }
    }

    pub fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
        let mut old_r = a.clone();
        let mut r = b.clone();

        let mut old_s = BigInt::one();
        let mut s = BigInt::zero();

        let mut old_t = BigInt::zero();
        let mut t = BigInt::one();

        while !r.is_zero() {
            let quotient = &old_r / &r;

            let temp_r = r.clone();
            r = &old_r - &quotient * &r;
            old_r = temp_r;

            let temp_s = s.clone();
            s = &old_s - &quotient * &s;
            old_s = temp_s;

            let temp_t = t.clone();
            t = &old_t - &quotient * &t;
            old_t = temp_t;
        }

        (old_r, old_s, old_t)
    }
}

impl<'a> Add for FpElement<'a> {
    type Output = FpElement<'a>;
    fn add(self, other: FpElement<'a>) -> FpElement<'a> {
        FpElement::new(self.context, self.val + other.val)
    }
}

impl<'a> Add for &FpElement<'a> {
    type Output = FpElement<'a>;
    fn add(self, other: &FpElement<'a>) -> FpElement<'a> {
        FpElement::new(self.context, &self.val + &other.val)
    }
}

impl<'a> Sub for FpElement<'a> {
    type Output = FpElement<'a>;
    fn sub(self, other: FpElement<'a>) -> FpElement<'a> {
        FpElement::new(
            self.context,
            (self.val + &self.context.p - other.val) % &self.context.p,
        )
    }
}

impl<'a> Sub for &FpElement<'a> {
    type Output = FpElement<'a>;
    fn sub(self, other: &FpElement<'a>) -> FpElement<'a> {
        FpElement::new(
            self.context,
            (&self.val + &self.context.p - &other.val) % &self.context.p,
        )
    }
}

impl<'a> Neg for FpElement<'a> {
    type Output = FpElement<'a>;
    fn neg(self) -> FpElement<'a> {
        if self.val == BigInt::zero() {
            self
        } else {
            FpElement::new(self.context, &self.context.p - self.val)
        }
    }
}

impl<'a> Neg for &FpElement<'a> {
    type Output = FpElement<'a>;
    fn neg(self) -> FpElement<'a> {
        if self.val == BigInt::zero() {
            self.clone()
        } else {
            FpElement::new(self.context, &self.context.p - &self.val)
        }
    }
}

impl<'a> Mul for FpElement<'a> {
    type Output = FpElement<'a>;
    fn mul(self, other: FpElement<'a>) -> FpElement<'a> {
        let res = (self.val * other.val) % &self.context.p;
        FpElement::new(self.context, res)
    }
}

impl<'a> Mul for &FpElement<'a> {
    type Output = FpElement<'a>;
    fn mul(self, other: &FpElement<'a>) -> FpElement<'a> {
        let res = (&self.val * &other.val) % &self.context.p;
        FpElement::new(self.context, res)
    }
}

impl<'a> Div for FpElement<'a> {
    type Output = FpElement<'a>;
    fn div(self, other: FpElement<'a>) -> FpElement<'a> {
        self * other.inverse()
    }
}

impl<'a> Div for &FpElement<'a> {
    type Output = FpElement<'a>;
    fn div(self, other: &FpElement<'a>) -> FpElement<'a> {
        let inv = other.inverse();
        self * &inv
    }
}

impl<'a> FieldElement<'a> for FpElement<'a> {
    fn zero(ctx: &'a FieldContext) -> Self {
        FpElement::new(ctx, BigInt::zero())
    }

    fn one(ctx: &'a FieldContext) -> Self {
        FpElement::new(ctx, BigInt::one())
    }

    fn is_zero(&self) -> bool {
        self.val == BigInt::zero()
    }

    fn inverse(&self) -> Self {
        let a = &self.val;
        let m = &self.context.p;
        let (g, x, _) = Self::extended_gcd(&a, &m);
        if g != BigInt::one() {
            panic!("No inverse exists!");
        }
        let inv = ((x % m) + m) % m;
        FpElement::new(self.context, inv)
    }

    fn pow(&self, exp: u64) -> Self {
        let mut base = self.val.clone();
        let mut result = BigInt::one();
        let mut e_val = exp;
        while e_val > 0 {
            if (e_val & 1) == 1 {
                result = result * &base;
            }
            base = &base * &base;
            e_val >>= 1;
        }
        FpElement::new(self.context, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::bigint::ToBigInt;

    #[test]
    fn test_fp_addition() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2.to_bigint().unwrap());
        let b = FpElement::new(&ctx, 3.to_bigint().unwrap());
        let c = FpElement::new(&ctx, 0.to_bigint().unwrap());

        assert_eq!((&a + &b).val, 5.to_bigint().unwrap());
        assert_eq!((&a + &c).val, a.val); // a+0 = a
                                          // Check wrap-around
        let x = FpElement::new(&ctx, 16.to_bigint().unwrap());
        let y = FpElement::new(&ctx, 5.to_bigint().unwrap());
        // 16+5=21 mod 17=4
        assert_eq!((x + y).val, 4.to_bigint().unwrap());
    }

    #[test]
    fn test_fp_subtraction() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2.to_bigint().unwrap());
        let b = FpElement::new(&ctx, 3.to_bigint().unwrap());
        assert_eq!((b - a).val, 1.to_bigint().unwrap());
        // Check wrap-around
        let x = FpElement::new(&ctx, 1.to_bigint().unwrap());
        let y = FpElement::new(&ctx, 0.to_bigint().unwrap());
        // 0-1 = -1 mod 17 = 16
        assert_eq!((y - x).val, 16.to_bigint().unwrap());
    }

    #[test]
    fn test_fp_multiplication() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2.to_bigint().unwrap());
        let b = FpElement::new(&ctx, 3.to_bigint().unwrap());
        assert_eq!((&a * &b).val, 6.to_bigint().unwrap());
        // Check multiplication by zero
        let zero = FpElement::new(&ctx, 0.to_bigint().unwrap());
        assert_eq!((a * zero).val, 0.to_bigint().unwrap());
        // Check wrap-around
        let big = FpElement::new(&ctx, 20.to_bigint().unwrap());
        // 20 mod 17 = 3, 3 * 3 = 9 mod 17
        assert_eq!((big * b).val, 9.to_bigint().unwrap());
    }

    #[test]
    fn test_fp_negation() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2.to_bigint().unwrap());
        assert_eq!((-a).val, 15.to_bigint().unwrap()); // since 17-2=15
        let zero = FpElement::new(&ctx, 0.to_bigint().unwrap());
        assert_eq!((-zero).val, 0.to_bigint().unwrap());
    }

    #[test]
    fn test_fp_inverse() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 3.to_bigint().unwrap());
        let inv_a = a.inverse();
        // Check a*inv_a=1
        assert_eq!((a * inv_a).val, 1.to_bigint().unwrap());
        // Try another element
        let b = FpElement::new(&ctx, 5.to_bigint().unwrap());
        let inv_b = b.inverse();
        assert_eq!((b * inv_b).val, 1.to_bigint().unwrap());
    }

    #[test]
    fn test_fp_division() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2.to_bigint().unwrap());
        let b = FpElement::new(&ctx, 3.to_bigint().unwrap());
        // a/b = a * b^-1
        let div = a / b;
        // 3^-1 mod17=6, so a/b=2*6=12 mod17
        assert_eq!(div.val, 12.to_bigint().unwrap());
    }

    #[test]
    fn test_fp_exponentiation() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2.to_bigint().unwrap());
        // a^5 = 32 mod17=15
        let exp = 5;

        let res = a.pow(exp);
        assert_eq!(res.val, 15.to_bigint().unwrap());

        // Check a bigger exponent
        let exp_big = 16; // a^(16)=2^16=65536 mod17
                          // 2^16 = (2^4)^4 = (16)^4 = (16 mod17=16)^2=256 mod17=256-255=1 again and again => actually 2^16 mod17= (2^(17-1))=1 by Fermat's little theorem
        let res_big = a.pow(exp_big);

        assert_eq!(res_big.val, 1.to_bigint().unwrap());
    }
}
