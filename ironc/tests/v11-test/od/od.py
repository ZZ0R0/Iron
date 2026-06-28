import sys

def main():
    data = open(sys.argv[1]).read()
    total = 0
    for c in data:
        v = ord(c)
        total += (v // 16) + (v % 16)
    print(total)

main()
