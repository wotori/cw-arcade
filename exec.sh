#!/bin/bash

query="$1"

case "$query" in
  "top-scores")
    archway query contract-state smart --args '{"ScoreList":{}}'
    ;;
  "admins")
    archway query contract-state smart --args '{"AdminsList":{}}'
    ;;
  "total-game-played")
    archway query contract-state smart --args '{"GameCounter":{}}'
    ;;
  "total-game-played-raw")
    archwayd query wasm contract-state smart archway19cmtglphcfhrkyr3hd39dh598gl26vg9j6f5kp7y43k3879cscrs2tz6y4 '{"GameCounter":{}}' --node https://rpc.constantine.archway.tech:443
    ;;
  "store-user-record")
    archway tx --args '{"AddTopUser": {"user": {"address":"archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y", "name":"Wotori", "score":27000}}}'
    ;;
  "store-user-record-raw")
    archwayd tx wasm execute --chain-id constantine-3 --gas auto --gas-prices $(archwayd q rewards estimate-fees 1 --node 'https://rpc.constantine.archway.tech:443' --output json | jq -r '.gas_unit_price | (.amount + .denom)') --gas-adjustment 1.4 archway1tykvjvpvfqr5g7f8uqqg5du8tp0h99jcgvf05xumtgcq3vf5vajsvp9v2e  '{"AddTopUser": {"user": {"address":"archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y", "name":"Wotori", "score":27000}}}' --from wallet_name --node https://rpc.constantine.archway.tech:443 -y
    ;;
  "add-admin")
    archway tx --args '{"AddAdmin": {"admins": ["archway1uwew6p8k70xa2lkzeujqcw430uky49zthsvc0y"]}}'
    ;;
  *)
    echo "Invalid query. Please enter a valid option."
    ;;
esac
