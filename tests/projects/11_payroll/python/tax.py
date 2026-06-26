# Progressive income tax computed on integer cents.
# Brackets are marginal: each (upper_cents, rate_bps) taxes the slice of
# taxable income that falls between the previous bound and `upper`.
# rate_bps is in basis points (10000 bps = 100%).

def fdiv(p, q):
    # Integer floor division as a named helper.
    return p // q


def fmod(p, q):
    # Integer modulo as a named helper.
    return p % q


def brackets():
    # (upper_bound_in_cents, marginal_rate_in_basis_points)
    # The last bracket uses a very large upper bound to act as "infinity".
    return [
        (1000000, 1000),
        (4000000, 1200),
        (8500000, 2200),
        (16000000, 2400),
        (20000000, 3200),
        (1000000000, 3500),
    ]


def income_tax(taxable):
    # Marginal tax over the progressive brackets, in cents (floored per slice).
    if taxable <= 0:
        return 0
    total = 0
    lower = 0
    bs = brackets()
    for i in range(len(bs)):
        upper = bs[i][0]
        rate = bs[i][1]
        if taxable > lower:
            top = taxable
            if top > upper:
                top = upper
            slice_amount = top - lower
            slice_tax = fdiv(slice_amount * rate, 10000)
            total = total + slice_tax
        lower = upper
    return total


def marginal_rate(taxable):
    # The marginal rate (bps) that applies to the last cent of income.
    bs = brackets()
    lower = 0
    rate = 0
    for i in range(len(bs)):
        upper = bs[i][0]
        if taxable > lower:
            rate = bs[i][1]
        lower = upper
    return rate


def fica(gross):
    # Flat payroll tax (social security + medicare) at 7.65% on gross.
    return fdiv(gross * 765, 10000)
