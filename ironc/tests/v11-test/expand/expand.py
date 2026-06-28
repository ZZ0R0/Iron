import sys

def main():
    data = open(sys.argv[1]).read()
    col = 0
    total = 0
    for c in data:
        if c == "\t":
            spaces = 8 - (col % 8)
            total += spaces
            col += spaces
        else:
            if c == "\n":
                col = 0
            else:
                col += 1
            total += 1
    print(total)

main()
