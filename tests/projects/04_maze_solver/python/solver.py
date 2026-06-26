# Breadth-first search over the grid. Everything is kept deterministic:
# a fixed neighbour order (from grid.moves) and a head-pointer FIFO queue
# (no pop(0), no sets iterated in arbitrary order).

from grid import moves


# Run BFS from (sr, sc). Returns a flat distance array indexed by cell id:
#   dist[cid] = number of steps from the start, or -1 if unreachable.
def bfs_dist(grid, sr, sc):
    n = grid.rows * grid.cols
    dist = []
    for i in range(n):
        dist.append(-1)
    start = grid.cell_id(sr, sc)
    if not grid.passable(sr, sc):
        return dist
    dist[start] = 0
    queue = []
    queue.append(start)
    head = 0
    mv = moves()
    while head < len(queue):
        cid = queue[head]
        head = head + 1
        r = grid.row_of(cid)
        c = grid.col_of(cid)
        for d in mv:
            nr = r + d[0]
            nc = c + d[1]
            if grid.passable(nr, nc):
                nid = grid.cell_id(nr, nc)
                if dist[nid] == -1:
                    dist[nid] = dist[cid] + 1
                    queue.append(nid)
    return dist


# BFS that also records, for each reached cell, the cell it came from.
# parent[cid] = previous cell id on a shortest path, or -1 for none/start.
def bfs_parent(grid, sr, sc):
    n = grid.rows * grid.cols
    dist = []
    parent = []
    for i in range(n):
        dist.append(-1)
        parent.append(-1)
    result = []
    start = grid.cell_id(sr, sc)
    if not grid.passable(sr, sc):
        result.append(dist)
        result.append(parent)
        return result
    dist[start] = 0
    queue = []
    queue.append(start)
    head = 0
    mv = moves()
    while head < len(queue):
        cid = queue[head]
        head = head + 1
        r = grid.row_of(cid)
        c = grid.col_of(cid)
        for d in mv:
            nr = r + d[0]
            nc = c + d[1]
            if grid.passable(nr, nc):
                nid = grid.cell_id(nr, nc)
                if dist[nid] == -1:
                    dist[nid] = dist[cid] + 1
                    parent[nid] = cid
                    queue.append(nid)
    result.append(dist)
    result.append(parent)
    return result


# Length of the shortest path in steps from start to goal, or -1 if blocked.
def shortest_path_length(grid, sr, sc, gr, gc):
    dist = bfs_dist(grid, sr, sc)
    return dist[grid.cell_id(gr, gc)]


# Whether the goal is reachable from the start.
def reachable(grid, sr, sc, gr, gc):
    return shortest_path_length(grid, sr, sc, gr, gc) >= 0


# How many cells are reachable from the start (the size of its region).
def count_reachable(grid, sr, sc):
    dist = bfs_dist(grid, sr, sc)
    total = 0
    for v in dist:
        if v >= 0:
            total = total + 1
    return total


# The farthest reachable distance from the start (0 if only the start).
def eccentricity(grid, sr, sc):
    dist = bfs_dist(grid, sr, sc)
    best = 0
    for v in dist:
        if v > best:
            best = v
    return best


# Reconstruct the shortest path from start to goal as a list of cell ids,
# ordered from start to goal. Empty list if the goal is unreachable.
def reconstruct_path(grid, sr, sc, gr, gc):
    pr = bfs_parent(grid, sr, sc)
    dist = pr[0]
    parent = pr[1]
    goal = grid.cell_id(gr, gc)
    path = []
    if dist[goal] < 0:
        return path
    cur = goal
    while cur != -1:
        path.append(cur)
        cur = parent[cur]
    path.reverse()
    return path
