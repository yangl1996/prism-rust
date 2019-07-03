let simulation = d3.forceSimulation(transactionBlocks)
    .force('collide', d3.forceCollide().radius(5*transactionBlockSize/8).iterations(10).strength(0.05))
    .force('x', d3.forceX(transactionScreenWidth/4).strength(0.1))
    .force('y', d3.forceY(transactionScreenHeight*0.2).strength(0.1))
    .alphaTarget(0.1)
    .on('tick', ticked)

function ticked() {
  transactionBlock.attr('x', d=> d.x)
        .attr('y', d => d.y)
}

const restart = () => {
  // Restart the simulation with new transaction block dataset
  transactionBlock = transactionBlock.data(transactionBlocks, d => d.blockId)

  transactionBlock.exit().transition()
      .attr('width', 0)
      .attr('height', 0)
      .remove()

  transactionBlock = transactionBlock.enter().append('rect')
          .attr('id', d => 'transactionBlock' + d.blockId )
          .attr('x', d => d.x )
          .attr('y', d => d.y )
          .attr('width', transactionBlockSize)
          .attr('height', transactionBlockSize)
          .attr('fill', d => 'grey')
          .attr('stroke', d => 'black')
          .attr('fill-opacity', 0.1)
      .call(transactionBlock => transactionBlock.transition().attr('width', transactionBlockSize).attr('height', transactionBlockSize).attr('fill', d => 'grey' ).attr('fill-opacity', 1.0).attr('stroke', d => 'black') )
    .merge(transactionBlock)
   
  // Restart simulation
  simulation.nodes(transactionBlocks)

  // If there are too many transaction blocks, turn off collision force

  simulation.alpha(0.1).restart()
}

restart()

const addTransactionBlock = (blockId, sourceNodeId) => {
  // Add a transaction block at the bottom of the screen
  const sourceNode = nodes.find(node => node.nodeId==sourceNodeId)
  const sourceNodeLocation = projection([sourceNode.longitude, sourceNode.latitude])
  const shardColor = d3.schemeCategory10[Math.floor(Math.random()*10)]
  transactionBlocks.push({x: sourceNodeLocation[0], y: sourceNodeLocation[1]+(height-worldMapScreenHeight), shardColor, blockId})
  restart()
}
