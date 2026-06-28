import sys

def main():
    data = open(sys.argv[1]).read()
    n = 0
    col = 0
    total = 0
    for c in data:
        if c == "\n":
            if col > 0:
                n += 1
                total += n * col
            col = 0
        else:
            col += 1
    print(total)

main()
