const ledgerHeight = longestChainScreenHeight/2
const ledgerX = width*0.4
const blocksToAdd = 10

const ledgerScale = d3.scaleLinear().domain([0, ledgerHeight]).range([0, ledgerBlockSize])

const removeFromTransactionPool = transactionBlockIds => {
  // Filter out transaction blocks
  transactionBlocks = transactionBlocks.filter(block => {
    for(let i=0; i<transactionBlockIds.length; i++){
      transactionGroup.select('#transactionBlock'+transactionBlockIds[i])
                       .transition()
                       .duration(t)
                       .style('opacity', 0)
                       .remove()
      if(block.blockId===transactionBlockIds[i]) return false
    }
    return true
  })
}

const scrollLedger = (nNewBlocks, scrolled) => {
  // Scale for ledger blocks
  const ledgerScale = d3.scaleLinear().domain([ledgerGroup.selectAll('.ledgerBlock').size(), 0]).range([ledgerBlockSize, 0])
  let _y = 0
  // Maintain a mapping for x, y coordinates for ledger links
  let yMapping = {}
  let xMapping = {}
  let timeOffset = 100
  
  ledgerGroup.selectAll('.ledgerBlock')
             .transition()
             .duration((d, i) => {
               // New blocks get added with an offset to create 'ordering' visualization 
               if(i>ledgerGroup.selectAll('.ledgerBlock').size()-nNewBlocks){
                 timeOffset+=100
                 return timeOffset+t
               }
               return t
             })
             .attr('transform', (d, i) => {
                _y += ledgerScale(i)/2
                yMapping[d.blockId] = _y
                return `translate(${ledgerX}, ${_y})`
              })

  ledgerGroup.selectAll('.ledgerBlock').select('rect')
     .transition()
     .style('opacity', 0.5)
     .duration((d, i) => {
       // New blocks get added with an offset to create 'ordering' visualization 
       if(i>ledgerGroup.selectAll('.ledgerBlock').size()-nNewBlocks){
         timeOffset+=100
         return timeOffset+t
       }
       return t
     })
    .attr('width', (d, i) => {
      xMapping[d.blockId] = ledgerX+ledgerScale(i)
      return ledgerScale(i)*1.25
    })
    .attr('height', (d, i) => {
      return ledgerScale(i)
    })

  let counter = 0, index = -1
  ledgerGroup.selectAll('.ledgerBlock').selectAll('line')
     .transition()
     .duration((d, i) => {
       // New blocks get added with an offset to create 'ordering' visualization 
       if(i>ledgerGroup.selectAll('.ledgerBlock').size()-nNewBlocks){
         timeOffset+=100
         return timeOffset+t
       }
       return t
     })
     .attr('x1', 3)
     .attr('x2', (d, i) => {
       if(counter%3==0)
         index+=1
       counter+=1
       return ledgerScale(index)
     })

  let linkOffset = 0
  timeOffset = 100
  ledgerGroup.selectAll('.ledgerLink')
    .transition('t1')
     .duration((d, i) => {
       if(i>ledgerGroup.selectAll('.ledgerLink').size()-nNewBlocks){
         timeOffset+=100
         return timeOffset+t
       }
       return t
     })
     // Ledger links go to where their ledger blocks are
     // unless ledger block is not visible.
     // If not visible, put ledger link at the top of left infinity ledger
     .attr('x2', (d, i) => {
        const ledgerBlockId = d.linkId.split('to')[1]
        if(ledgerGroup.select('#ledgerBlock'+ledgerBlockId).size()==0)
          return ledgerX
        return xMapping[ledgerBlockId]
     })
     .attr('y2', (d, i) => {
        const ledgerBlockId = d.linkId.split('to')[1]
        if(ledgerGroup.select('#ledgerBlock'+ledgerBlockId).size()==0) 
          return 0
        return yMapping[ledgerBlockId]
      })
      .on('end', () => {
        // Remove ledger links if proposer block sources are not on screen
        ledgerGroup.selectAll('.ledgerLink')
                   .each((d, i) => {
                     if(d.source.y2<=-2*longestChainBlockSize) {
                       ledgerGroup.select('#referenceLink'+d.linkId).remove()
                     }
                   })
      })
  ledgerGroup.selectAll('.ledgerLink')
     .transition('t2')
     .duration(t)
     .attr('x1', (d, i) => {
        return d.source.x2
     })
     .attr('y1', (d, i) => {
        return d.source.y2
     })
  // Remove from ledger
  let removals = ledgerGroup.selectAll('.ledgerBlock').size()-5*blocksToAdd
  ledgerGroup.selectAll('.ledgerBlock').each((d, i) => {
    if(removals>=0){
      ledgerGroup.select('#'+'ledgerBlock'+d.blockId)
        .transition()
        .duration(t)
        .style('opacity', 0)
        .remove()
      removals-=1
    }
   })

}

