use num::bigint::{BigInt, ToBigInt};
use std::fmt;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

use super::fp::FpElement;
use crate::{FieldContext, FieldElement};

/// Polynomial-based field extension element: F_{p^k}.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FpPolynomialElement<'a> {
    context: &'a FieldContext,
    coeffs: Vec<FpElement<'a>>,
}

impl fmt::Display for FpPolynomialElement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_printed_term = false;
        let degree = self.coeffs.len().saturating_sub(1);

        for i in (0..=degree).rev() {
            let c = &self.coeffs[i];
            if c.is_zero() {
                continue;
            }

            if has_printed_term {
                write!(f, " + ")?;
            }

            let exp = i;
            let val_str = format!("{}", c);
            if exp == 0 {
                write!(f, "{}", val_str)?;
            } else if exp == 1 {
                write!(f, "{}*x", val_str)?;
            } else {
                write!(f, "{}*x^{}", val_str, exp)?;
            }

            has_printed_term = true;
        }

        if !has_printed_term {
            write!(f, "0")?;
        }

        Ok(())
    }
}

impl<'a> FpPolynomialElement<'a> {
    pub fn new(ctx: &'a FieldContext, coeffs: Vec<FpElement<'a>>) -> Self {
        if !ctx.is_poly() {
            panic!("Field context is not a polynomial field");
        }

        let k = ctx.get_irreducible_poly_degree();
        let mut el = Self {
            context: ctx,
            coeffs,
        };
        el.normalize(k);
        el
    }

    pub fn from_vec(context: &'a FieldContext, coeffs: Vec<i64>) -> Self {
        let big_int_coeffs: Vec<BigInt> = coeffs.iter().map(|c| c.to_bigint().unwrap()).collect();
        let fp_coeffs = Self::poly_to_fp(context, &big_int_coeffs);
        Self::new(context, fp_coeffs)
    }

    fn normalize(&mut self, k: usize) {
        while self.coeffs.len() < k {
            self.coeffs.push(FpElement::zero(self.context));
        }
        self.coeffs = Self::poly_mod(&self.coeffs, self.context);
    }

