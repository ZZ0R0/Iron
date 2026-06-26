# Block-level parser for mini-Markdown. Splits raw text into lines, then
# groups consecutive lines into typed blocks: headings, list items and
# paragraphs. Blank lines act as separators.


class Block:
    def __init__(self, kind, level):
        self.kind = kind
        self.level = level
        self.lines = []

    def add(self, line):
        self.lines.append(line)

    def size(self):
        return len(self.lines)


def split_lines(text):
    raw = text.split("\n")
    out = []
    for piece in raw:
        out.append(piece)
    return out


def is_blank(line):
    return len(line.strip()) == 0


def is_heading(line):
    return line.startswith("#")


def heading_level(line):
    level = 0
    while level < len(line) and line[level:level + 1] == "#":
        level = level + 1
    if level > 6:
        level = 6
    return level


def heading_text(line):
    level = heading_level(line)
    rest = line[level:]
    return rest.strip()


def is_list_item(line):
    stripped = line.strip()
    return stripped.startswith("- ")


def list_text(line):
    stripped = line.strip()
    return stripped[2:].strip()


def parse_blocks(lines):
    blocks = []
    current = None
    for line in lines:
        if is_blank(line):
            current = None
        elif is_heading(line):
            b = Block("heading", heading_level(line))
            b.add(heading_text(line))
            blocks.append(b)
            current = None
        elif is_list_item(line):
            if current is None or current.kind != "list":
                current = Block("list", 0)
                blocks.append(current)
            current.add(list_text(line))
        else:
            if current is None or current.kind != "para":
                current = Block("para", 0)
                blocks.append(current)
            current.add(line.strip())
    return blocks


def count_kind(blocks, kind):
    total = 0
    for b in blocks:
        if b.kind == kind:
            total = total + 1
    return total


def count_list_items(blocks):
    total = 0
    for b in blocks:
        if b.kind == "list":
            total = total + b.size()
    return total
