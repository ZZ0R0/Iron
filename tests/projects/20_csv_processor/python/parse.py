# CSV parsing layer: split raw text into rows and fields, then build typed
# Record objects (dept, name, age, salary). A "table" is just a list[Record].


class Record:
    def __init__(self, dept, name, age, salary):
        self.dept = dept
        self.name = name
        self.age = age
        self.salary = salary


def field_of(r, key):
    if key == "dept":
        return r.dept
    if key == "name":
        return r.name
    return ""


def number_of(r, key):
    if key == "age":
        return r.age
    if key == "salary":
        return r.salary
    return 0


def split_line(line, sep):
    fields = []
    for raw in line.split(sep):
        fields.append(raw.strip())
    return fields


def split_rows(text):
    rows = []
    for line in text.split("\n"):
        cleaned = line.strip()
        if len(cleaned) > 0:
            rows.append(cleaned)
    return rows


def parse_table(text, sep):
    rows = split_rows(text)
    records = []
    i = 0
    for row in rows:
        # skip the header line (first non-empty row)
        if i == 0:
            i = i + 1
            continue
        cells = split_line(row, sep)
        if len(cells) < 4:
            continue
        dept = cells[0]
        name = cells[1]
        age = int(cells[2])
        salary = int(cells[3])
        records.append(Record(dept, name, age, salary))
        i = i + 1
    return records


def header_of(text, sep):
    rows = split_rows(text)
    if len(rows) == 0:
        return []
    return split_line(rows[0], sep)
