let computeLongestChain = () => {
  let longestChain = []
  let block = longestChainBlocks.reduce((prev, current) => (prev.depth > current.depth) ? prev : current)
  while(block!==null){
    longestChain.push(block)
    block=block.parent
  }

  return longestChain
}
