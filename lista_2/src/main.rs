use std::fmt;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

/// Trait for FieldElements over some field.
/// Now it is lifetime-parameterized to ensure elements don't outlive their context.
pub trait FieldElement<'a>:
    Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Neg<Output = Self>
    + Div<Self, Output = Self>
    + Eq
    + Clone
{
    fn zero(ctx: &'a FieldContext) -> Self;
    fn one(ctx: &'a FieldContext) -> Self;
    fn is_zero(&self) -> bool;
    fn inverse(&self) -> Self;
    fn pow(&self, exp: &[u64]) -> Self;
    fn const_time_pow(&self, exp: &[u64]) -> Self;
}

/// Holds the parameters of the field:
/// - `p`: Prime modulus for Fp
/// - `irreducible_poly`: coefficients of the irreducible polynomial for extension fields.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FieldContext {
    pub p: u64,
    pub irreducible_poly: Vec<u64>,
}

impl FieldContext {
    pub fn new(p: u64, irreducible_poly: Vec<u64>) -> Self {
        Self {
            p,
            irreducible_poly,
        }
    }

    fn to_fp<'a>(&'a self, val: u64) -> FpElement<'a> {
        FpElement::new(self, val)
    }
}

/// An element in the prime field Fp, referencing a `FieldContext`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FpElement<'a> {
    context: &'a FieldContext,
    val: u64,
}

impl<'a> fmt::Display for FpElement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Just display the value field
        write!(f, "{}", self.val)
    }
}

impl<'a> FpElement<'a> {
    pub fn new(context: &'a FieldContext, val: u64) -> Self {
        Self {
            context,
            val: val % context.p,
        }
    }

    fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
        if b == 0 {
            (a, 1, 0)
        } else {
            let (g, x, y) = Self::extended_gcd(b, a % b);
            (g, y, x - (a / b) * y)
        }
    }

    fn ct_select(a: FpElement<'a>, b: FpElement<'a>, bit: u64) -> FpElement<'a> {
        if bit == 0 {
            a
        } else {
            b
        }
    }
}

impl<'a> Add for FpElement<'a> {
    type Output = FpElement<'a>;
    fn add(self, other: FpElement<'a>) -> FpElement<'a> {
        FpElement::new(self.context, self.val + other.val)
    }
}

impl<'a> Sub for FpElement<'a> {
    type Output = FpElement<'a>;
    fn sub(self, other: FpElement<'a>) -> FpElement<'a> {
        FpElement::new(
            self.context,
            (self.val + self.context.p - other.val) % self.context.p,
        )
    }
}

impl<'a> Neg for FpElement<'a> {
    type Output = FpElement<'a>;
    fn neg(self) -> FpElement<'a> {
        if self.val == 0 {
            self
        } else {
            FpElement::new(self.context, self.context.p - self.val)
        }
    }
}

impl<'a> Mul for FpElement<'a> {
    type Output = FpElement<'a>;
    fn mul(self, other: FpElement<'a>) -> FpElement<'a> {
        let res = (self.val as u128 * other.val as u128) % (self.context.p as u128);
        FpElement::new(self.context, res as u64)
    }
}

impl<'a> Div for FpElement<'a> {
    type Output = FpElement<'a>;
    fn div(self, other: FpElement<'a>) -> FpElement<'a> {
        self * other.inverse()
    }
}

impl<'a> FieldElement<'a> for FpElement<'a> {
    fn zero(ctx: &'a FieldContext) -> Self {
        FpElement::new(ctx, 0)
    }

    fn one(ctx: &'a FieldContext) -> Self {
        FpElement::new(ctx, 1)
    }

    fn is_zero(&self) -> bool {
        self.val == 0
    }

    fn inverse(&self) -> Self {
        let a = self.val as i128;
        let m = self.context.p as i128;
        let (g, x, _) = Self::extended_gcd(a, m);
        if g != 1 {
            panic!("No inverse exists!");
        }
        let inv = ((x % m) + m) % m;
        FpElement::new(self.context, inv as u64)
    }

