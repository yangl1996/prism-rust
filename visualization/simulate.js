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
let nodeIndex = 0
let transactionBlockId = 0
let proposerBlockId = 0 
let votingBlockId = 0
if(mock){
  if(protocol==='prism'){
    const forkProbability = 0.05

    // Add 1 transaction block every 0.2 seconds
    d3.interval(() => {
      if(transactionBlocks.length>500) return
      if(nodeIndex<1) return
      const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeIndex))
      addTransactionBlock(transactionBlockId, sourceNodeId)
      transactionBlockId++
    }, t/5)

    // Add 1 proposer block every 5 seconds
    d3.interval(() => {
      let parent = null
      if(proposerBlocks.length!==0) parent = proposerBlocks[proposerBlocks.length-1] 
      if(proposerBlocks.length>1){
        if(Math.random()<forkProbability){
          parent = proposerBlocks[proposerBlocks.length-2]
        }
        else{
          parent = proposerBlocks[proposerBlocks.length-1]
        }
      }
      const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeIndex))
      let transactionBlockIds = transactionBlocks.map(block => block.blockId).filter(() => Math.random()<0.9)
      addProposerBlock(proposerBlockId, parent, sourceNodeId, transactionBlockIds)
      proposerBlockId++
    }, 5*t)

    // Each voting chain grows at the same rate as the proposer chain
    d3.interval(() => {
      if(nodeIndex<1) return
      const randomChain = Math.floor(Math.random() * Math.floor(numChains))
      const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeIndex))
      const parentId = chainsData[randomChain].blocks[chainsData[randomChain].blocks.length-1].blockId
      mineVotingBlock(randomChain, votingBlockId, sourceNodeId, parentId)
      votingBlockId++
    }, 5*t/numChains)
  }
  if(protocol==='longest-chain'){
    let blocks = []
    const Node = d3.hierarchy.prototype.constructor
    const root = new Node
    root.id = '0'
    root.parent = null
    root.depth = 0
    layoutTree(root)
    longestChainBlocks.push(root)
    drawLongestChain()
    d3.csv('blocksClean.csv').then(data => {
      blocks = data
    })
    let index = 1
    let ugly = false
    let interval = d3.interval(() => {
      if(reset){
        d3.csv('blocksUgly.csv').then(data => {
          blocks = data
        })
        index = 1
        longestChainBlocksGroup = longestChainScreen.append('g').attr('id', 'longestChainBlocksMessy')
        longestChainLinksGroup = longestChainScreen.append('g').attr('id', 'longestChainLinksMessy')
        longestChainBlocks = []
        links = []
        root.finalized = false
        layoutTree(root)
        longestChainBlocks.push(root)
        drawLongestChain()
        ugly = true
        reset = false
      }
      else{
        const newBlock = new Node
        newBlock.id = blocks[index]['id']
        newBlock.parent = longestChainBlocks.find(i => i.id===blocks[index]['parentId'])
        newBlock.depth = newBlock.parent.depth+1
        if(newBlock.parent.children) newBlock.parent.children.push(newBlock)
        else newBlock.parent.children = [newBlock]

        const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeIndex))

        newBlock.sourceNodeId = sourceNodeId
        const prevRootY = root.y
        layoutTree(root)
        root.y = prevRootY ? prevRootY : root.y
        if(ugly){
          newBlock.xShift = d3.randomUniform(-20, 20)()
          newBlock.yShift = d3.randomUniform(-10, 0)()
        }
        else{
          newBlock.xShift = 0
          newBlock.yShift = 0
        }
        longestChainBlocks.push(newBlock)
        for(let i=0; i<longestChainBlocks.length; i++){
            if(longestChainBlocks[i].id!=='0'){
              longestChainBlocks[i].y = longestChainBlocks[i].parent.y+2*longestChainBlockSize+longestChainBlocks[i].yShift
              longestChainBlocks[i].x += longestChainBlocks[i].xShift
            }
        }
        links.push({source: newBlock,
                    target: newBlock.parent, id: `${newBlock.id}-${newBlock.parent.id}`})
        pingNode(sourceNodeId)
        drawLongestChain()
        index++
        if(index>blocks.length) interval.stop()
      }
    }, 4*t)
  }
}

let simulateAttack = () => {
    const parent = proposerBlocks[proposerBlocks.length-2]
    const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeIndex))
    let transactionBlockIds = transactionBlocks.map(block => block.blockId).filter(() => Math.random()<0.9)
    addMaliciousBlock(proposerBlockId, parent, sourceNodeId, transactionBlockIds)
    proposerBlockId++
}