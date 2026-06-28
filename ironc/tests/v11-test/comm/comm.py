import sys

def main():
    path_a = sys.argv[1]
    path_b = path_a.replace("sorted_a", "sorted_b")
    a = open(path_a).read().splitlines()
    b = open(path_b).read().splitlines()
    i = 0
    j = 0
    common = 0
    while i < len(a) and j < len(b):
        x = int(a[i])
        y = int(b[j])
        if x == y:
            common += 1
            i += 1
            j += 1
        else:
            if x < y:
                i += 1
            else:
                j += 1
    print(common)

main()
