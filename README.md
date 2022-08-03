# Polkadot Academy Final Assigment 
## Assigment: Liquid Staking Protocol.

Logic mostly reside in a new pallet `frame/liquid-staking`

# Description

- Stake funds and receive a liquid fungible token
- Unbond and withdraw staked funds according to the logic defined in the `pallet_staking`
- Owners of the liquid token, are able to vote on the gobernance (locking the liquid asset in the pallet)


# Missing

- Calculate rewards/slash and pool percentage per stake contribution.
- Voting mechanism to choose validators to nominate.
- A lot of test coverage.
