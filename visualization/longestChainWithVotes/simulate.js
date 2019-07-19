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
  // Initialize the chains spaced by votingChainScreenWidth/numChains
  let chain = 0, x=0
  let scale = d3.scaleLinear().domain([0, numChainsToDisplay]).range([1.0, 0.0])
  let votingBlockId = 0
  while(chain<numChainsToDisplay){
    chainsData.push({x, y: 0, blocks: [], links: [], lastVotedBlock: 0, fakeBlocks: [], fakeLinks: [], drawn: false})
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
  d3.interval(() => {
    const randomChain = Math.floor(Math.random() * Math.floor(numChains))
    if(!chainsData[randomChain].drawn) return
    const sourceNodeId = Math.floor(Math.random() * Math.floor(nodes.length))
    const parentId = chainsData[randomChain].blocks[chainsData[randomChain].blocks.length-1].blockId
    mineVotingBlock(randomChain, votingBlockId, sourceNodeId, parentId)
    votingBlockId++
  }, 4*t/numChains)
}
