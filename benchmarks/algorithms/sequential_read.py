def find_last_element(starting_key, store):
    curr_val = store[starting_key]
    prev_val = curr_val
    while curr_val != None:
        prev_val = curr_val
        curr_val = store[curr_val] if curr_val in store else None

    return prev_val

def build_store():
    store = {0:1}
    for i in range(1, 100000000):
        store[i] = i + 1

    return store


stor = build_store()
print(find_last_element(0, stor))
