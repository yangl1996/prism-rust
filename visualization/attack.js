let fakeBlocks = []

/*
let attack = () => {
  for(let i=0; i<numChains; i++){
    let chainBlocks = chainsData[i].blocks
    if(chainBlocks.length>2 && proposerBlocks.length>2){
      const ppb = proposerBlocks[proposerBlocks.length-2]
      simulateAttack(ppb, i)
      break
    }
  }
}

let castFakeVote = (chainIndex) => {
  const targetX = fakeBlocks[fakeBlocks.length-1].x + worldMapScreenWidth
  const targetY = fakeBlocks[fakeBlocks.length-1].y + proposerBlockSize/2
  const sourceBlock = chainsData[chainIndex].fakeBlocks[chainsData[chainIndex].fakeBlocks.length-1]
  const sourceX = sourceBlock.x+worldMapScreenWidth+proposerScreenWidth
  const sourceY = sourceBlock.y

  const data = [[sourceX, sourceY], [sourceX-50,targetY+100], [targetX, targetY]]
  const curve = d3.line().x(d => d[0]).y(d => d[1]).curve(d3.curveBasis)
  const path = svg.append('path')
    .attr('class', 'voteLink ' + 'vote'+votingBlockId + ' fakeVote')
    .attr('d', curve(data))
  const totalLength = path.node().getTotalLength();

  path.attr('stroke-dasharray', totalLength + ' ' + totalLength)
      .attr('stroke-dashoffset', totalLength)
      .transition()
      .duration(t)
      .attr('stroke-dashoffset', 0)
      .on('end', () => {
        d3.select('.fakeVote').remove()
      })


}

let addFakeProposerBlock = (parentBlock) => {
  fakeBlocks.push({x: proposerScreenWidth/2+100, y: parentBlock.y+2*proposerBlockSize})
  const fakeBlock = {parent, blockId, children: [], sourceNodeId, finalizationLevel: 0.3, finalized: false, transactionBlockIds} 

  let fakeBlocksEnter = proposerBlocksGroup.selectAll('g.fakeBlock').data(fakeBlocks)

  fakeBlocksEnter.enter().append('rect')
                     .attr('class', 'fakeProposerBlock')
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
                          return node ? projection([node.longitude, node.latitude])[1]+(height-0.6*height) : d.y
                        }
                     )
                     .attr('rx', 3)
                     .style('fill-opacity', 0.0) 
                     .transition()
                     .duration(t)
                     .attr('height', proposerBlockSize)
                     .attr('width', proposerBlockSize*1.25)
                     .attr('x', d => d.x-proposerBlockSize/2) 
                     .attr('y', d => d.y)
                     .attr('x', d => { 
                         return d.x-proposerBlockSize/2
                     })
                     .attr('y', d => {
                       return d.y
                     })
                    .on('end', d => {
                      const didScroll = scrollProposerChain()
                      if(proposerBlocks[proposerBlocks.length-1].transactionBlockIds.length>0 && !didScroll){ 
                        captureTransactionBlocks(fakeBlocks[fakeBlocks.length-1], false) 
                      }
                    })


                
}

let growFakeChain = (ppb, chainIndex) => {
  let parent = chainsData[chainIndex].blocks[chainsData[chainIndex].blocks.length-1]
  if(chainsData[chainIndex].fakeBlocks.length!==0)
    parent = chainsData[chainIndex].fakeBlocks[chainsData[chainIndex].fakeBlocks.length-1]
  const newNode = {parent, blockId: votingBlockId, children: []} 
  parent.children.push(newNode)
  chainsData[chainIndex].fakeBlocks.push(newNode)
    let chainGroup = chainsGroup.select('#chain'+chainIndex)
    let fakeVotingBlock = chainGroup.selectAll('g.fakeVotingBlock').data(chainsData[chainIndex].fakeBlocks)

  // Add group tags for each fakeVotingBlock
  let fakeVotingBlockEnter = fakeVotingBlock.enter().append('g')
                      .attr('class', 'fakeVotingBlock')

  // 1) Draw block
  fakeVotingBlockEnter.append('rect')
         .attr('class', 'votingBlock')
         .attr('id', d => 'votingBlock'+d.blockId)
         .attr('height', votingBlockSize)
         .attr('width', votingBlockSize)
         .attr('x', d => { 
           d.x = chainsData[chainIndex].x + 1.5*votingBlockSize
           return d.x - votingBlockSize/2
         })
         .attr('y', d => {
           if(!d.parent) d.y = votingBlockSize/2
           else d.y = d.parent.y+2*votingBlockSize
           return d.y
         })

  // Merge existing and updating elements to update main chain colors
  fakeVotingBlockEnter.merge(fakeVotingBlock)
           .style('fill', 'red')

  fakeVotingBlock.exit().remove()

  castFakeVote(chainIndex)
  votingBlockId+=1
}
*/
const addMaliciousBlock = (blockId, parent=null, sourceNodeId, transactionBlockIds) => {
  const check = proposerBlocks.find(b => b.blockId===blockId) 
  if(check==undefined){
    pingNode(sourceNodeId, true)
    const newNode = {parent, blockId, children: [], sourceNodeId, finalizationLevel: 0.3, finalized: false, transactionBlockIds, malicious: true} 
    if(parent.children.length>1) return
    if(parent) parent.children.push(newNode)
    proposerBlocks.push(newNode)
    drawProposerChain()
  }
}
