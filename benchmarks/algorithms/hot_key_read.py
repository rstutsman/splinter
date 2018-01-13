def main():
    store = {0:1}
    for i in range(1:100000000):
        store[i] = i + 1

    foo = 0
    for i in range(1:100000000):
        foo += store[i % 1028]

    print(foo)


