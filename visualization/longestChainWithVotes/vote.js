let chainVotes = []
const drawVotes = (voteData) => {
  let vote = voteData ? voteGroup.selectAll('.voteLink').data(voteData, d=>d.id) :
                                   voteGroup.selectAll('.voteLink').data(chainVotes, d=>d.id)

  vote.exit().remove()
  let voteTransition = vote.enter().append('path')
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
              .transition('voteDraw')
              .duration(t)
              .attr('stroke-dashoffset', 0)
              .on('interrupt', (d) => {
                if(!voteData)
                  d3.select('#'+d.id)
                    .attr('stroke-dasharray', null)
                    .attr('stroke-dashoffset', 0)
                    .style('stroke-width', 1.0)
                    .style('stroke', '#e6e6e6')
               })
              .on('end', (d) => {
                if(!voteData)
                  d3.select('#'+d.id)
                    .attr('stroke-dasharray', null)
                    .attr('stroke-dashoffset', 0)
                    .style('stroke-width', 1.0)
                    .style('stroke', '#e6e6e6')
              })
  if(voteData){
    voteTransition.transition()
        .duration(t)
        .style('opacity', 0)
        .remove()
  }
}

let computeLongestChain = () => {
  let longestChain = []
  let block = longestChainBlocks.reduce((prev, current) => (prev.depth > current.depth) ? prev : current)
  while(block!==null){
    longestChain.push(block)
    block=block.parent
  }

  return longestChain
}


const castVotes = (votingChain) => {
  const lastBlock = longestChainBlocks[longestChainBlocks.length-1]
  if(votingChain==null){
    const sourceX = lastBlock.x-longestChainBlockSize/2+width/3
    const sourceY = lastBlock.y+longestChainBlockSize/2+longestChainBlockSize
    // Get the last block on chain
    const longestChain = computeLongestChain()
    let voteData = []
    for(let i=0; i<longestChain.length; i++){
      const target = longestChain[i]
      if(target.id==lastBlock.id) continue
      const targetX = target.x - longestChainBlockSize/2+width/3
      const targetY = target.y + longestChainBlockSize/2 + longestChainBlockSize

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
  else{
    // Get the last block on voting chain
    const lastBlock = chainsData[votingChain].blocks[chainsData[votingChain].blocks.length-1]
    const sourceX = lastBlock.x + 0.6*width
    const sourceY = lastBlock.y + +votingBlockSize + votingBlockSize/2
    let voteToCast = chainsData[votingChain].lastVotedBlock+1
    while(voteToCast<longestChainBlocks.length){
      // Get the block to vote for
      const longestChainBlock = longestChainBlocks.find(block => block.id==voteToCast)

      if(longestChainBlock===undefined || longestChainBlock.finalized || !longestChainBlock.x || Number.isNaN(longestChainBlock.y)) {
        voteToCast++
        continue
      }
      const targetX = longestChainBlock.x + longestChainBlockSize/2+width/3
      const targetY = longestChainBlock.y + longestChainBlockSize/2 + longestChainBlockSize

      const data = `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY}`
      const voteObj = {from: lastBlock.blockId, to: longestChainBlock.id, fromChain: votingChain, id: 'vote'+lastBlock.blockId+'-'+longestChainBlock.id, curve: `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY}`}
      let tempPath = voteGroup.append('path')
                              .attr('id', 'tempPath')
                              .attr('d', voteObj.curve)
      voteObj.totalLength = tempPath.node().getTotalLength()
      voteGroup.select('#tempPath').remove()
      chainVotes.push(voteObj)
      chainsData[votingChain].lastVotedBlock = voteToCast
      longestChainBlock.finalizationLevel+=0.01
      d3.select('#longestChainBlock'+longestChainBlock.id)
      if(longestChainBlock.finalizationLevel>finalizationThreshold) confirmBlock(longestChainBlock)
      voteToCast++
    }
    drawVotes()
  }
}

const mineVotingBlock = (votingChain, blockId, sourceNodeId, parentId, votes) => {
  const check = chainsData[votingChain].blocks.find(b => b.blockId===blockId) 
  if(check==undefined){
    pingNode(sourceNodeId)
    if(votingChain>=numChainsToDisplay) return
    addVotingBlock(votingChain, blockId, sourceNodeId, parentId, votes)
  }
}
