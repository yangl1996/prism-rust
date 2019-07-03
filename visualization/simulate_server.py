import asyncio
import datetime
import websockets
import random

async def simulate(websocket, path):
    cities = [
      [-6.13,39.31],
      [35.68,139.76],
      [-36.85,174.78],
      [13.75,100.48],
      [29.01,77.38],
      [1.36,103.75],
      [-15.67,-47.43],
      [-22.90,-43.24],
      [43.64,-79.40],
      [-27.11,-109.36],
      [47.61,-122.33]
    ]
    nodeId = 0
    addNode_t = None
    addNode_dt = datetime.timedelta(seconds=1)
    addTx_t = None
    addTx_dt = datetime.timedelta(seconds=0.25)
    transactionBlockId = 0
    addPb_t = None
    addPb_dt = datetime.timedelta(seconds=10)
    addVb_t = None
    addVb_dt = datetime.timedelta(seconds=1)
    while True:
        # Add node
        if addNode_t is None and nodeId<len(cities): 
            latitude = cities[nodeId][0]
            longitude = cities[nodeId][1]
            shardColor = '%06x' % random.randint(0, 0xFFFFFF)
            await websocket.send(f'ADDNODE,{latitude},{longitude},{shardColor},{nodeId}')
            addNode_t = datetime.datetime.now()
            nodeId+=1
        elif datetime.datetime.now()>addNode_t + addNode_dt and nodeId<len(cities):
            latitude = cities[nodeId][0]
            longitude = cities[nodeId][1]
            shardColor = '%06x' % random.randint(0, 0xFFFFFF)
            await websocket.send(f'ADDNODE,{latitude},{longitude},{shardColor},{nodeId}')
            addNode_t = datetime.datetime.now()
            nodeId+=1
        # Add transaction block
        if addTx_t is None: 
            await websocket.send(f'ADDTXBLOCK,{transactionBlockId}')
            addTx_t = datetime.datetime.now()
            transactionBlockId+=1
        elif datetime.datetime.now()>addTx_t + addTx_dt:
            await websocket.send(f'ADDTXBLOCK,{transactionBlockId}')
            addTx_t = datetime.datetime.now()
            transactionBlockId+=1

        # Add proposer block
        if addPb_t is None: 
            await websocket.send('addPb')
            addPb_t = datetime.datetime.now()
        elif datetime.datetime.now()>addPb_t+ addPb_dt:
            await websocket.send('addPb')
            addPb_t = datetime.datetime.now()

        # Mine voting block
        if addVb_t is None: 
            await websocket.send('addVb')
            addVb_t = datetime.datetime.now()
        elif datetime.datetime.now()>addVb_t+ addVb_dt:
            await websocket.send('addVb')
            addVb_t = datetime.datetime.now()


start_server = websockets.serve(simulate, '127.0.0.1', 6789)

asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()
