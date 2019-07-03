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

/*
// Add 1 transaction block every 2 seconds
d3.interval(() => {
  if(transactionBlocks.length>500) return
  if(nodeId<1) return
  const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeId))
  addTransactionBlock(transactionBlockId, sourceNodeId)
  transactionBlockId++
}, t/10)

// Add 1 proposer block every 10 seconds
d3.interval(() => {
  let parent = null
  if(proposerBlock.length!==0) parent = proposerBlocks[proposerBlocks.length-1] 
  if(proposerBlocks.length>1){
    if(Math.random()<0.05)
      parent = proposerBlocks[proposerBlocks.length-2]
    else
      parent = proposerBlocks[proposerBlocks.length-1]
  }
  const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeId))
  let transactionBlockIds = transactionBlocks.map(block => block.blockId).filter(() => Math.random()<0.9)
  addProposerBlock(proposerBlockId, parent, sourceNodeId, transactionBlockIds)
  proposerBlockId++
}, 10*t)

// Mine 1 voting block every second
d3.interval(() => {
  if(nodeId<1) return
  const randomChain = Math.floor(Math.random() * Math.floor(numChains))
  const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeId))
  const parentId = chainsData[randomChain].blocks[chainsData[randomChain].blocks.length-1].blockId
  mineVotingBlock(randomChain, votingBlockId, sourceNodeId, parentId)
  votingBlockId++
}, t/numChains * numChainsToDisplay)
*/
