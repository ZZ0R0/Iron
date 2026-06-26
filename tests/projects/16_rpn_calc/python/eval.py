# Evaluator for the RPN calculator: an integer stack machine with named
# variables. Operators, unary ops and stack commands are dispatched by name.

from lexer import tokenize, is_binary_op, is_unary_op, is_command, is_reserved


def ipow(base, exp):
    r = 1
    i = 0
    while i < exp:
        r = r * base
        i = i + 1
    return r


def iabs(x):
    if x < 0:
        return -x
    return x


class Stack:
    def __init__(self):
        self.data = []

    def push(self, v):
        self.data.append(v)

    def pop(self):
        n = len(self.data)
        top = self.data[n - 1]
        self.data.pop()
        return top

    def peek(self):
        n = len(self.data)
        return self.data[n - 1]

    def size(self):
        return len(self.data)

    def total(self):
        acc = 0
        for v in self.data:
            acc = acc + v
        return acc

    def clear(self):
        self.data = []


def apply_binary(op, a, b):
    if op == "+":
        return a + b
    elif op == "-":
        return a - b
    elif op == "*":
        return a * b
    elif op == "/":
        return a // b
    elif op == "%":
        return a % b
    else:
        return ipow(a, b)


def apply_unary(op, a):
    if op == "neg":
        return -a
    return iabs(a)


class RpnCalc:
    def __init__(self):
        self.stack = Stack()
        self.vars = {}

    def set_var(self, name, value):
        self.vars[name] = value

    def get_var(self, name):
        return self.vars[name]

    def do_command(self, tok):
        if tok == "dup":
            self.stack.push(self.stack.peek())
        elif tok == "drop":
            self.stack.pop()
        elif tok == "swap":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.push(b)
            self.stack.push(a)
        elif tok == "over":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.push(a)
            self.stack.push(b)
            self.stack.push(a)
        elif tok == "sum":
            t = self.stack.total()
            self.stack.clear()
            self.stack.push(t)
        else:
            self.stack.clear()

    def evaluate(self, expr):
        toks = tokenize(expr)
        for tok in toks:
            if is_binary_op(tok):
                b = self.stack.pop()
                a = self.stack.pop()
                self.stack.push(apply_binary(tok, a, b))
            elif is_unary_op(tok):
                a = self.stack.pop()
                self.stack.push(apply_unary(tok, a))
            elif is_command(tok):
                self.do_command(tok)
            elif tok in self.vars:
                self.stack.push(self.vars[tok])
            else:
                self.stack.push(int(tok))
        return self.stack.peek()

    def count_reserved(self, expr):
        toks = tokenize(expr)
        c = 0
        for tok in toks:
            if is_reserved(tok):
                c = c + 1
        return c
