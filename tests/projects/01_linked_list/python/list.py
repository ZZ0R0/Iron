from node import Node


class LinkedList:
    head: object
    size: int

    def __init__(self):
        self.head = None
        self.size = 0

    def push_front(self, v):
        self.head = Node(v, self.head)
        self.size = self.size + 1

    def push_back(self, v):
        newn = Node(v, None)
        if self.head is not None:
            cur = self.head
            going = True
            while going:
                nxt = cur.next
                if nxt is not None:
                    cur = nxt
                else:
                    cur.next = newn
                    going = False
        else:
            self.head = newn
        self.size = self.size + 1

    def pop_front(self):
        result = -1
        if self.head is not None:
            node = self.head
            result = node.value
            self.head = node.next
            self.size = self.size - 1
        return result

    def length(self):
        n = 0
        cur = self.head
        going = True
        while going:
            if cur is not None:
                n = n + 1
                cur = cur.next
            else:
                going = False
        return n

    def total(self):
        s = 0
        cur = self.head
        going = True
        while going:
            if cur is not None:
                s = s + cur.value
                cur = cur.next
            else:
                going = False
        return s

    def to_list(self):
        out = []
        cur = self.head
        going = True
        while going:
            if cur is not None:
                out.append(cur.value)
                cur = cur.next
            else:
                going = False
        return out

    def contains(self, target):
        found = False
        cur = self.head
        going = True
        while going:
            if cur is not None:
                if cur.value == target:
                    found = True
                    going = False
                else:
                    cur = cur.next
            else:
                going = False
        return found

    def index_of(self, target):
        idx = 0
        found = -1
        cur = self.head
        going = True
        while going:
            if cur is not None:
                if cur.value == target:
                    found = idx
                    going = False
                else:
                    idx = idx + 1
                    cur = cur.next
            else:
                going = False
        return found

    def get_at(self, pos):
        idx = 0
        result = -1
        cur = self.head
        going = True
        while going:
            if cur is not None:
                if idx == pos:
                    result = cur.value
                    going = False
                else:
                    idx = idx + 1
                    cur = cur.next
            else:
                going = False
        return result

    def max_value(self):
        best = -1000000
        cur = self.head
        going = True
        while going:
            if cur is not None:
                if cur.value > best:
                    best = cur.value
                cur = cur.next
            else:
                going = False
        return best

    def min_value(self):
        best = 1000000
        cur = self.head
        going = True
        while going:
            if cur is not None:
                if cur.value < best:
                    best = cur.value
                cur = cur.next
            else:
                going = False
        return best

    def count_value(self, target):
        c = 0
        cur = self.head
        going = True
        while going:
            if cur is not None:
                if cur.value == target:
                    c = c + 1
                cur = cur.next
            else:
                going = False
        return c

    def reverse(self):
        prev = None
        cur = self.head
        going = True
        while going:
            if cur is not None:
                nxt = cur.next
                cur.next = prev
                prev = cur
                cur = nxt
            else:
                going = False
        self.head = prev

    def remove_value(self, target):
        removed = 0
        kept = LinkedList()
        vals = self.to_list()
        for v in vals:
            if v == target:
                removed = removed + 1
            else:
                kept.push_back(v)
        self.head = kept.head
        self.size = kept.size
        return removed
