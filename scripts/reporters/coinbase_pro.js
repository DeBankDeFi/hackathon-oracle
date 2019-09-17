const CoinbasePro = require('coinbase-pro');
const Decimal = require('decimal.js');
const moment = require('moment');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const testKeyring = require('@polkadot/keyring/testing');
const BN = require('bn.js');
const [Alice, Charlie, BOB] = ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"]

const snooze = ms => new Promise(resolve => setTimeout(resolve, ms));

async function main() {
  let changed = false

  const websocket = new CoinbasePro.WebsocketClient(['BTC-USD']);
  const provider = new WsProvider('ws://127.0.0.1:9944');
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
  let key = keyring.getPair(Alice);
  let last_reported = null

  websocket.on('message', data => {
    if(!changed){
      websocket.unsubscribe({ channels: ['full'] })
      websocket.subscribe({ product_ids: ['BTC-USD'], channels: ['ticker'] })
      changed=true
    }

    let now = moment()
    console.log(moment.duration(now.diff(last_reported)).seconds(), data.type)
    if(data.type === "ticker" && (last_reported === null || moment.duration(now.diff(last_reported)).seconds() > 60)){
      console.log("pusing price")
      let price = new BN(new Decimal(data.price).mul(10000).toString())
      let price_report = api.tx.price.report(price)
      api.tx.oracleMembers.execute(price_report).signAndSend(key, ({ events = [], status }) => {
        console.log("pushed price", price.toString())
      })
      last_reported = now
    }
  });

  websocket.on('error', err => {
  });
  websocket.on('close', () => {
  });

  while(true){
    await snooze(100000)
  }
}


main().catch(console.error).finally(() => process.exit());
