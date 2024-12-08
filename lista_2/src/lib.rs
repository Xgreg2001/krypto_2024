pub mod field;

use field::fp::FpElement;
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
}

impl FieldContext {
    pub fn new(p: BigInt, irreducible_poly: Vec<BigInt>) -> Self {
        Self {
            p,
            irreducible_poly,
        }
    }

    pub fn to_fp<'a>(&'a self, val: BigInt) -> FpElement<'a> {
        FpElement::new(self, val)
    }
}
