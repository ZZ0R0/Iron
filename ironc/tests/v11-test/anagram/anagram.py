import sys

def main():
    data = open(sys.argv[1]).read()
    seen = {}
    for word in data.splitlines():
        cs = []
        for c in word:
            cs.append(c)
        cs.sort()
        key = ""
        for c in cs:
            key = key + c
        if key in seen:
            seen[key] = seen[key] + 1
        else:
            seen[key] = 1
    print(len(seen))

main()
