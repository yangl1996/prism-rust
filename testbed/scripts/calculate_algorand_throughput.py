#!/usr/local/bin/python3

import sys
import math

block_size = int(sys.argv[1])
size_in_txns = int(block_size / 100000 * 429)
deadline = int(size_in_txns / 3521 * 250)

print("Actual block size: {} txns, deadline: {} ms".format(size_in_txns, deadline))
