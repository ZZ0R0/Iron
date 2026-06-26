from tree import Tree


def build(values: list) -> Tree:
    t = Tree()
    for v in values:
        t.insert(v)
    return t


def yes_no(b: bool) -> str:
    if b:
        return "yes"
    return "no"


def main():
    values = [50, 30, 70, 20, 40, 60, 80, 10, 25, 35, 45]
    t = build(values)

    print(f"inserted={len(values)} size={t.size}")

    inorder = t.inorder()
    print(f"inorder: {inorder}")

    preorder = t.preorder()
    print(f"preorder: {preorder}")

    print(f"height={t.height()} count={t.count()}")
    print(f"min={t.min_value()} max={t.max_value()}")

    probes = [10, 15, 35, 50, 80, 99, 45, 5]
    for p in probes:
        print(f"contains({p})={yes_no(t.contains(p))}")

    # duplicates must be ignored
    t.insert(50)
    t.insert(30)
    print(f"after_dup_size={t.size}")

    # a second, degenerate tree (sorted input -> skewed)
    skewed = build([1, 2, 3, 4, 5, 6])
    print(f"skewed_height={skewed.height()} skewed_count={skewed.count()}")
    print(f"skewed_inorder: {skewed.inorder()}")
    print(f"skewed_min={skewed.min_value()} skewed_max={skewed.max_value()}")

    # a single-node tree
    one = build([42])
    print(f"one_height={one.height()} one_min={one.min_value()} one_max={one.max_value()}")
    print(f"one_contains_42={yes_no(one.contains(42))} one_contains_7={yes_no(one.contains(7))}")


main()
