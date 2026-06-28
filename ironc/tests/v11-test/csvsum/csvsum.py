import sys

def main():
    data = open(sys.argv[1]).read()
    total = 0
    val = 0
    f = 0
    for c in data:
        if c == "\n":
            if f == 0:
                total += val
            f = 0
            val = 0
        else:
            if c == ",":
                f += 1
            else:
                if f == 0:
                    val = val * 10 + (ord(c) - 48)
    print(total)

main()
