# Employee model and payslip computation, all in integer cents.
from tax import income_tax, fica, marginal_rate, fdiv, fmod


def pad2(n):
    # Two-digit zero-padded cents component as an explicit string.
    if n < 10:
        return "0" + str(n)
    return str(n)


def format_cents(c):
    # Render a cent amount as "D.CC" using only integer math.
    sign = ""
    v = c
    if v < 0:
        sign = "-"
        v = -v
    dollars = fdiv(v, 100)
    cents = fmod(v, 100)
    return sign + str(dollars) + "." + pad2(cents)


class Employee:
    name: str
    annual_gross: int
    retirement_bps: int
    health_premium: int

    def __init__(self, name, annual_gross, retirement_bps, health_premium):
        self.name = name
        self.annual_gross = annual_gross
        self.retirement_bps = retirement_bps
        self.health_premium = health_premium

    def retirement(self):
        # Pre-tax retirement contribution (e.g. 401k) on annual gross.
        return fdiv(self.annual_gross * self.retirement_bps, 10000)

    def pretax_deductions(self):
        # All deductions taken before income tax: retirement + health premium.
        return self.retirement() + self.health_premium

    def taxable_income(self):
        t = self.annual_gross - self.pretax_deductions()
        if t < 0:
            t = 0
        return t

    def income_tax(self):
        return income_tax(self.taxable_income())

    def payroll_tax(self):
        return fica(self.annual_gross)

    def total_tax(self):
        return self.income_tax() + self.payroll_tax()

    def net_annual(self):
        return self.annual_gross - self.pretax_deductions() - self.total_tax()

    def net_monthly(self):
        return fdiv(self.net_annual(), 12)

    def effective_rate_bps(self):
        # Total tax as basis points of gross (integer).
        if self.annual_gross == 0:
            return 0
        return fdiv(self.total_tax() * 10000, self.annual_gross)

    def marginal_bps(self):
        return marginal_rate(self.taxable_income())
