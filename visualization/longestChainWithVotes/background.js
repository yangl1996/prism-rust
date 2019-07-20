
let backgroundGlow = glow('backgroundGlow').rgb('white').stdDeviation(20)
backgroundGlow(svg)

let background = {
  nodes:d3.range(0, 20).map(d => { return {label: 'l'+d,
      r:~~d3.randomUniform(8, 15)(), 
      x: ~~d3.randomUniform(-100, width+100)(), 
      y: ~~d3.randomUniform(-20, height+20)()} 
  })
}


background.links = d3.range(0, 30).map(d => {
  const source = background.nodes[~~d3.randomUniform(20)()]
  const target = background.nodes[~~d3.randomUniform(20)()]
  return {source, target}
})

let link = svg.append('g')
    .attr('class', 'links')
    .selectAll('line')
    .data(background.links)
    .enter()
    .append('line')
    .attr('stroke', 'white')
    .style('stroke-opacity', 0.09)
    .style('filter', 'url(#backgroundGlow)')
    .attr('x1', d => d.source.x)
    .attr('y1', d => d.source.y)
    .attr('x2', d => d.target.x)
    .attr('y2', d => d.target.y)

let node = svg.append('g')
    .attr('class', 'nodes')
    .selectAll('circle')
    .data(background.nodes)
    .enter().append('circle')
    .style('fill', 'white')
    .style('filter', 'url(#backgroundGlow)')
    .style('opacity', 0.08)
    .attr('r', d => d.r)
    .attr('cx', d => d.x)
    .attr('cy', d => d.y)
