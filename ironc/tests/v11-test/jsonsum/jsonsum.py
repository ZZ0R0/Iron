import sys

def main():
    data = open(sys.argv[1]).read()
    total = 0
    val = 0
    has = False
    for c in data:
        v = ord(c)
        if v >= 48 and v <= 57:
            val = val * 10 + (v - 48)
            has = True
        else:
            if has:
                total += val
            val = 0
            has = False
    if has:
        total += val
    print(total)

main()
