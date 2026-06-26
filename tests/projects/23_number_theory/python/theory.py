def gcd(a, b):
    while b != 0:
        t = b
        b = a % b
        a = t
    return a

def lcm(a, b):
    return a // gcd(a, b) * b

def collatz_length(n):
    steps = 0
    while n != 1:
        if n % 2 == 0:
            n = n // 2
        else:
            n = 3 * n + 1
        steps = steps + 1
    return steps

def divisor_sum(n):
    total = 0
    d = 1
    while d < n:
        if n % d == 0:
            total = total + d
        d = d + 1
    return total
