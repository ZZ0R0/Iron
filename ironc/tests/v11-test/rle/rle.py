import sys

def main():
    data = open(sys.argv[1]).read()
    runs = 0
    prev = -1
    count = 0
    for c in data:
        v = ord(c)
        if v == prev:
            count += 1
        else:
            if count > 0:
                runs += 1
            prev = v
            count = 1
    if count > 0:
        runs += 1
    print(runs)

main()
