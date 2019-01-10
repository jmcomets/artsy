import random
import string
from itertools import takewhile

def take(iterable, n):
    for _, value in takewhile(lambda x: x[0] < n, enumerate(iterable)):
        yield value

CHARS = string.ascii_lowercase

def generate_sequential_entries(nb_entries, max_entry_length):
    q = ['']
    nb = 0
    while q:
        current = q.pop(0)
        if len(current) > max_entry_length:
            continue
        if nb == nb_entries:
            break
        if current:
            yield current
            nb += 1
        for c in CHARS:
            q.append(current + c)

def generate_random_entries(nb_entries, max_entry_length):
    for _ in range(nb_entries):
        yield "".join(random.choice(CHARS) for _ in range(random.randint(1, max_entry_length)))

def generate_entries(nb_entries, max_entry_length, yield_random_entries):
    if yield_random_entries:
        yield from generate_random_entries(nb_entries, max_entry_length)
    else:
        yield from generate_sequential_entries(nb_entries, max_entry_length)

def main(nb_entries, max_entry_length, yield_random_entries):
    entries = list(generate_entries(nb_entries, max_entry_length, yield_random_entries))
    for i, key in enumerate(entries):
        print(f"put {key} {i}")
    for key in entries:
        print(f"get {key}")

if __name__ == "__main__":
    import sys
    if "-h" in sys.argv or "--help" in sys.argv:
        print("usage: generate_entries.py [<nb_entries>] [<max_entry_length>]", file=sys.stderr)
    else:
        try:
            sys.argv.remove("--random")
        except ValueError:
            yield_random_entries = False
        else:
            yield_random_entries = True
        nb_entries = int(sys.argv[1]) if len(sys.argv) > 1 else 10
        max_entry_length = int(sys.argv[2]) if len(sys.argv) > 2 else 10
        try:
            main(nb_entries, max_entry_length, yield_random_entries)
        except KeyboardInterrupt:
            pass
