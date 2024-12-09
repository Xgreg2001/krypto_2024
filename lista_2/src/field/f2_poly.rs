use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use num::Zero;

use crate::{get_binary_poly_degree, FieldContext, FieldElement};

/// Represents a polynomial over a finite field F2.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct F2PolynomialElement<'a> {
    context: &'a FieldContext,
    coeffs: Vec<u64>,
}

impl<'a> FieldElement<'a> for F2PolynomialElement<'a> {
    fn zero(ctx: &'a FieldContext) -> Self {
        return F2PolynomialElement {
            context: ctx,
            coeffs: vec![0],
        };
    }

    fn one(ctx: &'a FieldContext) -> Self {
        return F2PolynomialElement {
            context: ctx,
            coeffs: vec![1],
        };
    }

    fn is_zero(&self) -> bool {
        return self.coeffs.iter().all(|&c| c == 0);
    }

    fn inverse(&self) -> Self {
        let ctx = self.context;
        let inv_poly = Self::poly_inv(ctx, &self.coeffs).unwrap();
        let res = F2PolynomialElement {
            context: ctx,
            coeffs: Self::poly_mod(ctx, &inv_poly),
        };
        res
    }

    fn pow(&self, exp: u64) -> Self {
        let ctx = self.context;
        let mut base = self.clone();
        let mut result = Self::one(ctx);
        let mut e = exp;

        while e > 0 {
            if (e & 1) == 1 {
                result = result * base.clone();
            }
            base = base.clone() * base.clone();
            e >>= 1;
        }
        result
    }
}

impl<'a> Display for F2PolynomialElement<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut has_printed_term = false;
        let mut degree = get_binary_poly_degree(&self.coeffs) - 1;
        let mut first_chunk = true;

        for &chunk in self.coeffs.iter() {
            if first_chunk {
                first_chunk = false;
                let leftmost_bit = 64 - chunk.leading_zeros();

                for i in (0..leftmost_bit).rev() {
                    if (chunk >> i) & 1 == 1 {
                        if has_printed_term {
                            write!(f, " + ")?;
                        }
                        if degree == 0 {
                            write!(f, "1")?;
                        } else if degree == 1 {
                            write!(f, "x")?;
                        } else {
                            write!(f, "x^{}", degree)?;
                        }
                        has_printed_term = true;
                    }

                    if degree == 0 {
                        break;
                    }

                    degree -= 1;
                }
            } else {
                for i in (0..64).rev() {
                    if (chunk >> i) & 1 == 1 {
                        if has_printed_term {
                            write!(f, " + ")?;
                        }
                        if degree == 0 {
                            write!(f, "1")?;
                        } else if degree == 1 {
                            write!(f, "x")?;
                        } else {
                            write!(f, "x^{}", degree)?;
                        }
                        has_printed_term = true;
                    }

                    if degree == 0 {
                        break;
                    }

                    degree -= 1;
                }
            }
        }

        if !has_printed_term {
            write!(f, "0")?;
        }

        Ok(())
    }
}

impl<'a> F2PolynomialElement<'a> {
    pub fn new(ctx: &'a FieldContext, coeffs: Vec<u64>) -> Self {
        assert!(ctx.is_binary());
        return F2PolynomialElement {
            context: ctx,
            coeffs,
        };
    }

    fn poly_add(a: &[u64], b: &[u64]) -> Vec<u64> {
        let max_len = a.len().max(b.len());
        let mut result = vec![0u64; max_len];

        for i in 0..max_len {
            result[i] = if i < a.len() { a[i] } else { 0 } ^ if i < b.len() { b[i] } else { 0 };
        }

        while result.last() == Some(&0) {
            result.pop();
        }

        result
    }

    fn poly_extended_gcd(a: &[u64], b: &[u64]) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
        let mut r0 = a.to_vec();
        let mut r1 = b.to_vec();
        let mut u0 = vec![1u64];
        let mut u1 = vec![0u64];
        let mut v0 = vec![0u64];
        let mut v1 = vec![1u64];

        while !r1.is_empty() {
            let q = Self::poly_div(&r0, &r1).0; // Quotient of r0 / r1
                                                // addition and subtraction are the same operations
            let new_r = Self::poly_add(&r0, &Self::poly_mul(&q, &r1));
            let new_u = Self::poly_add(&u0, &Self::poly_mul(&q, &u1));
            let new_v = Self::poly_add(&v0, &Self::poly_mul(&q, &v1));

            r0 = r1;
            r1 = new_r;
            u0 = u1;
            u1 = new_u;
            v0 = v1;
            v1 = new_v;
        }

