from primes import sieve, factorize
from theory import gcd, lcm, collatz_length, divisor_sum

def main():
    primes = sieve(50)
    print(primes)
    print(len(primes))
    for n in [12, 60, 97, 100]:
        fs = factorize(n)
        print(f"{n} -> {fs}")
    print(f"gcd(48,36)={gcd(48, 36)} lcm(4,6)={lcm(4, 6)}")
    for n in [6, 7, 27]:
        print(f"collatz({n})={collatz_length(n)}")
    perfects = []
    for n in range(2, 100):
        if divisor_sum(n) == n:
            perfects.append(n)
    print(f"perfects: {perfects}")

main()
