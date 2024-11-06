
table = {
    "a1": 
"""
a1,6 = 0, a1,12 = 0, a1,22 = 1, a1,26 = 0, a1,27 = 1, a1,28 = 0, a1,32 = 1
""",
    "d1":
"""
d1,2 = 0, d1,3 = 0, d1,6 = 0, d1,7 = a1,7, d1,8 = a1,8, d1,12 = 1, d1,13 = a1,13, d1,16 = 0,
d1,17 = a1,17, d1,18 = a1,18, d1,19 = a1,19, d1,20 = a1,20, d1,21 = a1,21, d1,22 = 0,
d1,26 = 0, d1,27 = 1, d1,28 = 1, d1,29 = a1,29, d1,30 = a1,30, d1,31 = a1,31, d1,32 = 1
""",
    "c1":
"""
c1,2 = 1, c1,3 = 1, c1,4 = d1,4, c1,5 = d1,5, c1,6 = 1, c1,7 = 1, c1,8 = 0, c1,9 = 1, c1,12 = 1,
c1,13 = 0, c1,17 = 1, c1,18 = 1, c1,19 = 1, c1,20 = 1, c1,21 = 1, c1,22 = 0, c1,26 = 1, c1,27 = 1,
c1,28 = 1, c1,29 = 1, c1,30 = 1, c1,31 = 0, c1,32 = 1
""",
    "b1":
"""
b1,1 = c1,1, b1,2 = 0, b1,3 = 0, b1,4 = 0, b1,5 = 1, b1,6 = 0, b1,7 = 0, b1,8 = 0, b1,9 = 0,
b1,10 = c1,10, b1,11 = c1,11, b1,12 = 0, b1,13 = 0, b1,17 = 0, b1,18 = 0, b1,19 = 1, b1,20 = 0,
b1,21 = 0, b1,22 = 0, b1,26 = 1, b1,27 = 0, b1,28 = 1, b1,29 = 1, b1,30 = 1, b1,31 = 0, b1,32 = 1
""",
    "a2":
"""
a2,1 = 0, a2,2 = 0, a2,3 = 0, a2,4 = 0, a2,5 = 1, a2,6 = 0, a2,7 = 1, a2,8 = 0, a2,9 = 0,
a2,10 = 1, a2,11 = 1, a2,12 = 1, a2,13 = 0, a2,17 = 1, a2,18 = 1, a2,19 = 1, a2,20 = 1,
a2,27 = 0, a2,28 = 1, a2,29 = 0, a2,30 = 0, a2,21 = 0, a2,22 = 1, a2,31 = 1, a2,32 = 0
""",
    "d2":
"""
d2,1 = 0, d2,2 = 1, d2,3 = 1, d2,4 = 0, d2,5 = 1, d2,6 = 0, d2,7 = 1, d2,8 = 0, d2,9 = 0,
d2,10 = 0, d2,11 = 1, d2,12 = 1, d2,13 = 0, d2,17 = 0, d2,18 = 1, d2,21 = 0, d2,22 = 1,
d2,26 = 0, d2,27 = 1, d2,28 = 0, d2,29 = 0, d2,32 = 0
""",
    "c2":
"""
c2,1 = 1, c2,7 = 0, c2,8 = 0, c2,9 = 0, c2,10 = 1, c2,11 = 1, c2,12 = 1, c2,13 = 1,
c2,16 = d2,16, c2,17 = 1, c2,18 = 0, c2,21 = 0, c2,22 = 0, c2,24 = d2,24, c2,25 = d2,25,
c2,26 = 1, c2,27 = 1, c2,28 = 0, c2,29 = 1, c2,32 = 1
""",
    "b2":
"""
b2,1 = 0, b2,2 = c2,2, b2,7 = 1, b2,8 = 1, b2,9 = 1, b2,10 = 1, b2,16 = 1, b2,17 = 0, b2,18 = 1,
b2,21 = 1, b2,22 = 1, b2,24 = 0, b2,25 = 0, b2,26 = 0, b2,27 = 1, b2,28 = 0, b2,29 = 0, b2,32 = 1
""",
    "a3":
"""
a3,1 = 1, a3,2 = 0, a3,7 = 1, a3,8 = 1, a3,9 = 1, a3,10 = 0, a3,13 = b2,13, a3,16 = 0,
a3,17 = 1, a3,18 = 0, a3,24 = 0, a3,25 = 0, a3,26 = 0, a3,27 = 1, a3,28 = 1, a3,29 = 1,
a3,32 = 1
""",
    "d3":
"""
d3,1 = 0, d3,2 = 0, d3,7 = 1, d3,8 = 1, d3,9 = 1, d3,10 = 1, d3,13 = 0, d3,16 = 1, d3,17 = 1,
d3,18 = 1, d3,19 = 0, d3,24 = 1, d3,25 = 1, d3,26 = 1, d3,27 = 1, d3,32 = 1
""",
    "c3":
"""
c3,1 = 1, c3,2 = 1, c3,7 = 1, c3,8 = 1, c3,9 = 1, c3,10 = 1, c3,13 = 0, c3,14 = d3,14,
c3,15 = d3,15, c3,16 = 1, c3,17 = 1, c3,18 = 0, c3,19 = 1, c3,20 = d3,20, c3,32 = 1
""",
    "b3":
"""
b3,8 = 1, b3,13 = 1, b3,14 = 0, b3,15 = 0, b3,16 = 0, b3,17 = 0, b3,18 = 0, b3,19 = 0,
b3,20 = 1, b3,25 = c3,25, b3,26 = c3,26, b3,27 = c3,27, b3,28 = c3,28, b3,29 = c3,29,
b3,30 = c3,30, b3,31 = c3,31, b3,32 = 1
""",
    "a4":
"""
a4,4 = 1, a4,8 = 0, a4,14 = 1, a4,15 = 1, a4,16 = 1, a4,17 = 1, a4,18 = 1, a4,19 = 1, a4,20 = 1,
a4,25 = 1, a4,26 = 1, a4,27 = 1, a4,28 = 1, a4,29 = 1, a4,30 = 1, a4,31 = 0, a4,32 = 0
""",
    "d4":
"""
d4,4 = 1, d4,8 = 1, d4,14 = 1, d4,15 = 1, d4,16 = 1, d4,17 = 1, d4,18 = 1, d4,19 = 0, d4,20 = 1,
d4,25 = 0, d4,26 = 0, d4,27 = 0, d4,28 = 0, d4,29 = 0, d4,30 = 0, d4,31 = 1, d4,32 = 0
""",
    "c4":
"""
c4,4 = 0, c4,16 = 0, c4,25 = 1, c4,26 = 0, c4,27 = 1, c4,28 = 1, c4,29 = 1, c4,30 = 1,
c4,31 = 1, c4,32 = 0
""",
    "b4":
"""
b4,30 = 1, b4,32 = 0
""",
    "a5":
"""
a5,4 = b4,4, a5,16 = b4,16, a5,18 = 0, a5,32 = 0
""",
    "d5":
"""
d5,18 = 1, d5,30 = a5,30, d5,32 = 0
""",
    "c5":
"""
c5,18 = 0, c5,32 = 0
""",
    "b5":
"""
b5,32 = 0
""",
    "a6 - b6":
"""
a6,18 = b5,18, a6,32 = 0, d6,32 = 0, c6,32 = 0, b6,32 = c6,32 + 1
""",
    "c9, b12":
"""
φ34,32 = 1, b12,32 = d12,32
""",
    "a13 - b13":
"""
a13,32 = c12,32, d13,32 = b12,32 + 1, c13,32 = a13,32, b13,32 = d13,32
""",
    "a14 - b14":
"""
a14,32 = c13,32, d14,32 = b13,32, c14,32 = a14,32, b14,32 = d14,32
""",
    "a15 - b15":
"""
a15,32 = c14,32, d15,32 = b14,32, c15,32 = a15,32, b15,32 = d15,32 + 1
""",
    "a16":
"""
a16,26 = 1, a16,32 = c15,32
""",
    "d16":
"""
d16,26 = 1, d16,32 = b15,32
""",
    "c16":
"""
c16,26 = 1, c16,32 = a16,32
""",
    "b16":
"""
b16,26 = 1
""",
}


