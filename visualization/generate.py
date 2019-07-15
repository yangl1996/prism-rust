import random, csv

blockId = 0
blocks = [{'id': blockId, 'children': [], 'parentId': None, 'depth': 0}]
blockId+=1

numBlocks = 100
forkProbability = 0.9

while blockId<numBlocks:
    newBlock = {'id': blockId, 'children': []}
    if random.random()<forkProbability:
        parentBlock = random.choice(blocks)
    else:
        maxDepth = -1
        parentBlock = None
        for b in blocks:
            if b['depth']>maxDepth: 
                parentBlock = b
                maxDepth = b['depth']
    newBlock['parentId'] = parentBlock['id']
    newBlock['depth'] = parentBlock['depth']+1
    parentBlock['children'].append(newBlock['id'])
    blocks.append(newBlock)
    blockId+=1
    

with open('blocks.csv', 'w+') as f:
    f.write('id,parentId\n')
    for b in blocks:
        if b['parentId']==None:
            f.write(f'{b["id"]},\n')
        else:
            f.write(f'{b["id"]},{b["parentId"]}\n')
