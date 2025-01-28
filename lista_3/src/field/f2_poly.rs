use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use num::{BigUint, One, Zero};

use crate::{get_binary_poly_degree, FieldContext, FieldElement};

/// Represents a polynomial over a finite field F2.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct F2PolynomialElement<'a> {
    context: &'a FieldContext,
    pub coeffs: BigUint,
}

impl<'a> FieldElement<'a> for F2PolynomialElement<'a> {
    fn zero(ctx: &'a FieldContext) -> Self {
        F2PolynomialElement {
            context: ctx,
            coeffs: BigUint::zero(),
        }
    }

    fn one(ctx: &'a FieldContext) -> Self {
        F2PolynomialElement {
            context: ctx,
            coeffs: BigUint::one(),
        }
    }

    fn is_zero(&self) -> bool {
        self.coeffs.is_zero()
    }

    fn inverse(&self) -> Self {
        let ctx = self.context;
        let inv_poly = Self::poly_inv(ctx, &self.coeffs).unwrap();
        F2PolynomialElement {
            context: ctx,
            coeffs: Self::poly_mod(ctx, &inv_poly),
        }
    }

    fn pow(&self, exp: &BigUint) -> Self {
        let ctx = self.context;
        let mut base = self.clone();
        let mut result = Self::one(ctx);
        let mut e = exp.clone();

        while e > BigUint::zero() {
            if (&e & BigUint::one()) == BigUint::one() {
                result = &result * &base;
            }
            base = &base * &base;
            e >>= 1;
        }
        result
    }

    fn pow_secure(&self, exp: &BigUint, subgroup_order: &BigUint) -> Self {
        let ctx = self.context;
        let mut base = self.clone();
        let mut result = Self::one(ctx);
        let mut dummy = Self::one(ctx);

        let exp = exp % subgroup_order;

        for shift in 0..subgroup_order.bits() {
            if ((&exp >> shift) & BigUint::one()) == BigUint::one() {
                result = &result * &base;
            } else {
                dummy = &dummy * &base;
            }
            base = &base * &base;
        }
        result
    }
}

impl<'a> Display for F2PolynomialElement<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut has_printed_term = false;
        let degree = get_binary_poly_degree(&self.coeffs);

        for i in (0..=degree).rev() {
            if (&self.coeffs >> i) & BigUint::one() == BigUint::one() {
                if has_printed_term {
                    write!(f, " + ")?;
                }
                if i == 0 {
                    write!(f, "1")?;
                } else if i == 1 {
                    write!(f, "x")?;
                } else {
                    write!(f, "x^{}", i)?;
                }
                has_printed_term = true;
            }
        }

        if !has_printed_term {
            write!(f, "0")?;
        }

        Ok(())
    }
}

impl<'a> F2PolynomialElement<'a> {
    pub fn new(ctx: &'a FieldContext, coeffs: BigUint) -> Self {
        assert!(ctx.is_binary());
        F2PolynomialElement {
            context: ctx,
            coeffs,
        }
    }

    fn poly_add(a: &BigUint, b: &BigUint) -> BigUint {
        a ^ b
    }

    fn poly_extended_gcd(a: &BigUint, b: &BigUint) -> (BigUint, BigUint, BigUint) {
        if b.is_zero() {
            return (a.clone(), BigUint::one(), BigUint::zero());
        }

        let mut r0 = a.clone();
        let mut r1 = b.clone();
        let mut u0 = BigUint::one();
        let mut u1 = BigUint::zero();
        let mut v0 = BigUint::zero();
        let mut v1 = BigUint::one();

        while !r1.is_zero() {
            let (q, r) = Self::poly_div(&r0, &r1);

            r0 = r1;
            r1 = r;

            let u_temp = u1.clone();
            u1 = &u0 ^ &(Self::poly_mul(&q, &u1));
            u0 = u_temp;

            let v_temp = v1.clone();
            v1 = &v0 ^ &(Self::poly_mul(&q, &v1));
            v0 = v_temp;
        }

        (r0, u0, v0)
    }