variables = {}
for constraints in table.values():
    constraints = constraints.replace("\n", " ")
    constraints_list = constraints.split(", ")
    for constraint in constraints_list:
        constraint = constraint.strip()
        left, right = constraint.split(" = ")
        var_name, bit_number = left.split(",")

        if var_name not in variables:
            variables[var_name] = {}
        
        if right == "1":
            if "one_bits" not in variables[var_name]:
                variables[var_name]["one_bits"] = []
            
            variables[var_name]["one_bits"] += [int(bit_number)]
        elif right == "0":
            if "zero_bits" not in variables[var_name]:
                variables[var_name]["zero_bits"] = []
            
            variables[var_name]["zero_bits"] += [int(bit_number)]
        elif right.startswith(("a", "b", "c", "d", "φ")):
            is_ending_with_plus_one = False
            if right.endswith(" + 1"):
                is_ending_with_plus_one = True
                right = right[:-4]
            elif right.endswith(" +1"):
                is_ending_with_plus_one = True
                right = right[:-3]
            elif right.endswith("+1"):
                is_ending_with_plus_one = True
                right = right[:-2]
            
            right_var_name, right_bit_number = right.split(",")
            if right_var_name not in variables[var_name]:
                variables[var_name][right_var_name] = {"same_bit":[], "different_bit":[]}
            
            if bit_number != right_bit_number:
                raise ValueError(f"Warning: Constraint '{constraint}': '{var_name}' and '{right_var_name}' have different bit numbers: '{bit_number}' and '{right_bit_number}', {is_ending_with_plus_one}")

            if is_ending_with_plus_one:
                variables[var_name][right_var_name]["different_bit"] += [int(bit_number)]
            else:
                variables[var_name][right_var_name]["same_bit"] += [int(bit_number)]

        else:
            raise ValueError(f"Warning: Constraint '{constraint}' is not valid")

