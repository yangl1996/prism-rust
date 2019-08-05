// Create glow for longest chain blocks
let blockGlow = glow('blockGlow').rgb('#17e9e0').stdDeviation(2)
blockGlow(svg)

let confirmBlock = (longestChainBlock) => {
 if(longestChainBlock.finalized) return
 longestChainBlock.finalized = true 
 const enlargement = 20
 d3.select('#longestChainBlock'+longestChainBlock.id).select('rect')
   .style('stroke-width', 4)
 d3.select('#longestChainBlock'+longestChainBlock.id).select('rect')
        .transition()
        .duration(t/2)
        .attr('x', -enlargement/(2*1.25))
        .attr('y', -enlargement/2)
        .attr('width', longestChainBlockSize+enlargement)
        .attr('height', longestChainBlockSize+enlargement)
        .transition()
        .duration(t/2)
        .attr('x', 0)
        .attr('y', 0)
        .attr('width', longestChainBlockSize*1.25)
        .attr('height', longestChainBlockSize)
  voteGroup.selectAll('.voteLink')
           .filter(d => d.to===longestChainBlock.id)
           .style('stroke-opacity', 1.0)
           .transition()
           .duration(t)
           .style('stroke-opacity', 0.0)
           .remove()
  chainVotes = chainVotes.filter(d => d.to!==longestChainBlock.id)
}

let shouldScroll = () => {
  // Check if last block is below appropriate height
  let lowestBlock = longestChainBlocks[0]
  for(let i=0; i<longestChainBlocks.length; i++)
    if(lowestBlock.y<longestChainBlocks[i].y){
      lowestBlock = longestChainBlocks[i]
    }
  return lowestBlock.y-2*longestChainBlockSize<height*0.5 ? false : true
}

let drawLongestChain = () => {
    // Create data join
    let longestChainBlock = longestChainBlocksGroup.selectAll('.longestChainBlock').data(longestChainBlocks, d => 'longestChainBlock'+d.id)


    const willScroll = shouldScroll()
    // Add new blocks
    let longestChainBlockEnter = longestChainBlock.enter().append('g')
           .attr('id', d => 'longestChainBlock'+d.id)
           .attr('class', 'longestChainBlock')
           // Cause group to shoot up from source node
           .attr('transform', d => {
            const node = d.sourceNodeId!==null ? globalNodesData.find(node => node.nodeId===d.sourceNodeId) : undefined
            const x = node ? node.x - width/3: d.x-longestChainBlockSize/2
            const y = node ? node.y : d.y
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

    longestChainBlockEnter.merge(longestChainBlock).transition()
                          .duration(t)
                          .attr('transform', d => {
                               return `translate(${d.x-longestChainBlockSize/2}, ${d.y})`
                          })
                          .on('end', (d, i) => {
                            if(willScroll && i==0) scrollLongestChain()
                            if(i==0 && !willScroll && longestChainVotes)
                              castVotes()
                             if(longestChainBlocks.length - d.depth>6 && longestChainVotes && !d.finalized){
                                 let timeout = willScroll ? 4*t : 2*t
                                 d.finalized=true
                                 d3.timeout(() => {
                                   confirmBlock(d)
                                  }, timeout)
                              }

                          })

    if(longestChainBlocks.length>1) captureTransactionBlocks(longestChainBlocks[longestChainBlocks.length-1], false)

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

    // Remove extra links
    link.exit().remove()

}

let scrollLongestChain = () => {
  let voted = false
  // Move longest chain blocks by -2*longestChainBlockSize
  longestChainBlocksGroup.selectAll('.longestChainBlock')
          .transition()
          .duration(t)
          .attr('transform', d => {
            d.y = d.y-2*longestChainBlockSize
            return `translate(${d.x-longestChainBlockSize/2}, ${d.y})`
          })
          .on('end', () => {
            if(!voted && longestChainVotes){ 
              voted = true
              d3.timeout(() => castVotes(), t)
            }
          })

  longestChainLinksGroup.selectAll('.longestChainLink')
    .transition()
    .duration(t)
    .attr('d', d => {
      return renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}})
    })

  // Move ledger link sources by -2*longestChainBlockSize
  ledgerGroup.selectAll('.ledgerLink')
     .transition('ledgerScroll')
     .duration(t)
     .attr('y1', d => {
       d.source.y2 = d.source.y2-2*longestChainBlockSize
       return d.source.y2
     })

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
    .transition('voteScroll')
    .duration(t)
    .attr('d', d => {
      return d.curve
    }) 
    .on('interrupt', d => {
        d3.select('#'+d.id).attr('d', d.curve)
   })
}

