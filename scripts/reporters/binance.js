const Decimal = require('decimal.js');
const moment = require('moment');
const Binance = require('binance-api-node').default
const { ApiPromise, WsProvider } = require('@polkadot/api');
const testKeyring = require('@polkadot/keyring/testing');
const BN = require('bn.js');
const [Alice, Charlie, BOB] = ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"]

const snooze = ms => new Promise(resolve => setTimeout(resolve, ms));

async function main() {
  const client = Binance()
  const provider = new WsProvider('wss://test-api.debank.io:2053/oracle/');
  const api = await ApiPromise.create(
    { provider,
      types: {
        "PriceReport": {
          "reporter": "AccountId",
          "price": "Price",
        },
        "Price": "u128",
        "Ledger": {
          "active": "Balance",
          "unbonds": "Vec<Unbind>"
        },
        "Unbind": {
          "amount": "Balance",
          "era": "BlockNumber"
        }
      }
    })
  const keyring = testKeyring.default();
  let key = keyring.getPair(Charlie);
  let last_reported = null

  client.ws.ticker('BTCUSDT', data => {
    let now = moment()
    console.log(moment.duration(now.diff(last_reported)).seconds(), data.eventType)
    if(data.eventType === "24hrTicker" && (last_reported === null || moment.duration(now.diff(last_reported)).seconds() > 30)){
      console.log("pusing price", data.curDayClose.toString(), data)
      let price = new BN(new Decimal(data.curDayClose.toString()).mul(10000).round().toString())
      console.log("pusing price--", price.toString())
      let price_report = api.tx.price.report(price)
      api.tx.oracleMembers.execute(price_report).signAndSend(key, ({ events = [], status }) => {
        console.log("pushed price", price.toString(), status.toString(), status.toString())
      })
      last_reported = now
    }




  });
  while(true){
    await snooze(100000)
  }
}


main().catch(console.error).finally(() => process.exit());
