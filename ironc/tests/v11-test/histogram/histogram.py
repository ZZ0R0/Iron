import sys

def main():
    data = open(sys.argv[1]).read()
    counts = [0] * 256
    for c in data:
        counts[ord(c)] += 1
    best = 0
    for v in counts:
        if v > best:
            best = v
    print(best)

main()
