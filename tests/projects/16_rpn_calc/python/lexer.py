# Lexer for the RPN calculator: turns a raw string into a list of token
# strings and classifies them (operators, stack commands, names/numbers).

BINARY_OPS = ["+", "-", "*", "/", "%", "^"]
UNARY_OPS = ["neg", "abs"]
COMMANDS = ["dup", "drop", "swap", "over", "sum", "clr"]


def tokenize(expr):
    raw = expr.split(" ")
    toks = []
    for piece in raw:
        cleaned = piece.strip()
        if len(cleaned) > 0:
            toks.append(cleaned)
    return toks


def is_binary_op(t):
    return t in BINARY_OPS


def is_unary_op(t):
    return t in UNARY_OPS


def is_command(t):
    return t in COMMANDS


def is_reserved(t):
    if is_binary_op(t):
        return True
    if is_unary_op(t):
        return True
    if is_command(t):
        return True
    return False
