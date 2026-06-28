import sys

def main():
    n = int(sys.argv[1])
    total = 0
    i = 1
    while i <= n:
        total += i
        i += 1
    print(total)

main()
