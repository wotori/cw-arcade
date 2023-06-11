use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{
        User, ADMINS, ARCADE, ARCADE_DENOM, GAME_COUNTER, MAX_TOP_SCORES,
        PRICE_PEER_GAME, TOP_USERS, TOTAL_PRICE_DISTRIBUTED,
    },
};
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admins: StdResult<Vec<_>> = msg
        .admins
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr))
        .collect();
    ADMINS.save(deps.storage, &admins?)?;
    ARCADE.save(deps.storage, &msg.arcade)?;
    MAX_TOP_SCORES.save(deps.storage, &msg.max_top_score)?;
    TOP_USERS.save(deps.storage, &Vec::<User>::new())?;
    GAME_COUNTER.save(deps.storage, &0)?;
    ARCADE_DENOM.save(deps.storage, &msg.denom)?;
    PRICE_PEER_GAME.save(deps.storage, &msg.price_peer_game)?;
    TOTAL_PRICE_DISTRIBUTED.save(deps.storage, &0)?;
    Ok(Response::new())
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        AddAdmin { admins } => exec::add_members(deps, info, admins),
        AddTopUser { user } => exec::add_user(deps, info, user, env),
        Leave {} => exec::leave(deps, info),
        Play {} => exec::play(deps, info, env),
        UpdatePrice { price } => exec::update_price(deps, price),
    }
}

mod exec {
    use std::collections::BinaryHeap;

    use cosmwasm_std::{coins, BankMsg};

    use super::*;
    use crate::{
        error::ContractError,
        state::{User, TOP_USERS},
        utils::{user_is_top},
    };
    use crate::utils::send_coins;

    pub fn add_user(
        mut deps: DepsMut,
        info: MessageInfo,
        user: User,
        env: Env,
    ) -> Result<Response, ContractError> {
        let admins = ADMINS.load(deps.storage)?;
        if admins.contains(&info.sender) {
            let mut resp = Response::new();
            let max = MAX_TOP_SCORES.load(deps.storage)?;
            let cur_top_users = TOP_USERS.load(deps.storage)?;
            let mut heap = BinaryHeap::from(cur_top_users);
            if heap.len() < max.into() {
                heap.push(user);
            } else if let Some(lowest_score_user) = heap.peek() {
                if lowest_score_user.score > user.score {
                    // check if user top score for send prize pool to his account
                    if user_is_top(&heap, &user) {
                        // send all accumulated coins to the winner.
                        resp = send_coins(&mut deps, &user, env)?;
                    }

                    // adding user to top score list // > used here because the lower the value, the greater it is
                    heap.pop();
                    heap.push(user);
                }
            }

            let vec = heap.into_vec();
            TOP_USERS.save(deps.storage, &vec)?;
            Ok(resp)
        } else {
            Err(ContractError::Unauthorized {
                sender: (info.sender),
            })
        }
    }

    pub fn add_members(
        deps: DepsMut,
        info: MessageInfo,
        admins: Vec<String>,
    ) -> Result<Response, ContractError> {
        let mut curr_admins = ADMINS.load(deps.storage)?;
        if !curr_admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
        }
        let admins: StdResult<Vec<_>> = admins
            .into_iter()
            .map(|addr| deps.api.addr_validate(&addr))
            .collect();
        curr_admins.append(&mut admins?);
        ADMINS.save(deps.storage, &curr_admins)?;
        Ok(Response::new())
    }

    pub fn leave(
        deps: DepsMut,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        ADMINS.update(deps.storage, move |admins| -> StdResult<_> {
            let admins = admins
                .into_iter()
                .filter(|admin| *admin != info.sender)
                .collect();
            Ok(admins)
        })?;

        Ok(Response::new())
    }

    pub fn play(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
    ) -> Result<Response, ContractError> {
        let price_peer_game = PRICE_PEER_GAME.load(deps.storage)?;
        //transfer token to admins
        let denom = ARCADE_DENOM.load(deps.storage)?;
        let mut admins = ADMINS.load(deps.storage)?;
        let self_address = env.contract.address;
        admins.push(self_address.clone());
        let tokens = cw_utils::must_pay(&info, &denom)?.u128();
        // TODO: check if tokens equal minimum amount (the price should cover next admin transaction)
        if tokens >= price_peer_game {
            let tokens_peer_admin = tokens / (admins.len() as u128);
            let left_tokens = tokens % (admins.len() as u128);
            let messages = admins.into_iter().map(|admin| BankMsg::Send {
                // this send is works as expected and testd
                to_address: admin.to_string(),
                amount: coins(tokens_peer_admin, &denom),
            });
            if left_tokens > 0 {
                // send tokens to arcade contract
                // TODO: check if it really works
                BankMsg::Send {
                    to_address: self_address.to_string(),
                    amount: coins(tokens_peer_admin, &denom),
                };
            }

            // increment game counter
            let mut counter = GAME_COUNTER.load(deps.storage)?;
            counter += 1;
            GAME_COUNTER.save(deps.storage, &counter)?;

            Ok(Response::new()
                .add_messages(messages)
                .add_attribute("recieved_tokens", tokens.to_string()))
        } else {
            // return coins to sender
            // TODO: need to be tested (not sure if it works) // potentially it is not even need to be here
            BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(tokens, &denom),
            };
            Err(ContractError::Unauthorized {
                sender: info.sender,
            })
        }
    }

    pub fn update_price(
        deps: DepsMut,
        price: u128,
    ) -> Result<Response, ContractError> {
        PRICE_PEER_GAME.save(deps.storage, &price)?;
        Ok(Response::new())
    }
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        AdminsList {} => to_binary(&query::admins_list(deps)?),
        ScoreList {} => to_binary(&query::scoreboard(deps)?),
        GameCounter {} => to_binary(&query::game_counter(deps)?),
        Price {} => to_binary(&query::get_price(deps)?),
        PrizePool {} => to_binary(&query::prize_pool(deps, env)?),
        TotalDistributed {} => to_binary(&query::total_distributed(deps)?),
    }
}

