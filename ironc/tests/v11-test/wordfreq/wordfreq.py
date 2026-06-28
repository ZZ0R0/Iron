import sys

def main():
    data = open(sys.argv[1]).read()
    freq = {}
    for word in data.split():
        if word in freq:
            freq[word] = freq[word] + 1
        else:
            freq[word] = 1
    best = 0
    for word in freq:
        if freq[word] > best:
            best = freq[word]
    print(best)

main()
