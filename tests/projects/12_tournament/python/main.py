from team import Team, strength_of, name_of, seed_ids, names_of
from bracket import standard_bracket, play, round_name


def build_teams():
    teams = []
    teams.append(Team(tid=1, name="Lions", strength=12, seed=0, wins=0))
    teams.append(Team(tid=2, name="Bears", strength=18, seed=0, wins=0))
    teams.append(Team(tid=3, name="Wolves", strength=15, seed=0, wins=0))
    teams.append(Team(tid=4, name="Hawks", strength=15, seed=0, wins=0))
    teams.append(Team(tid=5, name="Foxes", strength=9, seed=0, wins=0))
    teams.append(Team(tid=6, name="Pumas", strength=20, seed=0, wins=0))
    teams.append(Team(tid=7, name="Rams", strength=11, seed=0, wins=0))
    teams.append(Team(tid=8, name="Owls", strength=14, seed=0, wins=0))
    return teams


def zeros(n):
    z = []
    for i in range(n):
        z.append(0)
    return z


def seed_rank(ordered, tid):
    for i in range(len(ordered)):
        if ordered[i] == tid:
            return i + 1
    return 0


def main():
    teams = build_teams()
    n = len(teams)
    ids = []
    for t in teams:
        ids.append(t.tid)

    ordered = seed_ids(teams, ids)

    print("=== Seeding ===")
    for i in range(len(ordered)):
        tid = ordered[i]
        print(f"seed {i + 1}: id {tid} {name_of(teams, tid)} str {strength_of(teams, tid)}")

    # Place teams into the bracket according to the standard seed order.
    seed_order = standard_bracket(n)
    bracket = []
    for s in seed_order:
        bracket.append(ordered[s - 1])

    print("=== Bracket draw ===")
    print(names_of(teams, bracket))

    wins = zeros(n + 1)
    total_matches = 0
    current = bracket

    while len(current) > 1:
        title = round_name(len(current))
        print(f"=== {title} ({len(current)} teams) ===")
        winners = []
        i = 0
        while i < len(current):
            a = current[i]
            b = current[i + 1]
            res = play(teams, a, b)
            sa = res[0]
            sb = res[1]
            w = res[2]
            total_matches = total_matches + 1
            wins[w] = wins[w] + 1
            sda = seed_rank(ordered, a)
            sdb = seed_rank(ordered, b)
            print(f"(s{sda}) {name_of(teams, a)} {sa}-{sb} {name_of(teams, b)} (s{sdb}) -> {name_of(teams, w)}")
            winners.append(w)
            i = i + 2
        print(f"advancing: {winners}")
        current = winners

    champ = current[0]
    print("=== Result ===")
    print(f"champion id {champ} {name_of(teams, champ)}")
    print(f"champion seed {seed_rank(ordered, champ)}")
    print(f"total matches {total_matches}")

    print("=== Wins per team ===")
    for tid in range(1, n + 1):
        print(f"{tid} {name_of(teams, tid)} {wins[tid]}")

    undefeated = []
    rounds_total = 0
    m = n
    while m > 1:
        m = m // 2
        rounds_total = rounds_total + 1
    for tid in range(1, n + 1):
        if wins[tid] == rounds_total:
            undefeated.append(tid)
    print(f"rounds {rounds_total}")
    print(f"undefeated {undefeated}")


main()
