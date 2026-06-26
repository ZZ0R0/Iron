def in_bounds(rows, cols, r, c):
    if r < 0:
        return False
    if c < 0:
        return False
    if r >= rows:
        return False
    if c >= cols:
        return False
    return True


def count_neighbors(cells, rows, cols, r, c):
    total = 0
    dr = -1
    while dr <= 1:
        dc = -1
        while dc <= 1:
            if dr != 0 or dc != 0:
                nr = r + dr
                nc = c + dc
                if in_bounds(rows, cols, nr, nc):
                    total = total + cells[nr][nc]
            dc = dc + 1
        dr = dr + 1
    return total


def next_state(alive, neighbors):
    if alive == 1:
        if neighbors == 2:
            return 1
        if neighbors == 3:
            return 1
        return 0
    if neighbors == 3:
        return 1
    return 0