    fn pow(&self, exp: &[u64]) -> Self {
        let mut e: u128 = 0;
        for &part in exp {
            e = (e << 64) | part as u128;
        }

        let mut base = *self;
        let mut result = FpElement::new(self.context, 1);
        let mut e_val = e;
        while e_val > 0 {
            if (e_val & 1) == 1 {
                result = result * base;
            }
            base = base * base;
            e_val >>= 1;
        }
        result
    }

    fn const_time_pow(&self, exp: &[u64]) -> Self {
        let mut e: u128 = 0;
        for &part in exp {
            e = (e << 64) | part as u128;
        }

        let mut r0 = FpElement::new(self.context, 1);
        let mut r1 = *self;

        let bits = 128 - e.leading_zeros();
        for i in (0..bits).rev() {
            let bit = ((e >> i) & 1) as u64;
            let r0_new = r0 * r0;
            let r1_new = r0_new * r1;
            let new_r0 = FpElement::ct_select(r0_new, r1_new, bit);
            let new_r1 = FpElement::ct_select(r0_new * r1, r0_new, bit);

            r0 = new_r0;
            r1 = new_r1;
        }

        r0
    }
}

/// Polynomial-based field extension element: F_{p^k}.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FpPolynomialElement<'a> {
    context: &'a FieldContext,
    coeffs: Vec<FpElement<'a>>,
}

impl<'a> fmt::Display for FpPolynomialElement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_printed_term = false;
        let degree = self.coeffs.len().saturating_sub(1);

        // Print from highest exponent down to 0
        for i in (0..=degree).rev() {
            let c = &self.coeffs[i];
            if c.is_zero() {
                // Skip zero coefficients
                continue;
            }

            // If this is not the first printed term, add a separator
            if has_printed_term {
                write!(f, " + ")?;
            }

            let exp = i;
            let val_str = format!("{}", c); // Uses FpElement's Display
            if exp == 0 {
                // Just the coefficient
                write!(f, "{}", val_str)?;
            } else if exp == 1 {
                // coeff*x
                write!(f, "{}*x", val_str)?;
            } else {
                // coeff*x^exp
                write!(f, "{}*x^{}", val_str, exp)?;
            }

            has_printed_term = true;
        }

        // If no terms were printed, it means polynomial is 0
        if !has_printed_term {
            write!(f, "0")?;
        }

        Ok(())
    }
}

impl<'a> FpPolynomialElement<'a> {
    pub fn new(context: &'a FieldContext, coeffs: Vec<FpElement<'a>>) -> Self {
        let k = context.irreducible_poly.len() - 1;
        let mut el = Self { context, coeffs };
        el.normalize(k);
        el
    }

    fn normalize(&mut self, k: usize) {
        while self.coeffs.len() < k {
            self.coeffs.push(FpElement::zero(self.context));
        }
        self.coeffs = Self::poly_mod(&self.coeffs, self.context);
    }

