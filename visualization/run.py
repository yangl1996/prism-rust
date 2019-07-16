import subprocess, time, webbrowser, signal, os, json, re


def signal_handler(sig, frame):
    print('Killing all processes')
    if PROTOCOL=='prism':
        if not MOCK:
            p1.kill()
            p3.kill()
        p2.kill()
        for pid in os.popen('pgrep prism'):
            os.kill(int(pid), signal.SIGKILL)
    if PROTOCOL=='longest-chain':
        p1.kill()
        p2.kill()

with open('config.json', 'r') as f:
    config = json.loads(f.read())
    URL = config['host']
    VIS_PORT = config['visualization port']
    PROTOCOL = config['protocol']
    if PROTOCOL=='prism':
        NUM_NODES = config['num_nodes']
        MOCK = config['mock']
    if PROTOCOL=='longest-chain': 
        FORK_PROBABILITY = config['fork probability']
if PROTOCOL=='prism':
    with open('main.js', 'r') as f:
        contents = f.read()
        s = 'const mock = true' if MOCK else 'const mock = false'
        fixed_contents = re.sub(r"const mock = \w*", s, contents)
    with open('main.js', 'w+') as f:
        f.write(fixed_contents)
    if not MOCK:
        p1 = subprocess.Popen(['node', 'relay_server.js'])
        time.sleep(2)
    p2 = subprocess.Popen(['python3', '-m', 'http.server', f'{VIS_PORT}'])
    webbrowser.open_new_tab(f'http://{URL}:{VIS_PORT}/prism.html')
    time.sleep(2)
    if not MOCK:
        os.chdir('../testbed')
        p3 = subprocess.Popen(['./local-experiment.sh', f'{NUM_NODES}'])
if PROTOCOL=='longest-chain':
    with open('generate.py', 'r') as f:
        contents = f.read()
        fixed_contents = re.sub(r"forkProbability = [0-9]*[.]?[0-9]*", f'forkProbability = {FORK_PROBABILITY}', contents)
    with open('generate.py', 'w+') as f:
        f.write(fixed_contents)
    p1 = subprocess.Popen(['python3', 'generate.py'])
    p2 = subprocess.Popen(['python3', '-m', 'http.server', f'{VIS_PORT}'])
    webbrowser.open_new_tab(f'http://{URL}:{VIS_PORT}/longestChain.html')
signal.signal(signal.SIGINT, signal_handler)
signal.pause()