    fn poly_div(a: &BigUint, b: &BigUint) -> (BigUint, BigUint) {
        if b.is_zero() {
            panic!("Division by zero polynomial");
        }

        let mut quotient = BigUint::zero();
        let mut remainder = a.clone();
        let divisor_degree = get_binary_poly_degree(b);

        // Handle special cases
        if divisor_degree == 0 {
            // Divisor is 1 (b = 1)
            return (a.clone(), BigUint::zero());
        }

        while get_binary_poly_degree(&remainder) >= divisor_degree {
            let remainder_degree = get_binary_poly_degree(&remainder);
            let shift = remainder_degree - divisor_degree;

            // Update the quotient
            quotient |= BigUint::one() << shift;

            // Subtract (XOR) the shifted divisor from the remainder
            remainder ^= b << shift;

            // Avoid infinite loops by checking if remainder decreases
            if get_binary_poly_degree(&remainder) >= remainder_degree {
                panic!("Infinite loop detected in poly_div: Remainder did not decrease");
            }
        }

        (quotient, remainder)
    }

    fn poly_inv(ctx: &FieldContext, a: &BigUint) -> Option<BigUint> {
        let irreducible = &ctx.irreducible_binary_poly;
        let (gcd, u, _) = Self::poly_extended_gcd(a, irreducible);

        if gcd == BigUint::one() {
            Some(u)
        } else {
            None
        }
    }

    fn poly_mod(ctx: &FieldContext, a: &BigUint) -> BigUint {
        let mut remainder = a.clone();
        let divisor = &ctx.irreducible_binary_poly;
        let divisor_degree = get_binary_poly_degree(divisor);

        while get_binary_poly_degree(&remainder) >= divisor_degree {
            let remainder_degree = get_binary_poly_degree(&remainder);
            let shift = remainder_degree - divisor_degree;

            // Perform single shift and XOR operation
            let shifted_divisor = divisor << shift;
            remainder ^= shifted_divisor;
        }

        remainder
    }

    pub fn poly_mul(a: &BigUint, b: &BigUint) -> BigUint {
        let mut result = BigUint::zero();
        let mut b = b.clone();
        for i in 0..a.bits() as usize {
            if (a >> i) & BigUint::one() == BigUint::one() {
                result ^= &b;
            }
            b <<= 1;
        }
        result
    }
}

impl<'a> Add for F2PolynomialElement<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let ctx = self.context;
        let added = Self::poly_add(&self.coeffs, &rhs.coeffs);
        F2PolynomialElement {
            context: ctx,
            coeffs: Self::poly_mod(ctx, &added),
        }
    }
}

impl<'a> Add for &F2PolynomialElement<'a> {
    type Output = F2PolynomialElement<'a>;

    fn add(self, rhs: Self) -> Self::Output {
        let ctx = self.context;
        let added = F2PolynomialElement::poly_add(&self.coeffs, &rhs.coeffs);
        F2PolynomialElement {
            context: ctx,
            coeffs: F2PolynomialElement::poly_mod(ctx, &added),
        }
    }
}

impl<'a> Sub for F2PolynomialElement<'a> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs
    }
}

impl<'a> Sub for &F2PolynomialElement<'a> {
    type Output = F2PolynomialElement<'a>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs
    }
}

impl<'a> Neg for F2PolynomialElement<'a> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self
    }
}

impl<'a> Neg for &F2PolynomialElement<'a> {
    type Output = F2PolynomialElement<'a>;

    fn neg(self) -> Self::Output {
        self.clone()
    }
}

impl<'a> Mul for F2PolynomialElement<'a> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let ctx = self.context;
        let mult = Self::poly_mul(&self.coeffs, &rhs.coeffs);

        F2PolynomialElement {
            context: ctx,
            coeffs: Self::poly_mod(ctx, &mult),
        }
    }
}

impl<'a> Mul for &F2PolynomialElement<'a> {
    type Output = F2PolynomialElement<'a>;

    fn mul(self, rhs: Self) -> Self::Output {
        let ctx = self.context;
        let mult = F2PolynomialElement::poly_mul(&self.coeffs, &rhs.coeffs);

        F2PolynomialElement {
            context: ctx,
            coeffs: F2PolynomialElement::poly_mod(ctx, &mult),
        }
    }
}

impl<'a> Div for F2PolynomialElement<'a> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.inverse()
    }
}

