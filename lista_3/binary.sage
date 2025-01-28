# Define the binary field F_2[x]/(x^4 + x + 1)
F.<a> = GF(2^4, modulus=[1,1,0,0,1])  # x^4 + x + 1

# Define the elliptic curve y^2 + xy = x^3 + x^2 + 1
E = EllipticCurve(F, [1,1,0,0,1])

# Print all points on the curve
points = E.points()
print("Points on the curve:")
for P in points:
    print(f"x={P[0]}, y={P[1]}")
    # Test doubling
    P2 = 2*P
    P4 = 4*P
    if not P2.is_zero() and not P4.is_zero():
        print("\nFound suitable point!")
        print(f"P  = {P}")
        print(f"2P = {P2}")
        print(f"4P = {P4}")
        
        print("\nBinary representations:")
        print(f"P  x coordinate (binary): {bin(P[0].integer_representation())[2:].zfill(4)}")
        print(f"P  y coordinate (binary): {bin(P[1].integer_representation())[2:].zfill(4)}")
        print(f"2P x coordinate (binary): {bin(P2[0].integer_representation())[2:].zfill(4)}")
        print(f"2P y coordinate (binary): {bin(P2[1].integer_representation())[2:].zfill(4)}")
        print(f"4P x coordinate (binary): {bin(P4[0].integer_representation())[2:].zfill(4)}")
        print(f"4P y coordinate (binary): {bin(P4[1].integer_representation())[2:].zfill(4)}")
        break

