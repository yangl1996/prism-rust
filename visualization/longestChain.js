const mock = true
const protocol = 'longest-chain'
let width = 1080,
    height = 600

let svg = d3.select('body').append('svg')
	.attr('width', width)
	.attr('height', height)
  .style('position', 'absolute')

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

const t = 1000
const treeSize = width/2
const renderLink = d3.linkVertical().x(d => d.x+(1.25-1)/2*longestChainBlockSize).y(d => d.y)
const longestChainBlockSize = 20
let layoutTree = d3.tree().size([treeSize, height-0.4*height])

let longestChainScreenWidth = treeSize, longestChainScreenHeight = height
let longestChainScreen = svg.append('g')
            .attr('id', 'longestChain')
            .attr('width', longestChainScreenWidth)
            .attr('height', longestChainScreenHeight)
            .attr('transform', `translate(${width/3}, ${longestChainBlockSize})`)
let longestChainBlocksGroup = longestChainScreen.append('g').attr('id', 'longestChainBlocks')
let longestChainLinksGroup = longestChainScreen.append('g').attr('id', 'longestChainLinks')

let computeLongestChain = () => {
  let block = longestChainBlocks.reduce( (prev, current) => {
    return (prev.depth > current.depth) ? prev : current
  })
  let depth = 0
  while(block!==null){
    if(depth>6) block.finalized = true
    block = block.parent
    depth++
  }  
}

let drawLongestChain = () => {
    computeLongestChain()

    // Create data join
    let longestChainBlock = longestChainBlocksGroup.selectAll('.longestChainBlock').data(longestChainBlocks, d => 'longestChainBlock'+d.id)

    longestChainBlock
           .transition()
           .duration(t)
           .style('fill-opacity', d => d.finalized ? 1.0 : 0.4)
           .attr('x', d => { 
               return d.x-longestChainBlockSize/2
           })
           .attr('y', d => {
             return d.y
           })


    // Add new blocks
    let longestChainBlockEnter = longestChainBlock.enter().append('rect')
           .attr('id', d => 'longestChainBlock'+d.id)
           .attr('class', 'longestChainBlock')
           .style('fill-opacity', 0.4)
           .attr('height', 0)
           .attr('width', 0)
           .attr('rx', 3)
           // Cause the block to shoot from the source node's location
           .attr('x', d => { 
               const node = d.sourceNodeId ? nodes.find(node => node.nodeId===d.sourceNodeId) : undefined
               return node ? projection([node.longitude, node.latitude])[0] - width/3 + worldMapShift: d.x-longestChainBlockSize/2 
              }
           )
           .attr('y', d => { 
                const node = d.sourceNodeId ? nodes.find(node => node.nodeId===d.sourceNodeId) : undefined
                return node ? projection([node.longitude, node.latitude])[1]+(height-0.6*height) : d.y
              }
           )
           .transition()
           .duration(t)
           // Tune the fill opacity based on finalizationLevel
           .attr('height', longestChainBlockSize)
           .attr('width', longestChainBlockSize*1.25)
           .attr('x', d => { 
               return d.x-longestChainBlockSize/2
           })
           .attr('y', d => {
             return d.y
           })

    // Remove extra blocks
    longestChainBlock.exit().remove()
    let link = longestChainLinksGroup.selectAll('.chainLink').data(links, d => `${d.source.id}-${d.target.id}`)

    
    link.transition()
        .duration(t)
        .attr('d', d => d.source ? renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}}) : null)

    // Add new links
    link.enter().append('path')
        .attr('id', d => `${d.source.id}-${d.target.id}`)
        .attr('class', 'chainLink')
        .attr('d', d => d.source ? renderLink({source: d.source, target: d.source}) : null)
        .transition()
        .delay(t)
        .duration(t)
        .attr('d', d => d.source ? renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}}) : null)
        .transition()
        .delay(1)
        .attr('marker-end', 'url(#small-arrow)')
        .on('end', () => scrollLongestChain())
        .on('interrupt', () => scrollLongestChain())
    // Remove extra links
    link.exit().remove()

}

let scrollLongestChain = () => {
  // Check if last block is below appropriate height
  let lowestBlock = longestChainBlocks[0]
  for(let i=0; i<longestChainBlocks.length; i++)
    if(lowestBlock.y<longestChainBlocks[i].y){
      lowestBlock = longestChainBlocks[i]
    }
  if(lowestBlock.y-2*longestChainBlockSize<height-0.5*height)
    return false
  // Move proposer blocks by -2*longestChainBlockSize
  longestChainBlocksGroup.selectAll('rect')
          .transition()
          .duration(t)
          .attr('y', d => {
            d.y = d.y-2*longestChainBlockSize
            return d.y
          })
  longestChainLinksGroup.selectAll('.chainLink')
    .transition()
    .duration(t)
    .attr('d', d => {
      return renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}})
    })
}

let longestChainBlocks = []
let links = []