impl<'a> Div for &F2PolynomialElement<'a> {
    type Output = F2PolynomialElement<'a>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self::Output {
        self * &F2PolynomialElement::inverse(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_polynomial_addition() {
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

        let sum = poly_a + poly_b;
        assert_eq!(
            sum,
            F2PolynomialElement::new(&ctx, BigUint::from(0b10110011011000u64))
        );
    }

    #[test]
    fn test_binary_polynomial_subtraction() {
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

        let diff_a_b = &poly_a - &poly_b;
        let diff_b_a = poly_b - poly_a;
        assert_eq!(
            diff_a_b,
            F2PolynomialElement::new(&ctx, BigUint::from(0b10110011011000u64))
        );
        assert_eq!(
            diff_b_a,
            F2PolynomialElement::new(&ctx, BigUint::from(0b10110011011000u64))
        );
    }

    #[test]
    fn test_binary_polynomial_multiplication() {
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

        let mult = poly_a * poly_b;
        assert_eq!(
            mult,
            F2PolynomialElement::new(&ctx, BigUint::from(0b1110001011001111u64))
        );
    }

    #[test]
    fn test_binary_polynomial_negation() {
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

        let neg_a = -poly_a;
        let neg_b = -poly_b;

        assert_eq!(
            neg_a,
            F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64))
        );
        assert_eq!(
            neg_b,
            F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64))
        );
    }

    #[test]
    fn test_binary_polynomial_inverse() {
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

        let inv_a = poly_a.inverse();
        let inv_b = poly_b.inverse();

        assert_eq!(&inv_a * &poly_a, F2PolynomialElement::one(&ctx));
        assert_eq!(&inv_b * &poly_b, F2PolynomialElement::one(&ctx));

        assert_eq!(
            inv_a,
            F2PolynomialElement::new(&ctx, BigUint::from(0b111001101011011u64))
        );
        assert_eq!(
            inv_b,
            F2PolynomialElement::new(&ctx, BigUint::from(0b1100101110010000u64))
        );
    }

    #[test]
    fn test_binary_polynomial_division_internals() {
        let a = BigUint::from(0b1011010u64);
        let b = BigUint::from(0b101u64);

        let (quotient, remainder) = F2PolynomialElement::poly_div(&a, &b);

        assert_eq!(quotient, BigUint::from(0b10010u64));
        assert_eq!(remainder, BigUint::from(0b0u64));

        let a = BigUint::from(0b1011010u64);
        let b = BigUint::from(0b11u64);

        let (quotient, remainder) = F2PolynomialElement::poly_div(&a, &b);

        assert_eq!(quotient, BigUint::from(0b110110u64));
        assert_eq!(remainder, BigUint::from(0b0u64));

        let a = BigUint::from(0b1011010u64);
        let b = BigUint::from(0b1011u64);

        let (quotient, remainder) = F2PolynomialElement::poly_div(&a, &b);

        assert_eq!(quotient, BigUint::from(0b1000u64));
        assert_eq!(remainder, BigUint::from(0b10u64));

        let a = BigUint::from(0b1011010u64);
        let b = BigUint::from(0b101011111u64);

        assert_eq!(get_binary_poly_degree(&a), 6);
        assert_eq!(get_binary_poly_degree(&b), 8);

        let (quotient, remainder) = F2PolynomialElement::poly_div(&a, &b);

        assert_eq!(quotient, BigUint::zero());
        assert_eq!(remainder, BigUint::from(0b1011010u64));
    }

    #[test]
    fn test_binary_polynomial_extended_gcd() {
        let a = BigUint::from(0b1011u64); // Polynomial a(x) = x^3 + x + 1
        let b = BigUint::from(0b11u64); // Polynomial b(x) = x + 1

        let (gcd, u, v) = F2PolynomialElement::poly_extended_gcd(&a, &b);

        // GCD of x^3 + x + 1 and x + 1 should be 1
        assert_eq!(gcd, BigUint::one());

        // Verify Bézout's identity: u * a + v * b = gcd
        let lhs = (F2PolynomialElement::poly_mul(&u, &a)) ^ (F2PolynomialElement::poly_mul(&v, &b));
        assert_eq!(lhs, gcd);

        let a = BigUint::from(0b110101u64); // Polynomial a(x) = x^5 + x^4 + x^2 + 1
        let b = BigUint::from(0b101u64); // Polynomial b(x) = x^2 + x + 1

        let (gcd, u, v) = F2PolynomialElement::poly_extended_gcd(&a, &b);

        // GCD of these polynomials should be x + 1
        assert_eq!(gcd, BigUint::from(0b11u64));

        // Verify Bézout's identity: u * a + v * b = gcd
        let lhs = (F2PolynomialElement::poly_mul(&u, &a)) ^ (F2PolynomialElement::poly_mul(&v, &b));
        assert_eq!(lhs, gcd);

        let a = BigUint::from(0b100011011u64); // Polynomial a(x) = x^8 + x^4 + x^3 + x + 1 (irreducible)
        let b = BigUint::from(0b0u64); // Polynomial b(x) = 0

        let (gcd, u, v) = F2PolynomialElement::poly_extended_gcd(&a, &b);

        // GCD with zero should be the non-zero polynomial
        assert_eq!(gcd, a);

        // u and v should satisfy Bézout's identity
        let lhs = (F2PolynomialElement::poly_mul(&u, &a)) ^ (F2PolynomialElement::poly_mul(&v, &b));
        assert_eq!(lhs, gcd);

        let a = BigUint::from(0b101010u64); // Polynomial a(x) = x^5 + x^3 + x
        let b = BigUint::from(0b1110u64); // Polynomial b(x) = x^3 + x^2 + x

        let (gcd, u, v) = F2PolynomialElement::poly_extended_gcd(&a, &b);

        // GCD of these polynomials should be x
        assert_eq!(gcd, BigUint::from(0b1110u64));

        // Verify Bézout's identity: u * a + v * b = gcd
        let lhs = (F2PolynomialElement::poly_mul(&u, &a)) ^ (F2PolynomialElement::poly_mul(&v, &b));
        assert_eq!(lhs, gcd);

        let a = BigUint::from(0b111u64); // Polynomial a(x) = x^2 + x + 1
        let b = BigUint::from(0b1u64); // Polynomial b(x) = 1

        let (gcd, u, v) = F2PolynomialElement::poly_extended_gcd(&a, &b);

        // GCD of any polynomial with 1 should be 1
        assert_eq!(gcd, BigUint::one());

        // Verify Bézout's identity: u * a + v * b = gcd
        let lhs = (F2PolynomialElement::poly_mul(&u, &a)) ^ (F2PolynomialElement::poly_mul(&v, &b));
        assert_eq!(lhs, gcd);
    }

    #[test]
    fn test_binary_polynomial_division() {
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

        let div_a_b = &poly_a / &poly_b;
        let div_b_a = poly_b / poly_a;

        assert_eq!(
            div_a_b,
            F2PolynomialElement::new(&ctx, BigUint::from(0b11001000101111u64))
        );
        assert_eq!(
            div_b_a,
            F2PolynomialElement::new(&ctx, BigUint::from(0b1101101111011110u64))
        );
    }

    #[test]
    fn test_binary_polynomial_display() {
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

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
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

        let order = BigUint::from(65535u32);

        let exp: BigUint = BigUint::from(5u64);

        let exp_a = poly_a.pow(&exp);
        let exp_b = poly_b.pow(&exp);

        assert_eq!(
            exp_a,
            &(&(&(&poly_a * &poly_a) * &poly_a) * &poly_a) * &poly_a
        );
        assert_eq!(
            exp_b,
            &(&(&(&poly_b * &poly_b) * &poly_b) * &poly_b) * &poly_b
        );

        assert_eq!(exp_a, poly_a.pow_secure(&exp, &order));
        assert_eq!(exp_b, poly_b.pow_secure(&exp, &order));
    }

    #[test]
    fn test_binary_polynomial_degree() {
        let irreducible_poly = BigUint::from(0b11111101111101001u64);
        let ctx = FieldContext::new_binary(irreducible_poly);

        let poly_a = F2PolynomialElement::new(&ctx, BigUint::from(0b1000101000011101u64));
        let poly_b = F2PolynomialElement::new(&ctx, BigUint::from(0b1010011011000101u64));

        assert_eq!(get_binary_poly_degree(&poly_a.coeffs), 15);
        assert_eq!(get_binary_poly_degree(&poly_b.coeffs), 15);

        assert_eq!(ctx.get_irreducible_poly_degree(), 16);
    }
}
