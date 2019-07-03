const ledgerHeight = proposerScreenHeight/2
const ledgerX = 3*width/8
const blocksToAdd = 10

const ledgerScale = d3.scaleLinear().domain([0, ledgerHeight]).range([0, ledgerBlockSize])

const removeFromTransactionPool = transactionBlockIds => {
  // Filter out transaction blocks
  transactionBlocks = transactionBlocks.filter(block => {
    for(let i=0; i<transactionBlockIds.length; i++){
      transactionScreen.select('#transactionBlock'+transactionBlockIds[i]).remove()
      if(block.blockId===transactionBlockIds[i]) return false
    }
    return true
  })
}

const scrollLedger = () => {
  // Calculate lowest ledger block
  let maxY = 0
  let size = 0.1
  const ledgerScale = d3.scaleLinear().domain([ledgerGroup.selectAll('.ledger').size(), 0]).range([ledgerBlockSize, 0])
  // Determine shift amount for ledger blocks and reference links
  let _y = 0
  ledgerGroup.selectAll('.ledger')
     .transition()
     .duration(t)
    .attr('width', (d, i) => {
      return ledgerScale(i)
    })
    .attr('height', (d, i) => {
      return ledgerScale(i)
    })
    .attr('y', (d, i) => {
      _y += ledgerScale(i)/2
      return _y
     })

  _y = 0
  let linkOffset = 0
  ledgerGroup.selectAll('.ledgerLink')
    .transition()
     .duration(t)
     .attr('y2', (d, i) => {
       const ledgerBlockId = d.linkId.split('to')[1]
       if(ledgerGroup.select('#ledgerBlock'+ledgerBlockId).size()==0) {
         linkOffset+=1
         return 0
       }
       _y += ledgerScale(i-linkOffset)/2
       return _y + ledgerScale(i)/2
     })

  let removals = ledgerGroup.selectAll('.ledger').size()-5*blocksToAdd
  ledgerGroup.selectAll('.ledger').each((d, i) => {
    if(removals>=0){
      ledgerGroup.select('#'+'ledgerBlock'+d.blockId)
        .transition()
        .duration(t)
        .style('opacity', 0)
        .remove()
      removals-=1
    }
   })

  ledgerGroup.selectAll('.ledgerLink')
             .attr('y1', (d, i) => {
               if(d.source.y2<=0) {
                 ledgerGroup.select('#referenceLink'+d.linkId).remove()
                 return
               }
               return d.source.y2 
             })

}

const drawLedger = (ledgerBlocks, referenceLinks, scrolled) => {

  ledgerGroup.selectAll('.ledgerLink')
    .transition()
     .delay(t)
     .duration(t)
     .attr('y1', d => {
       // Shift source y by 2*proposerBlockSize if proposerChain also scrolled
       if(scrolled)
         d.source.y2= d.source.y2-2*proposerBlockSize
       return d.source.y2
     })

  // Draw ledger new blocks and reference links
  let ledgerBlock = ledgerGroup.selectAll('.ledgerBlock')
  ledgerBlock = ledgerBlock.data(ledgerBlocks, d => d.blockId)
  ledgerBlock = ledgerBlock.enter().append('rect')
          .attr('class', 'ledgerBlock')
          .attr('id', d => 'ledgerBlock' + d.blockId)
          .attr('x', d => d.x)
          .attr('y', d => d.y)
          .attr('width', ledgerBlockSize)
          .attr('height', ledgerBlockSize)
          .attr('fill', d => 'grey')
          .attr('stroke', d => 'black')
          .attr('opacity',1.0)
          .transition()
          .duration((d, i) => 100*i + t)
          .attr('x', ledgerX)
          .attr('y', d=>d._y+5*ledgerBlockSize)
          .attr('opacity',0.5)
          .transition()
          .duration(t)
          .attr('y', d=>d._y)
          .attr('class', 'ledger')
          .on('end', (d, i) => {if(i==ledgerBlocks.length-1) scrollLedger()})

    let referenceLink = ledgerGroup.selectAll('.referenceLink')
    referenceLink = referenceLink.data(referenceLinks, d=>d.linkId)

    referenceLink = referenceLink.enter().append('line')
                   .attr('class', 'referenceLink')
                   .attr('id', d => 'referenceLink'+d.linkId)
                   .merge(referenceLink)
                   .attr('x1', d=>d.source.x1)
                   .attr('y1', d=>d.source.y1)
                   .attr('x2', d=>d.target.x1)
                   .attr('y2', d=>d.target.y1)
                   .transition()
                   .duration(t)
                   .attr('x1', d=>d.source.x2)
                   .attr('y1', d=> { 
                     if(scrolled) d.source.y2 = d.source.y2+2*proposerBlockSize
                     return d.source.y2
                   })
                   .attr('x2', d=>d.target.x2-ledgerBlockSize/2)
                   .attr('y2', d=>d.target.y2+5*ledgerBlockSize)
                   .transition()
                   .duration(t)
                   .attr('y2', d=>d.target.y2)
                   .attr('y1', d=> { 
                     if(scrolled) d.source.y2 = d.source.y2-2*proposerBlockSize
                     return d.source.y2
                   })
                   .attr('class', 'referenceLink ledgerLink')
                  

  // No need to remove ledger blocks and reference links since they are removed by id when they
  // go above screen
}

const captureTransactionBlocks = (transactionBlockIds, proposerBlockId, scrolled) => {

  // Get proposerBlock and proposerBlock location
  const proposerBlock = proposerBlocks.find(block => block.blockId===proposerBlockId)
  const node = nodes.find(node => node.nodeId==proposerBlock.sourceNodeId)
  const sourceNodeLocation = projection([node.longitude, node.latitude])
  const sourceX = proposerBlock.x + width/3
  const sourceY = proposerBlock.y + proposerBlockSize/2

  // Get ledger blocks
  let referenceLinks = []
  let ledgerBlocks = []
  let _y = ledgerHeight
  for(let i=0; i<transactionBlocks.length; i++){
    if(ledgerBlocks.length>blocksToAdd-1) break
    let tb = transactionBlocks[i]
    for(let j=0; j<transactionBlockIds.length; j++) {
      if(tb.blockId==transactionBlockIds[j]){
        referenceLinks.push({source: {x1: sourceNodeLocation[0], y1: sourceNodeLocation[1]+(height-worldMapScreenHeight), 
                                      x2: sourceX, y2: sourceY},
                             target: {x1: tb.x+transactionBlockSize/2, y1: tb.y+transactionBlockSize/2,
                                      x2: ledgerX+ledgerBlockSize/2, y2: _y+ledgerBlockSize/2},
                             linkId: `from${proposerBlockId}to${tb.blockId}`
                            })
          
        tb._y= _y
        ledgerBlocks.push(transactionBlocks[i])
        _y+=ledgerBlockSize
        break
      }
    }
  }

  removeFromTransactionPool(transactionBlockIds)
  drawLedger(ledgerBlocks, referenceLinks, scrolled)
}
