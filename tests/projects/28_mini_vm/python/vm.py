# The stack virtual machine: an operand stack, an integer register file and an
# instruction pointer. `run` executes a program until it hits HALT (or runs off
# the end) and records every value produced by an EMIT instruction. All
# arithmetic is on non-negative integers in the sample programs, so `//` and `%`
# behave identically to truncating division.

from op import PUSH, LOAD, STORE, ADD, SUB, MUL, DIV, MOD, NEG
from op import DUP, SWAP, DROP, CMPEQ, CMPLT, JMP, JZ, JNZ, EMIT, HALT


class Stack:
    data: list

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


def new_stack():
    s = Stack()
    s.data = []
    return s


class VM:
    stack: Stack
    regs: dict
    output: list
    steps: int

    def get_reg(self, r):
        if r in self.regs:
            return self.regs[r]
        return 0

    def set_reg(self, r, v):
        self.regs[r] = v

    def run(self, prog):
        ip = 0
        n = len(prog)
        running = True
        while running:
            if ip >= n:
                running = False
            else:
                op = prog[ip]
                ip = ip + 1
                self.steps = self.steps + 1
                k = op.kind
                if k == PUSH:
                    self.stack.push(op.arg)
                elif k == LOAD:
                    rv = self.get_reg(op.arg)
                    self.stack.push(rv)
                elif k == STORE:
                    sv = self.stack.pop()
                    self.set_reg(op.arg, sv)
                elif k == ADD:
                    b = self.stack.pop()
                    a = self.stack.pop()
                    self.stack.push(a + b)
                elif k == SUB:
                    b = self.stack.pop()
                    a = self.stack.pop()
                    self.stack.push(a - b)
                elif k == MUL:
                    b = self.stack.pop()
                    a = self.stack.pop()
                    self.stack.push(a * b)
                elif k == DIV:
                    b = self.stack.pop()
                    a = self.stack.pop()
                    self.stack.push(a // b)
                elif k == MOD:
                    b = self.stack.pop()
                    a = self.stack.pop()
                    self.stack.push(a % b)
                elif k == NEG:
                    a = self.stack.pop()
                    self.stack.push(-a)
                elif k == DUP:
                    a = self.stack.peek()
                    self.stack.push(a)
                elif k == SWAP:
                    b = self.stack.pop()
                    a = self.stack.pop()
                    self.stack.push(b)
                    self.stack.push(a)
                elif k == DROP:
                    self.stack.pop()
                elif k == CMPEQ:
                    b = self.stack.pop()
                    a = self.stack.pop()
                    if a == b:
                        self.stack.push(1)
                    else:
                        self.stack.push(0)
                elif k == CMPLT:
                    b = self.stack.pop()
                    a = self.stack.pop()
                    if a < b:
                        self.stack.push(1)
                    else:
                        self.stack.push(0)
                elif k == JMP:
                    ip = op.arg
                elif k == JZ:
                    c = self.stack.pop()
                    if c == 0:
                        ip = op.arg
                elif k == JNZ:
                    c = self.stack.pop()
                    if c != 0:
                        ip = op.arg
                elif k == EMIT:
                    ev = self.stack.pop()
                    self.output.append(ev)
                else:
                    running = False
        return self.steps


def new_vm():
    m = VM()
    m.stack = new_stack()
    m.regs = {}
    m.output = []
    m.steps = 0
    return m


def execute(prog):
    m = new_vm()
    m.run(prog)
    return m
