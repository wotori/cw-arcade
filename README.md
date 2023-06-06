# cw-arcade
A Cosmos-based (CosmWasm) smart contract for building a decentralized arcade where users can play arcades like Tetris or Pacman by paying a quarter, just like in old-school gaming arcade machines. Scores are stored securely on the blockchain. You can choose the maximum amount of score records in the scoreboard when instantiating the smart contract.

# commands
building wasm
`cargo build --target wasm32-unknown-unknown --release`

generate schema
`cargo schema`

### deploy msg
`{"admins": ["archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y", "archway10mxcxvyjnpcmnkg0sxf7r25f3wzjqdz6jp4jux", "archway1a8dq0wced6q29rppdug7yvk8ek0dsrqwe3hxcz"], "arcade": "Pac-Man", "max_top_score": 10}`

### query msg
**get top scores**
`archway query contract-state smart --args '{"ScoreList":{}}'`

**admins**
`archway query contract-state smart --args '{"AdminsList":{}}'`
### execute
**store user record**
`archway tx --args '{"AddTopUser": {"user": {"address":"archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y", "name":"Wotori", "score":27000}}}'`

**store user record raw**
`archwayd tx wasm execute --chain-id constantine-3 --gas auto --gas-prices $(archwayd q rewards estimate-fees 1 --node 'https://rpc.constantine.archway.tech:443' --output json | jq -r '.gas_unit_price | (.amount + .denom)') --gas-adjustment 1.4 archway1tykvjvpvfqr5g7f8uqqg5du8tp0h99jcgvf05xumtgcq3vf5vajsvp9v2e  '{"AddTopUser": {"user": {"address":"archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y", "name":"Wotori", "score":27000}}}' --from wallet_name --node https://rpc.constantine.archway.tech:443 -y`

**add admin**
`archway tx --args '{"AddAdmin": {"admins": ["archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y"]}}'`