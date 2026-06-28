import sys

def main():
    data = open(sys.argv[1]).read()
    field = 2
    total = 0
    cur = 0
    f = 0
    val = 0
    for c in data:
        if c == "\n":
            if f == field:
                total += val
            f = 0
            val = 0
            cur = 0
        else:
            if c == ",":
                f += 1
            else:
                if f == field:
                    val = val * 10 + (ord(c) - 48)
    print(total)

main()
