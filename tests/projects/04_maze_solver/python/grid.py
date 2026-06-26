# Grid model for the maze: 0 = open cell, 1 = wall.
# A cell is identified by a flat id = r * cols + c, used by the solver
# for its distance and parent arrays (keeps everything in 1D, deterministic).

class Grid:
    def __init__(self, rows, cols, cells):
        self.rows = rows
        self.cols = cols
        self.cells = cells

    def at(self, r, c):
        return self.cells[r][c]

    def in_bounds(self, r, c):
        if r < 0:
            return False
        if c < 0:
            return False
        if r >= self.rows:
            return False
        if c >= self.cols:
            return False
        return True

    def passable(self, r, c):
        if not self.in_bounds(r, c):
            return False
        return self.cells[r][c] == 0

    def cell_id(self, r, c):
        return r * self.cols + c

    def row_of(self, cid):
        return cid // self.cols

    def col_of(self, cid):
        return cid % self.cols

    def count_open(self):
        total = 0
        for r in range(self.rows):
            for c in range(self.cols):
                if self.cells[r][c] == 0:
                    total = total + 1
        return total

    def count_walls(self):
        return self.rows * self.cols - self.count_open()


def make_grid(data):
    return Grid(rows=len(data), cols=len(data[0]), cells=data)


# The four moves, in a fixed order so BFS is fully deterministic:
# up, down, left, right.
def moves():
    return [(-1, 0), (1, 0), (0, -1), (0, 1)]
