import string
import random

def generate_entries(nb_entries, max_entry_length):
    chars = string.ascii_lowercase
    digits = string.digits
    for value in range(nb_entries):
        key = "".join(random.choice(chars) for _ in range(random.randint(1, max_entry_length)))
        yield key, value

def main(prefix, nb_entries, max_entry_length):
    for key, value in generate_entries(nb_entries, max_entry_length):
        print(f"{prefix} {key} {value}")

if __name__ == "__main__":
    import sys
    if "-h" in sys.argv or "--help" in sys.argv:
        print("usage: generate_entries.py [<nb_entries>] [<max_entry_length>]", file=sys.stderr)
    else:
        prefix = "put"
        nb_entries = int(sys.argv[1]) if len(sys.argv) > 1 else 10
        max_entry_length = int(sys.argv[2]) if len(sys.argv) > 2 else 10
        main(prefix, nb_entries, max_entry_length)
