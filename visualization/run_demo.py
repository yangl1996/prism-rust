import subprocess, signal, os

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
