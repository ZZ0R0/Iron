from grid import make_grid
from solver import shortest_path_length, reachable, count_reachable
from solver import eccentricity, reconstruct_path, bfs_dist


def yesno(b):
    if b:
        return "yes"
    return "no"


# Maze 1: a small grid with a wall down the middle and one gap.
# 0 = open, 1 = wall.
def maze_one():
    return [
        [0, 0, 0, 0, 0],
        [0, 1, 1, 1, 0],
        [0, 1, 0, 0, 0],
        [0, 1, 0, 1, 1],
        [0, 0, 0, 0, 0],
    ]


# Maze 2: a spiral-ish corridor; the only path is long and winding.
def maze_two():
    return [
        [0, 0, 0, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [0, 0, 0, 0, 0, 1, 0],
        [0, 1, 1, 1, 0, 1, 0],
        [0, 1, 0, 0, 0, 1, 0],
        [0, 1, 1, 1, 1, 1, 0],
        [0, 0, 0, 0, 0, 0, 0],
    ]


# Maze 3: two open regions fully separated by a wall column -> unreachable.
def maze_three():
    return [
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
    ]


def report(label, data, sr, sc, gr, gc):
    g = make_grid(data)
    print(label)
    print(f"size={g.rows}x{g.cols}")
    print(f"open={g.count_open()} walls={g.count_walls()}")
    plen = shortest_path_length(g, sr, sc, gr, gc)
    print(f"path_len={plen}")
    print(f"reachable={yesno(reachable(g, sr, sc, gr, gc))}")
    print(f"region={count_reachable(g, sr, sc)}")
    print(f"eccentricity={eccentricity(g, sr, sc)}")
    path = reconstruct_path(g, sr, sc, gr, gc)
    print(f"path_cells={len(path)}")
    print(path)


def main():
    report("maze1", maze_one(), 0, 0, 4, 4)
    report("maze2", maze_two(), 0, 0, 6, 6)
    report("maze3", maze_three(), 0, 0, 3, 4)

    # Distance map of maze 1 from the corner, printed row by row as ints.
    print("dist_map maze1")
    g = make_grid(maze_one())
    dist = bfs_dist(g, 0, 0)
    for r in range(g.rows):
        row = []
        for c in range(g.cols):
            row.append(dist[g.cell_id(r, c)])
        print(row)

    # A few point-to-point reachability checks across maze 2.
    print("checks maze2")
    g2 = make_grid(maze_two())
    targets = [(0, 6), (4, 2), (3, 3), (1, 0)]
    for t in targets:
        d = shortest_path_length(g2, 0, 0, t[0], t[1])
        ok = yesno(reachable(g2, 0, 0, t[0], t[1]))
        print(f"to({t[0]},{t[1]}) len={d} reach={ok}")


main()
