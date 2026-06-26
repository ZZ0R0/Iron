# Opcode definitions for the stack VM. An instruction is an `Op` value carrying
# an integer kind (the opcode) and a single integer operand (used by PUSH /
# LOAD / STORE / JMP / JZ / JNZ; ignored otherwise). Module-level constructors
# build each instruction so callers never touch the raw kind constants.

PUSH = 0
LOAD = 1
STORE = 2
ADD = 3
SUB = 4
MUL = 5
DIV = 6
MOD = 7
NEG = 8
DUP = 9
SWAP = 10
DROP = 11
CMPEQ = 12
CMPLT = 13
JMP = 14
JZ = 15
JNZ = 16
EMIT = 17
HALT = 18


class Op:
    kind: int
    arg: int


def make(kind, arg):
    op = Op()
    op.kind = kind
    op.arg = arg
    return op


def push(v):
    return make(PUSH, v)

def load(r):
    return make(LOAD, r)

def store(r):
    return make(STORE, r)

def add():
    return make(ADD, 0)

def sub():
    return make(SUB, 0)

def mul():
    return make(MUL, 0)

def div():
    return make(DIV, 0)

def modulo():
    return make(MOD, 0)

def neg():
    return make(NEG, 0)

def dup():
    return make(DUP, 0)

def swap():
    return make(SWAP, 0)

def drop():
    return make(DROP, 0)

def cmpeq():
    return make(CMPEQ, 0)

def cmplt():
    return make(CMPLT, 0)

def jmp(t):
    return make(JMP, t)

def jz(t):
    return make(JZ, t)

def jnz(t):
    return make(JNZ, t)

def emit():
    return make(EMIT, 0)

def halt():
    return make(HALT, 0)


def mnemonic(op):
    k = op.kind
    if k == PUSH:
        return "PUSH " + str(op.arg)
    elif k == LOAD:
        return "LOAD " + str(op.arg)
    elif k == STORE:
        return "STORE " + str(op.arg)
    elif k == ADD:
        return "ADD"
    elif k == SUB:
        return "SUB"
    elif k == MUL:
        return "MUL"
    elif k == DIV:
        return "DIV"
    elif k == MOD:
        return "MOD"
    elif k == NEG:
        return "NEG"
    elif k == DUP:
        return "DUP"
    elif k == SWAP:
        return "SWAP"
    elif k == DROP:
        return "DROP"
    elif k == CMPEQ:
        return "CMPEQ"
    elif k == CMPLT:
        return "CMPLT"
    elif k == JMP:
        return "JMP " + str(op.arg)
    elif k == JZ:
        return "JZ " + str(op.arg)
    elif k == JNZ:
        return "JNZ " + str(op.arg)
    elif k == EMIT:
        return "EMIT"
    else:
        return "HALT"
