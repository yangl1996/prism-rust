import random, csv, re

blockId = 0
blocks = [{'id': blockId, 'children': [], 'parentId': None, 'depth': 0}]
blockId+=1

numBlocks = 100
forkProbability = 0.6

if forkProbability>0:
    xString = 'newBlock.xShift = d3.randomUniform(-20, 20)()'
    yString = 'newBlock.yShift = d3.randomUniform(-10, 0)()'
else:
    xString = 'newBlock.xShift = d3.randomUniform(0, 0)()'
    yString = 'newBlock.yShift = d3.randomUniform(0, 0)()'

with open('simulate.js', 'r') as f:
    contents = f.read()
    fixed_contents = re.sub(r"newBlock.xShift = d3.randomUniform\([-]?\d*, [-]?\d*\)\(\)", xString, contents)
    fixed_contents = re.sub(r"newBlock.yShift = d3.randomUniform\([-]?\d*, [-]?\d*\)\(\)", yString, fixed_contents)
with open('simulate.js', 'w+') as f:
    f.write(fixed_contents)


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
    

with open('blocks.csv', 'w+') as f:
    f.write('id,parentId\n')
    for b in blocks:
        if b['parentId']==None:
            f.write(f'{b["id"]},\n')
        else:
            f.write(f'{b["id"]},{b["parentId"]}\n')
