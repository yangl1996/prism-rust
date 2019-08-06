let nodesGroup = null
let realNodesGroup = null
let landGlow = glow('landGlow').rgb('#3b945e').stdDeviation(10)
landGlow(svg)

// Fine tuned projection parameters
let projection = d3.geoNaturalEarth1()
    .scale(80)
    .clipExtent([[0, 0], [width,height-150]])

let projectCoordinate = d => projection([d.longitude, d.latitude])

let path = d3.geoPath()
    .projection(projection);

d3.json('world-continents.json', function(error) {
  if(error) throw error
}).then(function(topology) {
  worldMapScreen.selectAll('path')
        .data(topojson.object(topology, topology.objects.continent).geometries)
      .enter()
        .append('path')
        .attr('class', 'land')
        .attr('d', path)
        .style('filter', 'url(#landGlow)')
  nodesGroup = worldMapScreen.append('g').attr('id', 'fakeNodesGroup')
  // Real nodes go above on a new svg
  let realNodesSvg = d3.select('body').append('svg').attr('id', 'realNodesSvg') 
    .style('position', 'absolute')
    .attr('width', width)
    .attr('height', height)
    .on('click', () => {
      if(longestChainVotes)
        addVotingChains()
      else if(!longestChainVotes && !showTransactionPool)
        addTransactionBlocks()
      else
        endSimulation()
    })
  realNodesGroup = realNodesSvg.append('g').attr('class', 'nodes').attr('id', 'nodesGroup')
  drawNodes()
})


let pingNode = (nodeId) => {
  // Ping a node to create a ripple effect
  const globalNode = globalNodesData.find(n => n.nodeId===nodeId)
  const isLargeNode = globalNode.nodeId===globalNodesData[0].nodeId
  for(let i=1; i<=5; i++) {
    for(let d=0; d<300; d+=100) {
        realNodesGroup.append('circle')
            .attr('class', 'ripple')
            .attr('cx', () => isLargeNode ? globalNode.x-12 : globalNode.x-7)
            .attr('cy', () => isLargeNode ? globalNode.y-28 : globalNode.y-20)
            .attr('r', () => isLargeNode ? 12 : 9)
            .transition()
            .delay(d)
            .style('stroke-opacity', 0.7)
            .duration(0.7*t)
            .style('stroke-opacity', 0)
            .attr('r', () => isLargeNode ? 25 : 15)
            .remove()
      }
   }
}

// Drop path defines the Google Map icon shape
const dropPath = 'M 243.44676,222.01677 C 243.44676,288.9638 189.17548,343.23508 122.22845,343.23508 C 55.281426,343.23508 1.0101458,288.9638 1.0101458,222.01677 C 1.0101458,155.06975 40.150976,142.95572 122.22845,0.79337431 C 203.60619,141.74374 243.44676,155.06975 243.44676,222.01677 z';
      
const drawNodes = () => {
  // Create new nodes using projection
  if(nodesGroup==null) return

  let newNodes = nodesGroup.selectAll('.node')
            .data(nodes, d => d)


  newNodes.attr('id', d=>'node'+d.nodeId)


  newNodes.enter().append('circle')
                  .attr('id', d=>'node'+d.nodeId)
                  .attr('class', 'node')
                   .attr('r', 0.1)
                    .attr('transform', d => {
                      const p = projectCoordinate(d)
                      return `translate(${p[0]}, ${p[1]})`
                   })

  newNodes.exit().remove()


  // Real nodes are based on globalNodesData. globalNodesData gets a node if we have a defined nodeId
  globalNodesData = []
  for(let i=0; i<nodes.length; i++){
    if(nodes[i].nodeId===undefined) continue
    const rect = document.getElementById('node'+nodes[i].nodeId).getBoundingClientRect()
    const x = rect.left + window.scrollX 
    const y = rect.top + window.scrollY
    globalNodesData.push({x, y, nodeId: nodes[i].nodeId})
  }
  let realNodes = realNodesGroup.selectAll('g.node').data(globalNodesData, d => 'globalNode'+d.nodeId)

  realNodes.exit().remove()

  realNodes.attr('transform', d => `translate(${d.x}, ${d.y-6})`)
           .attr('class', 'node')
           .attr('transform', d => `translate(${d.x}, ${d.y-6})`)

  let realNodesEnter = realNodes.enter().append('g')
                                        .attr('class', 'node')
                                        .attr('transform', d => `translate(${d.x}, ${d.y-6})`)
                                        .attr('id', d=>'globalNode'+d.nodeId)


  realNodesEnter.append('path')
                .attr('d', dropPath)
                .attr('transform', d => `rotate(180)scale(0.001)`)
                .transition()
                .duration(t)
                .attr('transform', (d, i) => i===0 ? `rotate(180)scale(0.1)` : `rotate(180)scale(0.06)`)

  realNodesEnter.append('circle')
               .attr('r', 0)
               .attr('transform', (d, i) => i===0 ? 'translate(-12, -20)' : 'translate(-7, -13)')
               .style('fill', 'black')
               .transition()
               .duration(t)
               .attr('r', (d, i) => i===0 ? nodeRadius+3 : nodeRadius)

}

for(let i=0; i<cities.length; i++){
  const latitude = cities[i][0]
  const longitude = cities[i][1]
  nodes.push({latitude, longitude, nodeId: i})
}
