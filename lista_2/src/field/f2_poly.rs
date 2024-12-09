use std::ops::{Add, Div, Mul, Neg, Sub};

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
        todo!()
    }

    fn pow(&self, exp: u64) -> Self {
        todo!()
    }
}

impl<'a> F2PolynomialElement<'a> {
    pub fn new(ctx: &'a FieldContext, coeffs: Vec<u64>) -> Self {
        assert!(ctx.is_binary());
        return F2PolynomialElement {
            context: ctx,
            coeffs: coeffs,
        };
    }

    fn poly_add(a: &[u64], b: &[u64]) -> Vec<u64> {
        let len = a.len().max(b.len());
        let mut res = Vec::with_capacity(len);
        for i in 0..len {
            let av = if i < a.len() { a[i] } else { 0 };
            let bv = if i < b.len() { b[i] } else { 0 };
            res.push(av ^ bv);
        }
        res
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

    fn normalize(&mut self, k: usize) {
        todo!()
    }
}

impl<'a> Add for F2PolynomialElement<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let ctx = self.context;
        let added = Self::poly_add(&self.coeffs, &rhs.coeffs);
        // let k = ctx.get_irreducible_poly_degree();
        let res = F2PolynomialElement {
            context: ctx,
            coeffs: Self::poly_mod(ctx, &added),
        };
        // res.normalize(k);
        res
    }
}

impl<'a> Sub for F2PolynomialElement<'a> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<'a> Neg for F2PolynomialElement<'a> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        todo!()
    }
}

impl<'a> Mul for F2PolynomialElement<'a> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<'a> Div for F2PolynomialElement<'a> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
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
    fn test_binary_polynomial_exponentiation() {
        todo!()
    }
}
