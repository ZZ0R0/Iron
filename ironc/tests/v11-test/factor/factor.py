import sys

def largest_prime_factor(n):
    largest = 1
    d = 2
    while d * d <= n:
        while n % d == 0:
            largest = d
            n = n // d
        d = d + 1
    if n > 1:
        largest = n
    return largest

def main():
    n_max = int(sys.argv[1])
    total = 0
    for n in range(2, n_max):
        total = total + largest_prime_factor(n)
    print(total)

main()
