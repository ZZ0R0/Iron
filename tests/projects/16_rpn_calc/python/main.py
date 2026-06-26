# Driver for the RPN calculator. Evaluates a batch of expressions on a fresh
# calculator each time, then a scripted session that builds up variables.

from eval import RpnCalc, ipow, iabs, apply_binary

EXPRS = [
    "3 4 +",
    "10 2 8 * + 3 -",
    "2 3 ^",
    "100 7 %",
    "5 dup *",
    "1 2 3 4 sum",
    "9 4 swap -",
    "7 neg abs",
    "20 6 over + +",
    "15 4 / 2 *",
]


def run_batch():
    print("== batch ==")
    for expr in EXPRS:
        calc = RpnCalc()
        r = calc.evaluate(expr)
        n = calc.count_reserved(expr)
        print(f"{expr} = {r} [{n}]")


def run_session():
    print("== session ==")
    calc = RpnCalc()
    vx = calc.evaluate("3 4 +")
    calc.set_var("x", vx)
    vy = calc.evaluate("x dup *")
    calc.set_var("y", vy)
    vz = calc.evaluate("y x -")
    calc.set_var("z", vz)
    vw = calc.evaluate("x y z + +")
    calc.set_var("w", vw)
    for name in sorted(calc.vars.keys()):
        print(f"{name}={calc.get_var(name)}")
    fin = calc.evaluate("w 2 /")
    print(f"final={fin}")


def run_helpers():
    print("== helpers ==")
    print(f"ipow(3,4)={ipow(3, 4)}")
    print(f"iabs(-12)={iabs(-12)}")
    total = 0
    for k in range(1, 6):
        total = total + apply_binary("*", k, k)
    print(f"sumsq(1..5)={total}")


def main():
    run_batch()
    run_session()
    run_helpers()


main()
