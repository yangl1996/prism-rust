const WebSocket = require('ws')

const wss = new WebSocket.Server({ port: 9000 })
let visSocket = null
let prismSocket = null

wss.on('connection', ws => {
  if(visSocket==null){
    visSocket = ws
    console.log('Connected to visualization')
  }
  else {
    prismSocket = ws
    console.log('Connected to prism')
  }
  if(prismSocket!==null){
    prismSocket.on('message', message => {
      console.log(message)
      if(visSocket!==null)
        visSocket.send(`${message}`)
    })
  }
})
