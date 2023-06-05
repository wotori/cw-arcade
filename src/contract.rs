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
    use crate::msg::{GreetResp, InstantiateMsg};

    #[test]
    fn greet_query() {
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
        )
    }
}
