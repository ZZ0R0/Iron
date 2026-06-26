# Array-based Huffman tree: nodes live at integer indices in parallel lists.
# Leaves carry a symbol id (>= 0); internal nodes use -1. The two lightest
# active nodes are chosen by (frequency, creation index), so the build is fully
# deterministic and the Iron clone makes identical merge decisions.


class HuffmanTree:
    sym: list
    freq: list
    left: list
    right: list
    alive: list
    root: int

    def __init__(self):
        self.sym = []
        self.freq = []
        self.left = []
        self.right = []
        self.alive = []
        self.root = -1

    def add_node(self, sym, freq, left, right):
        idx = len(self.sym)
        self.sym.append(sym)
        self.freq.append(freq)
        self.left.append(left)
        self.right.append(right)
        self.alive.append(1)
        return idx

    def pick_min(self):
        # Smallest (freq, index) among alive nodes; -1 if none alive.
        best = -1
        for i in range(len(self.sym)):
            if self.alive[i] == 1:
                if best == -1:
                    best = i
                elif self.freq[i] < self.freq[best]:
                    best = i
        return best

    def alive_count(self):
        n = 0
        for i in range(len(self.sym)):
            if self.alive[i] == 1:
                n = n + 1
        return n

    def build(self, syms, freqs):
        # Leaves are created in the given symbol order (caller sorts first).
        for i in range(len(syms)):
            self.add_node(syms[i], freqs[i], -1, -1)
        if len(syms) == 1:
            # Degenerate alphabet: wrap the single leaf so it gets length 1.
            only = 0
            parent = self.add_node(-1, self.freq[only], only, -1)
            self.alive[only] = 0
            self.root = parent
            return
        while self.alive_count() > 1:
            a = self.pick_min()
            self.alive[a] = 0
            b = self.pick_min()
            self.alive[b] = 0
            merged = self.add_node(-1, self.freq[a] + self.freq[b], a, b)
            self.root = merged

    def code_lengths(self):
        # Depth of every leaf, keyed by symbol id. Iterative stack walk.
        lengths = {}
        if self.root == -1:
            return lengths
        stack = []
        stack.append((self.root, 0))
        while len(stack) > 0:
            n = len(stack)
            top = stack[n - 1]
            stack.pop()
            node = top[0]
            depth = top[1]
            if self.sym[node] >= 0:
                lengths[self.sym[node]] = depth
            else:
                lt = self.left[node]
                rt = self.right[node]
                if lt != -1:
                    stack.append((lt, depth + 1))
                if rt != -1:
                    stack.append((rt, depth + 1))
        return lengths

    def internal_count(self):
        n = 0
        for i in range(len(self.sym)):
            if self.sym[i] == -1:
                n = n + 1
        return n
