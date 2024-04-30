use cosmwasm_std::{DepsMut, Response, StdResult};

use crate::{
    msg::InstantiateMsg,
    state::{COUNTER, MINIMAL_DONATION},
};

pub fn instantiate(deps: DepsMut, msg: InstantiateMsg) -> StdResult<Response> {
    COUNTER.save(deps.storage, &0)?;
    MINIMAL_DONATION.save(deps.storage, &msg.minimal_donation)?;
    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{msg::ValuerResp, state::COUNTER};

    pub fn value(deps: Deps) -> StdResult<ValuerResp> {
        let value = COUNTER.load(deps.storage)?;
        Ok(ValuerResp { value })
    }
}

pub mod exec {
    use crate::state::{COUNTER, MINIMAL_DONATION};
    use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        // Two way of changing state in contract
        //COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;
        let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;

        let mut value = COUNTER.load(deps.storage)?;

        if info.funds.iter().any(|coin| {
            coin.denom == minimal_donation.denom && coin.amount >= minimal_donation.amount
        }) {
            value += 1;
            COUNTER.save(deps.storage, &value)?;
        }

        let resp = Response::new()
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", value.to_string());

        Ok(resp)
    }
}
