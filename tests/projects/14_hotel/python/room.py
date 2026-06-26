from dataclasses import dataclass


@dataclass
class Room:
    number: int
    capacity: int
    rate: int

    def revenue(self, nights: int) -> int:
        return self.rate * nights


def make_rooms() -> list:
    rooms = []
    rooms.append(Room(number=101, capacity=2, rate=80))
    rooms.append(Room(number=102, capacity=2, rate=80))
    rooms.append(Room(number=201, capacity=4, rate=120))
    rooms.append(Room(number=202, capacity=1, rate=60))
    return rooms


def find_room(rooms: list, number: int) -> int:
    idx = 0
    for r in rooms:
        if r.number == number:
            return idx
        idx = idx + 1
    return -1


def total_capacity(rooms: list) -> int:
    total = 0
    for r in rooms:
        total = total + r.capacity
    return total
