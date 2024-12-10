#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use execute::{execute_disperse, execute_receive};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "disperse_cosmwasm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Disperse { recipients } => execute_disperse(deps, env, info, recipients),
        ExecuteMsg::Receive(cw20_msg) => execute_receive(deps, env, info, cw20_msg),
    }
}

pub mod execute {
    use cosmwasm_std::{
        from_json, to_json_binary, Addr, BankMsg, Coin, CosmosMsg, StdError, Uint128, WasmMsg,
    };
    use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

    use crate::msg::{Cw20Recipient, ReceiveMsg, Recipient};

    use super::*;

    pub fn execute_disperse(
        _deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        recipients: Vec<Recipient>,
    ) -> Result<Response, ContractError> {
        // Calculate the total amount to be dispersed
        let mut total_amount: Vec<Coin> = vec![];

        for recipient in &recipients {
            for coin in &recipient.amount {
                match total_amount.iter_mut().find(|c| c.denom == coin.denom) {
                    Some(total_coin) => total_coin.amount += coin.amount,
                    None => total_amount.push(coin.clone()),
                }
            }
        }

        // Ensure the sender has sent the correct total amount
        if info.funds != total_amount {
            return Err(ContractError::InvalidFunds {});
        }

        // Create send messages for each recipient
        let messages = recipients.into_iter().map(|recipient| BankMsg::Send {
            to_address: recipient.address,
            amount: recipient.amount,
        });

        Ok(Response::new().add_messages(messages))
    }

    pub fn execute_receive(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        cw20_msg: Cw20ReceiveMsg,
    ) -> Result<Response, ContractError> {
        let cw20_token_address = info.sender;
        let sender = cw20_msg.sender;
        let amount = cw20_msg.amount;

        let msg: ReceiveMsg = from_json(&cw20_msg.msg)?;
        match msg {
            ReceiveMsg::DisperseCw20 { recipients } => {
                execute_cw20_disperse(deps, env, sender, cw20_token_address, amount, recipients)
                    .map_err(ContractError::Std)
            }
        }
    }

    pub fn execute_cw20_disperse(
        _deps: DepsMut,
        _env: Env,
        sender: String,
        cw20_token_address: Addr,
        total_amount: Uint128,
        recipients: Vec<Cw20Recipient>,
    ) -> StdResult<Response> {
        // Calculate the total amount to be dispersed
        let calculated_total: Uint128 = recipients.iter().map(|r| r.amount).sum();

        // Ensure the total amount matches the amount received
        if calculated_total != total_amount {
            return Err(StdError::generic_err(
                "Total amount does not match sum of recipient amounts",
            ));
        }

        // Create transfer messages for each recipient
        let messages: Vec<CosmosMsg> = recipients
            .into_iter()
            .map(|recipient| {
                let transfer_msg = Cw20ExecuteMsg::Transfer {
                    recipient: recipient.address,
                    amount: recipient.amount,
                };

                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: cw20_token_address.clone().into_string(),
                    msg: to_json_binary(&transfer_msg).unwrap(),
                    funds: vec![],
                })
            })
            .collect();

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "cw20_disperse")
            .add_attribute("sender", sender))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

pub mod query {}

#[cfg(test)]
mod tests {
    use crate::msg::{Cw20Recipient, ReceiveMsg, Recipient};

    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coins, to_json_binary, Uint128};
    use cw20::Cw20ReceiveMsg;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};

        let creator = deps.api.addr_make("creator");

        // Create message info
        let info = message_info(&creator, &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(2, res.attributes.len());
        assert_eq!("method", res.attributes[0].key);
        assert_eq!("instantiate", res.attributes[0].value);
        assert_eq!("owner", res.attributes[1].key);
        assert_eq!(creator.into_string(), res.attributes[1].value);
    }

    #[test]
    fn test_disperse_native() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let sender = deps.api.addr_make("sender");

        // First initialize the contract
        let msg = InstantiateMsg {};
        let info = message_info(&creator, &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Now test the disperse function
        let recipients = vec![
            Recipient {
                address: "recipient1".to_string(),
                amount: coins(50, "atom"),
            },
            Recipient {
                address: "recipient2".to_string(),
                amount: coins(30, "atom"),
            },
        ];

        let info = message_info(&sender, &coins(80, "atom"));
        let msg = ExecuteMsg::Disperse { recipients };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check that we have 2 messages
        assert_eq!(2, res.messages.len());
    }

    #[test]
    fn test_disperse_invalid_funds() {
        let mut deps = mock_dependencies();

        let creator = deps.api.addr_make("creator");
        let sender = deps.api.addr_make("sender");
        // First initialize the contract
        let msg = InstantiateMsg {};

        let info = message_info(&creator, &[]);

        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to disperse with incorrect funds
        let recipients = vec![Recipient {
            address: "recipient1".to_string(),
            amount: coins(50, "atom"),
        }];

        let info = message_info(&sender, &coins(40, "atom")); // Sending less than required
        let msg = ExecuteMsg::Disperse { recipients };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        // Verify we get an invalid funds error
        match err {
            ContractError::InvalidFunds {} => {}
            e => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_disperse_cw20() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");

        // First initialize the contract
        let msg = InstantiateMsg {};
        let info = message_info(&creator, &[]);

        let cw20_contract = deps.api.addr_make("cw20_contract");
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Test CW20 disperse
        let cw20_recipients = vec![
            Cw20Recipient {
                address: "recipient1".to_string(),
                amount: Uint128::new(50),
            },
            Cw20Recipient {
                address: "recipient2".to_string(),
                amount: Uint128::new(30),
            },
        ];

        let total_amount = Uint128::new(80);
        let cw20_msg = Cw20ReceiveMsg {
            sender: "user".to_string(),
            amount: total_amount,
            msg: to_json_binary(&ReceiveMsg::DisperseCw20 {
                recipients: cw20_recipients,
            })
            .unwrap(),
        };


        let info = message_info(&cw20_contract, &[]);
        let msg = ExecuteMsg::Receive(cw20_msg);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check that we have 2 messages (one for each recipient)
        assert_eq!(2, res.messages.len());
    }
}
