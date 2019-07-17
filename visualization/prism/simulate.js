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
}
