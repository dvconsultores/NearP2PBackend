use crate::*;


#[near_bindgen]
impl NearP2P {
    pub fn desplegar(&mut self) {
        //let contract_name: AccountId = AccountId::new_unchecked(self.contract_list.get(&env::signer_account_id()).expect("the user does not have a sub contract deployed").to_string());
        /*ext_subcontract::block_balance_token(
            AccountId::new_unchecked(CONTRACT_USDC.to_string()),
            "USDC".to_string(),
            U128(1000000),
            contract_name,
            0,
            Gas(30_000_000_000_000),
        )*/
        let signer: AccountId = AccountId::new_unchecked(env::signer_account_id().as_str().split('.').collect::<Vec<&str>>()[0].to_string());
        let subaccount_id: AccountId = AccountId::new_unchecked(
        format!("2{}.{}", signer, env::current_account_id())
        );
        Promise::new(subaccount_id.clone())
            .create_account()
            .transfer(1600000000000000000000000)
            .deploy_contract(CODE.to_vec())
            .then(ext_subcontract::new(
                env::current_account_id(),
                env::current_account_id(),
                AccountId::new_unchecked("v.nearp2p.testnet".to_string()),
                subaccount_id.clone(),
                0,
                BASE_GAS,
            ));

            ext_usdc::storage_deposit(
                true,
                subaccount_id.clone(),
                AccountId::new_unchecked(CONTRACT_USDC.to_string()),
                100000000000000000000000,
                BASE_GAS,
            );
    }

  

    #[payable]
    pub fn transferir(&mut self, sub_contract: AccountId) -> Promise {
        ext_subcontract::transfer(
            env::signer_account_id(),
            U128(48982376179746617400000006 - 1412439322253799699999999),                            
            U128(0),
            None,
            true,
            "NEAR".to_string(),
            sub_contract,
            1,
            Gas(30_000_000_000_000),
        )
    }

    #[payable]
    pub fn transferir2(&mut self, sub_contract: AccountId) -> Promise {
        ext_usdc::ft_transfer(
            sub_contract,
            U128(1000000),
            None,
            AccountId::new_unchecked(CONTRACT_USDC.to_string()),
            1,
            BASE_GAS,
        )
        /*ext_subcontract::transfer(
            env::signer_account_id(),
            U128(1000000000000000000000000u128),
            U128(1000000000000),
            Some(AccountId::new_unchecked(CONTRACT_USDC.to_string())),
            false,
            "USDC".to_string(),
            sub_contract,
            2,
            Gas(8_000_000_000_000),
        )*/
    }
    
