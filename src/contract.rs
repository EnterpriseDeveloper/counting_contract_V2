use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Item;

use crate::{
    error::ContractError,
    msg::InstantiateMsg,
    state::{State, OWNER, STATE},
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(
        deps.storage,
        &State {
            counter: 0,
            minimal_donation: msg.minimal_donation,
        },
    )?;
    OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::new())
}

pub fn migrate(mut deps: DepsMut) -> Result<Response, ContractError> {
    let contract = get_contract_version(deps.storage)?;

    if contract.contract != CONTRACT_NAME {
        return Err(ContractError::InvalidName(contract.contract));
    }

    let resp = match contract.version.as_str() {
        "0.1.0" => migrate_0_1_0(deps.branch())?,
        CONTRACT_VERSION => return Ok(Response::new()),
        _ => return Err(ContractError::InvalidVersion(contract.version.to_string())),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(resp)
}

pub fn migrate_0_1_0(deps: DepsMut) -> StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");

    let counter = COUNTER.load(deps.storage)?;
    let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;
    STATE.save(
        deps.storage,
        &State {
            counter: counter,
            minimal_donation: minimal_donation,
        },
    )?;

    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{msg::ValuerResp, state::STATE};

    pub fn value(deps: Deps) -> StdResult<ValuerResp> {
        let value = STATE.load(deps.storage)?.counter;
        Ok(ValuerResp { value })
    }
}

pub mod exec {
    use crate::{
        error::ContractError,
        state::{OWNER, STATE},
    };
    use cosmwasm_std::{BankMsg, DepsMut, Env, MessageInfo, Response, StdResult};

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        // Two way of changing state in contract
        //COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;
        let mut state = STATE.load(deps.storage)?;

        if info.funds.iter().any(|coin| {
            coin.denom == state.minimal_donation.denom
                && coin.amount >= state.minimal_donation.amount
        }) {
            state.counter += 1;
            STATE.save(deps.storage, &state)?;
        }

        let resp = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", state.counter.to_string());

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }

        let funds = deps.querier.query_all_balances(env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: funds,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }
}
