from poly import make, poly_str


def main():
    # p = 3x^2 + 2x + 1 ; q = x^3 - x + 4
    p = make([1, 2, 3])
    q = make([4, -1, 0, 1])

    print(f"p = {poly_str(p)}")
    print(f"q = {poly_str(q)}")
    print(f"deg p = {p.degree()}")
    print(f"deg q = {q.degree()}")

    print("-- evaluate p (Horner) --")
    for x in [-2, -1, 0, 1, 2, 3]:
        print(f"p({x}) = {p.evaluate(x)}")

    print("-- evaluate q (Horner) --")
    for x in [-2, -1, 0, 1, 2, 3]:
        print(f"q({x}) = {q.evaluate(x)}")

    s = p.add(q)
    d = p.sub(q)
    print(f"p+q = {poly_str(s)}")
    print(f"p+q coeffs {s.coeff_list()}")
    print(f"p-q = {poly_str(d)}")
    print(f"p-q coeffs {d.coeff_list()}")

    prod = p.mul(q)
    print(f"p*q = {poly_str(prod)}")
    print(f"p*q coeffs {prod.coeff_list()}")
    print(f"deg p*q = {prod.degree()}")
    for x in [-1, 0, 1, 2]:
        lhs = prod.evaluate(x)
        rhs = p.evaluate(x) * q.evaluate(x)
        ok = "yes"
        if lhs != rhs:
            ok = "no"
        print(f"check p*q({x})={lhs} eq={ok}")

    dp = p.derivative()
    dq = q.derivative()
    print(f"p' = {poly_str(dp)}")
    print(f"p' coeffs {dp.coeff_list()}")
    print(f"q' = {poly_str(dq)}")
    print(f"q' coeffs {dq.coeff_list()}")

    sc = p.scale(3)
    print(f"3p = {poly_str(sc)}")
    print(f"3p coeffs {sc.coeff_list()}")

    # (x + 1)^4 via repeated multiplication
    base = make([1, 1])
    pw = base.power(4)
    print(f"(x+1)^4 = {poly_str(pw)}")
    print(f"(x+1)^4 coeffs {pw.coeff_list()}")
    for x in [0, 1, 2]:
        print(f"(x+1)^4 at {x} = {pw.evaluate(x)}")

    # second derivative of q
    dq2 = dq.derivative()
    print(f"q'' = {poly_str(dq2)}")
    print(f"q'' coeffs {dq2.coeff_list()}")

    # zero polynomial behaviour
    zp = make([0, 0, 0])
    zflag = "yes"
    if not zp.is_zero():
        zflag = "no"
    print(f"zero is_zero {zflag}")
    print(f"zero deg {zp.degree()}")
    print(f"zero str = {poly_str(zp)}")

    # distributivity: p*(q+sc) == p*q + p*sc
    left = p.mul(q.add(sc))
    right = p.mul(q).add(p.mul(sc))
    dist = "yes"
    for x in [-1, 0, 1, 2, 3]:
        if left.evaluate(x) != right.evaluate(x):
            dist = "no"
    print(f"distributive {dist}")
    print(f"left coeffs {left.coeff_list()}")
    print(f"right coeffs {right.coeff_list()}")


main()
