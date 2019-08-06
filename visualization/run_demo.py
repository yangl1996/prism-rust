import subprocess, signal, os
import sys
import keyboard
sys.path.append('..')

def print_pressed_keys(e):
    key = list(keyboard._pressed_events.keys())
    if len(key)>0 and key[0]==126:
        os.system('killall LiveSlides')

keyboard.hook(print_pressed_keys)
keyboard.wait()

def signal_handler(sig, frame):
    print('Killing all processes')
    p1.kill()
    p2.kill()

os.chdir('longestChainWithForking/')
p1 = subprocess.Popen(['python3', '-m', 'http.server', '5000'])
os.chdir('../longestChainWithVotes/')
p2 = subprocess.Popen(['python3', '-m', 'http.server', '5001'])
os.chdir('..')

signal.signal(signal.SIGINT, signal_handler)
signal.pause()
