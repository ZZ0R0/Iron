from models import Book, Member, Loan, fine_for, LOAN_PERIOD


class Library:
    def __init__(self):
        self.books = []
        self.members = []
        self.loans = []

    def add_book(self, id, title, total):
        self.books.append(Book(id=id, title=title, total=total, available=total, borrow_count=0))

    def add_member(self, id, name):
        self.members.append(Member(id=id, name=name, active=0, borrowed_total=0, fines=0))

    def book_index(self, book_id):
        idx = 0
        for b in self.books:
            if b.id == book_id:
                return idx
            idx = idx + 1
        return -1

    def member_index(self, member_id):
        idx = 0
        for m in self.members:
            if m.id == member_id:
                return idx
            idx = idx + 1
        return -1

    def active_loan_index(self, book_id, member_id):
        idx = 0
        for ln in self.loans:
            if ln.book_id == book_id and ln.member_id == member_id and ln.returned_day < 0:
                return idx
            idx = idx + 1
        return -1

    def checkout(self, book_id, member_id, day):
        bi = self.book_index(book_id)
        mi = self.member_index(member_id)
        if bi < 0 or mi < 0:
            return 0
        if self.books[bi].available <= 0:
            return 0
        self.books[bi].available = self.books[bi].available - 1
        self.books[bi].borrow_count = self.books[bi].borrow_count + 1
        self.members[mi].active = self.members[mi].active + 1
        self.members[mi].borrowed_total = self.members[mi].borrowed_total + 1
        due = day + LOAN_PERIOD
        self.loans.append(Loan(book_id=book_id, member_id=member_id, checkout_day=day, due_day=due, returned_day=-1))
        return 1

    def return_book(self, book_id, member_id, day):
        li = self.active_loan_index(book_id, member_id)
        if li < 0:
            return 0
        self.loans[li].returned_day = day
        fine = fine_for(self.loans[li], day)
        bi = self.book_index(book_id)
        mi = self.member_index(member_id)
        self.books[bi].available = self.books[bi].available + 1
        self.members[mi].active = self.members[mi].active - 1
        self.members[mi].fines = self.members[mi].fines + fine
        return fine

    def overdue_book_ids(self, current_day):
        out = []
        for ln in self.loans:
            if ln.returned_day < 0 and current_day > ln.due_day:
                out.append(ln.book_id)
        out.sort()
        return out

    def available_book_ids(self):
        out = []
        for b in self.books:
            if b.available > 0:
                out.append(b.id)
        out.sort()
        return out

    def total_copies_out(self):
        total = 0
        for b in self.books:
            total = total + (b.total - b.available)
        return total

    def most_borrowed_id(self):
        best_id = -1
        best_count = -1
        for b in self.books:
            if b.borrow_count > best_count:
                best_count = b.borrow_count
                best_id = b.id
        return best_id
