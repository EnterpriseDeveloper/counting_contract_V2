use cosmwasm_std::{coins, Addr, Coin, Empty};
use cw_multi_test::{App, Contract, ContractWrapper};

use crate::{error::ContractError, execute, instantiate, multitest::CountingContract, query};

fn counting_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

const ATOM: &str = "atom";

#[test]
fn query_value() {
    let mut app = App::default();

    let sender = Addr::unchecked("sender");
    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        Coin::new(10u128, ATOM),
    )
    .unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn donate() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        Coin::new(10u128, ATOM),
    )
    .unwrap();

    contract.donate(&mut app, &sender, &[]).unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
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

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        Coin::new(10u128, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10u128, ATOM))
        .unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 1);

    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(10, ATOM)
    );

    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
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

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        "Counting contract",
        Coin::new(10u128, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender1, &coins(10u128, ATOM))
        .unwrap();

    contract
        .donate(&mut app, &sender2, &coins(5u128, ATOM))
        .unwrap();

    contract.withdraw(&mut app, &owner).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(15, ATOM)
    );
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        vec![]
    );
    assert_eq!(app.wrap().query_all_balances(sender1).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender2).unwrap(), vec![]);
}

#[test]
fn unauthorized_withdraw() {
    let mut app = App::default();

    let owner = app.api().addr_make("owner");
    let member = app.api().addr_make("member");

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        "Counting contract",
        Coin::new(10u128, ATOM),
    )
    .unwrap();

    let err = contract.withdraw(&mut app, &member).unwrap_err();

    assert_eq!(
        err,
        ContractError::Unauthorized {
            owner: owner.into()
        },
    );
}
