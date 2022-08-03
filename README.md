# Delegated PoS for Block Validators

## Overview
This runtime provides the DPoS pallet that is linked to the Aura pallet for 
block validation. The block rewards pallet was not finished in time.

Docs for the DPoS pallet can be found throughout it's cargo docs.

It provides two main extrinsics:
- `register_validator` - Provided a session key, a validator can register to be up for 
nomination
- `nominate` - Users can nominate validators or another nominator to delegate 
their stake to block validation

The pallet hooks into `on_initialize` to conduct the selections of validators 
for Aura. After every `N` blocks, a new set of validators is elected if there 
are validators up for election.

A test suit has been provided within the DPoS crate to demonstrate success and 
expected failure states of the election system.

## TODO

- The block rewards pallet still needs to be implemented. While I don't believe 
this would be a challenging task, I didn't balance my time well enough to have 
the resources to do it.

- Abstract DPoS over Aura, Babe, and more. This implementation is tightly 
coupled with Aura. I only learned about the Sessions pallet the night before 
this was due and was unable to dedicate the time to use it instead of just Aura.

- Runtime testing. I did not have time to ensure proper state transition over 
blocks in a runtime. 

## Requirements

Basic Direct Delegation Proof of Stake system:
- [X] A pallet which manages the DPoS System
  - [X] Where one set of users can register to be a validator by providing a session 
  key for Aura / BABE.
  - [X] Where any user can use their tokens to delegate (vote) for the set of 
  validators.
  - [X] Where every N blocks, the current “winners” are selected, and Aura / BABE is 
  updated.
  - [X] As a bonus, try to support delegation chains, where you can back a delegator 
  who themselves pick the validator.
- [ ] A pallet which gives block rewards to the current block producer.
  - [ ] As a bonus, you can think about and implement some kind of slashing for 
  validators if they “misbehave”

