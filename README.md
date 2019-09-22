# Oracle Module for Substrate

An [oracle](oracle) module for substrate, used together with `srml_collective`.

* substrate version: `polkadot-master`
* demo page: [https://oracle.debank.com/](https://oracle.debank.com/)
* ws: `wss://test-api.debank.io:2053/oracle/`

## Design Guide Lines

1. Oracle management and business logics should be decoupled.
2. Should follow conventions of `substrate` itself.

## Design

1. Staking/Rewarding/Slashing
    * One should stake a specific amount before becoming an oracle.
    * Oracle will receive rewards if it successfully witnessed an offline event.
    * Oracle will be slashed if it missed a reporting window.
    * Oracle can be slashed by major parties if its malicious activity is agreed upon. (parties such as council)
2. Oracle Election: oracles will be elected by staking amount every specific duration.
3. Reporting Cycle: an oracle should report an event in a specific duration. If so, it'll be paid, if not, it'll be slashed.
4. Unlock Duration: an oracle's staked coin will not be unlocked until a future time.

## Usage
### Initial Parameters

* `Currency`: Currency type.
* `OracleFee`: The amount of fee that should be paid to each oracle during each reporting cycle.
* `MissReportSlash`: The amount that'll be slashed if one oracle missed its reporting window.
* `MinStaking`: The minimum amount to stake for an oracle candidate.
* `MaliciousSlashOrigin`: The origin that's responsible for slashing malicious oracles.
* `Count`: The maxium count of working oracles.
* `ReportInteval`: The duration in which oracles should report and be paid.
* `ElectionEra`: The duration between oracle elections.
* `LockedDuration`: The locked time of staked amount.
* `ChangeMembers`: The actual oracle membership management type. (Usually the `srml_collective::Trait`)

### Extrinsics

* `bid(amount: Balance)`: bind amount to list as oraclce candidates.
* `slash_by_vote(who: AcocuntId, amount: Balnace)`: slash oracle by third parties.
* `unbind(amount: Balance)`: unbind amount.

### Public Trait

Oracle module has implemented the following trait. Business module should use this trait to
communicate with oracle module.

```rust
pub trait OracleMixedIn<T: system::Trait> {
    /// tell oracle module that an event is reported by a speicifc oracle.
    fn on_witnessed(who: &T::AccountId);
    /// predicate if one oracle is valid.
    fn is_valid(who: &T::AccountId) -> bool;
}
```

### Storage

* `Oracles`: acting oracles.
* `OracleLedger`: staking ledger of oracle/candidates.
* `WitnessReport`: blockstamp of each oracle's last event report.
* `OracleCandidates`: oracle candidates.
* `CurrentEra`: Current election era.
* `OracleLastRewarded`: oracle reward records.

### Events


* `OracleBonded(AccountId, Balance)`: Amount bonded by one oracle.
* `OracleUnbonded(AccountId, Balance)`: Amount unbonded by one oracle.
* `OracleSlashed(AccountId, Balance)`: Amount slashed to one oracle.
* `OraclePaid(AccountId, Balance)`: Amount paid to one oracle.
* `CandidatesAdded(AccountId)`: Candidate added.
* `CandidatesRemoved(AccountId)`: Candidate remove.
* `OracleStakeReleased(AccountId, Balance)`: Amount unlocked for one oracle.

## Example

Current repo has an example of coin price oracle ([link](price/src/lib.rs)), build use:

```bash
$ cargo build
```

And reporters are listed in [scripts/reporters/](scripts/reporters):

```bash
$ cd scripts/
$ npm install
$ node reporters/binance.js ...
```
