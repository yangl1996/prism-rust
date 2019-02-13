import sys

intext = sys.stdin.read()

left_ptr = -1
right_ptr = -1
tot = 0

while True:
    left_ptr = intext.find('(', left_ptr + 1)
    right_ptr = intext.find(')', right_ptr + 1)
    if left_ptr == -1:
        break
    num = int(intext[left_ptr+1:right_ptr])
    tot += num

print(tot)
