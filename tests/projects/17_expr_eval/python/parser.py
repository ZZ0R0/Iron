# Shunting-yard: convert an infix token list into a postfix (RPN) token list.
#
# All four binary operators (+ - * /) are left-associative, so an operator on
# top of the stack with precedence >= the incoming operator is popped to the
# output before the incoming one is pushed. Parentheses group sub-expressions
# and never reach the output.
#
# The operator stack is a plain list of operator codes (ints); parentheses are
# represented on it by OP_LPAREN. The output is a (kind, value) token list,
# exactly the same encoding tokenize() produces, minus the parentheses.

from tokens import KIND_NUM, KIND_OP, OP_LPAREN, OP_RPAREN, precedence


def to_rpn(toks):
    output = []
    ops = []
    for t in toks:
        kind = t[0]
        val = t[1]
        if kind == KIND_NUM:
            output.append((KIND_NUM, val))
        elif val == OP_LPAREN:
            ops.append(OP_LPAREN)
        elif val == OP_RPAREN:
            while len(ops) > 0 and ops[len(ops) - 1] != OP_LPAREN:
                top = ops[len(ops) - 1]
                ops.pop()
                output.append((KIND_OP, top))
            if len(ops) > 0:
                ops.pop()
        else:
            while len(ops) > 0 and ops[len(ops) - 1] != OP_LPAREN and precedence(ops[len(ops) - 1]) >= precedence(val):
                top = ops[len(ops) - 1]
                ops.pop()
                output.append((KIND_OP, top))
            ops.append(val)
    while len(ops) > 0:
        top = ops[len(ops) - 1]
        ops.pop()
        output.append((KIND_OP, top))
    return output


def max_stack_depth(toks):
    rpn = to_rpn(toks)
    depth = 0
    best = 0
    for t in rpn:
        if t[0] == KIND_NUM:
            depth = depth + 1
        else:
            depth = depth - 1
        if depth > best:
            best = depth
    return best