    fn poly_to_fp<'b>(context: &'b FieldContext, poly: &[BigInt]) -> Vec<FpElement<'b>> {
        poly.iter().map(|v| context.to_fp(v.clone())).collect()
    }

    fn poly_add(
        ctx: &'a FieldContext,
        a: &[FpElement<'a>],
        b: &[FpElement<'a>],
    ) -> Vec<FpElement<'a>> {
        let len = a.len().max(b.len());
        let mut res = Vec::with_capacity(len);
        let zero = FpElement::zero(ctx);
        for i in 0..len {
            let av = if i < a.len() { &a[i] } else { &zero };
            let bv = if i < b.len() { &b[i] } else { &zero };
            res.push(av + bv);
        }
        res
    }

    fn poly_sub(
        ctx: &'a FieldContext,
        a: &[FpElement<'a>],
        b: &[FpElement<'a>],
    ) -> Vec<FpElement<'a>> {
        let len = a.len().max(b.len());
        let mut res = Vec::with_capacity(len);
        let zero = FpElement::zero(ctx);
        for i in 0..len {
            let av = if i < a.len() { &a[i] } else { &zero };
            let bv = if i < b.len() { &b[i] } else { &zero };
            res.push(av - bv);
        }
        res
    }

    fn poly_mul(
        ctx: &'a FieldContext,
        a: &[FpElement<'a>],
        b: &[FpElement<'a>],
    ) -> Vec<FpElement<'a>> {
        let mut res = vec![FpElement::zero(ctx); a.len() + b.len() - 1];
        for i in 0..a.len() {
            for j in 0..b.len() {
                res[i + j] = &res[i + j] + &(&a[i] * &b[j]);
            }
        }

        res
    }

    fn poly_mod(a: &[FpElement<'a>], ctx: &'a FieldContext) -> Vec<FpElement<'a>> {
        let irreducible_poly = Self::poly_to_fp(ctx, &ctx.irreducible_poly);
        let deg_mod = ctx.get_irreducible_poly_degree();
        let mut r = a.to_vec();
        while r.len() > deg_mod {
            let leading = &r[r.len() - 1];
            if leading.is_zero() {
                r.pop();
                continue;
            }
            let diff = r.len() - irreducible_poly.len();
            let mut temp = vec![FpElement::zero(ctx); diff];
            for c in &irreducible_poly {
                temp.push(c * leading);
            }
            r = Self::poly_sub(ctx, &r, &temp);
            while r.last().map(|x| x.is_zero()) == Some(true) {
                r.pop();
            }
        }
        if r.is_empty() {
            r.push(FpElement::zero(ctx));
        }
        r
    }

    fn poly_is_zero(a: &[FpElement<'a>]) -> bool {
        for c in a.iter() {
            if !c.is_zero() {
                return false;
            }
        }
        true
    }

    fn poly_div(
        ctx: &'a FieldContext,
        a: &[FpElement<'a>],
        b: &[FpElement<'a>],
    ) -> (Vec<FpElement<'a>>, Vec<FpElement<'a>>) {
        if Self::poly_is_zero(b) {
            panic!("Division by zero polynomial");
        }
        let mut aa = a.to_vec();
        let mut qq = Vec::<FpElement<'a>>::new();

        while aa.len() >= b.len() && !Self::poly_is_zero(&aa) {
            let factor = &aa[aa.len() - 1] * &b[b.len() - 1].inverse();
            let deg_diff = aa.len() - b.len();
            let mut temp = vec![FpElement::zero(ctx); deg_diff];
            for c in b {
                temp.push(c * &factor);
            }
            while qq.len() < deg_diff + 1 {
                qq.push(FpElement::zero(ctx));
            }
            qq[deg_diff] = &qq[deg_diff] + &factor;
            aa = Self::poly_sub(ctx, &aa, &temp);
            while aa.last().map(|x| x.is_zero()) == Some(true) {
                aa.pop();
            }
        }

        if qq.is_empty() {
            qq.push(FpElement::zero(ctx));
        }
        if aa.is_empty() {
            aa.push(FpElement::zero(ctx));
        }

        (qq, aa)
    }

    fn poly_inv(a: &[FpElement<'a>], ctx: &'a FieldContext) -> Vec<FpElement<'a>> {
        let irreducible_poly = Self::poly_to_fp(ctx, &ctx.irreducible_poly);
        let mut r0 = irreducible_poly.clone();
        let mut r1 = a.to_vec();
        let mut s0 = vec![FpElement::one(ctx)];
        let mut s1 = vec![FpElement::zero(ctx)];
        let mut t0 = vec![FpElement::zero(ctx)];
        let mut t1 = vec![FpElement::one(ctx)];

        while !Self::poly_is_zero(&r1) {
            let (q, r) = Self::poly_div(ctx, &r0, &r1);
            let r2 = r.clone();
            let s2 = Self::poly_sub(ctx, &s0, &Self::poly_mul(ctx, &q, &s1));
            let t2 = Self::poly_sub(ctx, &t0, &Self::poly_mul(ctx, &q, &t1));

            r0 = r1;
            r1 = r2;
            s0 = s1;
            s1 = s2;
            t0 = t1;
            t1 = t2;
        }

        let inv_lead = r0.last().unwrap().inverse();
        let inv = Self::poly_mul(ctx, &t0, &[inv_lead]);
        Self::poly_mod(&inv, ctx)
    }
}

impl<'a> Index<usize> for FpPolynomialElement<'a> {
    type Output = FpElement<'a>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.coeffs[index]
    }
}

impl IndexMut<usize> for FpPolynomialElement<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.coeffs[index]
    }
}

impl<'a> Add for FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn add(self, other: FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let added = Self::poly_add(ctx, &self.coeffs, &other.coeffs);
        let k = ctx.get_irreducible_poly_degree();
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: added,
        };
        res.coeffs = Self::poly_mod(&res.coeffs, ctx);
        res.normalize(k);
        res
    }
}

