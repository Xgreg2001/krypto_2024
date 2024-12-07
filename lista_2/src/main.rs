mod field;
use field::{FieldContext, FieldElement, FpElement, FpPolynomialElement};
use num::bigint::ToBigInt;

fn main() {
    let p = 17.to_bigint().unwrap();
    let irreducible_poly = vec![
        3.to_bigint().unwrap(),
        1.to_bigint().unwrap(),
        1.to_bigint().unwrap(),
    ];
    let ctx = FieldContext::new(p, irreducible_poly);

    println!("Field context: {:?}", ctx);

    let a = FpElement::new(&ctx, 2.to_bigint().unwrap());
    let b = FpElement::new(&ctx, 3.to_bigint().unwrap());
    let c = &a + &b;
    println!("Fp: a = {}, b = {}", a, b);
    println!("Fp: a+b = {}", c);

    let inv_a = a.inverse();
    println!("Fp: a^-1 = {}", inv_a);
    println!("Fp: a*a^-1 = {}", &a * &inv_a);

    let prod = &a * &b;
    println!("Fp: a*b = {}", prod);

    let div = a / b;
    println!("Fp: a/b = {}", div);

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

    println!("F_{{p^2}}: a = {}, b = {}", poly_a, poly_b);

    let poly_sum = poly_a.clone() + poly_b.clone();
    println!("F_{{p^2}}: a+b = {}", poly_sum);

    let poly_prod = poly_a.clone() * poly_b.clone();
    println!("F_{{p^2}}: a*b = {}", poly_prod);

    let poly_inv_b = poly_b.inverse();
    println!("F_{{p^2}}: b^-1 = {}", poly_inv_b);

    let poly_div = poly_a.clone() / poly_b.clone();
    println!("F_{{p^2}}: a/b = {}", poly_div);

    let poly_exp = poly_a.pow(5);
    println!("F_{{p^2}}: a^5 = {}", poly_exp);
}
