def lcg_sequence(seed, count, modulo):
    out = []
    state = seed
    i = 0
    while i < count:
        state = (state * 1103515245 + 12345) % 2147483648
        out.append(state % modulo)
        i = i + 1
    return out

def reversed_list(n):
    out = []
    i = n
    while i >= 1:
        out.append(i)
        i = i - 1
    return out

def sorted_list(n):
    out = []
    i = 1
    while i <= n:
        out.append(i)
        i = i + 1
    return out

def all_equal(n, value):
    out = []
    i = 0
    while i < n:
        out.append(value)
        i = i + 1
    return out

def checksum(xs):
    total = 0
    for i, v in enumerate(xs):
        total = total + (i + 1) * v
    return total
