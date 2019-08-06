const mock = false
let body = document.getElementsByTagName('body')[0]

let width = body.clientWidth,
    height = body.clientHeight

let svg = d3.select('body').append('svg')
	.attr('width', width)
	.attr('height', height)
  .style('position', 'absolute')

svg.append('rect')
    .attr('width', width)
    .attr('height', height)
    .style('fill', 'url(#background-gradient)')

const focusedOpacities = {proposerChain: 1.0, transactionPool: 1.0, votingChains: 1.0, proposerChain: 1.0, worldMap: 1.0, ledger: 1.0, vote: 1.0, nodesGroup: 1.0}
const unfocusedOpacities = {proposerChain: 0.3, transactionPool: 0.3, votingChains: 0.3, proposerChain: 0.3, worldMap: 0.3, ledger: 0.3, vote: 0.3, nodesGroup: 0.3}

const transTime = 5000
let worldMapFocused = false
const tStep = 10
const aScale = d3.scaleLinear().domain([0, transTime]).range([1.0, 0.6])
const bScale = d3.scaleLinear().domain([0, transTime]).range([0.0, -0.002])
const cScale = d3.scaleLinear().domain([0, transTime]).range([0, 250])
const xTranslateScale = d3.scaleLinear().domain([0, transTime]).range([-600, -400])
const yTranslateScale = d3.scaleLinear().domain([0, transTime]).range([-300, 0])
const scaleScale = d3.scaleLinear().domain([0, transTime]).range([3, 1.5])
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
              .style('opacity', focusedOpacities['worldMap'])
              .on('click', () => focusWorldMap())

worldMapScreen.attr('transform', `translate(${xTranslateScale(transTime)}, ${yTranslateScale(transTime)})scale(${scaleScale(transTime)})`)

const nodeRadius = 3
let nodes = []
let globalNodesData = []

/*
let interval = d3.interval((elapsed) => {
  if(elapsed>transTime) interval.stop()
  M = `matrix3d(1, 0, 0, 0, 0, ${aScale(elapsed)}, 0, ${bScale(elapsed)}, 0, 0, 1, 0, 0, ${cScale(elapsed)}, 0, 1)`
  svgTransform.style('transform', M)
  worldMapScreen.attr('transform', `translate(${xTranslateScale(elapsed)}, ${yTranslateScale(elapsed)})scale(${scaleScale(elapsed)})`)
  drawNodes()
}, tStep)
*/
const t = 1000

// Proposer Chain Screen sizes
let proposerScreenWidth = width/3, proposerScreenHeight = height
let proposerScreen = svg.append('g')
            .attr('id', 'proposerChain')
            .style('opacity', focusedOpacities['proposerChain'])
            .attr('width', proposerScreenWidth)
            .attr('height', proposerScreenHeight)
            .attr('transform', 'translate(' + 1*width/3 + ',0)')
            .on('click', () => focusProposerChain())


const proposerBlockSize = 20
let proposerBlocks = []
const finalizationThreshold = 0.35

// Transaction Screen sizes
let transactionScreenWidth = width/3, transactionScreenHeight = height
let transactionScreen = svg.append('g')
            .attr('id', 'transactionPool')
            .style('opacity', focusedOpacities['transactionPool'])
            .attr('width', transactionScreenWidth)
            .attr('height', transactionScreenHeight)
            .on('click', () => focusTransactionPool())

const transactionBlockSize = 20
const ledgerBlockSize = 20
let transactionBlocks = []

let transactionGroup = transactionScreen.append('g').attr('id', 'transactionGroup').style('opacity', 'inherit')
let transactionBlock = transactionGroup.selectAll('.transactionBlock')

// Voting Chain Screen sizes
let votingChainScreenWidth = width*0.4, votingChainScreenHeight = height
let votingChainScreen = svg.append('g')
              .attr('id', 'votingChains')
              .style('opacity', focusedOpacities['votingChains'])
              .attr('width', votingChainScreenWidth)
              .attr('height', votingChainScreenHeight)
              .attr('transform', 'translate(' + 0.6*width + ',0)')
              .on('click', () => focusVotingChain())


const numChains = 100
const numChainsToDisplay = 10
const votingBlockSize = 20
let chainsData = []


let ledgerGroup = svg.append('g')
                     .attr('id', 'ledger')
                     .style('opacity', focusedOpacities['ledger'])

let voteGroup = svg.append('g')
                   .attr('id', 'vote')
                   .style('opacity', focusedOpacities['vote'])

let voteData = []


