const drawVotes = () => {
  voteGroup.selectAll('.voteLink').transition()
  let voteEnter = voteGroup.selectAll('.voteLink').data(voteData, d => d.voteId)

  voteEnter.exit()
           .transition()
           .duration(t)
           .style('opacity', 0.0)
           .remove()

  const curve = d3.line().x(d => d[0]).y(d => d[1]).curve(d3.curveBasis)
  voteEnter.transition()
           .duration(t)
           .attrTween('d', function (d) {
            const previous = d3.select(this).attr('d')
            const current = curve(d.data)
            d.scrolling = true
            return d3.interpolatePath(previous, current)
          })
  voteEnter.enter().append('path')
           .attr('class', 'voteLink')
           .attr('id', d => 'voteLink' + d.voteId)
           .style('stroke-width', 3.0)
           .style('stroke', '#e6e6e6')
           .style('filter', 'url(#blur)')
           .attr('d', d => d.scrolling ? d3.select(this).attr('d') : curve([d.data[0]])) 
           .transition()
           .duration(t)
           .attrTween('d', function (d) {
            const previous = d3.select(this).attr('d')
            const current = curve(d.data)
            d.scrolling = true
            return d3.interpolatePath(previous, current)
          })
          .on('interrupt', d => {
            d3.select('#voteLink'+d.voteId)
             .style('stroke-width', 1.0)
             .style('stroke', '#e6e6e6')
             .style('filter', 'url(#glow)')
          })
          .on('end', d => {
            d3.select('#voteLink'+d.voteId)
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

    if(votedProposerBlock===undefined || votedProposerBlock.finalized) return

    const targetX = votedProposerBlock.x + width/3
    const targetY = votedProposerBlock.y + proposerBlockSize/2

    const data = [[sourceX, sourceY], [sourceX-50,targetY+100], [targetX, targetY]]
    const voteObj = {from: lastBlock.blockId, fromChain: votingChain, to: votedProposerBlock.blockId, voteId: lastBlock.blockId+'-'+votedProposerBlock.blockId, data, scrolling: false}
    voteData.push(voteObj)
    votedProposerBlock.finalizationLevel+=0.01
    d3.select('#proposerBlock'+votedProposerBlock.blockId)
      .style('fill-opacity', votedProposerBlock.finalizationLevel)
    if(votedProposerBlock.finalizationLevel>finalizationThreshold) confirmBlock(votedProposerBlock)
    index-=1
  }

  chainsData[votingChain].lastVotedBlock = proposerBlocks[proposerBlocks.length-1].blockId

  drawVotes()
}

const mineVotingBlock = (votingChain, votingBlockId, sourceNodeId, parentId) => {
  // 1) Ping source node by finding id
  pingNode(sourceNodeId)
  if(votingChain>=numChainsToDisplay) return
  // 2) Draw voting block
  // 3) Draw voting link
  // 4) Scroll relevant voting chain 
  addVotingBlock(votingChain, votingBlockId, sourceNodeId, parentId)
  // 5) Voting block votes for ALL blocks after last voted block
}
