# cell codes: 0=empty 1=X 2=O
# winner codes: 0=none/ongoing 1=X 2=O 3=draw
from dataclasses import dataclass

WIN_LINES = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8],
    [0, 3, 6], [1, 4, 7], [2, 5, 8],
    [0, 4, 8], [2, 4, 6],
]


@dataclass
class Board:
    cells: list

    def place(self, idx, player):
        if self.cells[idx] != 0:
            return 0
        self.cells[idx] = player
        return 1

    def clear(self, idx):
        self.cells[idx] = 0

    def with_move(self, idx, player):
        next_cells = []
        for v in self.cells:
            next_cells.append(v)
        next_cells[idx] = player
        return Board(cells=next_cells)

    def count_filled(self):
        c = 0
        for v in self.cells:
            if v != 0:
                c = c + 1
        return c

    def is_full(self):
        if self.count_filled() == 9:
            return 1
        return 0

    def line_winner(self):
        for line in WIN_LINES:
            a = self.cells[line[0]]
            b = self.cells[line[1]]
            c = self.cells[line[2]]
            if a != 0 and a == b and b == c:
                return a
        return 0

    def winner(self):
        w = self.line_winner()
        if w != 0:
            return w
        if self.is_full() == 1:
            return 3
        return 0

    def available(self):
        out = []
        i = 0
        while i < 9:
            if self.cells[i] == 0:
                out.append(i)
            i = i + 1
        return out

    def render(self):
        rows = []
        r = 0
        while r < 3:
            chars = []
            k = 0
            while k < 3:
                v = self.cells[r * 3 + k]
                if v == 1:
                    chars.append("X")
                elif v == 2:
                    chars.append("O")
                else:
                    chars.append(".")
                k = k + 1
            rows.append(join(chars, ""))
            r = r + 1
        return join(rows, "|")


def join(xs, sep):
    out = ""
    i = 0
    while i < len(xs):
        if i > 0:
            out = out + sep
        out = out + xs[i]
        i = i + 1
    return out


def make_board():
    cells = []
    i = 0
    while i < 9:
        cells.append(0)
        i = i + 1
    return Board(cells=cells)
