class Graph:
    def __init__(self, adj):
        self.adj = adj

    def add_node(self, u):
        if u not in self.adj:
            self.adj[u] = []

    def add_edge(self, u, v):
        self.add_node(u)
        self.add_node(v)
        if v not in self.adj[u]:
            cu = self.adj[u]
            cu.append(v)
            self.adj[u] = cu
        if u not in self.adj[v]:
            cv = self.adj[v]
            cv.append(u)
            self.adj[v] = cv

    def neighbors(self, u):
        return sorted(self.adj[u])

    def nodes(self):
        return sorted(self.adj.keys())

    def node_count(self):
        return len(self.adj.keys())

    def edge_count(self):
        total = 0
        for u in sorted(self.adj.keys()):
            total = total + len(self.adj[u])
        return total // 2

    def degree(self, u):
        return len(self.adj[u])

    def has_edge(self, u, v):
        if u not in self.adj:
            return False
        found = False
        for w in self.adj[u]:
            if w == v:
                found = True
        return found