    fn poly_to_fp<'b>(context: &'b FieldContext, poly: &[u64]) -> Vec<FpElement<'b>> {
        poly.iter().map(|&v| context.to_fp(v)).collect()
    }

    fn poly_add(a: &[FpElement<'a>], b: &[FpElement<'a>]) -> Vec<FpElement<'a>> {
        let len = a.len().max(b.len());
        let mut res = Vec::with_capacity(len);
        for i in 0..len {
            let av = if i < a.len() {
                a[i]
            } else {
                FpElement::zero(a[0].context)
            };
            let bv = if i < b.len() {
                b[i]
            } else {
                FpElement::zero(b[0].context)
            };
            res.push(av + bv);
        }
        res
    }

    fn poly_sub(a: &[FpElement<'a>], b: &[FpElement<'a>]) -> Vec<FpElement<'a>> {
        let len = a.len().max(b.len());
        let ctx = a[0].context;
        let mut res = Vec::with_capacity(len);
        for i in 0..len {
            let av = if i < a.len() {
                a[i]
            } else {
                FpElement::zero(ctx)
            };
            let bv = if i < b.len() {
                b[i]
            } else {
                FpElement::zero(ctx)
            };
            res.push(av - bv);
        }
        res
    }

    fn poly_mul(a: &[FpElement<'a>], b: &[FpElement<'a>]) -> Vec<FpElement<'a>> {
        let ctx = a[0].context;
        let mut res = vec![FpElement::zero(ctx); a.len() + b.len() - 1];
        for i in 0..a.len() {
            for j in 0..b.len() {
                res[i + j] = res[i + j] + (a[i] * b[j]);
            }
        }
        res
    }

    fn poly_mod(a: &[FpElement<'a>], context: &'a FieldContext) -> Vec<FpElement<'a>> {
        let irreducible_poly = Self::poly_to_fp(context, &context.irreducible_poly);
        let deg_mod = irreducible_poly.len() - 1;
        let mut r = a.to_vec();
        while r.len() > deg_mod {
            let leading = r[r.len() - 1];
            if leading.is_zero() {
                r.pop();
                continue;
            }
            let diff = r.len() - irreducible_poly.len();
            let mut temp = vec![FpElement::zero(context); diff];
            for c in &irreducible_poly {
                temp.push(*c * leading);
            }
            r = Self::poly_sub(&r, &temp);
            while r.last().map(|x| x.is_zero()) == Some(true) {
                r.pop();
            }
        }
        if r.is_empty() {
            r.push(FpElement::zero(context));
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
        a: &[FpElement<'a>],
        b: &[FpElement<'a>],
    ) -> (Vec<FpElement<'a>>, Vec<FpElement<'a>>) {
        if Self::poly_is_zero(b) {
            panic!("Division by zero polynomial");
        }
        let ctx = a[0].context;
        let mut aa = a.to_vec();
        let mut qq = Vec::<FpElement<'a>>::new();

        while aa.len() >= b.len() && !Self::poly_is_zero(&aa) {
            let factor = aa[aa.len() - 1] * b[b.len() - 1].inverse();
            let deg_diff = aa.len() - b.len();
            let mut temp = vec![FpElement::zero(ctx); deg_diff];
            for c in b {
                temp.push(*c * factor);
            }
            while qq.len() < deg_diff + 1 {
                qq.push(FpElement::zero(ctx));
            }
            qq[deg_diff] = qq[deg_diff] + factor;
            aa = Self::poly_sub(&aa, &temp);
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

    fn poly_inv(a: &[FpElement<'a>], context: &'a FieldContext) -> Vec<FpElement<'a>> {
        let irreducible_poly = Self::poly_to_fp(context, &context.irreducible_poly);
        let mut r0 = irreducible_poly.clone();
        let mut r1 = a.to_vec();
        let mut s0 = vec![FpElement::one(context)];
        let mut s1 = vec![FpElement::zero(context)];
        let mut t0 = vec![FpElement::zero(context)];
        let mut t1 = vec![FpElement::one(context)];

        while !Self::poly_is_zero(&r1) {
            let (q, r) = Self::poly_div(&r0, &r1);
            let r2 = r.clone();
            let s2 = Self::poly_sub(&s0, &Self::poly_mul(&q, &s1));
            let t2 = Self::poly_sub(&t0, &Self::poly_mul(&q, &t1));

            r0 = r1;
            r1 = r2;
            s0 = s1;
            s1 = s2;
            t0 = t1;
            t1 = t2;
        }

        let inv_lead = r0.last().unwrap().inverse();
        let inv = Self::poly_mul(&t0, &vec![inv_lead]);
        Self::poly_mod(&inv, context)
    }
}

impl<'a> Index<usize> for FpPolynomialElement<'a> {
    type Output = FpElement<'a>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.coeffs[index]
    }
}

impl<'a> IndexMut<usize> for FpPolynomialElement<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.coeffs[index]
    }
}

impl<'a> Add for FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn add(self, other: FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let added = Self::poly_add(&self.coeffs, &other.coeffs);
        let k = ctx.irreducible_poly.len() - 1;
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: added,
        };
        res.coeffs = Self::poly_mod(&res.coeffs, ctx);
        res.normalize(k);
        res
    }
}

impl<'a> Sub for FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn sub(self, other: FpPolynomialElement<'a>) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let subbed = Self::poly_sub(&self.coeffs, &other.coeffs);
        let k = ctx.irreducible_poly.len() - 1;
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: subbed,
        };
        res.coeffs = Self::poly_mod(&res.coeffs, ctx);
        res.normalize(k);
        res
    }
}

