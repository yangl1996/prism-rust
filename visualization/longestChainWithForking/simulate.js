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
const Node = d3.hierarchy.prototype.constructor
const root = new Node
root.id = 0
root.parent = null
root.depth = 0
root.sourceNodeId = null
layoutTree(root)
longestChainBlocks.push(root)
drawLongestChain()
let index = 1
let mineLowRate = d3.interval(() => {
  const newBlock = new Node
  newBlock.id = index 
  newBlock.parent = longestChainBlocks[longestChainBlocks.length-1]
  newBlock.depth = newBlock.parent.depth+1
  newBlock.finalized = false
  if(newBlock.parent.children) newBlock.parent.children.push(newBlock)
  else newBlock.parent.children = [newBlock]
  newBlock.yShift=0
  newBlock.xShift=0

  const sourceNodeId = Math.floor(Math.random() * Math.floor(nodes.length))

  newBlock.sourceNodeId = sourceNodeId
  const prevRootY = root.y
  const oldXVals = longestChainBlocks.map(b => b.x)
  layoutTree(root)
  root.y = prevRootY ? prevRootY : root.y
  longestChainBlocks.push(newBlock)
  for(let i=0; i<longestChainBlocks.length; i++)
      if(longestChainBlocks[i].id!==0){
        longestChainBlocks[i].y = longestChainBlocks[i].parent.y+2*longestChainBlockSize
        if(i<oldXVals.length)
          longestChainBlocks[i].x = oldXVals[i]
      }
  links.push({source: newBlock,
              target: newBlock.parent, id: `${newBlock.id}-${newBlock.parent.id}`})
  pingNode(sourceNodeId)
  drawLongestChain()
  index++
}, 4*t)

let modifyProtocol = () => {
    let addBlocks = blocksToMine => {
      const lastBlockLevel = longestChainBlocks[longestChainBlocks.length-1].depth
      for(let x=0; x<blocksToMine; x++){
        const newBlock = new Node
        newBlock.id = index
        const possibleParents = longestChainBlocks.filter(b => b.depth==lastBlockLevel)
        newBlock.parent = possibleParents[Math.floor(Math.random()*possibleParents.length)]
        newBlock.depth = newBlock.parent.depth+1
        if(newBlock.parent.children) newBlock.parent.children.push(newBlock)
        else newBlock.parent.children = [newBlock]

        const sourceNodeId = Math.floor(Math.random() * Math.floor(nodes.length))
        pingNode(sourceNodeId)

        newBlock.sourceNodeId = sourceNodeId
        const prevRootY = root.y
        const oldXVals = longestChainBlocks.map(b => b.x)
        layoutTree(root)
        root.y = prevRootY ? prevRootY : root.y
        newBlock.xShift = d3.randomUniform(-10, 10)()
        newBlock.yShift = d3.randomUniform(-10, 0)()
        longestChainBlocks.push(newBlock)
        for(let i=0; i<longestChainBlocks.length; i++){
            if(longestChainBlocks[i].id!==0){
              longestChainBlocks[i].y = longestChainBlocks[i].parent.y+2*longestChainBlockSize+longestChainBlocks[i].yShift
            }
            if(i<oldXVals.length){
              longestChainBlocks[i].x = oldXVals[i]
            }
        }
        links.push({source: newBlock,
                    target: newBlock.parent, id: `${newBlock.id}-${newBlock.parent.id}`})
        index++
      }
      drawLongestChain()
    }

    mineLowRate.stop()
    addBlocks(2)
    let i = d3.interval(() => {
      const p = Math.random()
      if(p<0.5)
        addBlocks(1)
      else
        addBlocks(2)
    }, 4*t/1.5)

    setTimeout(() => {
      i.stop()
      addBlocks(2)
      i = d3.interval(() => {
        const p = Math.random()
        if(p<0.2)
          addBlocks(2)
        else
          addBlocks(3)
      }, 4*t/2)
    }, 5000)

    setTimeout(() => {
      i.stop()
      addBlocks(3)
      i = d3.interval(() => {
        const p = Math.random()
        if(p<0.2)
          addBlocks(2)
        else if(p<0.5)
          addBlocks(3)
        else
          addBlocks(4)
      }, 4*t/2.5)
    }, 10000)

    setTimeout(() => {
      i.stop()
      i = d3.interval(() => {
        const p = Math.random()
        if(p<0.1)
          addBlocks(2)
        else if(p<0.3)
          addBlocks(3)
        else
          addBlocks(4)
      }, 4*t/3)
    }, 12000)

    setTimeout(() => {
      i.stop()
      i = d3.interval(() => {
        const p = Math.random()
        if(p<0.1)
          addBlocks(2)
        else if(p<0.3)
          addBlocks(3)
        else if(p<0.7)
          addBlocks(4)
        else
          addBlocks(5)
      }, 4*t/3.5)
    }, 15000)

    setTimeout(() => {
      i.stop()
      i = d3.interval(() => {
        const p = Math.random()
        if(p<0.1)
          addBlocks(3)
        else if(p<0.3)
          addBlocks(4)
        else if(p<0.8)
          addBlocks(5)
        else
          addBlocks(6)
      }, 4*t/4)
    }, 20000)

}
