let width = 1080,
    height = 600
let longestChainVotes = true
let showTransactionPool = false

let svg = d3.select('body').append('svg')
  .attr('id', 'untransformedSvg')
	.attr('width', width)
	.attr('height', height)
  .style('position', 'absolute')


const transTime = 5000
let worldMapFocused = false
const tStep = 10
const aScale = d3.scaleLinear().domain([0, transTime]).range([1.0, 0.6])
const bScale = d3.scaleLinear().domain([0, transTime]).range([0.0, -0.002])
const cScale = d3.scaleLinear().domain([0, transTime]).range([0, 250])
const xTranslateScale = d3.scaleLinear().domain([0, transTime]).range([-400, -280])
const yTranslateScale = d3.scaleLinear().domain([0, transTime]).range([-200, 0])
const scaleScale = d3.scaleLinear().domain([0, transTime]).range([2, 1])
const worldMapShift = -280



let M = `matrix3d(1.0, 0, 0, 0, 0, ${aScale(transTime)}, 0, ${bScale(transTime)}, 0, 0, 1, 0, 0, ${cScale(transTime)}, 0, 1)`

let svgTransform = d3.select('body').append('svg')
    .style('position', 'absolute')
    .attr('width', width)
    .attr('height', height)
    .attr('id', 'svgTransform')
    .style('transform', M)

// World Map Screen sizes
let worldMapScreenWidth = 0.7*width, worldMapScreenHeight = 0.6*height
let worldMapScreen = svgTransform.append('g')
              .attr('id', 'worldMap')
              .attr('transform', `translate(-280, 0)scale(2)`)

worldMapScreen.attr('transform', `translate(${xTranslateScale(transTime)}, ${yTranslateScale(transTime)})scale(${scaleScale(transTime)})`)

const nodeRadius = 3
let nodes = []


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
            .on('click', () => addVotingChains())
let longestChainBlocksGroup = longestChainScreen.append('g').attr('id', 'longestChainBlocksClean')
let longestChainLinksGroup = longestChainScreen.append('g').attr('id', 'longestChainLinksClean')

let ledgerGroup = svg.append('g')
                     .attr('id', 'ledger')

// Voting Chain Screen sizes
let votingChainScreenWidth = width*0.4, votingChainScreenHeight = height
let votingChainScreen = svg.append('g')
              .attr('id', 'votingChains')
              .attr('width', votingChainScreenWidth)
              .attr('height', votingChainScreenHeight)
              .attr('transform', `translate(${width*0.6}, ${longestChainBlockSize})`)
            .on('click', () => addTransactionBlocks())
const numChains = 100
const numChainsToDisplay = 10
const votingBlockSize = 20
let chainsData = []

// Transaction Screen sizes
let transactionScreenWidth = width/3, transactionScreenHeight = height
let transactionScreen = svg.append('g')
            .attr('id', 'transactionPool')
            .attr('width', transactionScreenWidth)
            .attr('height', transactionScreenHeight)
const transactionBlockSize = 20
const ledgerBlockSize = 20
let transactionBlocks = []

let transactionGroup = transactionScreen.append('g').attr('id', 'transactionGroup').style('opacity', 'inherit')
let transactionBlock = transactionGroup.selectAll('.transactionBlock')

let voteGroup = svg.append('g').attr('id', 'votes')

let longestChainBlocks = []
let links = []
