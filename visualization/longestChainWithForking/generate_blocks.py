import random
import numpy as np

timestamp = 0
duration = 100
f = 10
delay_parameter = 1
num_nodes = 10
filename = 'high_forking'

def network_delay():
    return delay_parameter
    #return np.random.exponential(delay_parameter)

timestamps = []

# generate proposal events
while timestamp<duration:
    timestamp = timestamp + np.random.exponential(1/f)
    timestamps.append(timestamp)

nodes = []

genesis = {'id': 0, 'parent': None, 'depth': 0, 'miner': None, 'timestamp': 0}
block_id = 1

# create nodes
for i in range(0, num_nodes):
    nodes.append({'id': i, 'blocks': [genesis]}) 

# set which node is 'me'
me = nodes[0]
for t in timestamps:
    # choose a random node
    n = random.choice(nodes)
    # look at all blocks the node has received prior to the event
    # find maximum depth to adhere to longest chain protocol
    max_depth = max([b['depth'] for b in n['blocks'] if b['timestamp']<t])
    # find all blocks at that depth and add to random one
    valid_blocks = [b for b in n['blocks'] if b['depth']==max_depth and b['timestamp']<t]
    parent = random.choice(valid_blocks)

    # create new block with timestamp t
    new_block = {'id': block_id, 'parent': parent['id'], 'depth':
            parent['depth']+1, 'timestamp': t, 'miner': n['id']}
    delayed_block = new_block.copy()
    # delayed block is received with an additional network delay
    delayed_block['timestamp']+=network_delay()
    n['blocks'].append(delayed_block)

    # broadcast to all other nodes
    for j in range(0, num_nodes): 
        if n['id']!=j:
            delayed_block = new_block.copy()
            # delayed block is received with an additional network delay
            delayed_block['timestamp']+=network_delay()
            nodes[j]['blocks'].append(delayed_block)
    block_id+=1

print('\n\n')
# To see the amount of forking, print all blocks at a certain depth
max_depth = max([b['depth'] for b in me['blocks']])
for i in range(0, max_depth):
    s = ''
    for b in me['blocks']:
        if b['depth']==i:
            s+=str(b['id']) + ','
    print(s)


sorted_blocks = sorted(me['blocks'], key=lambda x: x['timestamp'])
with open(filename+'.csv', 'w+') as f:
    f.write('id,parentId\n')
    for b in sorted_blocks:
        if b['parent']!=None:
            f.write(f'{b["id"]},{b["parent"]}')
        else:
            f.write(f'{b["id"]},')
        f.write('\n')
