import random, csv, re

blockId = 0
blocks = [{'id': blockId, 'children': [], 'parentId': None, 'depth': 0}]
blockId+=1

numBlocks = 100
forkProbability = 0.6

while blockId<numBlocks:
    newBlock = {'id': blockId, 'children': []}
    fork = True if random.random()<forkProbability else False
    maxDepth = -1
    parentBlock = None
    for b in blocks:
        if b['depth']>maxDepth: 
            parentBlock = b
            maxDepth = b['depth']
    if fork and maxDepth>1:
        depths = list(range(1, maxDepth))
        selectedDepth = random.choices(depths, weights=depths, k=1)[0]
        for b in blocks:
            if b['depth']==selectedDepth:
                parentBlock = b
                break
    newBlock['parentId'] = parentBlock['id']
    newBlock['depth'] = parentBlock['depth']+1
    parentBlock['children'].append(newBlock['id'])
    blocks.append(newBlock)
    blockId+=1
    
if forkProbability>0:
    with open('blocksUgly.csv', 'w+') as f:
        f.write('id,parentId\n')
        for b in blocks:
            if b['parentId']==None:
                f.write(f'{b["id"]},\n')
            else:
                f.write(f'{b["id"]},{b["parentId"]}\n')
else:
    with open('blocksClean.csv', 'w+') as f:
        f.write('id,parentId\n')
        for b in blocks:
            if b['parentId']==None:
                f.write(f'{b["id"]},\n')
            else:
                f.write(f'{b["id"]},{b["parentId"]}\n')

