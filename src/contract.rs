use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::msg;

pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: msg::InstantiateMsg,
) -> StdResult<Response> {
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

pub fn query(_deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;

    match msg {
        Greet {} => to_binary(&query::greet()?),
    }
}

mod query {
    use crate::msg::GreetResp;

    use super::*;

    pub fn greet() -> StdResult<GreetResp> {
        let resp = GreetResp {
            message: "Hello, world!".to_owned(),
        };
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use super::*;
    use crate::msg::{GreetResp, InstantiateMsg, QueryMsg};

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
            admin: "Wotori".to_string(),
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
            admin: "owner".to_string(),
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
}
