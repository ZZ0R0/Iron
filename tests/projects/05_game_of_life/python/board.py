from rules import count_neighbors, next_state


class Board:
    cells: list
    rows: int
    cols: int

    def __init__(self, cells, rows, cols):
        self.cells = cells
        self.rows = rows
        self.cols = cols

    def get(self, r, c):
        return self.cells[r][c]

    def set(self, r, c, v):
        self.cells[r][c] = v

    def place_pattern(self, coords):
        i = 0
        while i < len(coords):
            rc = coords[i]
            self.cells[rc[0]][rc[1]] = 1
            i = i + 1

    def population(self):
        total = 0
        r = 0
        while r < self.rows:
            c = 0
            while c < self.cols:
                total = total + self.cells[r][c]
                c = c + 1
            r = r + 1
        return total

    def step(self):
        nxt = []
        r = 0
        while r < self.rows:
            row = []
            c = 0
            while c < self.cols:
                alive = self.cells[r][c]
                n = count_neighbors(self.cells, self.rows, self.cols, r, c)
                row.append(next_state(alive, n))
                c = c + 1
            nxt.append(row)
            r = r + 1
        self.cells = nxt

    def render(self):
        lines = []
        r = 0
        while r < self.rows:
            line = ""
            c = 0
            while c < self.cols:
                if self.cells[r][c] == 1:
                    line = line + "#"
                else:
                    line = line + "."
                c = c + 1
            lines.append(line)
            r = r + 1
        return lines


def make_board(rows, cols):
    cells = []
    r = 0
    while r < rows:
        row = []
        c = 0
        while c < cols:
            row.append(0)
            c = c + 1
        cells.append(row)
        r = r + 1
    return Board(cells=cells, rows=rows, cols=cols)
