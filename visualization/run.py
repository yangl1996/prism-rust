import subprocess, time, webbrowser, signal, os

def signal_handler(sig, frame):
    print('Killing all processes')
    p1.kill()
    p2.kill()
    p3.kill()
    for line in os.popen('ps'):
        if 'prism' in line:
            pid = line.split(' ')[0]
            os.kill(int(pid), signal.SIGKILL)

URL = 'localhost'
PORT = 5000
NUM_NODES = 1

p1 = subprocess.Popen(['node', 'index.js'])
time.sleep(2)
p2 = subprocess.Popen(['python3', '-m', 'http.server', f'{PORT}'])
webbrowser.open_new_tab(f'http://{URL}:{PORT}/')
time.sleep(2)
os.chdir('../testbed')
p3 = subprocess.Popen(['./local-experiment.sh', f'{NUM_NODES}'])

signal.signal(signal.SIGINT, signal_handler)
signal.pause()
