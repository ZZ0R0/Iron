from list import LinkedList


def build_from(xs):
    lst = LinkedList()
    for x in xs:
        lst.push_back(x)
    return lst


def yesno(b):
    if b:
        return "yes"
    return "no"


def main():
    lst = build_from([5, 3, 8, 1, 9, 2, 7])
    print(f"length {lst.length()}")
    print(f"sum {lst.total()}")
    print(lst.to_list())

    lst.push_front(100)
    lst.push_back(200)
    print(f"length {lst.length()}")
    print(lst.to_list())
    print(f"front {lst.get_at(0)}")
    print(f"at3 {lst.get_at(3)}")

    print(f"max {lst.max_value()}")
    print(f"min {lst.min_value()}")

    print(f"has8 {yesno(lst.contains(8))}")
    print(f"has42 {yesno(lst.contains(42))}")
    print(f"idx9 {lst.index_of(9)}")
    print(f"idx42 {lst.index_of(42)}")

    popped = lst.pop_front()
    print(f"popped {popped}")
    print(f"length {lst.length()}")
    print(lst.to_list())

    lst.reverse()
    print("reversed")
    print(lst.to_list())
    print(f"sum {lst.total()}")

    dup = build_from([4, 7, 4, 1, 4, 9, 4])
    print(f"count4 {dup.count_value(4)}")
    removed = dup.remove_value(4)
    print(f"removed {removed}")
    print(dup.to_list())
    print(f"length {dup.length()}")
    print(f"sum {dup.total()}")

    empty = LinkedList()
    print(f"emptylen {empty.length()}")
    print(f"emptypop {empty.pop_front()}")
    print(f"emptyhas {yesno(empty.contains(1))}")

    chain = LinkedList()
    acc = 0
    for i in range(1, 11):
        chain.push_back(i)
        acc = acc + i
    print(f"chainlen {chain.length()}")
    print(f"chainsum {chain.total()}")
    print(f"chainacc {acc}")
    chain.reverse()
    print(chain.to_list())
    print(f"get5 {chain.get_at(5)}")


main()
