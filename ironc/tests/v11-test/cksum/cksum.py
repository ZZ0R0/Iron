import sys

def main():
    data = open(sys.argv[1]).read()
    modulo = 4294967291
    h = 0
    for c in data:
        h = (h * 131 + ord(c)) % modulo
    print(h)

main()
