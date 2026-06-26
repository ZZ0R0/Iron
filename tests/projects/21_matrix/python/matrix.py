class Matrix:
    rows: int
    cols: int
    data: list

    def __init__(self, rows, cols, data):
        self.rows = rows
        self.cols = cols
        self.data = data

    def get(self, r, c):
        return self.data[r][c]

    def set(self, r, c, v):
        self.data[r][c] = v

    def add(self, other):
        out = []
        r = 0
        while r < self.rows:
            row = []
            c = 0
            while c < self.cols:
                row.append(self.data[r][c] + other.data[r][c])
                c = c + 1
            out.append(row)
            r = r + 1
        return make(self.rows, self.cols, out)

    def sub(self, other):
        out = []
        r = 0
        while r < self.rows:
            row = []
            c = 0
            while c < self.cols:
                row.append(self.data[r][c] - other.data[r][c])
                c = c + 1
            out.append(row)
            r = r + 1
        return make(self.rows, self.cols, out)

    def scale(self, k):
        out = []
        r = 0
        while r < self.rows:
            row = []
            c = 0
            while c < self.cols:
                row.append(self.data[r][c] * k)
                c = c + 1
            out.append(row)
            r = r + 1
        return make(self.rows, self.cols, out)

    def mul(self, other):
        out = []
        r = 0
        while r < self.rows:
            row = []
            c = 0
            while c < other.cols:
                acc = 0
                k = 0
                while k < self.cols:
                    acc = acc + self.data[r][k] * other.data[k][c]
                    k = k + 1
                row.append(acc)
                c = c + 1
            out.append(row)
            r = r + 1
        return make(self.rows, other.cols, out)

    def transpose(self):
        out = []
        c = 0
        while c < self.cols:
            row = []
            r = 0
            while r < self.rows:
                row.append(self.data[r][c])
                r = r + 1
            out.append(row)
            c = c + 1
        return make(self.cols, self.rows, out)

    def trace(self):
        total = 0
        i = 0
        while i < self.rows:
            total = total + self.data[i][i]
            i = i + 1
        return total

    def frobenius(self):
        total = 0
        r = 0
        while r < self.rows:
            c = 0
            while c < self.cols:
                v = self.data[r][c]
                total = total + v * v
                c = c + 1
            r = r + 1
        return total

    def is_symmetric(self):
        if self.rows != self.cols:
            return False
        r = 0
        while r < self.rows:
            c = 0
            while c < self.cols:
                if self.data[r][c] != self.data[c][r]:
                    return False
                c = c + 1
            r = r + 1
        return True

    def equals(self, other):
        if self.rows != other.rows:
            return False
        if self.cols != other.cols:
            return False
        r = 0
        while r < self.rows:
            c = 0
            while c < self.cols:
                if self.data[r][c] != other.data[r][c]:
                    return False
                c = c + 1
            r = r + 1
        return True


def make(rows, cols, data):
    m = Matrix(rows=rows, cols=cols, data=data)
    return m


def zeros(rows, cols):
    out = []
    r = 0
    while r < rows:
        row = []
        c = 0
        while c < cols:
            row.append(0)
            c = c + 1
        out.append(row)
        r = r + 1
    return make(rows, cols, out)


def identity(n):
    out = []
    r = 0
    while r < n:
        row = []
        c = 0
        while c < n:
            if r == c:
                row.append(1)
            else:
                row.append(0)
            c = c + 1
        out.append(row)
        r = r + 1
    return make(n, n, out)
