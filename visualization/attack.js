let fakeBlocks = []

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

  let fakeBlocksEnter = proposerBlocksGroup.selectAll('g.fakeBlock').data(fakeBlocks)

  fakeBlocksEnter.enter().append('rect')
                     .attr('class', 'fakeProposerBlock')
                     .attr('height', 0)
                     .attr('width', 0)
                     .attr('x', 0)
                     .attr('y', 0)
                     .attr('fill', 'red')
                     .transition()
                     .duration(t)
                     .attr('height', proposerBlockSize)
                     .attr('width', proposerBlockSize)
                     .attr('x', d => d.x-proposerBlockSize/2) 
                     .attr('y', d => d.y)


                
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

const simulateAttack = (ppb, chain) => {
  addFakeProposerBlock(ppb)
  let interval = d3.interval(() => {
    if(chainsData[chain].fakeBlocks.length>10){
       interval.stop()
    }
    growFakeChain(ppb, chain)
  }, t)
}
