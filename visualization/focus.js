const unfocus = (unfocusWorldMap=true) => {
  for (const screen of Object.keys(focusedOpacities)) {
    if(!unfocusWorldMap && (screen=='worldMap' || screen=='nodesGroup')) continue
    d3.select(`#${screen}`)
      .transition()
      .duration(2*t)
      .style('opacity', unfocusedOpacities[screen])
  }
}

const focus = () => {
  svg.selectAll('use')
     .transition()
     .duration(2*t)
     .style('opacity', 0) 
     .on('end', () => {
        svg.selectAll('use').remove()
     })
  for (const screen of Object.keys(focusedOpacities)) {
    d3.select(`#${screen}`)
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
      .attr('transform', `translate(${width/6}, 0)scale(2)`)
      .style('position', 'absolute')
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
      .attr('transform', `translate(${width/3},0)scale(2)`)
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
  if(worldMapFocused){
    focus()
    let interval = d3.interval((elapsed) => {
      if(elapsed>transTime) interval.stop()
      M = `matrix3d(1, 0, 0, 0, 0, ${aScale(elapsed)}, 0, ${bScale(elapsed)}, 0, 0, 1, 0, 0, ${cScale(elapsed)}, 0, 1)`
      svgTransform.style('transform', M)
      worldMapScreen.attr('transform', `translate(${xTranslateScale(elapsed)}, ${yTranslateScale(elapsed)})scale(${scaleScale(elapsed)})`)
      drawNodes()
    }, tStep)
    worldMapFocused = false
  }
  else{
    unfocus(false)
    console.log('focusing world map')
    let interval = d3.interval((elapsed) => {
      if(elapsed>transTime) interval.stop()
      M = `matrix3d(1, 0, 0, 0, 0, ${aScale(transTime-elapsed)}, 0, ${bScale(transTime-elapsed)}, 0, 0, 1, 0, 0, ${cScale(transTime-elapsed)}, 0, 1)`
      svgTransform.style('transform', M)
      worldMapScreen.attr('transform', `translate(${xTranslateScale(transTime-elapsed)}, ${yTranslateScale(transTime-elapsed)})scale(${scaleScale(transTime-elapsed)})`)
      drawNodes()
    }, tStep)
    worldMapFocused = true
  }
}

