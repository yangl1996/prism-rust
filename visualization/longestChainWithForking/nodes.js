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

let me = 0
let blockId = 0
let NUM_NODES = 10
let f = 0.1
let nodes = []
const networkDelay = 3
const Node = d3.hierarchy.prototype.constructor
const genesis = new Node
genesis.id = blockId
blockId++
genesis.parent = null
genesis.depth = 0
for(let i=0; i<NUM_NODES; i++)
  nodes.push({'nodeId': i, blocks: [genesis], latitude: cities[i][0], longitude: cities[i][1]})

let broadcast = (receiver, newBlock) => {
  d3.timeout(() => {
    nodes[receiver].blocks.push(newBlock)
    clearTimeout(nodes[receiver].timeout)
    mine(receiver)
    if(receiver==me){
      addToTree(newBlock)
    }
  }, networkDelay*1000)
}

let mine = (miner) => {
  const timeoutFn = setTimeout(() => {
    const newBlock = new Node
    newBlock.id = blockId
    blockId++
    const parent = nodes[miner].blocks.reduce((prev, current) => (prev.depth > current.depth) ? prev : current)
    newBlock.parent = parent
    newBlock.depth = parent.depth+1
    if(parent.children) parent.children.push(newBlock)
    else parent.children = [newBlock]
    newBlock.sourceNodeId = miner
    for(let j=0; j<NUM_NODES; j++){
      if(miner!==j)
        broadcast(j, newBlock)
    }
    if(miner==me) {
      addToTree(newBlock)
    }
  }, d3.randomExponential(f)()*1000)
  nodes[miner].timeout = timeoutFn
}
for(let i=0; i<NUM_NODES; i++){
  mine(i)
 }
