def print_curve_info(curve, points, operations, is_poly=False):
    print(f"Field characteristic: {curve.base_field().characteristic()}")
    if is_poly:
        print(f"Field polynomial: {curve.base_field().modulus()}")
    print(f"Curve equation: y^2 = x^3 + {curve.a4()}*x + {curve.a6()}")
    print(f"Curve order: {curve.order()}")
    print("\nTest Points:")
    for i, P in enumerate(points):
        if P:
            print(f"P{i+1} = ({P[0]}, {P[1]})")

    print("\nOperations:")
    for op, result in operations:
        if result:
            print(f"{op} = ({result[0]}, {result[1]})")
        else:
            print(f"{op} = infinity")

# Test Case 1: Curve over F_23[x]/(x^2 + 1)
p = 23
R.<x> = PolynomialRing(GF(p))
poly = x^2 + 1
F23x = GF(p^2, modulus=poly, name='a')
E1 = EllipticCurve(F23x, [2, 3])  # y^2 = x^3 + 2x + 3

# Find points on the curve
points1 = E1.points()[:3]
P1 = points1[1]
P2 = points1[2]
double_P1 = 2*P1
P1_plus_P2 = P1 + P2
triple_P1 = 3*P1

operations1 = [
    ("2*P1", double_P1),
    ("P1 + P2", P1_plus_P2),
    ("3*P1", triple_P1)
]

print("=== Test Case 1: Curve over F_23[x]/(x^2 + 1) ===")
print_curve_info(E1, [P1, P2], operations1, is_poly=True)

# Test Case 2: Curve over F_11[x]/(x^3 + 2x + 7)
p = 11
R.<x> = PolynomialRing(GF(p))
poly = x^3 + 2*x + 7  # Changed to an irreducible polynomial
F11x = GF(p^3, modulus=poly, name='a')
E2 = EllipticCurve(F11x, [5, 7])  # y^2 = x^3 + 5x + 7

points2 = E2.points()[:3]
P1 = points2[1]
P2 = points2[2]
double_P1 = 2*P1
P1_plus_P2 = P1 + P2
triple_P1 = 3*P1

operations2 = [
    ("2*P1", double_P1),
    ("P1 + P2", P1_plus_P2),
    ("3*P1", triple_P1)
]

print("\n=== Test Case 2: Curve over F_11[x]/(x^3 + 2x + 7) ===")
print_curve_info(E2, [P1, P2], operations2, is_poly=True)

# Test Case 3: Edge cases over F_7[x]/(x^2 + x + 3)
p = 7
R.<x> = PolynomialRing(GF(p))
poly = x^2 + x + 3  # Changed to an irreducible polynomial
F7x = GF(p^2, modulus=poly, name='a')
E3 = EllipticCurve(F7x, [1, 1])  # y^2 = x^3 + x + 1

points3 = E3.points()[:3]
P1 = points3[1]
P2 = -P1  # Point and its negative
P3 = points3[2]

operations3 = [
    ("P1 + (-P1)", P1 + P2),  # Should be infinity
    ("2*P3", 2*P3),
    ("P1 + P3", P1 + P3)
]

print("\n=== Test Case 3: Edge Cases over F_7[x]/(x^2 + x + 3) ===")
print_curve_info(E3, [P1, P2, P3], operations3, is_poly=True)

# Print polynomial representation helper
print("\nPolynomial Representations:")
print("For F_23[x]/(x^2 + 1):")
for i in range(5):
    elem = F23x.random_element()
    print(f"{elem} = {elem.polynomial()}")

print("\nFor F_11[x]/(x^3 + 2x + 7):")
for i in range(5):
    elem = F11x.random_element()
    print(f"{elem} = {elem.polynomial()}")

print("\nFor F_7[x]/(x^2 + x + 3):")
for i in range(5):
    elem = F7x.random_element()
    print(f"{elem} = {elem.polynomial()}")
