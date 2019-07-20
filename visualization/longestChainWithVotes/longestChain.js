
let blockGlow = glow('blockGlow').rgb('yellow').stdDeviation(3)
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

let setLongestChain = () => {
  let block = longestChainBlocks.reduce( (prev, current) => {
    return (prev.depth > current.depth) ? prev : current
  })
  let depth = 0
  while(block!==null){
    if(depth>6) {
      block.finalized = true
      chainVotes = chainVotes.filter(d => d.to!==block.id)
    }
    block = block.parent
    depth++
  }  
}

let drawLongestChain = () => {
    setLongestChain()

    // Create data join
    let longestChainBlock = longestChainBlocksGroup.selectAll('.longestChainBlock').data(longestChainBlocks, d => 'longestChainBlock'+d.id)

    longestChainBlock
           .transition()
           .duration(t/2)
           .style('fill-opacity', d => d.finalized ? 1.0 : d.finalizationLevel)
           .attr('x', d => { 
               return d.x-longestChainBlockSize/2
           })
           .attr('y', d => {
             return d.y
           })


    // Add new blocks
    let longestChainBlockEnter = longestChainBlock.enter().append('g')
           .attr('id', d => 'longestChainBlock'+d.id)
           .attr('class', 'longestChainBlock')
    longestChainBlockEnter.append('rect')
                           .style('fill-opacity', 0.4)
                           .style('filter', 'url(#blockGlow)')
                           .attr('height', 0)
                           .attr('width', 0)
                           .attr('rx', 3)
                           // Cause the block to shoot from the source node's location
                           .attr('x', d => { 
                               const node = d.sourceNodeId!==null ? nodes.find(node => node.nodeId===d.sourceNodeId) : undefined
                               return node ? projection([node.longitude, node.latitude])[0] - width/3 + worldMapShift: d.x-longestChainBlockSize/2 
                              }
                           )
                           .attr('y', d => { 
                                const node = d.sourceNodeId!==null ? nodes.find(node => node.nodeId===d.sourceNodeId) : undefined
                                return node ? projection([node.longitude, node.latitude])[1]+(height-0.6*height) : d.y
                              }
                           )
                           .transition()
                           .duration(t)
                           // Tune the fill opacity based on finalizationLevel
                           .attr('height', longestChainBlockSize)
                  //.style('filter', 'url(#myGlow)')
                           .attr('width', longestChainBlockSize*1.25)
                           .attr('x', d => { 
                               return d.x-longestChainBlockSize/2
                           })
                           .attr('y', d => {
                             return d.y
                           })

    if(!showTransactionPool){
      for(let y=6; y<15; y+=3){
        longestChainBlockEnter.append('line')
                              .attr('x1', d => d.x - longestChainBlockSize/2+4)
                              .attr('y1', d => d.y+y)
                              .attr('x2', d => d.x + longestChainBlockSize/2)
                              .attr('y2', d => d.y+y)
                              .style('stroke', 'white')
                              .style('opacity', 0)
                              .transition()
                              .duration(t)
                              .style('opacity', 0.4)
      }
   }

    // Remove extra blocks
    longestChainBlock.exit().remove()
    let link = longestChainLinksGroup.selectAll('.chainLink').data(links, d => `${d.source.id}-${d.target.id}`)

    
    link.transition()
        .duration(t)
        .attr('d', d => d.source ? renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}}) : null)

    // Add new links
    link.enter().append('path')
        .attr('id', d => `${d.source.id}-${d.target.id}`)
        .attr('class', 'chainLink')
        .attr('d', d => d.source ? renderLink({source: d.source, target: d.source}) : null)
        .transition()
        .duration(t)
        .attr('d', d => d.source ? renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}}) : null)
        .transition()
        .delay(1)
        .attr('marker-end', 'url(#small-arrow)')
        .on('end', () => {
          const didScroll = scrollLongestChain()
          if(!didScroll && longestChainVotes)
            castVotes()
          if(longestChainBlocks[longestChainBlocks.length-1].transactionBlockIds.length>0 && !didScroll)
              captureTransactionBlocks(longestChainBlocks[longestChainBlocks.length-1], false) 
        })
        .on('interrupt', () => {
          const didScroll = scrollLongestChain()
          if(!didScroll && longestChainVotes)
            castVotes()
          if(longestChainBlocks[longestChainBlocks.length-1].transactionBlockIds.length>0 && !didScroll)
              captureTransactionBlocks(longestChainBlocks[longestChainBlocks.length-1], false) 
        })
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
  // Move proposer blocks by -2*longestChainBlockSize
  longestChainBlocksGroup.selectAll('rect')
          .transition()
          .duration(t)
          .attr('y', d => {
            d.y = d.y-2*longestChainBlockSize
            return d.y
          })
  if(!showTransactionPool){
    longestChainBlocksGroup.selectAll('line')
            .transition()
            .duration(t)
            .attr('y1', (d, i) => {
              if(i%3==0) return d.y+5
              if(i%3==1) return d.y+8
              if(i%3==2) return d.y+11
            })
            .attr('y2', (d, i) => {
              if(i%3==0) return d.y+5
              if(i%3==1) return d.y+8
              if(i%3==2) return d.y+11
            })
 }
  longestChainLinksGroup.selectAll('.chainLink')
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

