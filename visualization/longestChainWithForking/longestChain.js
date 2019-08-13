let blockGlow = glow('blockGlow').rgb('#17e9e0').stdDeviation(2)
blockGlow(svg)

let drawLongestChain = () => {
    // Create data join
    let longestChainBlock = longestChainBlocksGroup.selectAll('.longestChainBlock').data(longestChainBlocks, d => 'longestChainBlock'+d.id)

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
    
    const enlargement = 20
    let didScroll = false
    longestChainBlockEnter.merge(longestChainBlock).transition()
                          .duration(t)
                          .attr('transform', d => {
                               return `translate(${d.x-longestChainBlockSize/2}, ${d.y})`
                          })
                          .on('end', (d, i) => {
                            didScroll = didScroll ? true : scrollLongestChain()
                            if(i==0 && !didScroll && longestChainVotes)
                              castVotes()
                             if(longestChainBlocks.length - d.depth>6 && longestChainVotes && !d.finalized){
                                 let timeout = didScroll ? 4*t : 2*t
                                 d.finalized=true
                                 d3.timeout(() => {
                                   d3.select('#longestChainBlock'+d.id).select('rect')
                                     .style('stroke-width', 4)
                                   d3.select('#longestChainBlock'+d.id).select('rect')
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
                                          .on('interrupt', () => {
                                            d3.select('#longestChainBlock'+longestChainBlock.blockId)
                                              .attr('x', d => d.x-longestChainBlockSize/2)
                                              .attr('y', d => d.y)
                                              .attr('width', longestChainBlockSize*1.25)
                                              .attr('height', longestChainBlockSize)
                                          })
                                  }, timeout)
                              }
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
    // Remove extra links
    link.exit().remove()

}

let scrollLongestChain = () => {
  // Check if last block is below appropriate height
  let lowestBlock = longestChainBlocks[longestChainBlocks.length-1]
  let scrolled = false
  while(lowestBlock.y-2*longestChainBlockSize>height-0.5*height){
    scrolled = true
  // Move proposer blocks by -2*longestChainBlockSize
  let voted = false
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
  }
  return scrolled
}

let shiftScreen = () => {
  if(!longestChainVotes) return
  longestChainVotes = false
  voteGroup.selectAll('.voteLink').remove()
  modifyProtocol()
}

let longestChainBlocks = []
let links = []
