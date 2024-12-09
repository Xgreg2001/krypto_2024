pub mod field;

use field::{f2_poly::F2PolynomialElement, fp::FpElement};
use num::bigint::BigInt;
use std::ops::{Add, Div, Mul, Neg, Sub};

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
    fn pow(&self, exp: u64) -> Self;
}

/// Holds the parameters of the field:
/// - `p`: Prime modulus for Fp
/// - `irreducible_poly`: coefficients of the irreducible polynomial for extension fields.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FieldContext {
    pub p: BigInt,
    pub irreducible_poly: Vec<BigInt>,
    pub irreducible_binary_poly: Vec<u64>,
}

impl FieldContext {
    pub fn new_poly(p: BigInt, irreducible_poly: Vec<BigInt>) -> Self {
        Self {
            p,
            irreducible_poly,
            irreducible_binary_poly: vec![],
        }
    }

    pub fn new_binary(irreducible_binary_poly: Vec<u64>) -> Self {
        Self {
            p: BigInt::from(2),
            irreducible_poly: vec![],
            irreducible_binary_poly,
        }
    }

    pub fn is_binary(&self) -> bool {
        self.irreducible_binary_poly.len() > 0
    }

    pub fn to_fp(&self, val: BigInt) -> FpElement<'_> {
        FpElement::new(self, val)
    }

    pub fn is_poly(&self) -> bool {
        self.irreducible_poly.len() > 0
    }

    fn get_irreducible_poly_degree(&self) -> usize {
        if self.is_binary() {
            get_binary_poly_degree(&self.irreducible_binary_poly)
        } else {
            return self.irreducible_poly.len() - 1;
        }
    }
}

pub fn get_binary_poly_degree(a: &Vec<u64>) -> usize {
    let mut degree = 0;
    for (i, &val) in a.iter().enumerate() {
        if val != 0 {
            degree = i * 64 + 63 - val.leading_zeros() as usize;
        }
    }
    degree
}
