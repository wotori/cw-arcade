use std::collections::BinaryHeap;

use cosmwasm_std::{BalanceResponse, BankMsg, BankQuery, DepsMut, Env, Response, StdError};
use crate::error::ContractError;
use cosmwasm_std::{coins};
use crate::state::{User, ARCADE_DENOM, TOTAL_PRICE_DISTRIBUTED};

pub fn user_is_top(heap: &BinaryHeap<User>, user: &User) -> bool {
    if let Some(highest_score_user) =
        heap.iter().min_by(|a, b| a.score.cmp(&b.score))
    {
        if highest_score_user > user {
            // > as score is Reverse<u16>
            return true;
        }
    }
    return false;
}

pub fn query_arcade_balance(
    deps: &DepsMut,
    env: Env,
) -> Result<u128, StdError> {
    let denom = ARCADE_DENOM.load(deps.storage)?;
    let address = env.contract.address.to_string();
    let balance_query = BankQuery::Balance { denom, address };
    let balance_response: BalanceResponse =
        deps.querier.query(&balance_query.into())?;
    let balance_u128 = balance_response.amount.amount.u128();
    Ok(balance_u128)
}

pub fn send_coins(deps: &mut DepsMut, user: &User, env: Env) -> Result<Response, ContractError> {
    let denom = ARCADE_DENOM.load(deps.storage).expect("Denom was not fetched");
    let balance = query_arcade_balance(deps, env).expect("Balance was not fetched");
    if balance == 0 {
        // return Err(ContractError::NoFunds); // this break the future calls logic
        return Ok(Response::new());
    }
    let distributed =
        TOTAL_PRICE_DISTRIBUTED.load(deps.storage);
    let total_distributed = distributed.unwrap() + balance;
    TOTAL_PRICE_DISTRIBUTED
        .save(deps.storage, &total_distributed).expect("Total price was not updated");

    let msg = BankMsg::Send {
        to_address: user.address.to_string(),
        amount: coins(
            balance.into(),
            &denom,
        ),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "send_coins")
        .add_attribute("sender", user.address.clone())
        .add_attribute("amount", balance.to_string());

    Ok(res)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;

    use super::*;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    fn test_user(name: &str, score: u16) -> User {
        User {
            name: name.to_string(),
            address: Addr::unchecked("test"),
            score: Reverse(score),
        }
    }

    #[test]
    fn test_user_is_top() {
        let mut heap = BinaryHeap::new();
        let user1 = test_user("user1", 10);
        let user2 = test_user("user2", 20);
        let user3 = test_user("user3", 30);

        heap.push(user1.clone());
        heap.push(user2.clone());

        assert!(!user_is_top(&heap, &user1));
        assert!(!user_is_top(&heap, &user2));
        assert!(user_is_top(&heap, &user3));

        heap.push(user3.clone());
        assert!(!user_is_top(&heap, &user3));
    }
}