mod query {
    use crate::msg::{PrizePoolResp, TotalDistributionResp};
    use crate::{
        msg::{
            AdminsListResp, GameCounterResp, GamePriceResp, ScoreboardListResp,
        },
        state::TOP_USERS,
    };
    use cosmwasm_std::{BalanceResponse, BankQuery};

    use super::*;

    pub fn scoreboard(deps: Deps) -> StdResult<ScoreboardListResp> {
        let scoreboard = TOP_USERS.load(deps.storage)?;
        let resp = ScoreboardListResp { scores: scoreboard };
        Ok(resp)
    }

    pub fn admins_list(deps: Deps) -> StdResult<AdminsListResp> {
        let admins = ADMINS.load(deps.storage)?;
        let resp = AdminsListResp { admins };
        Ok(resp)
    }

    pub fn game_counter(deps: Deps) -> StdResult<GameCounterResp> {
        let counter = GAME_COUNTER.load(deps.storage)?;
        let resp = GameCounterResp {
            game_counter: counter,
        };
        Ok(resp)
    }

    pub fn get_price(deps: Deps) -> StdResult<GamePriceResp> {
        let price = PRICE_PEER_GAME.load(deps.storage)?;
        let resp = GamePriceResp { price };
        Ok(resp)
    }

    pub fn prize_pool(deps: Deps, env: Env) -> StdResult<PrizePoolResp> {
        // returns available amount of coins that will be distributed to the winner that hist the scoreboard
        // TODO: replace with query_arcade_balance function (code duplicate)
        let denom = ARCADE_DENOM.load(deps.storage)?;
        let address = env.contract.address.to_string();
        let balance_query = BankQuery::Balance { denom, address };
        let balance_response: BalanceResponse =
            deps.querier.query(&balance_query.into())?;
        let balance_u128 = balance_response.amount.amount.u128();
        Ok(PrizePoolResp {
            prize_pool: balance_u128,
        })
    }

