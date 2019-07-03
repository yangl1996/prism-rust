const unfocus = (unfocusWorldMap=true) => {
  for (const screen of Object.keys(focusedOpacities)) {
    if(screen=='worldMap' && !unfocusWorldMap) continue
    d3.select(`.${screen}`)
      .style('opacity', focusedOpacities[screen])
      .transition()
      .duration(2*t)
      .style('opacity', unfocusedOpacities[screen])
  }
}

const focus = (focusWorldMap=true) => {
  svg.selectAll('use')
     .transition()
     .duration(2*t)
     .style('opacity', 0) 
     .on('end', () => {
        svg.selectAll('use').remove()
     })
  for (const screen of Object.keys(focusedOpacities)) {
    if(screen=='worldMap' && !focusWorldMap) continue
    d3.select(`.${screen}`)
      .style('opacity', unfocusedOpacities[screen])
      .transition()
      .duration(2*t)
      .style('opacity', focusedOpacities[screen])
  }
}

const focusProposerChain = () => {
  if(svg.selectAll('use').size()>0)
    focus()
  else {
    unfocus()
    svg.append('use')
      .attr('xlink:href','#proposerBlocks')
      .attr('transform', `translate(${proposerScreenWidth/2}, 0)scale(2)`)
      .style('opacity', 0.0)
      .transition()
      .duration(2*t)
      .style('opacity', 1.0)
  }

}

const focusTransactionPool = () => {
  if(svg.selectAll('use').size()>0)
    focus()
  else {
    unfocus()
    svg.append('use')
      .attr('xlink:href','#transactionGroup')
      .attr('transform', `translate(${width/3}, ${-height/2})scale(2)`)
      .style('opacity', 0.0)
      .transition()
      .duration(2*t)
      .style('opacity', 1.0)
  }
}

const focusVotingChain = () => {
  if(svg.selectAll('use').size()>0)
    focus()
  else {
    unfocus()
    svg.append('use')
      .attr('xlink:href','#chainsGroup')
      .attr('transform', `translate(${votingChainScreenWidth/2}, 0)scale(2)`)
      .style('opacity', 0.0)
      .transition()
      .duration(2*t)
      .style('opacity', 1.0)
  }
}

const focusWorldMap = () => {
  if(worldMapScreen.style('opacity')==0.3){
    unfocus(false)
    worldMapScreen
      .transition()
      .duration(2*t)
      .style('opacity', 1.0)
  }
  else{
    focus(false)
    worldMapScreen
      .transition()
      .duration(2*t)
      .style('opacity', focusedOpacities['worldMap'])
  }
}

