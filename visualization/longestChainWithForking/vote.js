const drawVotes = (voteData) => {
  let vote = voteGroup.selectAll('.voteLink').data(voteData, d=>d.id)

  vote.exit().remove()
  vote.enter().append('path')
      .attr('id', d => d.id)
      .attr('class', d => 'voteLink to' + d.to)
      .attr('d', d=>{
        return d.curve
      })
      .style('stroke-width', 3.0)
      .attr('stroke', 'url(#linear-gradient)')
      .style('filter', 'url(#blur)')
      .attr('stroke-dasharray', d => d.totalLength + ' ' + d.totalLength)
      .attr('stroke-dashoffset', d => d.totalLength)
      .transition()
      .duration(t)
      .attr('stroke-dashoffset', 0)
      .transition()
      .duration(t)
      .style('opacity', 0)
      .remove()
}


const castVotes = () => {
  if(!showVotes) return
  // Get the last block on chain
  const lastBlock = longestChainBlocks[longestChainBlocks.length-1]
  const sourceX = lastBlock.x-longestChainBlockSize/2
  const sourceY = lastBlock.y+longestChainBlockSize/2
  const longestChain = computeLongestChain()
  let voteData = []
  for(let i=0; i<longestChain.length; i++){
    const target = longestChain[i]
    if(target.id==lastBlock.id) continue
    const targetX = target.x - longestChainBlockSize/2
    const targetY = target.y + longestChainBlockSize/2

    const data = `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY}`
    const voteObj = {from: lastBlock.id, to: target.id, id: 'vote'+lastBlock.id+'-'+target.id, curve: `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY}`}
    let tempPath = voteGroup.append('path')
                            .attr('id', 'tempPath')
                            .attr('d', voteObj.curve)
    voteObj.totalLength = tempPath.node().getTotalLength()
    voteGroup.select('#tempPath').remove()
    voteData.push(voteObj)
  }
  drawVotes(voteData)
}
