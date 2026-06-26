# Tokenizer + token model for the expression evaluator.
#
# A source expression is a space-separated string of tokens, e.g.
# "3 + 4 * ( 2 - 1 )". Each piece is either an operator/parenthesis or a
# non-negative integer literal. To keep the data flowing through the parser
# and evaluator purely numeric (no strings in lists), every token is encoded
# as a (kind, value) pair of integers:
#
#   kind == KIND_NUM (0): value is the integer literal.
#   kind == KIND_OP  (1): value is an operator code (see OP_* below).

KIND_NUM = 0
KIND_OP = 1

OP_ADD = 0
OP_SUB = 1
OP_MUL = 2
OP_DIV = 3
OP_LPAREN = 4
OP_RPAREN = 5


def op_code(sym):
    if sym == "+":
        return OP_ADD
    if sym == "-":
        return OP_SUB
    if sym == "*":
        return OP_MUL
    if sym == "/":
        return OP_DIV
    if sym == "(":
        return OP_LPAREN
    return OP_RPAREN


def op_symbol(code):
    if code == OP_ADD:
        return "+"
    if code == OP_SUB:
        return "-"
    if code == OP_MUL:
        return "*"
    if code == OP_DIV:
        return "/"
    if code == OP_LPAREN:
        return "("
    return ")"


def is_operator_symbol(sym):
    if sym == "+":
        return True
    if sym == "-":
        return True
    if sym == "*":
        return True
    if sym == "/":
        return True
    if sym == "(":
        return True
    if sym == ")":
        return True
    return False


def precedence(code):
    if code == OP_MUL:
        return 2
    if code == OP_DIV:
        return 2
    if code == OP_ADD:
        return 1
    if code == OP_SUB:
        return 1
    return 0


def tokenize(expr):
    raw = expr.split(" ")
    toks = []
    for piece in raw:
        cleaned = piece.strip()
        if len(cleaned) == 0:
            continue
        if is_operator_symbol(cleaned):
            toks.append((KIND_OP, op_code(cleaned)))
        else:
            toks.append((KIND_NUM, int(cleaned)))
    return toks


def count_operators(toks):
    c = 0
    for t in toks:
        if t[0] == KIND_OP:
            code = t[1]
            if code != OP_LPAREN:
                if code != OP_RPAREN:
                    c = c + 1
    return c


def render_tokens(toks):
    parts = []
    for t in toks:
        if t[0] == KIND_NUM:
            parts.append(str(t[1]))
        else:
            parts.append(op_symbol(t[1]))
    return " ".join(parts)
