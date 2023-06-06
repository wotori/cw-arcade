# cw-arcade
A Cosmos-based (CosmWasm) smart contract for building a decentralized arcade where users can play arcades like Tetris or Pacman by paying a quarter, just like in old-school gaming arcade machines. Scores are stored securely on the blockchain. You can choose the maximum amount of score records in the scoreboard when instantiating the smart contract.

# commands
building wasm
`cargo build --target wasm32-unknown-unknown --release`

generate schema
`cargo schema`
