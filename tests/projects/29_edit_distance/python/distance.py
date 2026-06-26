# Core string-distance metrics built on dynamic programming.
# Every routine works on lists of single-character strings produced by
# to_chars, so the same indexing logic mirrors one-to-one in Iron.


def to_chars(s):
    out = []
    for ch in s:
        out.append(ch)
    return out


def min3(a, b, c):
    m = a
    if b < m:
        m = b
    if c < m:
        m = c
    return m


def levenshtein(s, t):
    a = to_chars(s)
    b = to_chars(t)
    n = len(a)
    m = len(b)
    dp = []
    for i in range(n + 1):
        row = []
        for j in range(m + 1):
            row.append(0)
        dp.append(row)
    for i in range(n + 1):
        dp[i][0] = i
    for j in range(m + 1):
        dp[0][j] = j
    for i in range(1, n + 1):
        for j in range(1, m + 1):
            cost = 0
            if a[i - 1] != b[j - 1]:
                cost = 1
            delete_cost = dp[i - 1][j] + 1
            insert_cost = dp[i][j - 1] + 1
            sub_cost = dp[i - 1][j - 1] + cost
            dp[i][j] = min3(delete_cost, insert_cost, sub_cost)
    return dp[n][m]


def lcs_length(s, t):
    a = to_chars(s)
    b = to_chars(t)
    n = len(a)
    m = len(b)
    dp = []
    for i in range(n + 1):
        row = []
        for j in range(m + 1):
            row.append(0)
        dp.append(row)
    for i in range(1, n + 1):
        for j in range(1, m + 1):
            if a[i - 1] == b[j - 1]:
                dp[i][j] = dp[i - 1][j - 1] + 1
            else:
                up = dp[i - 1][j]
                left = dp[i][j - 1]
                if up >= left:
                    dp[i][j] = up
                else:
                    dp[i][j] = left
    return dp[n][m]


def hamming(s, t):
    a = to_chars(s)
    b = to_chars(t)
    if len(a) != len(b):
        return -1
    total = 0
    for i in range(len(a)):
        if a[i] != b[i]:
            total = total + 1
    return total


def is_one_edit_away(s, t):
    d = levenshtein(s, t)
    if d == 1:
        return True
    return False


def common_prefix_len(s, t):
    a = to_chars(s)
    b = to_chars(t)
    limit = len(a)
    if len(b) < limit:
        limit = len(b)
    count = 0
    i = 0
    while i < limit:
        if a[i] == b[i]:
            count = count + 1
        else:
            break
        i = i + 1
    return count
