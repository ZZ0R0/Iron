# Polynomials with integer coefficients.
# Representation: coeffs[i] is the coefficient of x**i (ascending power order).

from dataclasses import dataclass


@dataclass
class Poly:
    coeffs: list

    def degree(self):
        d = len(self.coeffs) - 1
        while d > 0 and self.coeffs[d] == 0:
            d = d - 1
        return d

    def is_zero(self):
        for c in self.coeffs:
            if c != 0:
                return False
        return True

    def evaluate(self, x):
        # Horner's method, walking from the highest power down.
        d = self.degree()
        acc = 0
        i = d
        while i >= 0:
            acc = acc * x + self.coeffs[i]
            i = i - 1
        return acc

    def add(self, other):
        n = len(self.coeffs)
        m = len(other.coeffs)
        size = n
        if m > size:
            size = m
        out = []
        i = 0
        while i < size:
            a = 0
            if i < n:
                a = self.coeffs[i]
            b = 0
            if i < m:
                b = other.coeffs[i]
            out.append(a + b)
            i = i + 1
        return make(out)

    def sub(self, other):
        n = len(self.coeffs)
        m = len(other.coeffs)
        size = n
        if m > size:
            size = m
        out = []
        i = 0
        while i < size:
            a = 0
            if i < n:
                a = self.coeffs[i]
            b = 0
            if i < m:
                b = other.coeffs[i]
            out.append(a - b)
            i = i + 1
        return make(out)

    def scale(self, k):
        out = []
        for c in self.coeffs:
            out.append(c * k)
        return make(out)

    def mul(self, other):
        n = len(self.coeffs)
        m = len(other.coeffs)
        out = []
        i = 0
        while i < n + m - 1:
            out.append(0)
            i = i + 1
        i = 0
        while i < n:
            j = 0
            while j < m:
                out[i + j] = out[i + j] + self.coeffs[i] * other.coeffs[j]
                j = j + 1
            i = i + 1
        return make(out)

    def derivative(self):
        out = []
        i = 1
        while i < len(self.coeffs):
            out.append(self.coeffs[i] * i)
            i = i + 1
        if len(out) == 0:
            out.append(0)
        return make(out)

    def power(self, e):
        bcoeffs = []
        for c in self.coeffs:
            bcoeffs.append(c)
        base = make(bcoeffs)
        result = make([1])
        i = 0
        while i < e:
            result = result.mul(base)
            i = i + 1
        return result

    def coeff_list(self):
        d = self.degree()
        out = []
        i = 0
        while i <= d:
            out.append(self.coeffs[i])
            i = i + 1
        return out


def make(coeffs):
    p = Poly(coeffs=coeffs)
    return p


def term_str(coef, power):
    if power == 0:
        return str(coef)
    if power == 1:
        return f"{coef}x"
    return f"{coef}x^{power}"


def poly_str(p):
    d = p.degree()
    parts = []
    i = d
    while i >= 0:
        c = p.coeffs[i]
        if c != 0:
            parts.append(term_str(c, i))
        i = i - 1
    if len(parts) == 0:
        return "0"
    return " + ".join(parts)