        (r0, u0, v0)
    }

    fn poly_div(a: &[u64], b: &[u64]) -> (Vec<u64>, Vec<u64>) {
        let mut quotient = vec![0u64; a.len()];
        let mut remainder = a.to_vec();
        let divisor_degree = get_binary_poly_degree(&b.to_vec());
        let divisor = b;

        while get_binary_poly_degree(&remainder) >= divisor_degree {
            let remainder_degree = get_binary_poly_degree(&remainder);
            let shift = remainder_degree - divisor_degree;

            for i in 0..divisor.len() {
                if i + shift / 64 < remainder.len() {
                    remainder[i + shift / 64] ^= divisor[i] << (shift % 64);
                    if shift % 64 > 0 && i + shift / 64 + 1 < remainder.len() {
                        remainder[i + shift / 64 + 1] ^= divisor[i] >> (64 - shift % 64);
                    }
                }
            }

            if shift / 64 < quotient.len() {
                quotient[shift / 64] |= 1u64 << (shift % 64);
            }
        }

        while remainder.last() == Some(&0) {
            remainder.pop();
        }

        while quotient.last() == Some(&0) {
            quotient.pop();
        }

        (quotient, remainder)
    }

    fn poly_inv(ctx: &FieldContext, a: &[u64]) -> Option<Vec<u64>> {
        let irreducible = &ctx.irreducible_binary_poly;
        let (gcd, u, _) = Self::poly_extended_gcd(a, irreducible);

        if gcd == vec![1] {
            Some(u)
        } else {
            None
        }
    }

    fn poly_mod(ctx: &FieldContext, a: &[u64]) -> Vec<u64> {
        let mut remainder = a.to_vec();
        let divisor = &ctx.irreducible_binary_poly;
        let divisor_degree = get_binary_poly_degree(divisor);

        loop {
            let remainder_degree = get_binary_poly_degree(&remainder);

            if remainder_degree < divisor_degree {
                break;
            }

            let shift = remainder_degree - divisor_degree;

            for i in 0..divisor.len() {
                if i + shift / 64 < remainder.len() {
                    remainder[i + shift / 64] ^= divisor[i] << (shift % 64);
                    if shift % 64 > 0 && i + shift / 64 + 1 < remainder.len() {
                        remainder[i + shift / 64 + 1] ^= divisor[i] >> (64 - shift % 64);
                    }
                }
            }
        }

        while remainder.last() == Some(&0) {
            remainder.pop();
        }

        remainder
    }

    pub fn poly_mul(a: &[u64], b: &[u64]) -> Vec<u64> {
        let max_len = a.len() + b.len();
        let mut result = vec![0u64; max_len];

        for &a_val in a.iter() {
            for j in 0..64 {
                if (a_val >> j) & 1 == 1 {
                    for (k, &b_val) in b.iter().enumerate() {
                        let shift = j + k * 64;
                        let word = shift / 64;
                        let bit = shift % 64;

                        if word < result.len() {
                            result[word] ^= b_val << bit;
                            if bit > 0 && word + 1 < result.len() {
                                result[word + 1] ^= b_val >> (64 - bit);
                            }
                        }
                    }
                }
            }
        }

        while result.last() == Some(&0) {
            result.pop();
        }

        result
    }
}

impl<'a> Add for F2PolynomialElement<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let ctx = self.context;
        let added = Self::poly_add(&self.coeffs, &rhs.coeffs);
        let res = F2PolynomialElement {
            context: ctx,
            coeffs: Self::poly_mod(ctx, &added),
        };
        res
    }
}

impl<'a> Sub for F2PolynomialElement<'a> {
    type Output = Self;

    // OPTIMIZE: do we need to clone here?
    fn sub(self, rhs: Self) -> Self::Output {
        return self.clone() + rhs.clone();
    }
}

impl<'a> Neg for F2PolynomialElement<'a> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        return self;
    }
}

impl<'a> Mul for F2PolynomialElement<'a> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let ctx = self.context;
        let mult = Self::poly_mul(&self.coeffs, &rhs.coeffs);
        let res = F2PolynomialElement {
            context: ctx,
            coeffs: Self::poly_mod(ctx, &mult),
        };
        res
    }
}

