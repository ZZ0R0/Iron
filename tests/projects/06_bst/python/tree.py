NONE = -1


class Tree:
    vals: list
    left: list
    right: list
    root: int
    size: int

    def __init__(self):
        self.vals = []
        self.left = []
        self.right = []
        self.root = NONE
        self.size = 0

    def new_node(self, v: int) -> int:
        idx = len(self.vals)
        self.vals.append(v)
        self.left.append(NONE)
        self.right.append(NONE)
        return idx

    def insert(self, v: int):
        node = self.new_node(v)
        if self.root == NONE:
            self.root = node
            self.size = self.size + 1
            return
        cur = self.root
        while cur != NONE:
            if v < self.vals[cur]:
                if self.left[cur] == NONE:
                    self.left[cur] = node
                    self.size = self.size + 1
                    cur = NONE
                else:
                    cur = self.left[cur]
            elif v > self.vals[cur]:
                if self.right[cur] == NONE:
                    self.right[cur] = node
                    self.size = self.size + 1
                    cur = NONE
                else:
                    cur = self.right[cur]
            else:
                cur = NONE

    def contains_at(self, node: int, v: int) -> bool:
        cur = node
        while cur != NONE:
            if v == self.vals[cur]:
                return True
            elif v < self.vals[cur]:
                cur = self.left[cur]
            else:
                cur = self.right[cur]
        return False

    def contains(self, v: int) -> bool:
        return self.contains_at(self.root, v)

    def inorder_at(self, node: int) -> list:
        out = []
        if node == NONE:
            return out
        left = self.inorder_at(self.left[node])
        for x in left:
            out.append(x)
        out.append(self.vals[node])
        right = self.inorder_at(self.right[node])
        for x in right:
            out.append(x)
        return out

    def inorder(self) -> list:
        return self.inorder_at(self.root)

    def preorder_at(self, node: int) -> list:
        out = []
        if node == NONE:
            return out
        out.append(self.vals[node])
        left = self.preorder_at(self.left[node])
        for x in left:
            out.append(x)
        right = self.preorder_at(self.right[node])
        for x in right:
            out.append(x)
        return out

    def preorder(self) -> list:
        return self.preorder_at(self.root)

    def height_at(self, node: int) -> int:
        if node == NONE:
            return 0
        lh = self.height_at(self.left[node])
        rh = self.height_at(self.right[node])
        if lh > rh:
            return lh + 1
        return rh + 1

    def height(self) -> int:
        return self.height_at(self.root)

    def count_at(self, node: int) -> int:
        if node == NONE:
            return 0
        return 1 + self.count_at(self.left[node]) + self.count_at(self.right[node])

    def count(self) -> int:
        return self.count_at(self.root)

    def min_value(self) -> int:
        cur = self.root
        while self.left[cur] != NONE:
            cur = self.left[cur]
        return self.vals[cur]

    def max_value(self) -> int:
        cur = self.root
        while self.right[cur] != NONE:
            cur = self.right[cur]
        return self.vals[cur]
