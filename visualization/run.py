import subprocess, time, webbrowser, signal, os, json

def signal_handler(sig, frame):
    print('Killing all processes')
    p1.kill()
    p2.kill()
    p3.kill()
    for pid in os.popen('pgrep prism'):
        os.kill(int(pid), signal.SIGKILL)

with open('config.json', 'r') as f:
    config = json.loads(f.read())
    URL = config['host']
    VIS_PORT = config['visualization port']
    NUM_NODES = config['num_nodes']

p1 = subprocess.Popen(['node', 'relay_server.js'])
time.sleep(2)
p2 = subprocess.Popen(['python3', '-m', 'http.server', f'{VIS_PORT}'])
webbrowser.open_new_tab(f'http://{URL}:{VIS_PORT}/')
time.sleep(2)
os.chdir('../testbed')
p3 = subprocess.Popen(['./local-experiment.sh', f'{NUM_NODES}'])
signal.signal(signal.SIGINT, signal_handler)
signal.pause()
