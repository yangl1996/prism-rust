let y = 0
let drawText = () => {
  const screenWidth = width/7
  const screenHeight = height/6
  const arrowLength = 60
  if(clicks==0){
    let textScreen1 = svg.append('g')
                     .attr('id', 'textScreen1')
                     .style('text-anchor', 'middle')
                     .attr('transform', `translate(${3*width/8}, ${height/5})`)

    textScreen1.append('text')
                .text('Proposing')
                .style('fill', 'white')
                .style('font-family', 'monospace')
                .style('font-size', 40)
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)
    y+=40
  }
  else if(clicks==1){
    d3.select('#textScreen1').append('text')
                .text('Low Mining Rate')
                .style('fill', 'white')
                .style('font-family', 'monospace')
                .style('font-size', 20)
                .attr('dy', y)
                .attr('dx', 10)
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)

    y+=20+arrowLength

    d3.select('#textScreen1').append('line')
                .attr('x1', 0)
                .attr('y1', 40+20)
                .attr('x2', 0)
                .attr('y2', y)
                .style('stroke', 'white')
                .style('stroke-width', 4)
                .style('fill', 'white')
                .attr('marker-end', 'url(#arrow)')
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)

    y+=30
          

    d3.select('#textScreen1').append('text')
                .text('Low Throughput')
                .style('fill', 'white')
                .style('font-family', 'monospace')
                .style('font-size', 20)
                .attr('dy', y)
                .attr('dx', 10)
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)
  }
  else if(clicks==2){
    y = 0

    let textScreen2 = svg.append('g')
                     .attr('id', 'textScreen2')
                     .style('text-anchor', 'middle')
                     .attr('transform', `translate(${5*width/8}, ${height/5})`)

    textScreen2.append('text')
                .text('Voting')
                .style('fill', 'white')
                .style('font-family', 'monospace')
                .style('font-size', 40)
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)

    y+=40
  }
  else if(clicks==3){
    d3.select('#textScreen2').append('text')
                .text('Low Mining Rate')
                .style('fill', 'white')
                .style('font-family', 'monospace')
                .style('font-size', 20)
                .attr('dy', y)
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)

    y+=20

    d3.select('#textScreen2').append('line')
                .attr('x1', 0)
                .attr('y1', y)
                .attr('x2', 0)
                .attr('y2', y+arrowLength)
                .style('stroke', 'white')
                .style('stroke-width', 4)
                .style('fill', 'white')
                .attr('marker-end', 'url(#arrow)')
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)

    y+=arrowLength+30
          

    d3.select('#textScreen2').append('text')
                .text('Low Voting Rate')
                .style('fill', 'white')
                .style('font-family', 'monospace')
                .style('font-size', 20)
                .attr('dy', y)
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)

    y+=20
  }
  else if(clicks==4){
    d3.select('#textScreen2').append('line')
                .attr('x1', 0)
                .attr('y1', y)
                .attr('x2', 0)
                .attr('y2', y+arrowLength)
                .style('stroke', 'white')
                .style('stroke-width', 4)
                .style('fill', 'white')
                .attr('marker-end', 'url(#arrow)')
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)

    y+=arrowLength+30

    d3.select('#textScreen2').append('text')
                .text('High Latency')
                .style('fill', 'white')
                .style('font-family', 'monospace')
                .style('font-size', 20)
                .attr('dy', y)
                .style('opacity', 0)
                .transition()
                .duration(t)
                .style('opacity', 1.0)
  }


}

let drawGraph = () => {

  let graphWidth = width/7
  let graphHeight = height/6

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
             .duration(t)
             .style('opacity', 1.0)
             .on('end', () => {
                  let line = graphScreen.append('path')
                                        .datum([[0.1, 0.5], [0.2, 0.05], [2.9, 0.03]])
                                        .attr('class', 'line')
                                        .style('fill', 'none')
                                        .attr('d', lineGenerator)
                  let totalLength = line.node().getTotalLength()
                  
                  line.attr('stroke-dasharray', totalLength + ' ' + totalLength)
                    .attr('stroke-dashoffset', totalLength)
                    .transition()
                    .duration(2*t)
                    .attr('stroke-dashoffset', 0)
             })

}
