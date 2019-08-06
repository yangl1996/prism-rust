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
root.finalizationLevel = 0.4
layoutTree(root)
longestChainBlocks.push(root)
drawLongestChain()
d3.csv('low_forking.csv').then(data => {
  blocks = data
})

// Mine slowly on main chain
let index = 1
let mineLowRate = d3.interval(() => {
  const newBlock = new Node
  newBlock.id = blocks[index]['id']
  newBlock.parent = longestChainBlocks.find(i => i.id===blocks[index]['parentId'])
  newBlock.depth = newBlock.parent.depth+1
  newBlock.finalizationLevel = 0.4
  newBlock.finalized = false
  if(newBlock.parent.children) newBlock.parent.children.push(newBlock)
  else newBlock.parent.children = [newBlock]

  const sourceNodeId = Math.floor(Math.random() * Math.floor(nodes.length))

  newBlock.sourceNodeId = sourceNodeId
  let transactionBlockIds = transactionBlocks.map(block => block.blockId).filter(() => Math.random()<0.9)
  newBlock.transactionBlockIds = transactionBlockIds
  const prevRootY = root.y
  layoutTree(root)
  root.y = prevRootY ? prevRootY : root.y
  longestChainBlocks.push(newBlock)

  for(let i=0; i<longestChainBlocks.length; i++)
      if(longestChainBlocks[i].id!=='0')
        longestChainBlocks[i].y = longestChainBlocks[i].parent.y+2*longestChainBlockSize
  if(longestChainVotes)
  links.push({source: newBlock,
              target: newBlock.parent, id: `${newBlock.id}-${newBlock.parent.id}`})
  pingNode(sourceNodeId)
  drawLongestChain()
  index++
  if(index>blocks.length) mineLowRate.stop()
}, 4*t)

let mineVotingChains = null

// Add voting chains
let addVotingChains = () => {
  // Initialize the chains spaced by votingChainScreenWidth/numChains
  longestChainVotes = false
  let chain = 0, x=0
  let scale = d3.scaleLinear().domain([0, numChainsToDisplay]).range([1.0, 0.0])
  let votingBlockId = 0
  const lastVotedBlock = parseInt(longestChainBlocks[longestChainBlocks.length-1]['id'])
  while(chain<numChainsToDisplay){
    chainsData.push({x, y: 0, blocks: [], links: [], lastVotedBlock, drawn: false})
    const genesisBlock = {parent: null, blockId: votingBlockId, children: [], sourceNodeLocation: null}
    chainsData[chain].blocks.push(genesisBlock)
    votingBlockId++
    let chainGroup = chainsGroup.append('g')
                                .attr('id', 'chain'+chain)
                                .style('opacity', scale(chain))
    let linkGroup = chainsGroup.append('g')
                               .attr('id', 'links'+chain)
                                .style('opacity', scale(chain))
    chain++
    x+=votingChainScreenWidth/(numChainsToDisplay+1)
  }

  while(chain<numChains){
    chainsData.push({blocks: [], lastVotedBlock: 0, fakeBlocks: [], fakeLinks: []})
    const genesisBlock = {parent: null, blockId: votingBlockId, children: [], sourceNodeLocation: null}
    chainsData[chain].blocks.push(genesisBlock)
    votingBlockId+=1
    chain++
  }
  chain = 0
  let interval = d3.interval(() => { 
    chainsData[chain].drawn = true
    drawVotingChain(chain) 
    chain++ 
    if(chain==numChainsToDisplay) interval.stop()
  }, t)

  // Mine on voting chains
  mineVotingChains = d3.interval(() => {
    const randomChain = Math.floor(Math.random() * Math.floor(numChains))
    if(!chainsData[randomChain].drawn) return
    const sourceNodeId = Math.floor(Math.random() * Math.floor(nodes.length))
    const parentId = chainsData[randomChain].blocks[chainsData[randomChain].blocks.length-1].blockId
    mineVotingBlock(randomChain, votingBlockId, sourceNodeId, parentId)
    votingBlockId++
  }, 4*t/numChains)
}

let mineTransactionBlocks = null

// Add transaction blocks
let addTransactionBlocks = () => {
  showTransactionPool = true
  let transactionBlockId = 0
  // Add 1 transaction block every 0.2 seconds
  mineTransactionBlocks = d3.interval(() => {
    if(transactionBlocks.length>500) return
    const sourceNodeId = Math.floor(Math.random() * Math.floor(nodes.length))
    addTransactionBlock(transactionBlockId, sourceNodeId)
    transactionBlockId++
  }, t/5)
}
let endSimulation = () => {
  d3.selectAll('svg').remove()
  mineLowRate.stop()
  mineVotingChains.stop()
  mineTransactionBlocks.stop()
}
