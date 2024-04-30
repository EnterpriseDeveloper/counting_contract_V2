use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult,
};

mod contract;
pub mod msg;
mod state;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    contract::instantiate(deps)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, _msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;

    match _msg {
        Value {} => to_json_binary(&contract::query::value(deps)?),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: msg::ExecMsg,
) -> StdResult<Response> {
    use msg::ExecMsg::*;

    match msg {
        Poke {} => contract::exec::poke(deps, info),
    }
}

#[cfg(test)]
mod test {
    use crate::msg::{ExecMsg, QueryMsg, ValuerResp};

    use super::*;

    use cosmwasm_std::{Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &QueryMsg::Value {},
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        let resp: ValuerResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValuerResp { value: 0 });
    }

    #[test]
    fn poke() {
        let mut app = App::default();
        let sender = Addr::unchecked("sender");

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &QueryMsg::Value {},
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(sender, contract_addr.clone(), &ExecMsg::Poke {}, &[])
            .unwrap();

        let resp: ValuerResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValuerResp { value: 1 });
    }
}
