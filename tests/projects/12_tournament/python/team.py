class Team:
    def __init__(self, tid, name, strength, seed, wins):
        self.tid = tid
        self.name = name
        self.strength = strength
        self.seed = seed
        self.wins = wins


def strength_of(teams, tid):
    for t in teams:
        if t.tid == tid:
            return t.strength
    return 0


def name_of(teams, tid):
    for t in teams:
        if t.tid == tid:
            return t.name
    return "?"


def seed_ids(teams, ids):
    # Selection sort ids by (strength desc, id asc). Strongest team -> seed 1.
    result = []
    for x in ids:
        result.append(x)
    n = len(result)
    for i in range(n):
        best = i
        for j in range(i + 1, n):
            sj = strength_of(teams, result[j])
            sb = strength_of(teams, result[best])
            if sj > sb:
                best = j
            elif sj == sb:
                if result[j] < result[best]:
                    best = j
        tmp = result[i]
        result[i] = result[best]
        result[best] = tmp
    return result


def names_of(teams, ids):
    parts = []
    for x in ids:
        parts.append(name_of(teams, x))
    return ",".join(parts)
