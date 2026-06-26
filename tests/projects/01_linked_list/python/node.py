class Node:
    value: int
    next: object

    def __init__(self, value, nxt):
        self.value = value
        self.next = nxt


def make_node(value):
    return Node(value, None)
