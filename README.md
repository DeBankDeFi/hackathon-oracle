# Oracle Module for Substrate

An oracle module for substrate, used together with `srml_collective`.

## Design Guide Lines

1. Oracle management and business logics should be decoupled.
2. Should follow conventions of `substrate` itself.

## Design

1. Staking/Rewarding/Rewarding
    * One should stake a specific amount before becoming an oracle.
    * Oracle will receive rewards if it successfully witnessed an offline event.
    * Oracle will be slashed if it missed a reporting window.
    * Oracle can be slashed if major parties determine its malicious activity. (parties such as coucil)
2. Time Cycles
    * Oracle Election: oracles will be elected by staking amount every specific duration.
    * Reporting Cycle: duration in which an oracle should report an event. If so, it'll be paid, if not, it'll be slashed.
    * Unlock Duration: minimum duration in which oracle's staking balance is locked after its unbonding action.

### Parameters

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

```rust
pub trait OracleMixedIn<T: system::Trait> {
    fn on_witnessed(who: &T::AccountId);
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

Current repo has an example of coin price oracle, build use:

```bash
$ cargo build
```

And reporters are listed in `scripts/reporters/`:

```bash
$ cd scripts/
$ npm install
$ node reporters/binance.js ...
```
