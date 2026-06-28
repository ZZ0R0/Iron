import sys

def main():
    data = open(sys.argv[1]).read()
    needle = "iron"
    count = 0
    for line in data.splitlines():
        if needle in line:
            count += 1
    print(count)

main()
