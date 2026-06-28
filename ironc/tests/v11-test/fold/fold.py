import sys

def main():
    data = open(sys.argv[1]).read()
    width = 20
    col = 0
    count = 0
    for c in data:
        if c == "\n":
            count += 1
            col = 0
        else:
            col += 1
            if col == width:
                count += 1
                col = 0
    print(count)

main()
