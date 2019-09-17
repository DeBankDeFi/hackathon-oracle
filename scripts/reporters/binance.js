const Binance = require('binance-api-node').default
console.log('e11111')
const client = Binance()

console.log('111111')
const clean = client.ws.ticker('BTCUSDT', depth => {
  console.log("????")
  console.log(depth)
})
