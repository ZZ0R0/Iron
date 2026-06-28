import sys

def main():
    data = open(sys.argv[1]).read()
    count = 0
    prev = ""
    first = True
    for line in data.splitlines():
        if first or line != prev:
            count += 1
            prev = line
            first = False
    print(count)

main()
