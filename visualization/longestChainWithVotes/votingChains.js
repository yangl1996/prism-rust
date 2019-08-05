let chainsGroup = votingChainScreen.append('g').attr('class', 'chains').attr('id', 'chainsGroup')

const renderVotingLink = d3.linkVertical().x(d => d.x+(1.25-1)/2*votingBlockSize).y(d => d.y)

const scrollVotingChain = idx => {
  let didScroll = false
  let lastBlock = chainsData[idx].blocks[chainsData[idx].blocks.length-1]
  // Check if last block is below the screen's height
  while(lastBlock.y-2*votingBlockSize>height-0.4*height){
    didScroll = true
    // Select all chains and links for that specific chain and scroll by 2*votingBlockSize
    let scrollingBlocks = chainsGroup.select('#chain'+idx).selectAll('rect')
    scrollingBlocks
          .transition()
          .duration(t)
          .attr('x', d => d.x - votingBlockSize/2)
          .attr('y', d => {
            d.y = d.y-2*votingBlockSize
            return d.y
          })
    let scrollingLinks = chainsGroup.select('#links'+idx).selectAll('.chainLink')
          .transition()
          .duration(t)
          .attr('d', d => {
            if(!d.source) return
            let l = renderVotingLink({source: d.target, target: {x: d.source.x, y: d.source.y+votingBlockSize}})
            return l
          })
          .attr('marker-end', 'url(#vote-arrow)')
    // Scroll voting links
    const regex = /M([^,]*),([^,]*) Q([^,]*),([^,]*) ([^,]*),([^,]*)/
    voteGroup.selectAll('.voteLink')
      .filter(d => d.fromChain==idx)
      .attr('d', d => {
        const groups = d.curve.match(regex)
        const sourceX = groups[1]
        const sourceY = parseInt(groups[2])
        const targetX = groups[5]
        const targetY = groups[6]
        d.curve = `M${sourceX},${sourceY-2*votingBlockSize} Q${sourceX-50},${sourceY-50-2*votingBlockSize} ${targetX},${targetY}`
        return `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY}`
       })
      .transition('voteLinkScroll')
      .duration(t)
      .attr('d', d => {
        return d.curve
      }) 
      .on('interrupt', d => {
          d3.select('#'+d.id).attr('d', d.curve)
       })

    // Remove out of screen voting links
    if(chainsData[idx].shouldShift){
      voteGroup.selectAll('.voteLink')
               .filter(d => d.from===chainsData[idx].blocks[0].blockId)
               .remove()
      chainVotes = chainVotes.filter(d => d.from!==chainsData[idx].blocks[0].blockId)
      chainsData[idx].blocks.shift()
      chainsData[idx].links.shift()
    }
    else
      chainsData[idx].shouldShift = true 
    lastBlock = chainsData[idx].blocks[chainsData[idx].blocks.length-1]
  }
  return didScroll
}

const drawVotingChain = (idx, votes) => {
  // Create data join
  let chainGroup = chainsGroup.select('#chain'+idx)
  let votingBlocks = chainGroup.selectAll('g.votingBlock').data(chainsData[idx].blocks, d => d.blockId)

  // Add group tags for each votingBlock
  let votingBlocksEnter = votingBlocks.enter().append('g')
                      .attr('class', 'votingBlock')

  // Add new blocks
  votingBlocksEnter.append('rect')
         .attr('class', 'votingBlock')
         .style('filter', 'url(#blockGlow)')
         .attr('id', d => 'votingBlock'+d.blockId)
         .attr('height', votingBlockSize)
         .attr('width', votingBlockSize*1.25)
         .attr('rx', 3)
         .attr('x', d => {
           // Voting block's x coordinate is equivalent to chain's x coordinate
           d.x = chainsData[idx].x
           return d.sourceNodeLocation ? d.sourceNodeLocation[0] - width*0.6 : d.x - votingBlockSize/2
          })
         .attr('y', d => {
           // Voting block's y coordinate is 2 below it's parent.
           // If parent does not exist, the block should appear at the top of the screen.
           d.y = d.parent ? d.parent.y+2*votingBlockSize : 0
           return d.sourceNodeLocation ? d.sourceNodeLocation[1] : d.y
         })
         .style('opacity', (d, i) => {
            if(i===0) return 0.0
            return 0.25
          })
         .transition()
         .duration(3*t)
         .style('opacity', 1.0)
         .attr('x', d => { 
           return d.x - votingBlockSize/2
         })
         .attr('y', d => {
           return d.y
         })
        .on('end', (d, i) => {
          if(i==chainsData[idx].blocks.length-1){
            const didScroll = scrollVotingChain(idx)
            if(didScroll){
              d3.timeout(() => castVotes(idx, votes), t)
            }
            else
              castVotes(idx, votes)
          }
        })

  // Remove extra blocks
  votingBlocks.exit().remove()

  // Create data join from specific link chain
  let linkGroup = chainsGroup.select('#links'+idx)
  let link = linkGroup.selectAll('.chainLink').data(chainsData[idx].links, d => d.target.blockId)

  // Add new links
  link.enter().append('path', '.votingBlock')
      .attr('class', 'chainLink')
      .attr('d', d => d.source ? renderVotingLink({source: d.target, target: d.target}) : null)
      .transition()
      .delay(t)
      .duration(2*t)
      .attr('d', d => d.source ? renderVotingLink({source: d.target, target: {x: d.source.x, y: d.source.y+votingBlockSize}}) : null)
      .transition()
      .delay(1)
      .attr('marker-end', 'url(#vote-arrow)')
  // Remove extra links
  link.exit().remove()

}

const addVotingBlock = (idx, blockId, sourceNodeId, parentId, votes) => {
  if(!chainsData[idx].blocks) return
  const sourceNode = globalNodesData.find(node => node.nodeId==sourceNodeId)
  const parent = parentId!==null ? chainsData[idx].blocks.find(b => b.blockId===parentId) : null
  const newNode = {parent, blockId, children: [], sourceNodeLocation: [sourceNode.x, sourceNode.y]} 
  if(parent) parent.children.push(newNode)
  chainsData[idx].links.push({source: parent, target: newNode})
  chainsData[idx].blocks.push(newNode)
  drawVotingChain(idx, votes)
}
