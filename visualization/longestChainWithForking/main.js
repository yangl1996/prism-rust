let body = document.getElementsByTagName('body')[0]

let width = body.clientWidth,
    height = body.clientHeight
let clicks = 0
let longestChainVotes = true

let svg = d3.select('body').append('svg')
  .attr('id', 'untransformedSvg')
	.attr('width', width)
	.attr('height', height)
  .style('position', 'absolute')

svg.append('rect')
    .attr('width', width)
    .attr('height', height)
    .style('fill', 'url(#background-gradient)')

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
              .attr('transform', `translate(-400, 0)scale(1.5)`)

const worldMapShift = -100

const nodeRadius = 3
let nodes = []

const treeSize = width/3
const renderLink = d3.linkVertical().x(d => d.x+(1.25-1)/2*longestChainBlockSize).y(d => d.y)
const longestChainBlockSize = 20
let layoutTree = d3.tree().size([treeSize, height-0.4*height])

// Longest Chain Screen
let longestChainScreenWidth = treeSize, longestChainScreenHeight = height
let longestChainScreen = svg.append('g')
            .attr('id', 'longestChain')
            .attr('width', longestChainScreenWidth)
            .attr('height', longestChainScreenHeight)
            .attr('transform', `translate(${width/3}, ${longestChainBlockSize})`)
let longestChainBlocksGroup = longestChainScreen.append('g').attr('id', 'longestChainBlocksClean')
let longestChainLinksGroup = longestChainScreen.append('g').attr('id', 'longestChainLinksClean')

// Votes
let voteGroup = longestChainScreen.append('g').attr('id', 'votes')
