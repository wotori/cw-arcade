use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{
        User, ADMINS, ARCADE, ARCADE_DENOM, GAME_COUNTER, MAX_TOP_SCORES,
        TOP_USERS,
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
    Ok(Response::new())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        AddAdmin { admins } => exec::add_members(deps, info, admins),
        AddTopUser { user } => exec::add_user(deps, info, user),
        Leave {} => exec::leave(deps, info),
        Play {} => exec::play(deps, info),
    }
}

mod exec {
    use std::collections::BinaryHeap;

    use cosmwasm_std::{coins, BankMsg};

    use super::*;
    use crate::{
        error::ContractError,
        state::{User, TOP_USERS},
    };

    pub fn add_user(
        deps: DepsMut,
        info: MessageInfo,
        user: User,
    ) -> Result<Response, ContractError> {
        println!("sender: {}", info.sender);
        let admins = ADMINS.load(deps.storage)?;
        if admins.contains(&info.sender) {
            let max = MAX_TOP_SCORES.load(deps.storage)?;
            let cur_top_users = TOP_USERS.load(deps.storage)?;
            let mut heap = BinaryHeap::from(cur_top_users);
            if heap.len() < max.into() {
                heap.push(user);
            } else if let Some(lowest_score_user) = heap.peek() {
                if lowest_score_user.score > user.score {
                    // > because the lower the value, the greater it is
                    heap.pop();
                    heap.push(user);
                }
            }
            let vec = heap.into_vec();
            TOP_USERS.save(deps.storage, &vec)?;
            Ok(Response::new())
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
    ) -> Result<Response, ContractError> {
        // increment game counter
        let mut counter = GAME_COUNTER.load(deps.storage)?;
        counter += 1;
        GAME_COUNTER.save(deps.storage, &counter)?;

        //transfer token to admins
        let denom = ARCADE_DENOM.load(deps.storage)?;
        let admins = ADMINS.load(deps.storage)?;
        let tokens = cw_utils::must_pay(&info, &denom)?.u128();
        // TODO: check if tokens equal minimum amount (the price should cover next admin transaction)
        let tokens_peer_admin = tokens / (admins.len() as u128);
        let messages = admins.into_iter().map(|admin| BankMsg::Send {
            to_address: admin.to_string(),
            amount: coins(tokens_peer_admin, &denom),
        });

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("recieved_tokens", tokens.to_string()))
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Greet {} => to_binary(&query::greet()?),
        AdminsList {} => to_binary(&query::admins_list(deps)?),
        ScoreList {} => to_binary(&query::scoreboard(deps)?),
        GameCounter {} => to_binary(&query::game_counter(deps)?),
    }
}

mod query {
    use crate::{
        msg::{AdminsListResp, GameCounterResp, GreetResp, ScoreboardListResp},
        state::TOP_USERS,
    };

    use super::*;

    pub fn greet() -> StdResult<GreetResp> {
        let resp = GreetResp {
            message: "Hello, world!".to_owned(),
        };
        Ok(resp)
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        msg::{
            AdminsListResp, GameCounterResp, InstantiateMsg, QueryMsg,
            ScoreboardListResp,
        },
        state::User,
    };
    use cosmwasm_std::{coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};

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
                    Addr::unchecked("admin2")
                ],
            }
        );
    }

    #[test]
    fn write_score() {
        let mut app = App::default();

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
                    denom: "aconst".to_string(),
                },
                &[],
                "cw-arcade",
                None,
            )
            .unwrap();

        let _resp = app
            .execute_contract(
                Addr::unchecked("wotori"),
                addr.clone(),
                &ExecuteMsg::AddTopUser {
                    user: User {
                        name: "ASHTON".to_string(),
                        address: Addr::unchecked("archway#######"),
                        score: std::cmp::Reverse(300),
                    },
                },
                &[],
            )
            .unwrap();

        let _resp = app
            .execute_contract(
                Addr::unchecked("wotori"),
                addr.clone(),
                &ExecuteMsg::AddTopUser {
                    user: User {
                        name: "WOTORI".to_string(),
                        address: Addr::unchecked("archway#######"),
                        score: std::cmp::Reverse(299),
                    },
                },
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
                scores: vec![User {
                    name: "ASHTON".to_string(),
                    score: std::cmp::Reverse(300),
                    address: Addr::unchecked("archway#######")
                }]
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
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    arcade: "pacman".to_string(),
                    admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                    max_top_score: 10,
                    denom: "aconst".to_string(),
                },
                &[],
                "Pac-Man Arcade",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Play {},
            &coins(5, "aconst"),
        )
        .unwrap();

        assert_eq!(
            app.wrap()
                .query_balance("user", "aconst")
                .unwrap()
                .amount
                .u128(),
            0
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "aconst")
                .unwrap()
                .amount
                .u128(),
            1
        );

        assert_eq!(
            app.wrap()
                .query_balance("admin1", "aconst")
                .unwrap()
                .amount
                .u128(),
            2
        );

        assert_eq!(
            app.wrap()
                .query_balance("admin2", "aconst")
                .unwrap()
                .amount
                .u128(),
            2
        );
    }
}
