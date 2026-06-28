import sys

def main():
    data = open(sys.argv[1]).read()
    modulo = 1000000007
    h = 0
    for c in data:
        v = ord(c)
        if v >= 97 and v <= 122:
            v = v - 32
        h = (h * 31 + v) % modulo
    print(h)

main()
