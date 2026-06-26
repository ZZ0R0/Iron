LOAN_PERIOD = 14
FINE_PER_DAY = 25


class Book:
    def __init__(self, id, title, total, available, borrow_count):
        self.id = id
        self.title = title
        self.total = total
        self.available = available
        self.borrow_count = borrow_count


class Member:
    def __init__(self, id, name, active, borrowed_total, fines):
        self.id = id
        self.name = name
        self.active = active
        self.borrowed_total = borrowed_total
        self.fines = fines


class Loan:
    def __init__(self, book_id, member_id, checkout_day, due_day, returned_day):
        self.book_id = book_id
        self.member_id = member_id
        self.checkout_day = checkout_day
        self.due_day = due_day
        self.returned_day = returned_day


def fine_for(loan, current_day):
    end = loan.returned_day
    if end < 0:
        end = current_day
    over = end - loan.due_day
    if over <= 0:
        return 0
    return over * FINE_PER_DAY
