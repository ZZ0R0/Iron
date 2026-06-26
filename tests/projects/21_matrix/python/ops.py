from matrix import Matrix, make, zeros, identity


def power(m, p):
    result = identity(m.rows)
    base = m
    e = p
    while e > 0:
        if e % 2 == 1:
            result = result.mul(base)
        base = base.mul(base)
        e = e // 2
    return result


def matrix_pow_naive(m, p):
    result = identity(m.rows)
    i = 0
    while i < p:
        result = result.mul(m)
        i = i + 1
    return result


def minor(m, drop_r, drop_c):
    out = []
    r = 0
    while r < m.rows:
        if r != drop_r:
            row = []
            c = 0
            while c < m.cols:
                if c != drop_c:
                    row.append(m.data[r][c])
                c = c + 1
            out.append(row)
        r = r + 1
    return make(m.rows - 1, m.cols - 1, out)


def determinant(m):
    n = m.rows
    if n == 1:
        return m.data[0][0]
    if n == 2:
        return m.data[0][0] * m.data[1][1] - m.data[0][1] * m.data[1][0]
    total = 0
    sign = 1
    c = 0
    while c < n:
        sub = minor(m, 0, c)
        total = total + sign * m.data[0][c] * determinant(sub)
        sign = sign - 2 * sign
        c = c + 1
    return total


def row_sums(m):
    out = []
    r = 0
    while r < m.rows:
        s = 0
        c = 0
        while c < m.cols:
            s = s + m.data[r][c]
            c = c + 1
        out.append(s)
        r = r + 1
    return out


def col_sums(m):
    out = []
    c = 0
    while c < m.cols:
        s = 0
        r = 0
        while r < m.rows:
            s = s + m.data[r][c]
            r = r + 1
        out.append(s)
        c = c + 1
    return out


def diagonal(m):
    out = []
    i = 0
    while i < m.rows:
        out.append(m.data[i][i])
        i = i + 1
    return out


def flatten(m):
    out = []
    r = 0
    while r < m.rows:
        c = 0
        while c < m.cols:
            out.append(m.data[r][c])
            c = c + 1
        r = r + 1
    return out
