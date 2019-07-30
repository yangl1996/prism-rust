// Create glow for longest chain blocks
let blockGlow = glow('blockGlow').rgb('#17e9e0').stdDeviation(2)
blockGlow(svg)

let confirmBlock = (longestChainBlock) => {
  voteGroup.selectAll('.voteLink')
           .filter(d => d.to===longestChainBlock.id)
           .style('stroke-opacity', 1.0)
           .transition()
           .duration(t)
           .style('stroke-opacity', 0.0)
           .remove()
  chainVotes = chainVotes.filter(d => d.to!==longestChainBlock.id)
}

let longestChainCallback = fromBlockEnd => {
  if(fromBlockEnd && longestChainVotes) return
  const didScroll = scrollLongestChain()
  if(!didScroll && longestChainVotes)
    castVotes()
  if(longestChainBlocks[longestChainBlocks.length-1].transactionBlockIds.length>0 && !didScroll)
      captureTransactionBlocks(longestChainBlocks[longestChainBlocks.length-1], false) 
}

let drawLongestChain = () => {
    // Create data join
    let longestChainBlock = longestChainBlocksGroup.selectAll('.longestChainBlock').data(longestChainBlocks, d => 'longestChainBlock'+d.id)


    // Add new blocks
    let longestChainBlockEnter = longestChainBlock.enter().append('g')
           .attr('id', d => 'longestChainBlock'+d.id)
           .attr('class', 'longestChainBlock')
           // Cause group to shoot up from source node
           .attr('transform', d => {
            const node = d.sourceNodeId!==null ? nodes.find(node => node.nodeId===d.sourceNodeId) : undefined
            const x = node ? projection([node.longitude, node.latitude])[0] - width/3 + worldMapShift: d.x-longestChainBlockSize/2 
            const y = node ? projection([node.longitude, node.latitude])[1]+(height-0.6*height) : d.y
            return `translate(${x}, ${y})`
           })

    // Add a rect to the group
    longestChainBlockEnter.append('rect')
                           .style('filter', 'url(#blockGlow)')
                           .attr('height', longestChainBlockSize)
                           .attr('width', longestChainBlockSize*1.25)
                           .attr('rx', 3)

    // If transaction blocks have not been added yet, longest chain blocks contain transactions
    if(!showTransactionPool){
      for(let y=6; y<15; y+=3){
        longestChainBlockEnter.append('line')
                              .attr('class', 'transaction')
                              .attr('x1', 4)
                              .attr('y1', y)
                              .attr('x2', 20)
                              .attr('y2', y)
      }
    }

    // Cause longest chain blocks to shoot to proper location
    longestChainBlockEnter.transition()
                          .duration(t)
                          .attr('transform', d => {
                            return `translate(${d.x-longestChainBlockSize/2}, ${d.y})`
                          })
                          .on('end', () => longestChainCallback(true))
                          .on('interrupt', () => longestChainCallback(true))


    // Remove extra blocks
    longestChainBlock.exit().remove()

    // Create data joins for links
    let link = longestChainLinksGroup.selectAll('.longestChainLink').data(links, d => `${d.source.id}-${d.target.id}`)

    // Add new links
    link.enter().append('path')
        .attr('id', d => `${d.source.id}-${d.target.id}`)
        .attr('class', 'longestChainLink')
        .attr('d', d => d.source ? renderLink({source: d.source, target: d.source}) : null)
        .transition()
        .duration(t)
        .attr('d', d => d.source ? renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}}) : null)
        .transition()
        .delay(1)
        .attr('marker-end', 'url(#longestChain-arrow)')
        .on('end', () => longestChainCallback(false))
        .on('interrupt', () => longestChainCallback(false))

    // Remove extra links
    link.exit().remove()

}

let scrollLongestChain = () => {
  // Check if last block is below appropriate height
  let lowestBlock = longestChainBlocks[0]
  for(let i=0; i<longestChainBlocks.length; i++)
    if(lowestBlock.y<longestChainBlocks[i].y){
      lowestBlock = longestChainBlocks[i]
    }

  if(lowestBlock.y-2*longestChainBlockSize<height-0.5*height)
    return false

  // Move longest chain blocks by -2*longestChainBlockSize
  longestChainBlocksGroup.selectAll('.longestChainBlock')
          .transition()
          .duration(t)
          .attr('transform', d => {
            d.y = d.y-2*longestChainBlockSize
            return `translate(${d.x-longestChainBlockSize/2}, ${d.y})`
          })

  longestChainLinksGroup.selectAll('.longestChainLink')
    .transition()
    .duration(t)
    .attr('d', d => {
      return renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}})
    })
    .on('end', () => longestChainVotes ? d3.timeout(() => castVotes(), t) : null)

  // Move ledger link sources by -2*longestChainBlockSize
  ledgerGroup.selectAll('.ledgerLink')
    .transition()
     .duration(t)
     .attr('y1', d => {
       d.source.y1 = d.source.y1-2*longestChainBlockSize
       return d.source.y1
     })
  
  d3.timeout(() => captureTransactionBlocks(longestChainBlocks[longestChainBlocks.length-1], true), t)

  // Shift targetY of voting links by -2*longestChainBlockSize
  const regex = /M([^,]*),([^,]*) Q([^,]*),([^,]*) ([^,]*),([^,]*)/
  voteGroup.selectAll('.voteLink')
    .attr('d', d => {
      const groups = d.curve.match(regex)
      const sourceX = groups[1]
      const sourceY = groups[2]
      const targetX = groups[5]
      const targetY = parseInt(groups[6])
      d.curve = `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY-2*longestChainBlockSize}`
      return `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY}`
     })
    .transition()
    .duration(t)
    .attr('d', d => {
      return d.curve
    }) 
    .on('interrupt', d => {
        d3.select('#'+d.id).attr('d', d.curve)
   })
  return true
}

