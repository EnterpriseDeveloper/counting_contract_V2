use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use msg::InstantiateMsg;

mod contract;
pub mod msg;
mod state;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
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
    env: Env,
    info: MessageInfo,
    msg: msg::ExecMsg,
) -> StdResult<Response> {
    use msg::ExecMsg::*;

    match msg {
        Donate {} => contract::exec::donate(deps, info),
        Withdraw {} => contract::exec::withdraw(deps, env, info),
    }
}

#[cfg(test)]
mod test {
    use crate::msg::{ExecMsg, QueryMsg, ValuerResp};

    use super::*;

    use cosmwasm_std::{coins, Addr, Coin, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    const ATOM: &str = "atom";

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10u128, ATOM),
                },
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
    fn donate() {
        let mut app = App::default();
        let sender = Addr::unchecked("sender");

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10u128, ATOM),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(sender, contract_addr.clone(), &ExecMsg::Donate {}, &[])
            .unwrap();

        let resp: ValuerResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValuerResp { value: 0 });
    }

    #[test]
    fn donate_with_funds() {
        let mut app = App::default();
        let sender = app.api().addr_make("sender");

        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, ATOM))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10u128, ATOM),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10u128, ATOM),
        )
        .unwrap();

        let resp: ValuerResp = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValuerResp { value: 1 });

        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            coins(10, ATOM)
        );

        assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]); //NOT WORKING
    }

    #[test]
    fn withdraw() {
        let mut app = App::default();

        let owner = app.api().addr_make("owner");
        let sender1 = app.api().addr_make("sender1");
        let sender2 = app.api().addr_make("sender2");

        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender1, coins(10, ATOM))
                .unwrap();

            router
                .bank
                .init_balance(storage, &sender2, coins(5, ATOM))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    minimal_donation: Coin::new(10u128, ATOM),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender1.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10u128, ATOM),
        )
        .unwrap();

        app.execute_contract(
            sender2.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(5u128, ATOM),
        )
        .unwrap();

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &ExecMsg::Withdraw {},
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap().query_all_balances(owner).unwrap(),
            coins(15, ATOM)
        );
        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            vec![]
        );
        assert_eq!(app.wrap().query_all_balances(sender1).unwrap(), vec![]);
        assert_eq!(app.wrap().query_all_balances(sender2).unwrap(), vec![]);
    }
}
