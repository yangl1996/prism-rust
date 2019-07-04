let width = 1080,
    height = 600

let svg = d3.select('body').append('svg')
	.attr('width', width)
	.attr('height', height)
  .style('position', 'absolute')

const transTime = 5000
const tStep = 10
const aScale = d3.scaleLinear().domain([0, transTime]).range([1.0, 0.6])
const bScale = d3.scaleLinear().domain([0, transTime]).range([0.0, -0.002])
const cScale = d3.scaleLinear().domain([0, transTime]).range([0, 250])

let M = `matrix3d(1.0, 0, 0, 0, 0, ${aScale(5000)}, 0, ${bScale(5000)}, 0, 0, 1, 0, 0, ${cScale(5000)}, 0, 1)`
let svgTransform = d3.select('body').append('svg')
    .style('position', 'absolute')
    .attr('width', width)
    .attr('height', height)
    .attr('id', 'svgTransform')
    .style('transform', M)


const t = 1000

svg.append('svg:defs').append('svg:marker')
    .attr('id', 'small-arrow')
    .attr('refX', 6)
    .attr('refY', 3)
    .attr('markerWidth', 12)
    .attr('markerHeight', 12)
    .attr('markerUnits','userSpaceOnUse')
    .attr('orient', 'auto')
    .append('path')
    .attr('d', 'M 0 0 L 6 3 L 0 6')
    .style('stroke', 'black')
    .style('fill', 'none')

let blurFilter = svg.append('svg:defs').append('filter')
    .attr('id','blur');
blurFilter.append('feGaussianBlur')
    .attr('stdDeviation','2')

let glowFilter = svg.append('svg:defs').append('filter')
    .attr('id','glow');
glowFilter.append('feGaussianBlur')
    .attr('stdDeviation','2')
    .attr('result','coloredBlur');

let feMerge = glowFilter.append('feMerge');
feMerge.append('feMergeNode')
    .attr('in','coloredBlur');
feMerge.append('feMergeNode')
    .attr('in','SourceGraphic');

const focusedOpacities = {proposerChain: 1.0, transactionPool: 1.0, votingChains: 1.0, proposerChain: 1.0, worldMap: 1.0, ledger: 1.0, vote: 1.0}
const unfocusedOpacities = {proposerChain: 0.3, transactionPool: 0.3, votingChains: 0.3, proposerChain: 0.3, worldMap: 0.3, ledger: 0.3, vote: 0.3}

// Proposer Chain Screen sizes
let proposerScreenWidth = width/3, proposerScreenHeight = height
let proposerScreen = svg.append('g')
            .attr('class', 'proposerChain')
            .style('opacity', focusedOpacities['proposerChain'])
            .attr('width', proposerScreenWidth)
            .attr('height', proposerScreenHeight)
            .attr('transform', 'translate(' + 1*width/3 + ',0)')
            .on('click', () => focusProposerChain())

const proposerBlockSize = 20
let proposerBlocks = []
let proposerBlockId = 0
const finalizationThreshold = 10000

// Transaction Screen sizes
let transactionScreenWidth = width/3, transactionScreenHeight = height
let transactionScreen = svg.append('g')
            .attr('class', 'transactionPool')
            .style('opacity', focusedOpacities['transactionPool'])
            .attr('width', transactionScreenWidth)
            .attr('height', transactionScreenHeight)
            .on('click', () => focusTransactionPool())

const transactionBlockSize = 20
const ledgerBlockSize = 20
let transactionBlocks = []
let transactionBlockId = 0

let transactionGroup = transactionScreen.append('g').attr('id', 'transactionGroup').style('opacity', 'inherit')
let transactionBlock = transactionGroup.append('g').selectAll('.transactionBlock')

// Voting Chain Screen sizes
let votingChainScreenWidth = width*0.4, votingChainScreenHeight = height
let votingChainScreen = svg.append('g')
              .attr('class', 'votingChains')
              .style('opacity', focusedOpacities['votingChains'])
              .attr('width', votingChainScreenWidth)
              .attr('height', votingChainScreenHeight)
              .attr('transform', 'translate(' + 2*width/3 + ',0)')
              .on('click', () => focusVotingChain())


const numChains = 100
const numChainsToDisplay = 8
const votingBlockSize = 20
let chainsData = []

// World Map Screen sizes
let worldMapScreenWidth = 0.7*width, worldMapScreenHeight = 0.6*height
let worldMapScreen = svgTransform.append('g')
              .attr('class', 'worldMap')
              .style('opacity', focusedOpacities['worldMap'])
              .on('click', () => focusWorldMap())

const nodeRadius = 10
let nodes = []

let ledgerGroup = svg.append('g')
                     .attr('class', 'ledger')
                     .style('opacity', focusedOpacities['ledger'])

let voteGroup = svg.append('g')
                   .attr('class', 'vote') 
                   .style('opacity', focusedOpacities['vote'])

let voteData = []


