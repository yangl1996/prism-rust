import subprocess, time, webbrowser, signal, os, json, re


def signal_handler(sig, frame):
    print('Killing all processes')
    if not MOCK:
        p1.kill()
        p3.kill()
    if DEMO_LOCATION=='apr-server':
        p = subprocess.run(['./run.sh', 'stop-instances'], input='1'.encode('utf-8'))
        print('Stopped prism')
    p2.kill()
    for pid in os.popen('pgrep prism'):
        os.kill(int(pid), signal.SIGKILL)

with open('config.json', 'r') as f:
    config = json.loads(f.read())
    URL = config['host']
    VIS_PORT = config['visualization port']
    NUM_NODES = config['num_nodes']
    DEMO_LOCATION = config['demo_location']
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
    if DEMO_LOCATION=='local':
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
if not MOCK and DEMO_LOCATION=='local':
    os.chdir('../../testbed')
    p3 = subprocess.Popen(['./local-experiment.sh', f'{NUM_NODES}'])

if not MOCK and DEMO_LOCATION=='apr-server':
    os.chdir('../../testbed')
    TOPO = config['topology']
    if TOPO=='randreg':
        topo_p = subprocess.run(['python3', 'scripts/generate_topo.py', f'{NUM_NODES}', f'{TOPO}', str(config['degree']), '>', 'randreg.json']) 
    else:
        topo_p = subprocess.run(['python3', 'scripts/generate_topo.py', f'{NUM_NODES}', f'{TOPO}', '>', 'clique.json']) 
    p4 = subprocess.run(['./run.sh', 'start-instances', f'{NUM_NODES}'], input='1'.encode('utf-8'))
    p5 = subprocess.run(['./run.sh', 'mount-nvme'])
    p6 = subprocess.run(['./run.sh', 'gen-payload', f'{TOPO}.json'])
    p7 = subprocess.run(['./run.sh', 'sync-payload'])
    p3 = subprocess.Popen(['./run.sh', 'show-demo'])

signal.signal(signal.SIGINT, signal_handler)
signal.pause()
