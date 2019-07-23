let simulation = d3.forceSimulation(transactionBlocks)
    .force('collide', d3.forceCollide().radius(5*transactionBlockSize/8).iterations(10).strength(0.05))
    .force('x', d3.forceX(transactionScreenWidth/4).strength(0.1))
    .force('y', d3.forceY(transactionScreenHeight*0.2).strength(0.1))
    .alphaTarget(0.1)
    .on('tick', ticked)

function ticked() {
  transactionBlock.attr('transform', d => `translate(${d.x}, ${d.y})`)
}

const restart = () => {
  // Restart the simulation with new transaction block dataset
  transactionBlock = transactionGroup.selectAll('g').data(transactionBlocks, d => d.blockId)

  transactionBlock.exit()
      .remove()

  transactionBlockEnter = transactionBlock.enter().append('g')
          .attr('id', d => 'transactionBlock' + d.blockId )
          .attr('class', 'transactionBlock')
          .attr('transform', d => `translate(${d.x}, ${d.y})`)

  transactionBlockEnter.append('rect').attr('rx', 3)
          .attr('width', transactionBlockSize*1.25)
          .attr('height', transactionBlockSize)
          .style('filter', 'url(#blockGlow)')

  transactionBlockEnter.append('line')
                       .attr('class', 'transaction')
                       .attr('x1', transactionBlockSize/2-6) 
                       .attr('y1', 5) 
                       .attr('x2', transactionBlockSize/2+10) 
                       .attr('x1', transactionBlockSize/2-6) 
                       .attr('y1', 5) 
                       .attr('x2', transactionBlockSize/2+10) 
                       .attr('y2', 5) 
  transactionBlockEnter.append('line')
                       .attr('class', 'transaction')
                       .attr('x1', transactionBlockSize/2-6) 
                       .attr('y1', 8) 
                       .attr('x2', transactionBlockSize/2+10) 
                       .attr('y2', 8) 
  transactionBlockEnter.append('line')
                       .attr('class', 'transaction')
                       .attr('x1', transactionBlockSize/2-6) 
                       .attr('y1', 11) 
                       .attr('x2', transactionBlockSize/2+10) 
                       .attr('y2', 11) 

  transactionBlockEnter
          .style('opacity', 0.0)
          .transition()
          .duration(t)
          .style('opacity', 1.0)

  transactionBlock = transactionBlock.merge(transactionBlock)
   
  // Restart simulation
  simulation.nodes(transactionBlocks)
  simulation.alpha(0.1).restart()
}

restart()

const addTransactionBlock = (blockId, sourceNodeId) => {
  // Check if already added
  const check = transactionBlocks.find(b => b.blockId===blockId) 
  if(check==undefined){
    // Add a transaction block at the bottom of the screen
    pingNode(sourceNodeId)
    const sourceNode = nodes.find(node => node.nodeId==sourceNodeId)
    const sourceNodeLocation = projection([sourceNode.longitude, sourceNode.latitude])
    transactionBlocks.push({x: sourceNodeLocation[0]+worldMapShift, y: sourceNodeLocation[1]+(height-0.6*height), blockId})
    restart()
 }
}
