#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, to_json_binary, Empty, Uint128};
    use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
    use cw_multi_test::{AppBuilder, Contract, ContractWrapper, Executor};

    use crate::{
        contract::{execute, instantiate, query},
        msg::{Cw20Recipient, InstantiateMsg, ReceiveMsg},
    };

    // Function to create your disperse contract
    fn contract_disperse() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            execute,
            instantiate,
            query, // Replace with your query function
        );
        Box::new(contract)
    }

    // Function to create the cw20-base contract
    fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    #[test]
    fn test_cw20_disperse_integration() {
        let deps = mock_dependencies();

        let sender = deps.api.addr_make("sender");
        let recipient1 = deps.api.addr_make("recipient1");
        let recipient2 = deps.api.addr_make("recipient2");

        // Initialize the app
        let mut app = AppBuilder::new().build(|_router, _api, _storage| {});

        // Store the code IDs for the contracts
        let cw20_code_id = app.store_code(contract_cw20());
        let disperse_code_id = app.store_code(contract_disperse());

        // Instantiate the CW20 contract with initial balance for the sender
        let initial_balances = vec![Cw20Coin {
            address: sender.to_string(),
            amount: Uint128::new(3000),
        }];
        let cw20_inst_msg = cw20_base::msg::InstantiateMsg {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 6,
            initial_balances,
            mint: None,
            marketing: None,
        };
        let cw20_contract_addr = app
            .instantiate_contract(
                cw20_code_id,
                sender.clone(),
                &cw20_inst_msg,
                &[],
                "CW20",
                None,
            )
            .unwrap();

        let cw20_contract = Cw20Contract(cw20_contract_addr.clone());

        // Instantiate your disperse contract
        let disperse_inst_msg = InstantiateMsg {}; // Update if you have fields
        let disperse_contract_addr = app
            .instantiate_contract(
                disperse_code_id,
                sender.clone(),
                &disperse_inst_msg,
                &[],
                "Disperse",
                None,
            )
            .unwrap();

        // Prepare the disperse message
        let recipients = vec![
            Cw20Recipient {
                address: recipient1.to_string(),
                amount: Uint128::new(1000),
            },
            Cw20Recipient {
                address: recipient2.to_string(),
                amount: Uint128::new(2000),
            },
        ];

        let disperse_msg = ReceiveMsg::DisperseCw20 { recipients };
        let cw20_send_msg = Cw20ExecuteMsg::Send {
            contract: disperse_contract_addr.to_string(),
            amount: Uint128::new(3000),
            msg: to_json_binary(&disperse_msg).unwrap(),
        };

        // Execute the send message from the sender to the disperse contract
        app.execute_contract(
            sender.clone(),
            cw20_contract_addr.clone(),
            &cw20_send_msg,
            &[],
        )
        .unwrap();

        // Check balances after dispersal
        let balance_sender = cw20_contract.balance(&app.wrap(), sender.clone()).unwrap();
        assert_eq!(balance_sender, Uint128::zero());

        let balance_recipient1 = cw20_contract
            .balance(&app.wrap(), recipient1.clone())
            .unwrap();
        assert_eq!(balance_recipient1, Uint128::new(1000));

        let balance_recipient2 = cw20_contract
            .balance(&app.wrap(), recipient2.clone())
            .unwrap();
        assert_eq!(balance_recipient2, Uint128::new(2000));

        // Ensure the disperse contract has zero balance
        let balance_disperse = cw20_contract
            .balance(&app.wrap(), disperse_contract_addr.clone())
            .unwrap();
        assert_eq!(balance_disperse, Uint128::zero());
    }
}
