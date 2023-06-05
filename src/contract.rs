use crate::{
    msg,
    state::{ADMINS, ARCADE},
};
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: msg::InstantiateMsg,
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
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: msg::ExecuteMsg,
) -> StdResult<Response> {
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;

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
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use super::*;
    use crate::msg::{AdminsListResp, GreetResp, InstantiateMsg, QueryMsg};

    #[test]
    fn greet_query() {
        // simple test
        let resp = query::greet().unwrap();
        assert_eq!(
            resp,
            msg::GreetResp {
                message: "Hello, world!".to_owned()
            }
        );
    }

    #[test]
    fn greet_query2() {
        // more comprehensive test
        let mut deps = mock_dependencies();
        let env = mock_env();
        let inst_msg = InstantiateMsg {
            arcade: "Pac-Man".to_string(),
            admins: vec!["wotori".to_string(), "senlin".to_string()],
        };

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            inst_msg,
        )
        .unwrap();

        let resp = query(deps.as_ref(), env, msg::QueryMsg::Greet {}).unwrap();
        let resp: GreetResp = from_binary(&resp).unwrap();

        assert_eq!(
            resp,
            GreetResp {
                message: "Hello, world!".to_owned()
            }
        );
    }
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn greet_query3() {
        // the most comprehensive test
        let mut app = App::default();
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let instantiate_msg = InstantiateMsg {
            arcade: "pacman".to_string(),
            admins: vec!["wotori".to_string(), "senlin".to_string()],
        };
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &instantiate_msg,
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: GreetResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::Greet {})
            .unwrap();

        assert_eq!(
            resp,
            GreetResp {
                message: "Hello, world!".to_owned()
            }
        )
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
}
