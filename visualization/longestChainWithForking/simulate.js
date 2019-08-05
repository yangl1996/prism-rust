let cities = [
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
let t = 1000
let blocks = []
const Node = d3.hierarchy.prototype.constructor
const root = new Node
root.id = '0'
root.parent = null
root.depth = 0
root.sourceNodeId = null
layoutTree(root)
longestChainBlocks.push(root)
drawLongestChain()
d3.csv('low_forking.csv').then(data => {
  blocks = data
})
let index = 1
let mineLowRate = d3.interval(() => {
  const newBlock = new Node
  newBlock.id = blocks[index]['id']
  newBlock.parent = longestChainBlocks.find(i => i.id===blocks[index]['parentId'])
  newBlock.depth = newBlock.parent.depth+1
  newBlock.finalized = false
  if(newBlock.parent.children) newBlock.parent.children.push(newBlock)
  else newBlock.parent.children = [newBlock]

  const sourceNodeId = Math.floor(Math.random() * Math.floor(nodes.length))

  newBlock.sourceNodeId = sourceNodeId
  const prevRootY = root.y
  layoutTree(root)
  root.y = prevRootY ? prevRootY : root.y
  longestChainBlocks.push(newBlock)
  for(let i=0; i<longestChainBlocks.length; i++)
      if(longestChainBlocks[i].id!=='0')
        longestChainBlocks[i].y = longestChainBlocks[i].parent.y+2*longestChainBlockSize
  links.push({source: newBlock,
              target: newBlock.parent, id: `${newBlock.id}-${newBlock.parent.id}`})
  pingNode(sourceNodeId)
  drawLongestChain()
  index++
  if(index>blocks.length) mineLowRate.stop()
}, 4*t)

let modifyProtocol = () => {
    t = 500
    d3.csv('high_forking.csv').then(data => {
      blocks = data
    })
    index = 1
    longestChainBlocksGroup = longestChainScreen.append('g').attr('id', 'longestChainBlocksMessy')
                                                            .attr('transform', 'translate(100, 0)')
    longestChainLinksGroup = longestChainScreen.append('g').attr('id', 'longestChainLinksMessy')
                                                            .attr('transform', 'translate(100, 0)')
    longestChainBlocks = []
    links = []
    root.finalized = false
    layoutTree(root)
    longestChainBlocks.push(root)
    drawLongestChain()
    let mineFastRate = d3.interval(() => {
      const p = Math.random()
      let blocksToMine = 1
      if(p<0.2)
        blocksToMine = 3
      else if (p<0.5)
        blocksToMine = 2
      for(let x=0; x<blocksToMine; x++){
        const newBlock = new Node
        newBlock.id = blocks[index]['id']
        newBlock.parent = longestChainBlocks.find(i => i.id===blocks[index]['parentId'])
        newBlock.depth = newBlock.parent.depth+1
        if(newBlock.parent.children) newBlock.parent.children.push(newBlock)
        else newBlock.parent.children = [newBlock]

        const sourceNodeId = Math.floor(Math.random() * Math.floor(nodes.length))
        pingNode(sourceNodeId)

        newBlock.sourceNodeId = sourceNodeId
        const prevRootY = root.y
        layoutTree(root)
        root.y = prevRootY ? prevRootY : root.y
        newBlock.xShift = d3.randomUniform(-20, 20)()
        newBlock.yShift = d3.randomUniform(-10, 0)()
        longestChainBlocks.push(newBlock)
        for(let i=0; i<longestChainBlocks.length; i++){
            if(longestChainBlocks[i].id!=='0'){
              longestChainBlocks[i].y = longestChainBlocks[i].parent.y+2*longestChainBlockSize+longestChainBlocks[i].yShift
              longestChainBlocks[i].x += longestChainBlocks[i].xShift
            }
        }
        links.push({source: newBlock,
                    target: newBlock.parent, id: `${newBlock.id}-${newBlock.parent.id}`})
        index++
        if(index>blocks.length) mineFastRate.stop()
      }
      drawLongestChain()
    }, 4*t/4)

}
