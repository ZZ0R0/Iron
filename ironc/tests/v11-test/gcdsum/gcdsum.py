import sys

def main():
    n = int(sys.argv[1])
    total = 0
    i = 1
    while i < n:
        a = i
        b = n
        while b != 0:
            t = b
            b = a % b
            a = t
        total += a
        i += 1
    print(total)

main()
