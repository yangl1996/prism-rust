let drawGraph = () => {

  let graphWidth = width/7
  let graphHeight = height/4

  let graphScreen = svg.append('g')
                       .attr('id', 'graph')
                       .attr('width', graphWidth)
                       .attr('height', graphHeight)
                       .style('opacity', 0)
                       .attr('transform', `translate(${1.8*width/8}, ${height/3})`)


  let xScale = d3.scaleLinear()
                 .domain([0, 3])
                 .range([0, graphWidth])

  let yScale = d3.scaleLinear()
                 .domain([0, 0.5])
                 .range([graphHeight, 0])

  let xAxis = graphScreen.append('g')
                         .attr('transform', `translate(0,${graphHeight})`)
                         .call(d3.axisBottom(xScale))

  let yAxis = graphScreen.append('g')
                         .call(d3.axisLeft(yScale))

  let lineGenerator = d3.line()
                        .x((d, i) => xScale(d[0]))
                        .y((d, i) => yScale(d[1]))
                        .curve(d3.curveBasis)

  yAxis.selectAll('.tick').style('stroke', 'white')
  xAxis.selectAll('.tick').remove()


  graphScreen.append('path')
    .datum([[0.1, 0.5], [0.2, 0.05], [2.9, 0.03]])
    .attr('class', 'line')
    .style('fill', 'none')
    .attr('d', lineGenerator)

  graphScreen.append('text')
    .text('Mining Rate')
    .style('fill', 'white')
    .style('font-family', 'monospace')
    .style('font-size', 20)
    .attr('transform', `translate(${graphWidth/2-50}, ${graphHeight})`)
    .attr('dy', 40)

  graphScreen.append('text')
    .text('Security')
    .style('fill', 'white')
    .style('font-family', 'monospace')
    .style('font-size', 20)
    .attr('transform', `translate(0, ${graphHeight/2})rotate(-90)`)
    .attr('dy', -40)
    .attr('dx', -40)
  
  graphScreen.transition()
             .duration(2*t)
             .style('opacity', 1.0)

}
