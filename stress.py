import random

text = "([])"

def terminal(depth):
    text = "()"

    for _ in range(0, depth):
        tags = random.choice(["()", "[]"])

        text = tags[0] + text + tags[1]

    return text

def gen_entry(max_depth, parallel_remaining=100):
    if max_depth < 5:
        return terminal(random.randint(0, 3))
    depth = random.randint(5, max_depth)

    if parallel_remaining < 2:
        parallel = 1
    else:
        parallel = random.randint(1, 1 + parallel_remaining // 2)
    parallel_remaining -= parallel

    return "".join([gen_entry(depth - 1, parallel_remaining / parallel) for _ in range(parallel)])

def gen_tag_name():
    length = random.randint(2, 12)

    return "".join([random.choice("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ") for _ in range(length)])

def gen_tags():
    name = gen_tag_name()

    return "<" + name + ">", "</" + name + ">"

def gen_tag_entry(max_depth, allowed_children):
    tags = gen_tags()
    if allowed_children < 1 or max_depth == 0:
        return "".join(tags)
    else:
        if max_depth > 0:
            children = random.randint(0, allowed_children // 3)
            return tags[0] + "".join([gen_tag_entry(max_depth - 1, (allowed_children - children) // children) for _ in range(children)]) + tags[1]
        return "".join(tags)

open("benches/stress.txt", "w").write("[" + gen_entry(100, 2000) + "]")
# open("src/stress.txt", "w").write("[" + gen_entry(500, 100000) + "]")
open("benches/stress_html.txt", "w").write(gen_tag_entry(50, 2000))
# open("src/stress_html.txt", "w").write(gen_tag_entry(500, 100000))