impl<'a> Neg for FpPolynomialElement<'a> {
    type Output = FpPolynomialElement<'a>;
    fn neg(self) -> FpPolynomialElement<'a> {
        let ctx = self.context;
        let negcoeffs: Vec<FpElement<'a>> = self.coeffs.into_iter().map(|c| c.neg()).collect();
        let k = ctx.irreducible_poly.len() - 1;
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
        let k = ctx.irreducible_poly.len() - 1;
        let mut prod = Self::poly_mul(&self.coeffs, &other.coeffs);
        prod = Self::poly_mod(&prod, ctx);
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

impl<'a> FieldElement<'a> for FpPolynomialElement<'a> {
    fn zero(ctx: &'a FieldContext) -> Self {
        let k = ctx.irreducible_poly.len() - 1;
        let coeffs = vec![FpElement::zero(ctx); k];
        FpPolynomialElement {
            context: ctx,
            coeffs,
        }
    }

    fn one(ctx: &'a FieldContext) -> Self {
        let k = ctx.irreducible_poly.len() - 1;
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
        let k = ctx.irreducible_poly.len() - 1;
        let mut res = FpPolynomialElement {
            context: ctx,
            coeffs: inv_poly,
        };
        res.normalize(k);
        res
    }

    fn pow(&self, exp: &[u64]) -> Self {
        let ctx = self.context;
        let mut base = self.clone();
        let mut result = Self::one(ctx);
        let mut e: u128 = 0;
        for &part in exp {
            e = (e << 64) | part as u128;
        }

        while e > 0 {
            if (e & 1) == 1 {
                result = result * base.clone();
            }
            base = base.clone() * base.clone();
            e >>= 1;
        }
        result
    }

    fn const_time_pow(&self, exp: &[u64]) -> Self {
        let ctx = self.context;
        let mut e: u128 = 0;
        for &part in exp {
            e = (e << 64) | part as u128;
        }

        let mut r0 = Self::one(ctx);
        let mut r1 = self.clone();

        let bits = 128 - e.leading_zeros();
        for i in (0..bits).rev() {
            let bit = ((e >> i) & 1) as u64;
            let (r0_new, r1_new) = if bit == 0 {
                (r0.clone() * r0.clone(), r0.clone() * r1.clone())
            } else {
                (r0.clone() * r1.clone(), r1.clone() * r1.clone())
            };

            // Simplified branching, but ideally constant-time selection.
            if bit == 0 {
                r1 = r1_new;
                r0 = r0_new;
            } else {
                r0 = r0_new;
                r1 = r1_new;
            }
        }

        r0
    }
}