    #[payable]
    pub fn create_subcontract_user(&mut self) -> Promise {
        require!(env::attached_deposit() >= 100000000000000000000000, "you have to deposit a minimum 0.1 NEAR");
        let signer: AccountId = AccountId::new_unchecked(env::signer_account_id().as_str().split('.').collect::<Vec<&str>>()[0].to_string());
        let subaccount_id: AccountId = AccountId::new_unchecked(
        format!("{}.{}", signer, env::current_account_id())
        );
        let result = Promise::new(subaccount_id.clone())
        .create_account()
        .transfer(1600000000000000000000000)
        .deploy_contract(CODE.to_vec())
        .then(ext_subcontract::new(
            env::current_account_id(),
            env::current_account_id(),
            AccountId::new_unchecked("v.nearp2p.testnet".to_string()),
            subaccount_id.clone(),
            0,
            BASE_GAS,
        ));

        self.contract_list_user.insert(env::signer_account_id(), subaccount_id.clone());

        ext_usdc::storage_deposit(
            true,
            subaccount_id,
            AccountId::new_unchecked(CONTRACT_USDC.to_string()),
            100000000000000000000000,
            BASE_GAS,
        );

        result
    }

    
    /// accept offer into the contract
    /// Params: offer_type: 1 = sell, 2 = buy
    #[payable]
    pub fn accept_offer(&mut self, offer_type: i8
        , offer_id: i128
        , amount: U128
        , payment_method: i128
        , datetime: String
        , rate: f64
    ) {
        let attached_deposit = env::attached_deposit();
        if offer_type == 1 {
            require!(attached_deposit >= 1, "you have to deposit a minimum one YoctoNEAR");

            let offer: usize = self.offers_sell.iter().position(|x| x.offer_id == offer_id).expect("Offer sell not found");
            let signer_id = env::signer_account_id();
            require!(self.offers_sell[offer].owner_id != signer_id, "you can not accept your own offer");


            #[warn(unused_assignments)]
            let contract_name: AccountId = AccountId::new_unchecked(self.contract_list.get(&signer_id).expect("the user does not have a sub contract deployed").to_string());
            
            match self.offers_sell[offer].asset.as_str() {
                "NEAR" => {
                    ext_subcontract::block_balance_near(
                        amount,
                        contract_name,
                        0,
                        GAS_FOR_BLOCK,
                    ).then(
                        int_offer::on_accept_offer_sell(
                            offer
                            , amount
                            , payment_method
                            , datetime
                            , rate
                            , env::current_account_id()
                            , 0
                            , BASE_GAS
                    ));
                }, 
                "USDC" => {
                    let result = promise_result_as_success();
                    if result.is_none() {
                        env::panic_str("Failed to activate token".as_ref());
                    }
                    ext_subcontract::block_balance_token(
                        AccountId::new_unchecked(CONTRACT_USDC.to_string()),
                        "USDC".to_string(),
                        amount,
                        contract_name,
                        0,
                        GAS_FOR_BLOCK,
                    ).then(
                        int_offer::on_accept_offer_sell(
                            offer
                            , amount
                            , payment_method
                            , datetime
                            , rate
                            , env::current_account_id()
                            , 0
                            , BASE_GAS
                    ));
                },
                _=> env::panic_str("The requested asset does not exist")
            };
        } else if offer_type == 2 {
            require!(attached_deposit >= 1, "you have to deposit a minimum of one YoctoNear");

            let offer: usize = self.offers_buy.iter().position(|x| x.offer_id == offer_id).expect("Offer buy not found");
            
            require!(self.offers_buy[offer].owner_id != env::signer_account_id(), "you can not accept your own offer");
            require!(self.offers_buy[offer].remaining_amount >= amount.0, "the quantity is greater than the offer buy amount");
                
            let remaining: u128 = self.offers_buy[offer].remaining_amount - amount.0;
            if remaining <= 0 {
                self.offers_buy[offer].status = 2;
            }

            if self.offers_buy[offer].max_limit > remaining {
                self.offers_buy[offer].max_limit = remaining;
            }
            if self.offers_buy[offer].min_limit > remaining {
                match self.offers_buy[offer].asset.as_str() {
                    "NEAR" => self.offers_buy[offer].min_limit = 1000000000000000000000000,
                    _=> self.offers_buy[offer].min_limit = 1000000,
                };
            }

            let fee: u128 = (amount.0 * FEE_TRANSACTION_NEAR) / 100000;
            //let fee_deducted = amount - fee;
            self.offers_buy[offer].remaining_amount = remaining;
            self.order_buy_id += 1;
            let data = OrderObject {
                offer_id: offer_id,
                order_id: self.order_buy_id,
                owner_id: self.offers_buy[offer].owner_id.clone(),
                asset: self.offers_buy[offer].asset.clone(),
                signer_id: env::signer_account_id(),
                exchange_rate: rate.to_string(),
                operation_amount: amount.0,
                amount_delivered: amount.0 - fee,
                fee_deducted: fee,
                payment_method: payment_method,
                fiat_method: self.offers_buy[offer].fiat_method,
                confirmation_owner_id: 0,
                confirmation_signer_id: 0,
                confirmation_current: 0,
                time: self.offers_buy[offer].time,
                datetime: datetime,
                terms_conditions: self.offers_buy[offer].terms_conditions.to_string(),
                status: 1,
            };
            self.orders_buy.push(data);
            //actualizar total ordenes owner_id
            let mut index = self.merchant.iter().position(|x| x.user_id == self.offers_buy[offer].owner_id.clone()).expect("owner not merchant");
            self.merchant[index].total_orders = self.merchant[index].total_orders + 1;
            self.merchant[index].percentaje_completion = (self.merchant[index].orders_completed as f64 / self.merchant[index].total_orders as f64) * 100.0;
            index = self.merchant.iter().position(|x| x.user_id == env::signer_account_id().clone()).expect("owner not merchant");
            self.merchant[index].total_orders = self.merchant[index].total_orders + 1;
            self.merchant[index].percentaje_completion = (self.merchant[index].orders_completed as f64 / self.merchant[index].total_orders as f64) * 100.0;

            env::log_str("Offer buy accepted");
        
        }   else {
            require!(attached_deposit >= 1, "you have to deposit a minimum of one YoctoNear");
            env::panic_str("Invalid offer type");
        }
    }


