# Driver: assemble a handful of small programs for the stack VM, run them and
# print only deterministic integer output (program sizes, disassembly mnemonics,
# emitted values, executed step counts and sorted register snapshots).

from op import push, load, store, add, sub, mul, modulo
from op import dup, swap, cmplt, jmp, jz, emit, halt
from op import mnemonic
from vm import execute


def disasm(prog):
    parts = []
    for op in prog:
        parts.append(mnemonic(op))
    return " | ".join(parts)


def reg_dump(m):
    parts = []
    for r in sorted(m.regs.keys()):
        parts.append(f"{r}:{m.regs[r]}")
    return ",".join(parts)


def report(label, prog):
    m = execute(prog)
    print(f"{label} size={len(prog)} steps={m.steps}")
    print(disasm(prog))
    print(m.output)
    print(reg_dump(m))


def prog_arith():
    p = []
    p.append(push(3))
    p.append(push(4))
    p.append(add())
    p.append(push(5))
    p.append(mul())
    p.append(push(6))
    p.append(sub())
    p.append(emit())
    p.append(halt())
    return p


def prog_sum(n):
    p = []
    p.append(push(0))
    p.append(store(0))
    p.append(push(n))
    p.append(store(1))
    p.append(load(1))
    p.append(jz(15))
    p.append(load(0))
    p.append(load(1))
    p.append(add())
    p.append(store(0))
    p.append(load(1))
    p.append(push(1))
    p.append(sub())
    p.append(store(1))
    p.append(jmp(4))
    p.append(load(0))
    p.append(emit())
    p.append(halt())
    return p


def prog_fact(n):
    p = []
    p.append(push(1))
    p.append(store(0))
    p.append(push(n))
    p.append(store(1))
    p.append(load(1))
    p.append(jz(15))
    p.append(load(0))
    p.append(load(1))
    p.append(mul())
    p.append(store(0))
    p.append(load(1))
    p.append(push(1))
    p.append(sub())
    p.append(store(1))
    p.append(jmp(4))
    p.append(load(0))
    p.append(emit())
    p.append(halt())
    return p


def prog_fib(k):
    p = []
    p.append(push(0))
    p.append(store(0))
    p.append(push(1))
    p.append(store(1))
    p.append(push(k))
    p.append(store(2))
    p.append(load(2))
    p.append(jz(23))
    p.append(load(0))
    p.append(emit())
    p.append(load(0))
    p.append(load(1))
    p.append(add())
    p.append(store(3))
    p.append(load(1))
    p.append(store(0))
    p.append(load(3))
    p.append(store(1))
    p.append(load(2))
    p.append(push(1))
    p.append(sub())
    p.append(store(2))
    p.append(jmp(6))
    p.append(halt())
    return p


def prog_gcd(a, b):
    p = []
    p.append(push(a))
    p.append(store(0))
    p.append(push(b))
    p.append(store(1))
    p.append(load(1))
    p.append(jz(15))
    p.append(load(0))
    p.append(load(1))
    p.append(modulo())
    p.append(store(2))
    p.append(load(1))
    p.append(store(0))
    p.append(load(2))
    p.append(store(1))
    p.append(jmp(4))
    p.append(load(0))
    p.append(emit())
    p.append(halt())
    return p


def prog_max(a, b):
    p = []
    p.append(push(a))
    p.append(store(0))
    p.append(push(b))
    p.append(store(1))
    p.append(load(0))
    p.append(load(1))
    p.append(cmplt())
    p.append(jz(10))
    p.append(load(1))
    p.append(jmp(11))
    p.append(load(0))
    p.append(emit())
    p.append(halt())
    return p


def prog_stackops():
    p = []
    p.append(push(10))
    p.append(push(20))
    p.append(swap())
    p.append(sub())
    p.append(emit())
    p.append(push(7))
    p.append(dup())
    p.append(mul())
    p.append(emit())
    p.append(halt())
    return p


def main():
    report("arith", prog_arith())
    report("sum10", prog_sum(10))
    report("sum100", prog_sum(100))
    report("fact5", prog_fact(5))
    report("fact10", prog_fact(10))
    report("fib10", prog_fib(10))
    report("gcd", prog_gcd(48, 36))
    report("gcd2", prog_gcd(1071, 462))
    report("max", prog_max(17, 42))
    report("max2", prog_max(99, 5))
    report("stackops", prog_stackops())


main()