impl<'a> Add for &FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn add(self, other: &FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let added = FpPolynomialElement::poly_add(ctx, &self.coeffs, &other.coeffs);
        let k = ctx.get_irreducible_poly_degree();
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: added,
        };
        res.coeffs = FpPolynomialElement::poly_mod(&res.coeffs, ctx);
        res.normalize(k);
        res
    }
}

impl<'a> Sub for FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn sub(self, other: FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let subbed = Self::poly_sub(ctx, &self.coeffs, &other.coeffs);
        let k = ctx.get_irreducible_poly_degree();
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: subbed,
        };
        res.coeffs = Self::poly_mod(&res.coeffs, ctx);
        res.normalize(k);
        res
    }
}

impl<'a> Sub for &FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn sub(self, other: &FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let subbed = FpPolynomialElement::poly_sub(ctx, &self.coeffs, &other.coeffs);
        let k = ctx.get_irreducible_poly_degree();
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: subbed,
        };
        res.coeffs = FpPolynomialElement::poly_mod(&res.coeffs, ctx);
        res.normalize(k);
        res
    }
}

impl<'a> Neg for FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn neg(self) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let negcoeffs: Vec<FpElement<'a>> = self.coeffs.into_iter().map(|c| c.neg()).collect();
        let k = ctx.get_irreducible_poly_degree();
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: negcoeffs,
        };
        res.normalize(k);
        res
    }
}

impl<'a> Neg for &FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn neg(self) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let negcoeffs: Vec<FpElement<'a>> = self.coeffs.iter().map(|c| c.neg()).collect();
        let k = ctx.get_irreducible_poly_degree();
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: negcoeffs,
        };
        res.normalize(k);
        res
    }
}

impl<'a> Mul for FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn mul(self, other: FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let k = ctx.get_irreducible_poly_degree();
        let mut prod = Self::poly_mul(ctx, &self.coeffs, &other.coeffs);
        prod = Self::poly_mod(&prod, ctx);
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: prod,
        };
        res.normalize(k);
        res
    }
}

impl<'a> Mul for &FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn mul(self, other: &FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let k = ctx.get_irreducible_poly_degree();
        let mut prod = FpPolynomialElement::poly_mul(ctx, &self.coeffs, &other.coeffs);
        prod = FpPolynomialElement::poly_mod(&prod, ctx);
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: prod,
        };
        res.normalize(k);
        res
    }
}

impl<'a> Div for FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn div(self, other: FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        self * other.inverse()
    }
}

impl<'a> Div for &FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn div(self, other: &FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        self * &FpPolynomialElement::inverse(other)
    }
}

impl<'a> FieldElement<'a> for FpPolynomialElement<'a> {
    fn zero(ctx: &'a FieldContext) -> Self {
        let k = ctx.get_irreducible_poly_degree();
        let coeffs = vec![FpElement::zero(ctx); k];
        FpPolynomialElement {
            context: ctx,
            coeffs,
        }
    }

    fn one(ctx: &'a FieldContext) -> Self {
        let k = ctx.get_irreducible_poly_degree();
        let mut coeffs = vec![FpElement::zero(ctx); k];
        coeffs[0] = FpElement::one(ctx);
        FpPolynomialElement {
            context: ctx,
            coeffs,
        }
    }

    fn is_zero(&self) -> bool {
        self.coeffs.iter().all(|c| c.is_zero())
    }

    fn inverse(&self) -> Self {
        let ctx = self.context;
        let inv_poly = Self::poly_inv(&self.coeffs, ctx);
        let k = ctx.get_irreducible_poly_degree();
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: inv_poly,
        };
        res.normalize(k);
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

#[cfg(test)]
mod tests {
    use super::*;
    use num::bigint::ToBigInt;

