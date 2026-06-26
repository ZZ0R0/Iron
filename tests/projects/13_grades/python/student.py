# Student model: a name plus parallel lists of course names and integer grades.

class Student:
    def __init__(self, name, courses, grades):
        self.name = name
        self.courses = courses
        self.grades = grades

    def count(self):
        return len(self.grades)

    def total(self):
        t = 0
        for g in self.grades:
            t = t + g
        return t

    def average(self):
        n = len(self.grades)
        if n == 0:
            return 0
        return self.total() // n

    def best(self):
        if len(self.grades) == 0:
            return 0
        return max(self.grades)

    def worst(self):
        if len(self.grades) == 0:
            return 0
        return min(self.grades)

    def passed(self, threshold):
        c = 0
        for g in self.grades:
            if g >= threshold:
                c = c + 1
        return c

    def grade_in(self, course):
        i = 0
        while i < len(self.courses):
            if self.courses[i] == course:
                return self.grades[i]
            i = i + 1
        return -1
