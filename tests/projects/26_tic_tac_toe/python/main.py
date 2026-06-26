from board import Board, make_board
from ai import minimax, best_move, other


def play_game(first):
    b = make_board()
    player = first
    move_no = 0
    print(f"game start: first={first}")
    while b.winner() == 0:
        m = best_move(b, player)
        b.place(m, player)
        move_no = move_no + 1
        print(f"move {move_no}: p{player} -> cell {m} board {b.render()}")
        player = other(player)
    w = b.winner()
    print(f"result: {w}")
    return w


def play_scripted(first, script):
    # player `first` (the AI) plays optimally; the opponent follows a script.
    b = make_board()
    ai_player = first
    player = first
    move_no = 0
    si = 0
    print(f"scripted start: ai=p{ai_player}")
    while b.winner() == 0:
        if player == ai_player:
            m = best_move(b, player)
        else:
            m = script[si]
            si = si + 1
        b.place(m, player)
        move_no = move_no + 1
        print(f"move {move_no}: p{player} -> cell {m} board {b.render()}")
        player = other(player)
    w = b.winner()
    print(f"scripted result: {w}")
    return w


def detect(cells):
    b = make_board()
    i = 0
    while i < 9:
        b.cells[i] = cells[i]
        i = i + 1
    return b.winner()


def main():
    # part 1: two perfect AIs face off (must be a draw)
    print("== perfect vs perfect ==")
    r1 = play_game(1)
    r2 = play_game(2)
    print(f"draws: {r1} {r2}")

    # part 2: AI must take an immediate win
    print("== ai takes the win ==")
    bb = make_board()
    bb.cells[0] = 1
    bb.cells[1] = 1
    bb.cells[4] = 2
    print(f"setup board {bb.render()}")
    mv = best_move(bb, 1)
    print(f"winning move for X = {mv}")
    bb.place(mv, 1)
    print(f"after move {bb.render()} winner {bb.winner()}")

    # part 3: AI must block opponent
    print("== ai blocks ==")
    cb = make_board()
    cb.cells[3] = 2
    cb.cells[4] = 2
    cb.cells[0] = 1
    print(f"setup board {cb.render()}")
    bm = best_move(cb, 1)
    print(f"blocking move for X = {bm}")

    # part 4: scripted opponent makes a mistake, AI punishes
    print("== scripted ==")
    sr = play_scripted(1, [4, 8, 2, 6])
    print(f"final {sr}")

    # part 5: winner detection table
    print("== detection ==")
    boards = []
    boards.append([1, 1, 1, 0, 2, 0, 2, 0, 0])
    boards.append([2, 0, 0, 2, 1, 0, 2, 0, 1])
    boards.append([1, 2, 1, 1, 2, 2, 2, 1, 1])
    boards.append([1, 2, 1, 2, 2, 1, 1, 1, 2])
    boards.append([0, 0, 0, 0, 0, 0, 0, 0, 0])
    idx = 0
    for bd in boards:
        w = detect(bd)
        print(f"board {idx}: winner {w}")
        idx = idx + 1

    # part 6: minimax score of the empty board for each starter
    print("== scores ==")
    e1 = make_board()
    print(f"empty score X-to-move {minimax(e1, 1, 0)}")
    e2 = make_board()
    print(f"empty score O-to-move {minimax(e2, 2, 0)}")


main()
