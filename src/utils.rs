use std::collections::BinaryHeap;

use cosmwasm_std::{BalanceResponse, BankQuery, DepsMut, Env, StdError};

use crate::state::{User, ARCADE_DENOM};

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
