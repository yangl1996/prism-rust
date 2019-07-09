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
          .attr('class', 'transactionBlock')
          .attr('x', d => d.x )
          .attr('y', d => d.y )
          .attr('rx', 3)
          .attr('width', transactionBlockSize*1.25)
          .attr('height', transactionBlockSize)
    .merge(transactionBlock)
   
  // Restart simulation
  simulation.nodes(transactionBlocks)

  simulation.alpha(0.1).restart()
}

restart()

const addTransactionBlock = (blockId, sourceNodeId) => {
  // Add a transaction block at the bottom of the screen
  pingNode(sourceNodeId)
  const sourceNode = nodes.find(node => node.nodeId==sourceNodeId)
  const sourceNodeLocation = projection([sourceNode.longitude, sourceNode.latitude])
  const shardColor = d3.schemeCategory10[Math.floor(Math.random()*10)]
  transactionBlocks.push({x: sourceNodeLocation[0]+worldMapShift, y: sourceNodeLocation[1]+(height-worldMapScreenHeight), shardColor, blockId})
  restart()
}
