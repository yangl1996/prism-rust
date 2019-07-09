let proposerBlocksGroup = proposerScreen.append('g').attr('id', 'proposerBlocks')

const confirmBlock = proposerBlock => {
  voteGroup.selectAll('.voteLink')
           .filter(d => d.to===proposerBlock.blockId)
           .style('stroke-opacity', 1.0)
           .transition()
           .duration(t)
           .style('stroke-opacity', 0.0)
           .remove()
  voteData = voteData.filter(d => d.to!==proposerBlock.blockId)
  if(proposerBlock.finalized) return
  proposerBlock.finalized = true
  proposerBlock.finalizationLevel = 1
  const enlargement = 20
  proposerBlock.finalized = true
  d3.select('#proposerBlock'+proposerBlock.blockId)
    .transition()
    .duration(t/2)
    .style('opacity', 1.0)
    .style('fill-opacity', 1.0)
    .attr('x', d => d.x-(enlargement+proposerBlockSize)/2)
    .attr('y', d => d.y-(enlargement)/2)
    .attr('width', proposerBlockSize+enlargement)
    .attr('height', proposerBlockSize+enlargement)
    .transition()
    .duration(t/2)
    .attr('x', d => d.x-proposerBlockSize/2)
    .attr('y', d => d.y)
    .attr('width', proposerBlockSize*1.25)
    .attr('height', proposerBlockSize)
    .on('interrupt', () => {
      d3.select('#proposerBlock'+proposerBlock.blockId)
        .attr('x', d => d.x-proposerBlockSize/2)
        .attr('y', d => d.y)
        .attr('width', proposerBlockSize*1.25)
        .attr('height', proposerBlockSize)
        .style('opacity', 1.0)
        .style('fill-opacity', 1.0)
    })
    for(let i=0; i<proposerBlock.transactionBlockIds.length; i++){
      let confirmedTxBlock = d3.select('#ledgerBlock'+proposerBlock.transactionBlockIds[i])
                               .style('opacity', 1.0)
                               .style('fill', '#ff1a1a')
    }

}

let drawProposerChain = () => {
    // Create data join
    let proposerBlock = proposerBlocksGroup.selectAll('.proposerBlock').data(proposerBlocks, d => 'proposerBlock'+d.blockId)

    // Add new blocks
    let proposerBlockEnter = proposerBlock.enter().append('rect')
           .attr('id', d => 'proposerBlock'+d.blockId)
           .attr('class', 'proposerBlock')
           .attr('height', 0)
           .attr('width', 0)
           .attr('rx', 3)
           // Cause the block to shoot from the source node's location
           .attr('x', d => { 
                const node = nodes.find(node => node.nodeId==d.sourceNodeId)
               // If no parent or only has one sibling, the block appears at center
               if(!d.parent || d.parent.children.length==1)
                 d.x = proposerScreenWidth/2
               // Otherwise if the block has a sibling, offset the block by 2 proposerBlocks
               else if(d.parent.children.length==2){
                 if(d.blockId==d.parent.children[0].blockId)
                  d.x = proposerScreenWidth/2-2*proposerBlockSize
                 else
                  d.x = proposerScreenWidth/2+2*proposerBlockSize
               }
                return node ? projection([node.longitude, node.latitude])[0]-width/3 + worldMapShift: d.x-proposerBlockSize/2 
              }
           )
           .attr('y', d => { 
                const node = nodes.find(node => node.nodeId==d.sourceNodeId)
               // The block is normal and should be offset by 2 proposerBlocks
               if(d.parent) 
                 d.y = d.parent.y+2*proposerBlockSize
               // If the block has no parent, the block appears at top of screen
               else 
                 d.y = proposerBlockSize/2
                return node ? projection([node.longitude, node.latitude])[1]+(height-worldMapScreenHeight) : d.y
              }
           )
           .style('fill-opacity', 0.0) 
           .transition()
           .duration(t)
           // Tune the fill opacity based on finalizationLevel
           .style('fill-opacity', d => d.finalizationLevel)
           .attr('height', proposerBlockSize)
           .attr('width', proposerBlockSize*1.25)
           .attr('x', d => { 
               return d.x-proposerBlockSize/2
           })
           .attr('y', d => {
             return d.y
           })
          .on('end', d => {
            const didScroll = scrollProposerChain()
            if(proposerBlocks[proposerBlocks.length-1].transactionBlockIds.length>0 && !didScroll){ 
              captureTransactionBlocks(proposerBlocks[proposerBlocks.length-1], false) 
            }
          })

    // Remove extra blocks
    proposerBlock.exit().remove()

}

const scrollProposerChain = () => {
  // Check if last block is below appropriate height
  const lastBlock = proposerBlocks[proposerBlocks.length-1]
  if(lastBlock.y-2*proposerBlockSize<height-0.4*height)
    return false
  // Move proposer blocks by -2*proposerBlockSize
  proposerBlocksGroup.selectAll('rect')
          .transition()
          .duration(t)
          .attr('x', d => d.x-proposerBlockSize/2)
          .attr('y', d => {
            d.y = d.y-2*proposerBlockSize
            return d.y
          })
          .attr('width', proposerBlockSize*1.25)
          .attr('height', proposerBlockSize)
  
  // Move ledger link sources by -2*proposerBlockSize
  ledgerGroup.selectAll('.ledgerLink')
    .transition()
     .duration(t)
     .attr('y1', d => {
       d.source.y1 = d.source.y1-2*proposerBlockSize
       return d.source.y1
     })
  
  // Shift targetY of voting links by -2*proposerBlockSize
  const regex = /M([^,]*),([^,]*) Q([^,]*),([^,]*) ([^,]*),([^,]*)/
  voteGroup.selectAll('.voteLink')
    .attr('d', d => {
      const groups = d.curve.match(regex)
      const sourceX = groups[1]
      const sourceY = groups[2]
      const targetX = groups[5]
      const targetY = parseInt(groups[6])
      d.curve = `M${sourceX},${sourceY} Q${sourceX-50},${sourceY-50} ${targetX},${targetY-2*proposerBlockSize}`
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

  d3.timeout(() => captureTransactionBlocks(proposerBlocks[proposerBlocks.length-1], true), t)
  voteGroup.selectAll('.voteLink')
           .filter(d => d.to===proposerBlocks[0].blockId)
           .remove()
  voteData = voteData.filter(d => d.to!==proposerBlocks[0].blockId)
  proposerBlocks.shift()
  // Indicate that we did scroll
  return true
}

const addProposerBlock = (blockId, parent=null, sourceNodeId, transactionBlockIds) => {
  const check = proposerBlocks.find(b => b.blockId===blockId) 
  if(check==undefined){
    const newNode = {parent, blockId, children: [], sourceNodeId, finalizationLevel: 0.3, finalized: false, transactionBlockIds} 
    if(parent.children.length>1) return
    if(parent) parent.children.push(newNode)
    proposerBlocks.push(newNode)
    drawProposerChain()
  }
}

const genesisBlock = {parent: null, blockId: ''.padStart(64, '0'), children: [], sourceNodeId: null, finalizationLevel: 0.3, transactionBlockIds: []}
proposerBlocks.push(genesisBlock)
drawProposerChain([])
