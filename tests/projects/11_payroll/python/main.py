# Payroll run: per-employee payslips + company totals, all in integer cents.
from employee import Employee, format_cents
from tax import brackets, income_tax, fdiv


def build_roster():
    roster = []
    roster.append(Employee(name="alice", annual_gross=4800000, retirement_bps=600, health_premium=240000))
    roster.append(Employee(name="bob", annual_gross=7200000, retirement_bps=1000, health_premium=300000))
    roster.append(Employee(name="carol", annual_gross=12500000, retirement_bps=1500, health_premium=360000))
    roster.append(Employee(name="dave", annual_gross=3600000, retirement_bps=400, health_premium=180000))
    roster.append(Employee(name="erin", annual_gross=21000000, retirement_bps=2000, health_premium=420000))
    return roster


def print_payslip(e):
    print("== " + e.name + " ==")
    print("gross " + format_cents(e.annual_gross))
    print("retirement " + format_cents(e.retirement()))
    print("health " + format_cents(e.health_premium))
    print("taxable " + format_cents(e.taxable_income()))
    print("income_tax " + format_cents(e.income_tax()))
    print("payroll_tax " + format_cents(e.payroll_tax()))
    print("net_year " + format_cents(e.net_annual()))
    print("net_month " + format_cents(e.net_monthly()))
    print(f"eff_bps {e.effective_rate_bps()} marg_bps {e.marginal_bps()}")


def bracket_revenue(roster):
    # How much income tax revenue each bracket ceiling contributes in total,
    # keyed by the bracket's upper bound (sorted for deterministic output).
    bs = brackets()
    revenue = {}
    for i in range(len(bs)):
        upper = bs[i][0]
        revenue[upper] = 0
    for e in roster:
        taxable = e.taxable_income()
        lower = 0
        for i in range(len(bs)):
            upper = bs[i][0]
            rate = bs[i][1]
            if taxable > lower:
                top = taxable
                if top > upper:
                    top = upper
                slice_amount = top - lower
                slice_tax = fdiv(slice_amount * rate, 10000)
                revenue[upper] = revenue[upper] + slice_tax
            lower = upper
    return revenue


def main():
    roster = build_roster()
    for e in roster:
        print_payslip(e)

    total_gross = 0
    total_pretax = 0
    total_tax = 0
    total_net = 0
    for e in roster:
        total_gross = total_gross + e.annual_gross
        total_pretax = total_pretax + e.pretax_deductions()
        total_tax = total_tax + e.total_tax()
        total_net = total_net + e.net_annual()

    print("== company ==")
    print("headcount " + str(len(roster)))
    print("gross " + format_cents(total_gross))
    print("pretax " + format_cents(total_pretax))
    print("tax " + format_cents(total_tax))
    print("net " + format_cents(total_net))
    avg_net = fdiv(total_net, len(roster))
    print("avg_net " + format_cents(avg_net))

    print("== ranking by net ==")
    pairs = []
    for i in range(len(roster)):
        pairs.append((-roster[i].net_annual(), i))
    ordered = sorted(pairs)
    rank = 1
    for p in ordered:
        idx = p[1]
        e = roster[idx]
        print(f"{rank} " + e.name + " " + format_cents(e.net_annual()))
        rank = rank + 1

    print("== bracket revenue ==")
    revenue = bracket_revenue(roster)
    for upper in sorted(revenue.keys()):
        print(format_cents(upper) + " -> " + format_cents(revenue[upper]))

    print("== checks ==")
    print(f"sample_tax {income_tax(5000000)}")
    print(f"sample_tax {income_tax(15000000)}")


main()
