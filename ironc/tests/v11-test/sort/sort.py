import sys

def main():
    data = open(sys.argv[1]).read()
    lines = data.splitlines()
    lines.sort()
    modulo = 1000000007
    h = 0
    k = 0
    for line in lines:
        if k >= 1000:
            print(h)
            return
        for c in line:
            h = (h * 31 + ord(c)) % modulo
        k += 1
    print(h)

main()