    #[test]
    fn test_polynomial_addition() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);
        // poly_a = 2+3x, poly_b=5+x
        let poly_a = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(2.to_bigint().unwrap()),
                ctx.to_fp(3.to_bigint().unwrap()),
            ],
        );

        let poly_b = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(5.to_bigint().unwrap()),
                ctx.to_fp(1.to_bigint().unwrap()),
            ],
        );

        // a+b = (2+5)+(3+1)x=7+4x
        let sum = &poly_a + &poly_b;
        assert_eq!(sum[0].val, 7.to_bigint().unwrap());
        assert_eq!(sum[1].val, 4.to_bigint().unwrap());
    }

    #[test]
    fn test_polynomial_subtraction() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);
        let poly_a = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(2.to_bigint().unwrap()),
                ctx.to_fp(3.to_bigint().unwrap()),
            ],
        );

        let poly_b = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(5.to_bigint().unwrap()),
                ctx.to_fp(1.to_bigint().unwrap()),
            ],
        );

        // b-a = (5-2) + (1-3)x = 3 + (-2)x =3+15x mod17
        let diff = &poly_b - &poly_a;
        assert_eq!(diff[0].val, 3.to_bigint().unwrap());
        assert_eq!(diff[1].val, 15.to_bigint().unwrap());
    }

    #[test]
    fn test_polynomial_multiplication() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);
        let poly_a = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(2.to_bigint().unwrap()),
                ctx.to_fp(3.to_bigint().unwrap()),
            ],
        );

        let poly_b = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(5.to_bigint().unwrap()),
                ctx.to_fp(1.to_bigint().unwrap()),
            ],
        );

        let prod = &poly_a * &poly_b;
        // Expected result from previous reasoning: 7 + 0*x
        assert_eq!(prod[0].val, 1.to_bigint().unwrap());
        assert_eq!(prod[1].val, 14.to_bigint().unwrap());

        // Check multiplication by zero polynomial
        let zero_poly = FpPolynomialElement::zero(&ctx);
        let zero_prod = &poly_a * &zero_poly;
        // should be zero polynomial:
        assert!(zero_prod.is_zero());
    }

    #[test]
    fn test_polynomial_negation() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);
        let poly_a = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(2.to_bigint().unwrap()),
                ctx.to_fp(3.to_bigint().unwrap()),
            ],
        );

        let neg_a = -&poly_a;
        // -2 mod17=15, -3mod17=14
        assert_eq!(neg_a[0].val, 15.to_bigint().unwrap());
        assert_eq!(neg_a[1].val, 14.to_bigint().unwrap());

        let zero_poly = FpPolynomialElement::zero(&ctx);
        assert!((-&zero_poly).is_zero());
    }

    #[test]
    fn test_polynomial_inverse() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);
        let poly_b = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(5.to_bigint().unwrap()),
                ctx.to_fp(1.to_bigint().unwrap()),
            ],
        );

        let inv_b = poly_b.inverse();
        // poly_b * inv_b = 1
        let one = FpPolynomialElement::one(&ctx);
        let check_inv = &poly_b * &inv_b;
        assert_eq!(check_inv, one);
    }

    #[test]
    fn test_polynomial_division() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);
        let poly_a = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(2.to_bigint().unwrap()),
                ctx.to_fp(3.to_bigint().unwrap()),
            ],
        );

        let poly_b = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(5.to_bigint().unwrap()),
                ctx.to_fp(1.to_bigint().unwrap()),
            ],
        );

        let quotient = &poly_a / &poly_b;
        let check_div = &quotient * &poly_b;
        assert_eq!(check_div, poly_a);
    }

    #[test]
    fn test_polynomial_exponentiation() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);
        let poly_a = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(2.to_bigint().unwrap()),
                ctx.to_fp(3.to_bigint().unwrap()),
            ],
        ); // 2+3x
        let exp = 5;

        assert_eq!(
            poly_a.pow(exp),
            FpPolynomialElement::new(
                &ctx,
                vec![
                    ctx.to_fp(15.to_bigint().unwrap()),
                    ctx.to_fp(4.to_bigint().unwrap())
                ]
            )
        );

        let exp_big = 16; // poly_a^(16)
        assert_eq!(
            poly_a.pow(exp_big),
            FpPolynomialElement::new(
                &ctx,
                vec![
                    ctx.to_fp(1.to_bigint().unwrap()),
                    ctx.to_fp(6.to_bigint().unwrap())
                ]
            )
        );
    }

    #[test]
    fn test_polynomial_display() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);

        // poly = 0
        let zero_poly = FpPolynomialElement::zero(&ctx);
        assert_eq!(format!("{}", zero_poly), "0");

        // poly = 2 + 3x
        let poly_a = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(2.to_bigint().unwrap()),
                ctx.to_fp(3.to_bigint().unwrap()),
            ],
        );
        // displayed as: "3*x + 2"
        let disp_a = format!("{}", poly_a);
        assert!(disp_a.contains("3*x"));
        assert!(disp_a.contains("2"));

        // poly = 5 (no x term)
        let poly_c = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(5.to_bigint().unwrap())]);
        // just "5"
        assert_eq!(format!("{}", poly_c), "5");
    }

    #[test]
    fn test_additional_polynomial_cases() {
        let p = 17.to_bigint().unwrap();
        let irreducible_poly = vec![
            3.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
            1.to_bigint().unwrap(),
        ];
        let ctx = FieldContext::new_poly(p, irreducible_poly);

        // Another polynomial
        let poly_x_only = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(0.to_bigint().unwrap()),
                ctx.to_fp(1.to_bigint().unwrap()),
            ],
        ); // x
           // x*x = x^2 = -1 mod poly => = p-1=16
        let prod = &poly_x_only * &poly_x_only;
        assert_eq!(prod[0].val, 14.to_bigint().unwrap());
        assert_eq!(prod[1].val, 16.to_bigint().unwrap());

        // Check that a polynomial times its inverse gives one
        let poly_rand = FpPolynomialElement::new(
            &ctx,
            vec![
                ctx.to_fp(4.to_bigint().unwrap()),
                ctx.to_fp(9.to_bigint().unwrap()),
            ],
        );

        let inv_rand = poly_rand.inverse();
        let check_inv = poly_rand * inv_rand;
        assert_eq!(check_inv, FpPolynomialElement::one(&ctx));
    }

    #[test]
    fn test_mizoz_example() {
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

        let p1 = FpPolynomialElement::from_vec(&ctx, vec![8, 6, 7, 7, 3, 9, 1]);
        let p2 = FpPolynomialElement::from_vec(&ctx, vec![3, 7, 0, 3, 4, 2, 4]);

        assert_eq!(
            &p1 + &p2,
            FpPolynomialElement::from_vec(&ctx, vec![0, 2, 7, 10, 7, 0, 5])
        );

        assert_eq!(
            -&p1,
            FpPolynomialElement::from_vec(&ctx, vec![3, 5, 4, 4, 8, 2, 10])
        );

        assert_eq!(
            -&p2,
            FpPolynomialElement::from_vec(&ctx, vec![8, 4, 0, 8, 7, 9, 7])
        );

        assert_eq!(
            &p1 - &p2,
            FpPolynomialElement::from_vec(&ctx, vec![5, 10, 7, 4, 10, 7, 8])
        );

        assert_eq!(
            &p2 - &p1,
            FpPolynomialElement::from_vec(&ctx, vec![6, 1, 4, 7, 1, 4, 3])
        );

        assert_eq!(
            &p1 * &p2,
            FpPolynomialElement::from_vec(&ctx, vec![4, 10, 10, 4, 10, 1, 3])
        );

        assert_eq!(
            p1.inverse(),
            FpPolynomialElement::from_vec(&ctx, vec![0, 5, 8, 4, 1, 5, 2])
        );

        assert_eq!(
            p2.inverse(),
            FpPolynomialElement::from_vec(&ctx, vec![10, 2, 2, 6, 2, 2, 10])
        );

        assert_eq!(
            &p1 / &p2,
            FpPolynomialElement::from_vec(&ctx, vec![6, 2, 0, 3, 0, 0, 9])
        );

        assert_eq!(
            p2 / p1,
            FpPolynomialElement::from_vec(&ctx, vec![7, 2, 4, 2, 1, 1, 10])
        );
    }
}
