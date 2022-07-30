# Single Delegation Validation

## Requirements

Basic Direct Delegation Proof of Stake system:
* A pallet which manages the DPoS System
  * Where one set of users can register to be a validator by providing a session key for Aura / BABE.
  * Where any user can use their tokens to delegate (vote) for the set of validators.
  * Where every N blocks, the current “winners” are selected, and Aura / BABE is updated.
  * As a bonus, try to support delegation chains, where you can back a delegator who themselves pick the validator.
* A pallet which gives block rewards to the current block producer.
  * As a bonus, you can think about and implement some kind of slashing for validators if they “misbehave”

