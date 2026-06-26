from student import Student
from gradebook import mention, ranking, distribution
from gradebook import course_sums, course_counts, course_high
from gradebook import class_average, honor_count

def build_roster():
    roster = []
    roster.append(Student("alice", ["math", "cs", "art"], [92, 88, 95]))
    roster.append(Student("bob", ["math", "cs", "art"], [60, 72, 55]))
    roster.append(Student("carol", ["math", "cs", "art"], [80, 80, 80]))
    roster.append(Student("dave", ["math", "cs", "art"], [45, 50, 65]))
    roster.append(Student("erin", ["math", "cs", "art"], [99, 91, 90]))
    roster.append(Student("frank", ["math", "cs", "art"], [70, 68, 74]))
    return roster

def main():
    roster = build_roster()

    print("=== averages ===")
    for s in roster:
        print(f"{s.name} avg={s.average()} best={s.best()} worst={s.worst()} passed={s.passed(70)}")

    print("=== ranking ===")
    rank = 1
    for p in ranking(roster):
        avg = -p[0]
        name = p[1]
        print(f"{rank} {name} {avg} {mention(avg)}")
        rank = rank + 1

    print("=== distribution ===")
    dist = distribution(roster)
    for m in sorted(dist.keys()):
        print(f"{m}={dist[m]}")

    print("=== courses ===")
    sums = course_sums(roster)
    cnts = course_counts(roster)
    for c in sorted(sums.keys()):
        avg = sums[c] // cnts[c]
        high = course_high(roster, c)
        print(f"{c} avg={avg} high={high} students={cnts[c]}")

    print("=== summary ===")
    print(f"students={len(roster)}")
    print(f"class_average={class_average(roster)}")
    print(f"honor_roll={honor_count(roster)}")
    top = ranking(roster)[0]
    print(f"top_student={top[1]}")

main()
