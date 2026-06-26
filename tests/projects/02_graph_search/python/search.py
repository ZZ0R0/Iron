def bfs(gr, start):
    order = []
    visited = {}
    visited[start] = True
    queue = [start]
    head = 0
    while head < len(queue):
        u = queue[head]
        head = head + 1
        order.append(u)
        for v in gr.neighbors(u):
            if v not in visited:
                visited[v] = True
                queue.append(v)
    return order

def dfs(gr, start):
    order = []
    visited = {}
    stack = [start]
    while len(stack) > 0:
        top = len(stack) - 1
        u = stack[top]
        stack.pop()
        if u not in visited:
            visited[u] = True
            order.append(u)
            nb = gr.neighbors(u)
            nb.reverse()
            for v in nb:
                if v not in visited:
                    stack.append(v)
    return order

def components(gr):
    visited = {}
    comps = []
    for s in gr.nodes():
        if s not in visited:
            comp = []
            queue = [s]
            head = 0
            visited[s] = True
            while head < len(queue):
                u = queue[head]
                head = head + 1
                comp.append(u)
                for v in gr.neighbors(u):
                    if v not in visited:
                        visited[v] = True
                        queue.append(v)
            comps.append(sorted(comp))
    return comps

def bfs_distances(gr, start):
    dist = {}
    dist[start] = 0
    queue = [start]
    head = 0
    while head < len(queue):
        u = queue[head]
        head = head + 1
        du = dist[u]
        for v in gr.neighbors(u):
            if v not in dist:
                dist[v] = du + 1
                queue.append(v)
    return dist

def shortest_path(gr, src, dst):
    parent = {}
    parent[src] = -1
    queue = [src]
    head = 0
    while head < len(queue):
        u = queue[head]
        head = head + 1
        for v in gr.neighbors(u):
            if v not in parent:
                parent[v] = u
                queue.append(v)
    path = []
    if dst in parent:
        cur = dst
        while cur != -1:
            path.append(cur)
            cur = parent[cur]
        path.reverse()
    return path

def is_reachable(gr, src, dst):
    dist = bfs_distances(gr, src)
    return dst in dist

def is_connected(gr):
    ns = gr.nodes()
    if len(ns) == 0:
        return True
    dist = bfs_distances(gr, ns[0])
    return len(dist.keys()) == len(ns)

def eccentricity(gr, start):
    dist = bfs_distances(gr, start)
    ecc = 0
    for k in sorted(dist.keys()):
        if dist[k] > ecc:
            ecc = dist[k]
    return ecc
