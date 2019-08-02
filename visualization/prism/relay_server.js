const WebSocket = require('ws')

const wss = new WebSocket.Server({ host: 'ec2-54-183-248-97.us-west-1.compute.amazonaws.com', port: 8080 })
let visSocket = null
let prismSocket = null

console.log('Waiting for incoming of visualization and prism')
wss.on('connection', ws => {
  if(ws.protocol=='visualization'){
    visSocket = ws
    console.log('Connected to visualization')
  }
  else {
    prismSocket = ws
    console.log('Connected to prism')
  }
  if(prismSocket!==null){
    prismSocket.on('message', message => {
      if(visSocket!==null)
        visSocket.send(`${message}`)
    })
  }
})
