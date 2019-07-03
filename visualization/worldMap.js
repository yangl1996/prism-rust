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
  /*
  worldMapScreen.selectAll('path')
        .data(topojson.object(topology, topology.objects.continent).geometries)
      .enter()
        .append('path')
        .attr('class', 'land')
        .attr('d', path)
  */
  nodesGroup = worldMapScreen.append('g').attr('class', 'nodes')
  drawNodes()
})


let pingNode = nodeId => {
  let node = nodes.find(n => n.nodeId==nodeId)
  if(node==undefined){
    for(let i=0; i<nodes.length; i++){
      if(!('nodeId' in nodes[i])){
        nodes[i].nodeId = nodeId
        node = nodes[i]
        break
      }
    }
  }
  const position = projectCoordinate(node)
  for(let i=1; i<=5; i++) {
    for(let d=0; d<300; d+=100) {
      worldMapScreen.append('circle')
            .attr('class', 'ripple')
            .attr('cx', position[0])
            .attr('cy', position[1])
            .attr('r', 9)
            .transition()
            .delay(d)
            .style('stroke-opacity', 0.8)
            .duration(0.7*t)
            .style('stroke-opacity', 0.1)
            .attr('r', 30)
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
                              .style('filter', 'url(#glow)')

  newNodesEnter.append('circle')
               .attr('r', 0)
                .attr('transform', d => {
                  const p = projectCoordinate(d) 
                  return `translate(${p[0]}, ${p[1]})`
               })
                .transition()
                .duration(t)
               .attr('r', 5)

  newNodes.exit().remove()

  const connectivity = 0.5

  let linkData = []
  for(let i=0; i<nodes.length-1; i++){
    const source = nodes[i]
    const p1 = projectCoordinate(source)
    for(let j=i+1; j<nodes.length; j++){
      if(Math.random()<1-connectivity) continue
      const target = nodes[j]
      const p2 = projectCoordinate(target)
      const link = {source: {x: p1[0], y: p1[1]}, target: {x: p2[0], y: p2[1]}}
      linkData.push(link)
    }
  }

  let linkEnter = nodesGroup.selectAll('.link')
                            .data(linkData)
                            .enter().append('line')
                            .attr('class', 'link')
                            .style('stroke-width', 0.0)
                            .attr('x1', d => d.source.x)
                            .attr('y1', d => d.source.y)
                            .attr('x2', d => d.target.x)
                            .attr('y2', d => d.target.y)
                            .transition()
                            .duration(t)
                            .style('stroke-width', 1.0)
}

let addNode = (latitude, longitude) => {
  nodes.push({latitude, longitude}) 
  drawNodes()
}

while(true) {
  if(nodes.length===cities.length) break
  const latitude = cities[nodeIndex][0]
  const longitude = cities[nodeIndex][1]
  addNode(latitude, longitude)
  nodeIndex++
}
