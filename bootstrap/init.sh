#!/bin/bash
python3 bootstrap.py
btcd &> /dev/null &
lnd &> /dev/null &
bash
