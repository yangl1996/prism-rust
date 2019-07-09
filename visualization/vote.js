const drawVotes = () => {
  voteData = voteData.filter(v => {
    const p = proposerBlocks.find(b => b.blockId===v.to)
    return !p.finalized
  })

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
        .on('interrupt', (d) => {
          d3.select('#'+d.id)
            .attr('stroke-dasharray', null)
            .attr('stroke-dashoffset', 0)
            .style('stroke-width', 1.0)
            .style('stroke', '#e6e6e6')
            .style('filter', 'url(#glow)')
         })
        .on('end', (d) => {
          d3.select('#'+d.id)
           .attr('stroke-dasharray', null)
           .attr('stroke-dashoffset', 0)
           .style('stroke-width', 1.0)
           .style('stroke', '#e6e6e6')
           .style('filter', 'url(#glow)')
        })
}


const castVotes = (votingChain) => {
  // Get the last block on voting chain
  const lastBlock = chainsData[votingChain].blocks[chainsData[votingChain].blocks.length-1]
  // Calculate the vote's source coordinate
  const sourceX = lastBlock.x + width/3 + proposerScreenWidth
  const sourceY = lastBlock.y + proposerBlockSize/2
  let index = proposerBlocks.length-1
  while(index>=0 && chainsData[votingChain].lastVotedBlock!==proposerBlocks[index].blockId){
    // Get the proposerBlock to vote for
    const votedProposerBlock = proposerBlocks.find(block => block.blockId==proposerBlocks[index].blockId)

    if(votedProposerBlock===undefined || votedProposerBlock.finalized || !votedProposerBlock.x || Number.isNaN(votedProposerBlock.y)) {
      index-=1
      continue
    }
    const targetX = votedProposerBlock.x + width/3 + proposerBlockSize*1.25/2
    const targetY = votedProposerBlock.y + proposerBlockSize/2

    const data = `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY}`
    const voteObj = {from: lastBlock.blockId, to: votedProposerBlock.blockId, fromChain: votingChain, id: 'vote'+lastBlock.blockId+'-'+votedProposerBlock.blockId, curve: `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY}`}
    let tempPath = voteGroup.append('path')
                            .attr('id', 'tempPath')
                            .attr('d', voteObj.curve)
    voteObj.totalLength = tempPath.node().getTotalLength()
    voteGroup.select('#tempPath').remove()
    votedProposerBlock.finalizationLevel+=0.01
    d3.select('#proposerBlock'+votedProposerBlock.blockId)
      .style('fill-opacity', votedProposerBlock.finalizationLevel)
    //if(votedProposerBlock.finalizationLevel>finalizationThreshold) confirmBlock(votedProposerBlock)
    voteData.push(voteObj)
    chainsData[votingChain].lastVotedBlock = proposerBlocks[proposerBlocks.length-1].blockId
    index-=1
  }
  drawVotes()
}

const mineVotingBlock = (votingChain, votingBlockId, sourceNodeId, parentId) => {
  pingNode(sourceNodeId)
  if(votingChain>=numChainsToDisplay) return
  addVotingBlock(votingChain, votingBlockId, sourceNodeId, parentId)
}