for var_name, var_data in variables.items():
    if "one_bits" in var_data:
        one_bits = var_data["one_bits"]
        bitmask = 0
        for bit in one_bits:
            bitmask |= 1 << (bit - 1)
        
        print(f"const {var_name.upper()}_ONE_BITS: u32 = 0x{bitmask:08X};")
    
    if "zero_bits" in var_data:
        zero_bits = var_data["zero_bits"]
        bitmask = 0
        for bit in zero_bits:
            bitmask |= 1 << (bit - 1)

        print(f"const {var_name.upper()}_ZERO_BITS: u32 = 0x{bitmask:08X};")
    
    for other_var_name, other_var_data in var_data.items():
        if other_var_name =="one_bits" or other_var_name == "zero_bits":
            continue

        if len(other_var_data["same_bit"]) > 0:
            same_bits = other_var_data["same_bit"]
            bitmask = 0
            for bit in same_bits:
                bitmask |= 1 << (bit - 1)

            print(f"const {var_name.upper()}_{other_var_name.upper()}_SAME_BITS: u32 = 0x{bitmask:08X};")
        
        if len(other_var_data["different_bit"]) > 0:
            different_bits = other_var_data["different_bit"]
            bitmask = 0
            for bit in different_bits:
                bitmask |= 1 << (bit - 1)
            
            print(f"const {var_name.upper()}_{other_var_name.upper()}_DIFFERENT_BITS: u32 = 0x{bitmask:08X};")
            
