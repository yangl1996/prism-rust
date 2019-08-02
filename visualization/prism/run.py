import subprocess, time, webbrowser, signal, os, json, re


def signal_handler(sig, frame):
    print('Killing all processes')
    if not MOCK:
        p1.kill()
        p3.kill()
    p2.kill()
    for pid in os.popen('pgrep prism'):
        os.kill(int(pid), signal.SIGKILL)

with open('config.json', 'r') as f:
    config = json.loads(f.read())
    URL = config['host']
    VIS_PORT = config['visualization port']
    NUM_NODES = config['num_nodes']
    MOCK = config['mock']

with open('main.js', 'r') as f:
    contents = f.read()
    s = 'const mock = true' if MOCK else 'const mock = false'
    fixed_contents = re.sub(r"const mock = \w*", s, contents)
with open('main.js', 'w+') as f:
    f.write(fixed_contents)
if not MOCK:
    with open('client.js', 'r') as f:
        contents = f.read()
        s = f"let websocket = new WebSocket('ws://{URL}:8080', 'visualization');"
        fixed_contents = re.sub(r"let websocket = new WebSocket\('ws:\/\/([^']*)', 'visualization'\);", s, contents) 
    with open('client.js', 'w+') as f:
        f.write(fixed_contents)
    with open('relay_server.js', 'r') as f:
        contents = f.read()
        s = "const wss = new WebSocket.Server({ host: '" + URL + "', port: 8080 })"
        fixed_contents = re.sub(r"const wss = new WebSocket\.Server\({ host: '([^']*)', port: 8080 }\)", s, contents) 
    with open('relay_server.js', 'w+') as f:
        f.write(fixed_contents)
    with open('../../testbed/local-experiment.sh', 'r') as f:
        contents = f.read()
        s = f'--demo ws://{URL}:' 
        fixed_contents = re.sub(r"--demo ws:\/\/([^:]*):", s, contents)
    with open('../../testbed/local-experiment.sh', 'w+') as f:
        f.write(fixed_contents)
    p1 = subprocess.Popen(['node', 'relay_server.js'])
    time.sleep(2)
p2 = subprocess.Popen(['python3', '-m', 'http.server', f'{VIS_PORT}'])
webbrowser.open_new_tab(f'http://{URL}:{VIS_PORT}/prism.html')
time.sleep(2)
if not MOCK:
    os.chdir('../../testbed')
    p3 = subprocess.Popen(['./local-experiment.sh', f'{NUM_NODES}'])

signal.signal(signal.SIGINT, signal_handler)
signal.pause()
