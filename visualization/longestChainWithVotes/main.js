let body = document.getElementsByTagName('body')[0]

let width = body.clientWidth,
    height = body.clientHeight
let longestChainVotes = true
let showTransactionPool = false
let svg = d3.select('body').append('svg')
  .attr('id', 'untransformedSvg')
	.attr('width', width)
	.attr('height', height)
  .style('position', 'absolute')

svg.append('rect')
    .attr('width', width)
    .attr('height', height)
    .style('fill', 'url(#background-gradient)')

const worldMapShift = -100

let M = `matrix3d(1.0, 0, 0, 0, 0, 0.6, 0, -0.002, 0, 0, 1, 0, 0, 250, 0, 1)`

let svgTransform = d3.select('body').append('svg')
    .style('position', 'absolute')
    .attr('width', width)
    .attr('height', height)
    .attr('id', 'svgTransform')
    .style('transform', M)

// World Map Screen
let worldMapScreenWidth = 0.7*width, worldMapScreenHeight = 0.6*height
let worldMapScreen = svgTransform.append('g')
              .attr('id', 'worldMap')

worldMapScreen.attr('transform', `translate(-400, 0)scale(1.5)`)

const nodeRadius = 3
let nodes = []
let globalNodesData = []

// Longest Chain Screen
const treeSize = width/3
const renderLink = d3.linkVertical().x(d => d.x+(1.25-1)/2*longestChainBlockSize).y(d => d.y)
const longestChainBlockSize = 20
const finalizationThreshold = 0.46
let layoutTree = d3.tree().size([treeSize, height-0.4*height])

let longestChainScreenWidth = treeSize, longestChainScreenHeight = height
let longestChainScreen = svg.append('g')
            .attr('id', 'longestChain')
            .attr('width', longestChainScreenWidth)
            .attr('height', longestChainScreenHeight)
            .attr('transform', `translate(${width/3}, ${longestChainBlockSize})`)
let longestChainBlocksGroup = longestChainScreen.append('g').attr('id', 'longestChainBlocksClean')
let longestChainLinksGroup = longestChainScreen.append('g').attr('id', 'longestChainLinksClean')


// Voting Chain Screen
let votingChainScreenWidth = width*0.4, votingChainScreenHeight = height
let votingChainScreen = svg.append('g')
              .attr('id', 'votingChains')
              .attr('width', votingChainScreenWidth)
              .attr('height', votingChainScreenHeight)
              .attr('transform', `translate(${width*0.6}, ${longestChainBlockSize})`)
const numChains = 100
const numChainsToDisplay = 10
const votingBlockSize = 20
let chainsData = []

// Transaction Screen
let transactionScreenWidth = width/3, transactionScreenHeight = height
let transactionScreen = svg.append('g')
            .attr('id', 'transactionPool')
            .attr('width', transactionScreenWidth)
            .attr('height', transactionScreenHeight)
const transactionBlockSize = 20
const ledgerBlockSize = 20
let transactionBlocks = []

let transactionGroup = transactionScreen.append('g').attr('id', 'transactionGroup').style('opacity', 'inherit')
let transactionBlock = transactionGroup.selectAll('g').data(transactionBlocks, d => d.blockId)

// Votes Group
let voteGroup = svg.append('g').attr('id', 'votes')

// Ledger Group
let ledgerGroup = svg.append('g').attr('id', 'ledger')

let longestChainBlocks = []
let links = []
