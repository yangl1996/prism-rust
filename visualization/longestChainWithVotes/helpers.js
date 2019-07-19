let computeLongestChain = () => {
  let longestChain = []
  let block = longestChainBlocks.reduce((prev, current) => (prev.depth > current.depth) ? prev : current)
  while(block!==null){
    longestChain.push(block)
    block=block.parent
  }

  return longestChain
}

let confirmBlock = (longestChainBlock) => {
  voteGroup.selectAll('.voteLink')
           .filter(d => d.to===longestChainBlock.id)
           .style('stroke-opacity', 1.0)
           .transition()
           .duration(t)
           .style('stroke-opacity', 0.0)
           .remove()
  chainVotes = chainVotes.filter(d => d.to!==longestChainBlock.id)
  if(longestChainBlock.finalized) return
  longestChainBlock.finalized = true
  longestChainBlock.finalizationLevel = 1
  const enlargement = 20
  longestChainBlock.finalized = true
  d3.select('#longestChainBlock'+longestChainBlock.blockId)
    .transition()
    .duration(t/2)
    .style('opacity', 1.0)
    .style('fill-opacity', 1.0)
    .attr('x', d => d.x-(enlargement+longestChainBlockSize)/2)
    .attr('y', d => d.y-(enlargement)/2)
    .attr('width', longestChainBlockSize+enlargement)
    .attr('height', longestChainBlockSize+enlargement)
    .transition()
    .duration(t/2)
    .attr('x', d => d.x-longestChainBlockSize/2)
    .attr('y', d => d.y)
    .attr('width', longestChainBlockSize*1.25)
    .attr('height', longestChainBlockSize)
    .on('interrupt', () => {
      d3.select('#longestChainBlock'+longestChainBlock.blockId)
        .attr('x', d => d.x-longestChainBlockSize/2)
        .attr('y', d => d.y)
        .attr('width', longestChainBlockSize*1.25)
        .attr('height', longestChainBlockSize)
        .style('opacity', 1.0)
        .style('fill-opacity', 1.0)
    })

}