    pub fn total_distributed(deps: Deps) -> StdResult<TotalDistributionResp> {
        let total_distributed = TOTAL_PRICE_DISTRIBUTED.load(deps.storage)?;
        Ok(TotalDistributionResp { total_distributed })
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Reverse;
    use super::*;
    use crate::{
        msg::{
            AdminsListResp, GameCounterResp, InstantiateMsg, PrizePoolResp,
            QueryMsg, ScoreboardListResp,
        },
        state::User,
    };
    use cosmwasm_std::{coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use crate::msg::GamePriceResp;

    #[test]
    fn play() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user"),
                    coins(5, "aconst"),
                )
                .unwrap()
        });
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("wotori"),
                &InstantiateMsg {
                    arcade: "pacman".to_string(),
                    admins: vec!["admin1".to_owned()],
                    max_top_score: 10,
                    price_peer_game: 1,
                    denom: "aconst".to_string(),
                },
                &[],
                "Pac-Man".to_string(),
                None,
            )
            .unwrap();

        let resp: GameCounterResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::GameCounter {})
            .unwrap();
        assert_eq!(resp, GameCounterResp { game_counter: 0 });

        let _resp = app
            .execute_contract(
                Addr::unchecked("user"),
                addr.clone(),
                &ExecuteMsg::Play {},
                &coins(5, "aconst"),
            )
            .unwrap();

        let resp: GameCounterResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::GameCounter {})
            .unwrap();
        assert_eq!(resp, GameCounterResp { game_counter: 1 })
    }

    #[test]
    fn instantiation() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    arcade: "pacman".to_string(),
                    admins: vec![],
                    max_top_score: 10,
                    price_peer_game: 1,
                    denom: "aconst".to_string(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();
        assert_eq!(resp, AdminsListResp { admins: vec![] });

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    arcade: "Pac-Man".to_string(),
                    admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                    max_top_score: 10,
                    price_peer_game: 1,
                    denom: "aconst".to_string(),
                },
                &[],
                "Contract 2",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(
            resp,
            AdminsListResp {
                admins: vec![
                    Addr::unchecked("admin1"),
                    Addr::unchecked("admin2"),
                ],
            }
        );
    }

    fn test_user(name: &str, score: u16, addr: String) -> User {
        User {
            name: name.to_string(),
            address: Addr::unchecked(addr),
            score: Reverse(score),
        }
    }

    #[test]
    fn write_score() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("arcade"),
                    coins(100000, "aconst"),
                )
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let max = 1;
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("wotori"),
                &InstantiateMsg {
                    admins: vec!["wotori".to_string()],
                    arcade: "Pac-Man".to_string(),
                    max_top_score: max,
                    price_peer_game: 1,
                    denom: "aconst".to_string(),
                },
                &[],
                "cw-arcade",
                None,
            )
            .unwrap();

        let user1 = test_user("user1", 299, "test".to_string());
        let _resp = app
            .execute_contract(
                Addr::unchecked("wotori"),
                addr.clone(),
                &ExecuteMsg::AddTopUser { user: user1.clone() },
                &[],
            )
            .unwrap();

        let user2 = test_user("user2", 300, "test".to_string());
        let _resp = app
            .execute_contract(
                Addr::unchecked("wotori"),
                addr.clone(),
                &ExecuteMsg::AddTopUser { user: user2.clone() },
                &[],
            )
            .unwrap();

        let resp: ScoreboardListResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::ScoreList {})
            .unwrap();

        assert_eq!(
            resp,
            ScoreboardListResp {
                scores: vec![user2]
            }
        );

        assert_eq!(resp.scores.len(), usize::from(max))
    }

    #[test]
    fn unauthorized() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec![],
                    arcade: "Pac-Man".to_string(),
                    max_top_score: 10,
                    price_peer_game: 1,
                    denom: "aconst".to_string(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked("user"),
                addr,
                &ExecuteMsg::AddAdmin {
                    admins: vec!["user".to_owned()],
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                sender: Addr::unchecked("user")
            },
            err.downcast().unwrap()
        );
    }

    #[test]
    fn play_with_pay() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user1"),
                    coins(100000, "aconst"),
                )
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    arcade: "pacman".to_string(),
                    admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                    max_top_score: 1,
                    price_peer_game: 123,
                    denom: "aconst".to_string(),
                },
                &[],
                "Pac-Man Arcade",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user1"),
            addr.clone(),
            &ExecuteMsg::Play {},
            &coins(333, "aconst"),
        )
            .unwrap();

        let resp: PrizePoolResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::PrizePool {})
            .unwrap();
        assert_eq!(resp, PrizePoolResp { prize_pool: 111 });

        let price: GamePriceResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::Price {})
            .unwrap();
        assert_eq!(price, GamePriceResp { price: 123 });

        let price: GameCounterResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::GameCounter {})
            .unwrap();
        assert_eq!(price, GameCounterResp { game_counter: 1 });

        assert_eq!(
            app.wrap()
                .query_balance("user1", "aconst")
                .unwrap()
                .amount
                .u128(),
            99667
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "aconst")
                .unwrap()
                .amount
                .u128(),
            111
        );

        assert_eq!(
            app.wrap()
                .query_balance("admin1", "aconst")
                .unwrap()
                .amount
                .u128(),
            111
        );

        assert_eq!(
            app.wrap()
                .query_balance("admin2", "aconst")
                .unwrap()
                .amount
                .u128(),
            111
        );

        let arcade_balance = app.wrap()
            .query_balance(addr.clone(), "aconst")
            .unwrap()
            .amount
            .u128();

        assert_eq!(arcade_balance, 111);

        let user2_balance_1 = app.wrap()
            .query_balance("test2", "aconst")
            .unwrap()
            .amount
            .u128();

        assert_eq!(user2_balance_1, 0);

        let user1 = test_user("user1", 299, "test1".to_string());
        let _resp = app
            .execute_contract(
                Addr::unchecked("admin1"),
                addr.clone(),
                &ExecuteMsg::AddTopUser { user: user1.clone() },
                &[],
            )
            .unwrap();

        let user2 = test_user("user2", 300, "test2".to_string());
        let _resp = app
            .execute_contract(
                Addr::unchecked("admin1"),
                addr.clone(),
                &ExecuteMsg::AddTopUser { user: user2.clone() },
                &[],
            )
            .unwrap();

        let arcade_balance_2 = app.wrap()
            .query_balance(addr.clone(), "aconst")
            .unwrap()
            .amount
            .u128();

        let user2_balance_2 = app.wrap()
            .query_balance("test2", "aconst")
            .unwrap()
            .amount
            .u128();

        // arcade balance should be empty as the whole balance should be sent to the winner user
        assert_eq!(arcade_balance_2, 0);
        assert_eq!(user2_balance_2, 111)
    }
}
