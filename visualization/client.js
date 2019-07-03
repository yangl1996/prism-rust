let websocket = new WebSocket('ws://127.0.0.1:9000/');
websocket.onmessage = function (event) {
  const data = JSON.parse(event.data)
  console.log(data)
  if('VoterBlock' in data){
    if(nodeId>1){
      const chain = data['VoterBlock']['chain']
      const votingBlockId = data['VoterBlock']['id']
      const randomChain = Math.floor(Math.random() * Math.floor(numChains))
      const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeId))
      const parentId = chainsData[chain].blocks[chainsData[chain].blocks.length-1].blockId
      mineVotingBlock(chain, votingBlockId, sourceNodeId, parentId)
    }
  }

  if('ProposerBlock' in data){
    if(nodeId>1){
      const proposerBlockId = data['ProposerBlock']['id']
      const parent = proposerBlocks.find(el => el.blockId==data['ProposerBlock']['parent'])
      const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeId))
      let transactionBlockIds = data['ProposerBlock']['transaction_refs']
      addProposerBlock(proposerBlockId, parent, sourceNodeId, transactionBlockIds)
    }
  }

  if('TransactionBlock' in data){
    if(nodeId>1){
      const transactionBlockId = data['TransactionBlock']['id']
      const sourceNodeId = Math.floor(Math.random() * Math.floor(nodeId))
      addTransactionBlock(transactionBlockId, sourceNodeId)
    }
  }
}
/* 
  Events:
  1) Add node
  Data: node id, node latitude, node longitude
  2) Add proposer block
  Data: source node id, block id, parent id, transaction block ids 
  3) Add transaction block
  Data: source node id, transaction block id 
  4) Add voting block
  Data: source node id, block id, voting chain number 
  5) Confirm proposer block
  Data: proposer block id
*/
