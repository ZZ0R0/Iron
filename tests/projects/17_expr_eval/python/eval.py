# Evaluate a postfix (RPN) token list with an integer stack machine.
#
# Numbers are pushed; an operator pops its two operands (right operand on top),
# applies the operation and pushes the result. Division is floor division on
# non-negative operands, which matches Rust integer division on the inputs the
# driver uses. The final stack top is the value of the expression.

from tokens import KIND_NUM, OP_ADD, OP_SUB, OP_MUL, OP_DIV


def apply_op(code, a, b):
    if code == OP_ADD:
        return a + b
    if code == OP_SUB:
        return a - b
    if code == OP_MUL:
        return a * b
    if code == OP_DIV:
        return a // b
    return 0


def eval_rpn(rpn):
    stack = []
    for t in rpn:
        kind = t[0]
        val = t[1]
        if kind == KIND_NUM:
            stack.append(val)
        else:
            n = len(stack)
            b = stack[n - 1]
            a = stack[n - 2]
            stack.pop()
            stack.pop()
            stack.append(apply_op(val, a, b))
    n = len(stack)
    return stack[n - 1]
