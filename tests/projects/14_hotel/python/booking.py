from dataclasses import dataclass


def idiv(a: int, b: int) -> int:
    return a // b


def occ_pct(used: int, horizon: int) -> int:
    return 100 * used // horizon


@dataclass
class Booking:
    room: int
    guest: int
    start: int
    end: int

    def nights(self) -> int:
        return self.end - self.start


def make_bookings() -> list:
    bs = []
    bs.append(Booking(room=101, guest=1, start=0, end=3))
    bs.append(Booking(room=101, guest=2, start=2, end=5))
    bs.append(Booking(room=101, guest=3, start=6, end=9))
    bs.append(Booking(room=102, guest=4, start=0, end=4))
    bs.append(Booking(room=201, guest=5, start=1, end=10))
    bs.append(Booking(room=201, guest=6, start=10, end=14))
    bs.append(Booking(room=202, guest=7, start=3, end=8))
    bs.append(Booking(room=202, guest=8, start=5, end=6))
    return bs


def overlaps(a: Booking, b: Booking) -> bool:
    if a.room != b.room:
        return False
    return a.start < b.end and b.start < a.end


def count_conflicts(bs: list) -> int:
    n = len(bs)
    conflicts = 0
    i = 0
    while i < n:
        j = i + 1
        while j < n:
            if overlaps(bs[i], bs[j]):
                conflicts = conflicts + 1
            j = j + 1
        i = i + 1
    return conflicts


def nights_for_room(bs: list, number: int) -> int:
    total = 0
    for b in bs:
        if b.room == number:
            total = total + b.nights()
    return total


def is_free_on(bs: list, number: int, day: int) -> bool:
    for b in bs:
        if b.room == number:
            if b.start <= day and day < b.end:
                return False
    return True


def bookings_for_room(bs: list, number: int) -> int:
    count = 0
    for b in bs:
        if b.room == number:
            count = count + 1
    return count


def free_days(bs: list, number: int, horizon: int) -> int:
    free = 0
    day = 0
    while day < horizon:
        if is_free_on(bs, number, day):
            free = free + 1
        day = day + 1
    return free
