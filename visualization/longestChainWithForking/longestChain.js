let blockGlow = glow('blockGlow').rgb('#17e9e0').stdDeviation(2)
blockGlow(svg)

let setLongestChain = () => {
  let block = longestChainBlocks.reduce( (prev, current) => {
    return (prev.depth > current.depth) ? prev : current
  })
  let depth = 0
  while(block!==null){
    if(depth>6) block.finalized = true
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
                          .attr('transform', d => {
                               return `translate(${d.x-longestChainBlockSize/2}, ${d.y})`
                          })


    // Add new blocks
    let longestChainBlockEnter = longestChainBlock.enter().append('g')
           .attr('id', d => 'longestChainBlock'+d.id)
           .attr('class', 'longestChainBlock')
           .attr('transform', d => {
               const node = d.sourceNodeId!==null ? nodes.find(node => node.nodeId===d.sourceNodeId) : undefined
               const x = node ? projection([node.longitude, node.latitude])[0] - width/3 + worldMapShift: d.x-longestChainBlockSize/2 
               const y = node ? projection([node.longitude, node.latitude])[1]+(height-0.6*height) : d.y
               return `translate(${x}, ${y})`
           })

    longestChainBlockEnter.append('rect')
           .style('filter', 'url(#blockGlow)')
           .attr('height', 0)
           .attr('width', 0)
           .attr('rx', 3)
           .attr('height', longestChainBlockSize)
           .attr('width', longestChainBlockSize*1.25)
    for(let y=6; y<15; y+=3){
      longestChainBlockEnter.append('line')
                            .attr('class', 'transaction')
                            .attr('x1', d => 4)
                            .attr('y1', d => y)
                            .attr('x2', d => 20)
                            .attr('y2', d => y)
      }
    
    longestChainBlockEnter.transition()
                          .duration(t)
                          .attr('transform', d => {
                               return `translate(${d.x-longestChainBlockSize/2}, ${d.y})`
                          })
    

    // Remove extra blocks
    longestChainBlock.exit().remove()
    let link = longestChainLinksGroup.selectAll('.longestChainLink').data(links, d => `${d.source.id}-${d.target.id}`)

    
    link.transition()
        .duration(t)
        .attr('d', d => d.source ? renderLink({source: d.source, target: {x: d.target.x, y: d.target.y+longestChainBlockSize}}) : null)

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
        .on('end', () => {
          const didScroll = scrollLongestChain()
          if(!didScroll && longestChainVotes)
            castVotes()
        })
        .on('interrupt', () => {
          const didScroll = scrollLongestChain()
          if(!didScroll && longestChainVotes)
            castVotes()
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
    .on('end', () => longestChainVotes ? d3.timeout(() => castVotes(), t): null)
  return true
}

let shiftScreen = () => {
  if(!longestChainVotes) return
  longestChainVotes = false
  longestChainBlocksGroup.transition()
                         .duration(t)
                         .attr('transform', `translate(-${1.3*width/3}, 0)`)

  longestChainLinksGroup.transition()
                         .duration(t)
                         .attr('transform', `translate(-${1.3*width/3}, 0)`)
                         .on('end', () => {
                           mineLowRate.stop()
                           let line = longestChainScreen.append('line')
                                                        .attr('x1', -width/8)
                                                        .attr('y1', height/4)
                                                        .attr('x2', -width/8)
                                                        .attr('y2', height/4)
                                                        .style('stroke', 'white')
                                                        .style('stroke-width', 2)
                                                        .transition()
                                                        .duration(t)
                                                        .attr('x2', width/20)
                                                        .attr('y2', height/4)
                                                        .style('stroke-width', 2)
                                                        .attr('marker-end', 'url(#arrow)')

                           let text = longestChainScreen.append('text')
                                                        .attr('x', -width/10)
                                                        .attr('y', height/4-20)
                                                        .attr('font-family', 'monospace')
                                                        .text('Increase mining rate')
                                                        .style('fill', 'white')
                                                        .style('font-size', '20px')
                                                        .style('opacity', 0)
                                                        .transition()
                                                        .duration(t)
                                                        .style('opacity', 1.0)
                           modifyProtocol()
                         })
}

let longestChainBlocks = []
let links = []
