let nodesGroup = null
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
  nodesGroup = worldMapScreen.append('g').attr('class', 'nodes')
  drawNodes()
})


let pingNode = node => {
  const position = projectCoordinate(node)
  for(let i=1; i<=5; i++) {
    for(let d=0; d<300; d+=100) {
      worldMapScreen.append('circle')
            .attr('class', 'ripple')
            .attr('cx', position[0]-1.5)
            .attr('cy', position[1]-6)
            .attr('r', 9)
            .transition()
            .delay(d)
            .style('stroke-opacity', 0.7)
            .duration(0.7*t)
            .style('stroke-opacity', 0)
            .attr('r', 15)
            .remove()
      }
   }
}

const dropPath = 'M 243.44676,222.01677 C 243.44676,288.9638 189.17548,343.23508 122.22845,343.23508 C 55.281426,343.23508 1.0101458,288.9638 1.0101458,222.01677 C 1.0101458,155.06975 40.150976,142.95572 122.22845,0.79337431 C 203.60619,141.74374 243.44676,155.06975 243.44676,222.01677 z';
      
const drawNodes = () => {
  // Create new nodes using projection
  if(nodesGroup==null) return
  const newNodes = nodesGroup.selectAll('g.node')
            .data(nodes)

  let newNodesEnter = newNodes.enter().append('g')
                              .attr('class', 'node')
                              .attr('id', d => 'node'+d.nodeId)

  newNodesEnter.append('path')
            .attr('d', dropPath)
            .attr('transform', d => {
              const p = projectCoordinate(d) 
              return `translate(${p[0]+10}, ${p[1]+10})rotate(180)scale(0.001)`
  
            })
            .transition()
            .duration(t)
            // Node position is determined by projection 
            .attr('transform', d => {
              const p = projectCoordinate(d) 
              return `translate(${p[0]+6}, ${p[1]+6})rotate(180)scale(0.06)`
  
            })

  newNodesEnter.append('circle')
               .attr('r', 0)
                .attr('transform', d => {
                  const p = projectCoordinate(d) 
                  return `translate(${p[0]-1}, ${p[1]-6})`
               })
               .style('fill', 'black')
                .transition()
                .duration(t)
               .attr('r', 3)

  newNodes.exit().remove()
}

let addNode = (nodeId, latitude, longitude, shardColor) => {
  nodes.push({latitude, longitude, shardColor, nodeId}) 
  drawNodes()
}

while(true) {
  if(nodes.length===cities.length) break
  const latitude = cities[nodeId][0]
  const longitude = cities[nodeId][1]
  const shardColor = '#'+((1<<24)*Math.random()|0).toString(16)
  addNode(nodeId, latitude, longitude, shardColor)
  nodeId++
}
