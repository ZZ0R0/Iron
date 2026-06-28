import sys

def main():
    n = int(sys.argv[1])
    a = []
    b = []
    i = 0
    while i < n:
        row_a = []
        row_b = []
        j = 0
        while j < n:
            row_a.append((i * n + j) % 7)
            row_b.append((i + j * n) % 5)
            j += 1
        a.append(row_a)
        b.append(row_b)
        i += 1
    trace = 0
    i = 0
    while i < n:
        j = 0
        while j < n:
            s = 0
            k = 0
            while k < n:
                s += a[i][k] * b[k][j]
                k += 1
            if i == j:
                trace += s
            j += 1
        i += 1
    print(trace)

main()
