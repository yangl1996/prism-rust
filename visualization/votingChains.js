let chainsGroup = votingChainScreen.append('g').attr('class', 'chains').attr('id', 'chainsGroup')

const scrollVotingChain = idx => {
  const lastBlock = chainsData[idx].blocks[chainsData[idx].blocks.length-1]
  // Check if last block is below the screen's height
  while((idx==0 && lastBlock.y-2*votingBlockSize>height-worldMapScreenHeight) || 
    (idx==1 && lastBlock.y-4*votingBlockSize>height-worldMapScreenHeight) ||
    (lastBlock.y-8*votingBlockSize>height-worldMapScreenHeight)){
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
            let l = renderLink({source: d.target, target: {x: d.source.x, y: d.source.y+votingBlockSize}})
            return l
          })
          .attr('marker-end', 'url(#small-arrow)')
    for(let i=0; i<voteData.length; i++){
      if(voteData[i].fromChain!==idx) continue
      const sourceX = voteData[i].data[0][0]
      const sourceY = voteData[i].data[0][1] - 2*votingBlockSize
      const targetX = voteData[i].data[2][0]
      const targetY = voteData[i].data[2][1]
      const newData = [[sourceX, sourceY], [sourceX-50,targetY+100], [targetX, targetY]]
      voteData[i].data = newData
    }
    drawVotes()
  }
}

const drawVotingChain = idx => {
  // Create data join
  let chainGroup = chainsGroup.select('#chain'+idx)
  let votingBlocks = chainGroup.selectAll('g.votingBlock').data(chainsData[idx].blocks)

  // Add group tags for each votingBlock
  let votingBlocksEnter = votingBlocks.enter().append('g')
                      .attr('class', 'votingBlock')

  // Add new blocks
  votingBlocksEnter.append('rect')
         .attr('class', 'votingBlock')
         .attr('id', d => 'votingBlock'+d.blockId)
         .attr('height', votingBlockSize)
         .attr('width', votingBlockSize)
         .attr('x', d => {
           // Voting block's x coordinate is equivalent to chain's x coordinate
           d.x = chainsData[idx].x
           return d.sourceNodeLocation ? d.sourceNodeLocation[0]-2*width/3 : d.x - votingBlockSize/2
          })
         .attr('y', d => {
           // Voting block's y coordinate is 2 below it's parent.
           // If parent does not exist, the block should appear at the top of the screen.
           d.y = d.parent ? d.parent.y+2*votingBlockSize : votingBlockSize/2
           return d.sourceNodeLocation ? d.sourceNodeLocation[1]+(height-worldMapScreenHeight) : d.y
         })
         .transition()
         .duration(t)
         .attr('x', d => { 
           return d.x - votingBlockSize/2
         })
         .attr('y', d => {
           return d.y
         })
  // Merge existing and updating elements to update colors
  votingBlocksEnter.merge(votingBlocks)
           .style('fill', 'grey')

  // Remove extra blocks
  votingBlocks.exit().remove()

  // Create data join from specific link chain
  let linkGroup = chainsGroup.select('#links'+idx)
  let link = linkGroup.selectAll('.chainLink').data(chainsData[idx].links)

  // Add new links
  link.enter().append('path', '.votingBlock')
      .attr('class', 'chainLink')
      .attr('d', d => d.source ? renderLink({source: d.target, target: d.target}) : null)
      .transition()
      .delay(t)
      .duration(t)
      .attr('d', d => d.source ? renderLink({source: d.target, target: {x: d.source.x, y: d.source.y+votingBlockSize}}) : null)
      .on('end', () => {
        scrollVotingChain(idx)
        castVotes(idx)
      })
      .transition()
      .delay(1)
      .attr('marker-end', 'url(#small-arrow)')
  // Remove extra links
  link.exit().remove()

}

const addVotingBlock = (idx, blockId, sourceNodeId, parentId=null) => {
  if(!chainsData[idx].blocks) return
  const sourceNode = nodes.find(node => node.nodeId==sourceNodeId)
  const sourceNodeLocation = projection([sourceNode.longitude, sourceNode.latitude])
  const parent = parentId ? chainsData[idx].blocks.find(b => b.blockId===parentId) : null
  const newNode = {parent, blockId, children: [], sourceNodeLocation} 
  if(parent) parent.children.push(newNode)
  chainsData[idx].links.push({source: parent, target: newNode})
  chainsData[idx].blocks.push(newNode)
  // 1) Add block to voting chain and draw
  drawVotingChain(idx)
}

// Initialize the chains spaced by votingChainScreenWidth/numChains
let chain = 0, x=0
let scale = d3.scaleLinear().domain([0, numChainsToDisplay]).range([1.0, 0.0])
let votingBlockId = 1
while(chain<numChainsToDisplay){
  let votingBlockIdStr = votingBlockId.toString(16)
  votingBlockIdStr = votingBlockIdStr.padStart(64, '0') 
  chainsData.push({x, y: 0, blocks: [], links: [], lastVotedBlock: -1, fakeBlocks: [], fakeLinks: []})
  const genesisBlock = {parent: null, blockId: votingBlockIdStr, children: [], sourceNodeLocation: null}
  chainsData[chain].blocks.push(genesisBlock)
  votingBlockId++
  let chainGroup = chainsGroup.append('g')
                              .attr('id', 'chain'+chain)
                              .style('opacity', scale(chain))
  let linkGroup = chainsGroup.append('g')
                             .attr('id', 'links'+chain)
                              .style('opacity', scale(chain))
  drawVotingChain(chain)
  chain++
  x+=votingChainScreenWidth/(numChainsToDisplay+1)
}

while(chain<numChains){
  let votingBlockIdStr = votingBlockId.toString(16)
  votingBlockIdStr = votingBlockIdStr.padStart(64, '0') 
  chainsData.push({x, y: 0, blocks: [], links: [], lastVotedBlock: -1, fakeBlocks: [], fakeLinks: []})
  const genesisBlock = {parent: null, blockId: votingBlockIdStr, children: [], sourceNodeLocation: null}
  chainsData[chain].blocks.push(genesisBlock)
  votingBlockId+=1
  let chainGroup = chainsGroup.append('g')
                              .attr('id', 'chain'+chain)
  let linkGroup = chainsGroup.append('g')
                             .attr('id', 'links'+chain)
  chain++
  x+=votingChainScreenWidth/(numChainsToDisplay+1)
}