    #[private]
    pub fn on_accept_offer_sell(&mut self, offer: usize
        , amount: U128
        , payment_method: i128
        , datetime: String
        , rate: f64
    ) { 
        require!(env::predecessor_account_id() == env::current_account_id(), "Only administrators");
        let result = promise_result_as_success();
       
        
        let valid: bool = near_sdk::serde_json::from_slice::<bool>(&result.unwrap()).expect("bool");
        require!(valid, "No balance");

        let remaining: u128 = self.offers_sell[offer].remaining_amount - amount.0;
        if remaining <= 0 {
            self.offers_sell[offer].status = 2;
        }
        
        if self.offers_sell[offer].max_limit > remaining {
            self.offers_sell[offer].max_limit = remaining;
        }
        if self.offers_sell[offer].min_limit > remaining {
            match self.offers_sell[offer].asset.as_str() {
                "NEAR" => self.offers_sell[offer].min_limit = 1000000000000000000000000,
                _=> self.offers_sell[offer].min_limit = 1000000,
            };
        }
        
        let fee: u128 = (amount.0 * FEE_TRANSACTION_NEAR) / 100000;

        self.offers_sell[offer].remaining_amount = remaining;
        self.order_sell_id += 1;
        let data = OrderObject {
            offer_id: self.offers_sell[offer].offer_id,
            order_id: self.order_sell_id,
            owner_id: self.offers_sell[offer].owner_id.clone(),
            asset: self.offers_sell[offer].asset.clone(),
            signer_id: env::signer_account_id(),
            exchange_rate: rate.to_string(),
            operation_amount: amount.0,
            amount_delivered: amount.0 - fee,
            fee_deducted: fee,
            payment_method: payment_method,
            fiat_method: self.offers_sell[offer].fiat_method,
            confirmation_owner_id: 0,
            confirmation_signer_id: 0,
            confirmation_current: 0,
            time: self.offers_sell[offer].time,
            datetime: datetime,
            terms_conditions: self.offers_sell[offer].terms_conditions.to_string(),
            status: 1,
        };
        self.orders_sell.push(data);
        //actualizar total ordenes owner_id
        let mut index = self.merchant.iter().position(|x| x.user_id == self.offers_sell[offer].owner_id.clone()).expect("owner not merchant");
        self.merchant[index].total_orders = self.merchant[index].total_orders + 1;
        self.merchant[index].percentaje_completion = (self.merchant[index].orders_completed as f64 / self.merchant[index].total_orders as f64) * 100.0;
        index = self.merchant.iter().position(|x| x.user_id == env::signer_account_id().clone()).expect("owner not merchant");
        self.merchant[index].total_orders = self.merchant[index].total_orders + 1;
        self.merchant[index].percentaje_completion = (self.merchant[index].orders_completed as f64 / self.merchant[index].total_orders as f64) * 100.0;
        
            
        env::log_str("Offer sell accepted");
    }
}