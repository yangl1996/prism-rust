let proposerBlocksGroup = proposerScreen.append('g').attr('id', 'proposerBlocks')
let proposerBlock = proposerBlocksGroup.selectAll('g.proposerBlock').data(proposerBlocks)

const renderLink = d3.linkVertical().x(d => d.x).y(d => d.y)

const confirmBlock = proposerBlock => {
  if(proposerBlock.finalized) return
  const enlargement = 20
  proposerBlock.finalized = true
  d3.select('#proposerBlock'+proposerBlock.blockId)
    .transition()
    .duration(t/2)
    .style('fill-opacity', d => {
      d.finalizationLevel = 1.0
      return d.finalizationLevel
    })
    .style('opacity', d => {
      d.finalizationLevel = 1.0
      return d.finalizationLevel
    })
    .attr('x', d => d.x-(enlargement+proposerBlockSize)/2)
    .attr('y', d => d.y-(enlargement)/2)
    .attr('width', proposerBlockSize+enlargement)
    .attr('height', proposerBlockSize+enlargement)
    .attr('fill', 'gold')
    .transition()
    .duration(t/2)
    .attr('x', d => d.x-proposerBlockSize/2)
    .attr('y', d => d.y)
    .attr('width', proposerBlockSize)
    .attr('height', proposerBlockSize)
    .on('end', () => {
      voteGroup.selectAll('.voteLink')
               .style('stroke-opacity', 1.0)
               .transition()
               .duration(t)
               .style('stroke-opacity', 0.0)
               .remove()
      voteData = voteData.filter(d => d.to!==proposerBlock.blockId)
      drawVotes()
    })

}

let drawProposerChain = transactionBlockIds => {
    // Create data join
    let proposerBlock = proposerBlocksGroup.selectAll('g.proposerBlock').data(proposerBlocks)
    
    // Add group tags for each proposerBlock
    let proposerBlockEnter = proposerBlock.enter().append('g')
                        .attr('class', 'proposerBlock')


    // Add new blocks
    proposerBlockEnter.append('rect')
           .attr('id', d => 'proposerBlock'+d.blockId)
           .attr('height', 0)
           .attr('width', 0)
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
                return node ? projection([node.longitude, node.latitude])[0]-width/3 : d.x-proposerBlockSize/2 
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
           .attr('width', proposerBlockSize)
           .attr('x', d => { 
               return d.x-proposerBlockSize/2
           })
           .attr('y', d => {
             return d.y
           })
    const didScroll = scrollProposerChain()
    if(transactionBlockIds.length>0) 
      captureTransactionBlocks(transactionBlockIds, proposerBlocks[proposerBlocks.length-1].blockId, didScroll) 

    // Remove extra blocks
    proposerBlock.exit().remove()

    // Create data join
}

const scrollProposerChain = () => {
  // Check if last block is below
  const lastBlock = proposerBlocks[proposerBlocks.length-1]
  if(lastBlock.y-2*proposerBlockSize<height-worldMapScreenHeight) return false
  proposerBlocksGroup.selectAll('rect')
          .transition()
          .delay(t)
          .duration(t)
          .attr('x', d => d.x-proposerBlockSize/2)
          .attr('y', d => {
            d.y = d.y-2*proposerBlockSize
            return d.y
          })
  for(let i=0; i<voteData.length; i++){
    const sourceX = voteData[i].data[0][0]
    const sourceY = voteData[i].data[0][1]
    const targetX = voteData[i].data[2][0]
    const targetY = voteData[i].data[2][1] - 2*proposerBlockSize
    const newData = [[sourceX, sourceY], [sourceX-50,targetY+100], [targetX, targetY]]
    voteData[i].data = newData
  }
  d3.timeout(() => drawVotes(), t)
  // Indicate that we did scroll
  return true
}

const addProposerBlock = (blockId, parent=null, sourceNodeId, transactionBlockIds) => {
  pingNode(sourceNodeId)
  const newNode = {parent, blockId, children: [], sourceNodeId, finalizationLevel: 0.3, finalized: false} 
  if(parent) parent.children.push(newNode)
  proposerBlocks.push(newNode)
  drawProposerChain(transactionBlockIds)
}
const genesisBlock = {parent: null, blockId: ''.padStart(64, '0'), children: [], sourceNodeId: null, finalizationLevel: 0.3}
proposerBlocks.push(genesisBlock)
drawProposerChain([])
