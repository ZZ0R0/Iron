from board import Board

# minimax from X's perspective: X (1) maximizes, O (2) minimizes.
# score: +(10 - depth) if X wins, -(10 - depth) if O wins, 0 draw.


def other(player):
    if player == 1:
        return 2
    return 1


def score_terminal(w, depth):
    if w == 1:
        return 10 - depth
    if w == 2:
        return depth - 10
    return 0


def minimax(b, player, depth):
    w = b.winner()
    if w != 0:
        return score_terminal(w, depth)
    moves = b.available()
    if player == 1:
        best = -1000
        for m in moves:
            child = b.with_move(m, player)
            s = minimax(child, other(player), depth + 1)
            if s > best:
                best = s
        return best
    best = 1000
    for m in moves:
        child = b.with_move(m, player)
        s = minimax(child, other(player), depth + 1)
        if s < best:
            best = s
    return best


def best_move(b, player):
    moves = b.available()
    best_idx = -1
    if player == 1:
        best_score = -1000
        for m in moves:
            child = b.with_move(m, player)
            s = minimax(child, other(player), 1)
            if s > best_score:
                best_score = s
                best_idx = m
        return best_idx
    best_score = 1000
    for m in moves:
        child = b.with_move(m, player)
        s = minimax(child, other(player), 1)
        if s < best_score:
            best_score = s
            best_idx = m
    return best_idx
