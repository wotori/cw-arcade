# cw-arcade
A Cosmos-based (CosmWasm) smart contract for building a decentralized arcade where users can play arcades like Tetris or Pacman by paying a quarter, just like in old-school gaming arcade machines. Scores are stored securely on the blockchain. You can choose the maximum amount of score records in the scoreboard when instantiating the smart contract along with the price for each game.

<!-- ![alt cw-arcade](cw-arcade.jpg) -->
<img src="cw-arcade.jpg" width="500" />

## Smart Contract Features

Investigate all potential queries and execute commands within the ./schema folder.

### deploy msg
`{"admins": ["archway10mxcxvyjnpcmnkg0sxf7r25f3wzjqdz6jp4jux"], "arcade": "Pac-Man", "max_top_score": 10, "denom":"aconst", "price_peer_game": "250000000000000000"}`

### 1. Admins List Management
Only an admin is authorized to execute a command to add a user to the scoreboard. The intelligent contract validates data before storing it in the storage. If the scoreboard is already full and a user achieves the highest score, securing the 1st place, they receive all the deposited coins in the contract.

- Add admins
- Remove admins
- List all admins

add admin:
`archway tx --args '{"AddAdmin": {"admins": ["archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y"]}}'`

query admins:
`archway query contract-state smart --args '{"AdminsList":{}}'`

### 2. Top Users Management
- Add top users with their address, nickname, and score

**store user record**
`archway tx --args '{"AddTopUser": {"user": {"address":"archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y", "name":"Wotori", "score":27000}}}'`

**store user record raw**
`archwayd tx wasm execute --chain-id constantine-3 --gas auto --gas-prices $(archwayd q rewards estimate-fees 1 --node 'https://rpc.constantine.archway.tech:443' --output json | jq -r '.gas_unit_price | (.amount + .denom)') --gas-adjustment 1.4 archway1tykvjvpvfqr5g7f8uqqg5du8tp0h99jcgvf05xumtgcq3vf5vajsvp9v2e  '{"AddTopUser": {"user": {"address":"archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y", "name":"Wotori", "score":27000}}}' --from wallet_name --node https://rpc.constantine.archway.tech:443 -y`

### 3. Score List Management
- Record and update scores for players
`archway query contract-state smart --args '{"ScoreList":{}}'`

### 4. Play
This execute function is initiated by players on the dApp side. It receives payment from the user (akin to inserting a quarter in old-school gaming machines). This sum of money is divided between the admins (for user store execution) and the smart contract itself, which accumulates prize coins for future distribution to the top-ranking winner.


- Allow users to participate in the game (for dApp confirmation)

### 6. Game Counter
- Increment and track the total number of games played

archway cli query:
`archway query contract-state smart --args '{"GameCounter":{}}'`

raw-query:
`archwayd query wasm contract-state smart archway19cmtglphcfhrkyr3hd39dh598gl26vg9j6f5kp7y43k3879cscrs2tz6y4 '{"GameCounter":{}}' --node https://rpc.constantine.archway.tech:443`


### 7. Price Management
- Set and update the price to participate in the game
- Inquire about the present quantity of tokens designated for the prize.
`archway query contract-state smart --args '{"PrizePool":{}}'`

### 8. Prize Pool Management
- Add funds to the prize pool (just send coins to the smart contract address)
- Distribute prizes to winners (top 1 recieve all collected const)

### 9. Total Distributed Prizes
- Update and track the total amount of prizes distributed
`archway query contract-state smart --args '{"TotalDistributed":{}}'`

# Demo
You can explore the functionalities of this cw-arcade smart contract and delight in playing [Pac-Man](https://wotori.github.io/arcade-pacman/).

![alet pac-man demo](pac-man-demo.png)
