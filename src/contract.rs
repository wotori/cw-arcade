use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{ADMINS, ARCADE},
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
    }
}

mod exec {
    use std::collections::BinaryHeap;

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
        // todo: add check if contract caller is an admin wit info
        println!("sender: {}", info.sender);
        let cur_top_users = TOP_USERS.load(deps.storage)?;
        let mut heap = BinaryHeap::from(cur_top_users);
        if heap.len() < 100 {
            // TODO: add to contract instantiation state
            heap.push(user);
        } else if let Some(lowest_score_user) = heap.peek() {
            if lowest_score_user.score < user.score {
                heap.pop();
                heap.push(user);
            }
        }
        let vec = heap.into_vec();
        TOP_USERS.save(deps.storage, &vec)?;
        Ok(Response::new())
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
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Greet {} => to_binary(&query::greet()?),
        AdminsList {} => to_binary(&query::admins_list(deps)?),
    }
}

mod query {
    use crate::msg::{AdminsListResp, GreetResp};

    use super::*;

    pub fn greet() -> StdResult<GreetResp> {
        let resp = GreetResp {
            message: "Hello, world!".to_owned(),
        };
        Ok(resp)
    }

    pub fn admins_list(deps: Deps) -> StdResult<AdminsListResp> {
        let admins = ADMINS.load(deps.storage)?;
        let resp = AdminsListResp { admins };
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{AdminsListResp, InstantiateMsg, QueryMsg};
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

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
                },
                &[],
                "Contract 2",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
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
}
