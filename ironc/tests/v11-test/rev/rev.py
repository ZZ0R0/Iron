import sys

def main():
    data = open(sys.argv[1]).read()
    modulo = 1000000007
    h = 0
    for line in data.splitlines():
        cs = []
        for c in line:
            cs.append(c)
        i = len(cs) - 1
        while i >= 0:
            h = (h * 31 + ord(cs[i])) % modulo
            i -= 1
    print(h)

main()