const drawLedger = (ledgerBlocks, referenceLinks, scrolled) => {
  // Draw ledger new blocks and reference links
  let ledgerBlock = ledgerGroup.selectAll('.newLedgerBlock')
  ledgerBlock = ledgerBlock.data(ledgerBlocks, d => d.blockId)
  ledgerBlockEnter = ledgerBlock.enter().append('g')
          .attr('id', d => 'ledgerBlock' + d.blockId)
          .attr('class', 'newLedgerBlock transactionBlock')
          .attr('transform', d => `translate(${d.x}, ${d.y})`)
          .attr('class', 'ledgerBlock transactionBlock')

  ledgerBlockEnter.append('rect')
          .attr('rx', 3)
          .attr('width', ledgerBlockSize*1.25)
          .attr('height', ledgerBlockSize)
          .style('filter', 'url(#blockGlow)')

  ledgerBlockEnter.append('line')
                 .attr('class', 'transaction')
                 .attr('x1', ledgerBlockSize/2-6) 
                 .attr('y1', 5) 
                 .attr('x2', ledgerBlockSize/2+10) 
                 .attr('y2', 5) 
  ledgerBlockEnter.append('line')
                       .attr('class', 'transaction')
                       .attr('x1', ledgerBlockSize/2-6) 
                       .attr('y1', 8) 
                       .attr('x2', ledgerBlockSize/2+10) 
                       .attr('y2', 8) 
  ledgerBlockEnter.append('line')
                       .attr('class', 'transaction')
                       .attr('x1', ledgerBlockSize/2-6) 
                       .attr('y1', 11) 
                       .attr('x2', ledgerBlockSize/2+10) 
                       .attr('y2', 11) 

    let referenceLink = ledgerGroup.selectAll('.referenceLink')
    referenceLink = referenceLink.data(referenceLinks, d=>d.linkId)
    referenceLink = referenceLink.enter().append('line')
                   .attr('class', 'referenceLink')
                   .attr('id', d => 'referenceLink'+d.linkId)
                   .attr('x1', d=>d.source.x1)
                   .attr('y1', d=>d.source.y1)
                   .attr('x2', d=>d.target.x1)
                   .attr('y2', d=>d.target.y1)
                   .attr('class', 'referenceLink ledgerLink')
  scrollLedger(ledgerBlocks.length, scrolled)
  // No need to remove ledger blocks and reference links since they are removed by id when they
  // go above screen
}

// Blocks that disappear as they are captured
let disappearingGroup = svg.append('g')
const drawDisappearingBlocks = (disappearingBlocks) => {
  let disappearingBlock = disappearingGroup.selectAll('.disappearingBlock') 
  disappearingBlock = disappearingBlock.data(disappearingBlocks, d => d.blockId)

  let maxY = 0

  ledgerGroup.selectAll('.ledgerBlock')
             .each((d, i) => { 
                const y = d3.select('#ledgerBlock'+d.blockId).attr('y')
               if(y>maxY) maxY = y 
             })

  const minY = maxY-100
  const xShift = 300

  disappearingBlock = disappearingBlock.enter().append('rect')
                                       .attr('class', 'disappearingBlock transactionBlock')
                                       .attr('rx', 3)
                                       .attr('x', d => d.x)
                                       .attr('y', d => d.y)
                                       .style('filter', 'url(#blockGlow)')
                                       .attr('width', ledgerBlockSize*1.25)
                                       .attr('height', ledgerBlockSize)
                                       .transition()
                                       .duration((d, i) => Math.random()*t)
                                       .attr('transform', `translate(${xShift}, ${Math.random() * (maxY - minY) + minY})`)
                                       .style('opacity', 0)
                                       .remove()

}

const captureTransactionBlocks = (longestChainBlock, scrolled) => {
  // Get longestChainBlock and longestChainBlock location
  const transactionBlockIds = longestChainBlock.transactionBlockIds
  const node = globalNodesData.find(node => node.nodeId==longestChainBlock.sourceNodeId)
  const x1 = node.x
  const y1 = node.y
  const x2 = longestChainBlock.x + width/3 - longestChainBlockSize*1.25/2
  const y2 = longestChainBlock.y + longestChainBlockSize/2 + longestChainBlockSize

  // Get ledger blocks
  let referenceLinks = []
  let ledgerBlocks = []
  let _y = ledgerHeight

  let disappearingBlocks = []
  for(let i=0; i<transactionBlocks.length; i++){
    let tb = transactionBlocks[i]
    for(let j=0; j<transactionBlockIds.length; j++) {
      if(tb.blockId==transactionBlockIds[j]){
        if(ledgerBlocks.length<blocksToAdd-1){
          referenceLinks.push({source: {x1, y1, x2, y2},
                               target: {x1: tb.x+transactionBlockSize/2, y1: tb.y+transactionBlockSize/2},
                               linkId: `from${longestChainBlock.id}to${tb.blockId}`
                              })
            
          ledgerBlocks.push(tb)
        }
        else {
          disappearingBlocks.push(tb)
        }
        break
      }
    }
  }

  removeFromTransactionPool(transactionBlockIds)
  drawLedger(ledgerBlocks, referenceLinks, scrolled)
  drawDisappearingBlocks(disappearingBlocks)
}
