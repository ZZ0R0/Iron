from matrix import Matrix, make, zeros, identity
from ops import power, matrix_pow_naive, determinant, minor
from ops import row_sums, col_sums, diagonal, flatten


def yesno(b):
    if b:
        return "yes"
    return "no"


def show(label, m):
    print(f"{label} {m.rows}x{m.cols}")
    r = 0
    while r < m.rows:
        print(m.data[r])
        r = r + 1


def main():
    a = make(2, 3, [[1, 2, 3], [4, 5, 6]])
    b = make(2, 3, [[7, 8, 9], [1, 0, 2]])
    show("A", a)
    show("B", b)

    show("A+B", a.add(b))
    show("A-B", a.sub(b))
    show("A*3", a.scale(3))

    bt = b.transpose()
    show("B^T", bt)
    show("A*B^T", a.mul(bt))

    print(f"A row_sums {row_sums(a)}")
    print(f"A col_sums {col_sums(a)}")
    print(f"A frobenius {a.frobenius()}")

    print("=== square ===")
    s = make(3, 3, [[2, 1, 0], [1, 3, 1], [0, 1, 2]])
    show("S", s)
    print(f"S trace {s.trace()}")
    print(f"S diagonal {diagonal(s)}")
    print(f"S det {determinant(s)}")
    print(f"S symmetric {yesno(s.is_symmetric())}")

    i3 = identity(3)
    show("I3", i3)
    print(f"S*I == S {yesno(s.mul(i3).equals(s))}")

    print("=== powers ===")
    m = make(2, 2, [[1, 1], [1, 0]])
    p = 1
    while p <= 8:
        mp = power(m, p)
        flat = flatten(mp)
        print(f"M^{p} {flat} det {determinant(mp)}")
        p = p + 1

    print(f"naive == fast {yesno(matrix_pow_naive(m, 7).equals(power(m, 7)))}")

    print("=== det checks ===")
    grid = make(3, 3, [[6, 1, 1], [4, -2, 5], [2, 8, 7]])
    show("G", grid)
    print(f"G det {determinant(grid)}")
    print(f"G minor(0,0) det {determinant(minor(grid, 0, 0))}")

    k = make(4, 4, [[1, 0, 2, -1], [3, 0, 0, 5], [2, 1, 4, -3], [1, 0, 5, 0]])
    kt = k.trace()
    print(f"K det {determinant(k)}")
    print(f"K trace {kt}")

    z = zeros(2, 4)
    show("Z", z)
    print(f"Z+Z == Z {yesno(z.add(z).equals(z))}")


main()
