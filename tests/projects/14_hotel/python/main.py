from room import make_rooms, find_room, total_capacity
from booking import make_bookings, count_conflicts, nights_for_room
from booking import is_free_on, bookings_for_room, free_days
from booking import occ_pct, idiv, overlaps


def main():
    horizon = 14
    rooms = make_rooms()
    bookings = make_bookings()

    print("=== hotel ===")
    print(len(rooms))
    print(total_capacity(rooms))
    print(len(bookings))

    print("=== occupancy ===")
    for r in rooms:
        used = nights_for_room(bookings, r.number)
        pct = occ_pct(used, horizon)
        free = free_days(bookings, r.number, horizon)
        cnt = bookings_for_room(bookings, r.number)
        print(f"room {r.number} cap {r.capacity} bookings {cnt} nights {used} occ {pct} free {free}")

    print("=== revenue ===")
    grand = 0
    for r in rooms:
        used = nights_for_room(bookings, r.number)
        rev = r.revenue(used)
        grand = grand + rev
        print(f"room {r.number} rate {r.rate} revenue {rev}")
    print(f"total {grand}")

    print("=== avg occupancy ===")
    total_nights = 0
    for r in rooms:
        total_nights = total_nights + nights_for_room(bookings, r.number)
    span = horizon * len(rooms)
    avg = occ_pct(total_nights, span)
    print(avg)

    print("=== conflicts ===")
    conflicts = count_conflicts(bookings)
    print(conflicts)
    has = conflicts > 0
    print("yes" if has else "no")

    print("=== conflict pairs ===")
    pairs = []
    n = len(bookings)
    i = 0
    while i < n:
        j = i + 1
        while j < n:
            if overlaps(bookings[i], bookings[j]):
                pairs.append(bookings[i].guest)
                pairs.append(bookings[j].guest)
            j = j + 1
        i = i + 1
    print(pairs)

    print("=== availability day 5 ===")
    for r in rooms:
        avail = is_free_on(bookings, r.number, 5)
        print(f"room {r.number}: " + ("free" if avail else "busy"))

    print("=== free count by room ===")
    counts = {}
    for r in rooms:
        counts[r.number] = free_days(bookings, r.number, horizon)
    for k in sorted(counts.keys()):
        print(f"{k}={counts[k]}")

    print("=== peak day ===")
    best_day = 0
    best_busy = -1
    day = 0
    while day < horizon:
        busy = 0
        for r in rooms:
            if not is_free_on(bookings, r.number, day):
                busy = busy + 1
        if busy > best_busy:
            best_busy = busy
            best_day = day
        day = day + 1
    print(f"day {best_day} busy {best_busy}")

    print("=== summary ===")
    idx = find_room(rooms, 201)
    print(idx)
    print(idiv(grand, len(rooms)))


main()