fn main() {
    let p = 17;
    let irreducible_poly = vec![1, 0, 1]; // x² + 1
    let context = FieldContext::new(p, irreducible_poly);

    println!("Field context: {:?}", context);

    let a = FpElement::new(&context, 2);
    let b = FpElement::new(&context, 3);
    let c = a + b;
    println!("Fp: a = {}, b = {}", a, b);
    println!("Fp: a+b = {}", c);

    let inv_a = a.inverse();
    println!("Fp: a^-1 = {}", inv_a);
    println!("Fp: a*a^-1 = {}", a * inv_a);

    let prod = a * b;
    println!("Fp: a*b = {}", prod);

    let div = a / b;
    println!("Fp: a/b = {}", div);

    let poly_a = FpPolynomialElement::new(&context, vec![context.to_fp(2), context.to_fp(3)]);
    let poly_b = FpPolynomialElement::new(&context, vec![context.to_fp(5), context.to_fp(1)]);

    println!("F_{{p^2}}: a = {}, b = {}", poly_a, poly_b);

    let poly_sum = poly_a.clone() + poly_b.clone();
    println!("F_{{p^2}}: a+b = {}", poly_sum);

    let poly_prod = poly_a.clone() * poly_b.clone();
    println!("F_{{p^2}}: a*b = {}", poly_prod);

    let poly_inv_b = poly_b.inverse();
    println!("F_{{p^2}}: b^-1 = {}", poly_inv_b);

    let poly_div = poly_a.clone() / poly_b.clone();
    println!("F_{{p^2}}: a/b = {}", poly_div);

    let exp = [5u64]; // a^5
    let poly_exp = poly_a.pow(&exp);
    println!("F_{{p^2}}: a^5 = {}", poly_exp);

    let poly_exp_ct = poly_a.const_time_pow(&exp);
    println!("F_{{p^2}}: a^5 (ct) = {}", poly_exp_ct);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fp_addition() {
        let p = 17;
        let irreducible_poly = vec![1, 0, 1];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2);
        let b = FpElement::new(&ctx, 3);
        let c = FpElement::new(&ctx, 0);

        assert_eq!((a + b).val, 5);
        assert_eq!((a + c).val, a.val); // a+0 = a
                                        // Check wrap-around
        let x = FpElement::new(&ctx, 16);
        let y = FpElement::new(&ctx, 5);
        // 16+5=21 mod 17=4
        assert_eq!((x + y).val, 4);
    }

    #[test]
    fn test_fp_subtraction() {
        let p = 17;
        let irreducible_poly = vec![1, 0, 1];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2);
        let b = FpElement::new(&ctx, 3);
        assert_eq!((b - a).val, 1);
        // Check wrap-around
        let x = FpElement::new(&ctx, 1);
        let y = FpElement::new(&ctx, 0);
        // 0-1 = -1 mod 17 = 16
        assert_eq!((y - x).val, 16);
    }

    #[test]
    fn test_fp_multiplication() {
        let p = 17;
        let irreducible_poly = vec![1, 0, 1];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2);
        let b = FpElement::new(&ctx, 3);
        assert_eq!((a * b).val, 6);
        // Check multiplication by zero
        let zero = FpElement::new(&ctx, 0);
        assert_eq!((a * zero).val, 0);
        // Check wrap-around
        let big = FpElement::new(&ctx, 20);
        // 20 mod 17 = 3, 3 * 3 = 9 mod 17
        assert_eq!((big * b).val, 9);
    }

    #[test]
    fn test_fp_negation() {
        let p = 17;
        let irreducible_poly = vec![1, 0, 1];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2);
        assert_eq!((-a).val, 15); // since 17-2=15
        let zero = FpElement::new(&ctx, 0);
        assert_eq!((-zero).val, 0);
    }

    #[test]
    fn test_fp_inverse() {
        let p = 17;
        let irreducible_poly = vec![1, 0, 1];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 3);
        let inv_a = a.inverse();
        // Check a*inv_a=1
        assert_eq!((a * inv_a).val, 1);
        // Try another element
        let b = FpElement::new(&ctx, 5);
        let inv_b = b.inverse();
        assert_eq!((b * inv_b).val, 1);
    }

    #[test]
    fn test_fp_division() {
        let p = 17;
        let irreducible_poly = vec![1, 0, 1];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2);
        let b = FpElement::new(&ctx, 3);
        // a/b = a * b^-1
        let div = a / b;
        // 3^-1 mod17=6, so a/b=2*6=12 mod17
        assert_eq!(div.val, 12);
    }

    #[test]
    fn test_fp_exponentiation() {
        let p = 17;
        let irreducible_poly = vec![1, 0, 1];
        let ctx = FieldContext::new(p, irreducible_poly);

        let a = FpElement::new(&ctx, 2);
        // a^5 = 32 mod17=15
        let exp = [5u64];

        let res = a.pow(&exp);
        assert_eq!(res.val, 15);

        assert_eq!(a.pow(&exp), a.const_time_pow(&exp));

        // Check a bigger exponent
        let exp_big = [0x10u64]; // a^(16)=2^16=65536 mod17
                                 // 2^16 = (2^4)^4 = (16)^4 = (16 mod17=16)^2=256 mod17=256-255=1 again and again => actually 2^16 mod17= (2^(17-1))=1 by Fermat's little theorem

        assert_eq!(a.pow(&exp_big), a.const_time_pow(&exp_big));
    }

    #[test]
    fn test_polynomial_addition() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);
        // poly_a = 2+3x, poly_b=5+x
        let poly_a = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(2), ctx.to_fp(3)]);
        let poly_b = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(5), ctx.to_fp(1)]);

        // a+b = (2+5)+(3+1)x=7+4x
        let sum = poly_a.clone() + poly_b.clone();
        assert_eq!(sum[0].val, 7);
        assert_eq!(sum[1].val, 4);
    }

    #[test]
    fn test_polynomial_subtraction() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);
        let poly_a = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(2), ctx.to_fp(3)]);
        let poly_b = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(5), ctx.to_fp(1)]);

        // b-a = (5-2) + (1-3)x = 3 + (-2)x =3+15x mod17
        let diff = poly_b.clone() - poly_a.clone();
        assert_eq!(diff[0].val, 3);
        assert_eq!(diff[1].val, 15);
    }

    #[test]
    fn test_polynomial_multiplication() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);
        let poly_a = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(2), ctx.to_fp(3)]);
        let poly_b = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(5), ctx.to_fp(1)]);

        let prod = poly_a.clone() * poly_b.clone();
        // Expected result from previous reasoning: 7 + 0*x
        assert_eq!(prod[0].val, 7);
        assert_eq!(prod[1].val, 0);

        // Check multiplication by zero polynomial
        let zero_poly = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(0), ctx.to_fp(0)]);
        let zero_prod = poly_a.clone() * zero_poly.clone();
        // should be zero polynomial:
        assert!(zero_prod.is_zero());
    }

    #[test]
    fn test_polynomial_negation() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);
        let poly_a = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(2), ctx.to_fp(3)]);
        let neg_a = -poly_a.clone();
        // -2 mod17=15, -3mod17=14
        assert_eq!(neg_a[0].val, 15);
        assert_eq!(neg_a[1].val, 14);

        let zero_poly = FpPolynomialElement::zero(&ctx);
        assert!((-zero_poly.clone()).is_zero());
    }

    #[test]
    fn test_polynomial_inverse() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);
        let poly_b = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(5), ctx.to_fp(1)]);

        let inv_b = poly_b.inverse();
        // poly_b * inv_b = 1
        let one = FpPolynomialElement::one(&ctx);
        let check_inv = poly_b.clone() * inv_b.clone();
        assert_eq!(check_inv, one);
    }

    #[test]
    fn test_polynomial_division() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);
        let poly_a = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(2), ctx.to_fp(3)]);
        let poly_b = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(5), ctx.to_fp(1)]);

        let quotient = poly_a.clone() / poly_b.clone();
        let check_div = quotient * poly_b.clone();
        assert_eq!(check_div, poly_a);
    }

    #[test]
    fn test_polynomial_exponentiation() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);
        let poly_a = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(2), ctx.to_fp(3)]);
        let exp = [5u64];

        // Check consistency of pow and const_time_pow
        assert_eq!(poly_a.pow(&exp), poly_a.const_time_pow(&exp));

        // Try another exponent
        let exp_big = [16u64]; // poly_a^(16)
        assert_eq!(poly_a.pow(&exp_big), poly_a.const_time_pow(&exp_big));
    }

    #[test]
    fn test_polynomial_display() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);

        // poly = 0
        let zero_poly = FpPolynomialElement::zero(&ctx);
        assert_eq!(format!("{}", zero_poly), "0");

        // poly = 2 + 3x
        let poly_a = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(2), ctx.to_fp(3)]);
        // displayed as: "3*x + 2"
        let disp_a = format!("{}", poly_a);
        assert!(disp_a.contains("3*x"));
        assert!(disp_a.contains("2"));

        // poly = 5 (no x term)
        let poly_c = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(5)]);
        // just "5"
        assert_eq!(format!("{}", poly_c), "5");
    }

    #[test]
    fn test_additional_polynomial_cases() {
        let p = 17;
        let ctx = FieldContext::new(p, vec![1, 0, 1]);

        // Another polynomial
        let poly_x_only = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(0), ctx.to_fp(1)]); // x
                                                                                            // x*x = x^2 = -1 mod poly => = p-1=16
        let prod = poly_x_only.clone() * poly_x_only.clone();
        assert_eq!(prod[0].val, 16);
        assert_eq!(prod[1].val, 0);

        // Check that a polynomial times its inverse gives one
        let poly_rand = FpPolynomialElement::new(&ctx, vec![ctx.to_fp(4), ctx.to_fp(9)]);
        let inv_rand = poly_rand.inverse();
        let check_inv = poly_rand * inv_rand;
        assert_eq!(check_inv, FpPolynomialElement::one(&ctx));
    }
}