impl<'a> Div for F2PolynomialElement<'a> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        return self * rhs.inverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_polynomial_addition() {
        let irreducible_poly = vec![0b11111101111101001];
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, vec![0b1000101000011101]);
        let poly_b = F2PolynomialElement::new(&ctx, vec![0b1010011011000101]);

        let sum = poly_a + poly_b;
        assert_eq!(sum, F2PolynomialElement::new(&ctx, vec![0b10110011011000]));
    }

    #[test]
    fn test_binary_polynomial_subtraction() {
        let irreducible_poly = vec![0b11111101111101001];
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, vec![0b1000101000011101]);
        let poly_b = F2PolynomialElement::new(&ctx, vec![0b1010011011000101]);

        let diff_a_b = poly_a.clone() - poly_b.clone();
        let diff_b_a = poly_b - poly_a;
        assert_eq!(
            diff_a_b,
            F2PolynomialElement::new(&ctx, vec![0b10110011011000])
        );
        assert_eq!(
            diff_b_a,
            F2PolynomialElement::new(&ctx, vec![0b10110011011000])
        );
    }

    #[test]
    fn test_binary_polynomial_multiplication() {
        let irreducible_poly = vec![0b11111101111101001];
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, vec![0b1000101000011101]);
        let poly_b = F2PolynomialElement::new(&ctx, vec![0b1010011011000101]);

        let mult = poly_a * poly_b;
        assert_eq!(
            mult,
            F2PolynomialElement::new(&ctx, vec![0b1110001011001111])
        );
    }

    #[test]
    fn test_binary_polynomial_negation() {
        let irreducible_poly = vec![0b11111101111101001];
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, vec![0b1000101000011101]);
        let poly_b = F2PolynomialElement::new(&ctx, vec![0b1010011011000101]);

        let neg_a = -poly_a;
        let neg_b = -poly_b;

        assert_eq!(
            neg_a,
            F2PolynomialElement::new(&ctx, vec![0b1000101000011101])
        );
        assert_eq!(
            neg_b,
            F2PolynomialElement::new(&ctx, vec![0b1010011011000101])
        );
    }

    #[test]
    fn test_binary_polynomial_inverse() {
        let irreducible_poly = vec![0b11111101111101001];
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, vec![0b1000101000011101]);
        let poly_b = F2PolynomialElement::new(&ctx, vec![0b1010011011000101]);

        let inv_a = poly_a.inverse();
        let inv_b = poly_b.inverse();

        assert_eq!(
            inv_a,
            F2PolynomialElement::new(&ctx, vec![0b111001101011011])
        );
        assert_eq!(
            inv_b,
            F2PolynomialElement::new(&ctx, vec![0b1100101110010000])
        );
    }

    #[test]
    fn test_binary_polynomial_division() {
        let irreducible_poly = vec![0b11111101111101001];
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, vec![0b1000101000011101]);
        let poly_b = F2PolynomialElement::new(&ctx, vec![0b1010011011000101]);

        let div_a_b = poly_a.clone() / poly_b.clone();
        let div_b_a = poly_b / poly_a;

        assert_eq!(
            div_a_b,
            F2PolynomialElement::new(&ctx, vec![0b11001000101111])
        );
        assert_eq!(
            div_b_a,
            F2PolynomialElement::new(&ctx, vec![0b1101101111011110])
        );
    }

    #[test]
    fn test_binary_polynomial_display() {
        let irreducible_poly = vec![0b11111101111101001];
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, vec![0b1000101000011101]);
        let poly_b = F2PolynomialElement::new(&ctx, vec![0b1010011011000101]);

        assert_eq!(
            format!("{}", poly_a),
            "x^15 + x^11 + x^9 + x^4 + x^3 + x^2 + 1"
        );
        assert_eq!(
            format!("{}", poly_b),
            "x^15 + x^13 + x^10 + x^9 + x^7 + x^6 + x^2 + 1"
        );
    }

    #[test]
    fn test_binary_polynomial_exponentiation() {
        let irreducible_poly = vec![0b11111101111101001];
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, vec![0b1000101000011101]);
        let poly_b = F2PolynomialElement::new(&ctx, vec![0b1010011011000101]);

        const EXP: u64 = 5;

        let exp_a = poly_a.pow(EXP);
        let exp_b = poly_b.pow(EXP);

        assert_eq!(
            exp_a,
            poly_a.clone() * poly_a.clone() * poly_a.clone() * poly_a.clone() * poly_a
        );
        assert_eq!(
            exp_b,
            poly_b.clone() * poly_b.clone() * poly_b.clone() * poly_b.clone() * poly_b
        );
    }
}
