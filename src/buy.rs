use crate::*;


#[near_bindgen]
impl NearP2P {
    /// Returns the order object loaded in contract
    /// Params: campo: String, valor: String
    pub fn get_offers_buy(self, amount: Option<f64>,
        fiat_method: Option<i128>,
        payment_method: Option<i128>,
        is_merchant: Option<bool>,
        owner_id: Option<AccountId>,
        status: Option<i8>,
        offer_id: Option<i128>,
        asset: Option<String>,
        signer_id: Option<AccountId>,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> SearchOfferObject {
        if self.offers_buy.len() > 0 {
            search_offer(self.offers_buy, amount, fiat_method, payment_method, is_merchant, owner_id, status, offer_id, asset, signer_id, from_index, limit)
        } else {
            SearchOfferObject {
                total_index: 0,
                data: [].to_vec(),
            }
        }
    }


    /// Set the offer buy object into the contract
    /// Params: owner_id: String, asset: String, exchange_rate: String, amount: String
    /// min_limit: String, max_limit: String, payment_method_id: String, status: i8
    /// This is a list of offers for buying operations, will be called by the user
    #[payable]
    pub fn set_offers_buy(&mut self, owner_id: AccountId
        , asset: String
        , exchange_rate: String
        , amount: f64
        , min_limit: f64
        , max_limit: f64
        , payment_method: Vec<PaymentMethodsOfferObject>
        , fiat_method: i128
        , time: i64
        , terms_conditions: String
    ) -> i128{
        let attached_deposit = env::attached_deposit();
        assert!(
            (attached_deposit as f64 / YOCTO_NEAR as f64) as f64 >= amount,
            "the deposit attached is less than the quantity supplied : {}",
            amount
        );
        self.offer_buy_id += 1;
        let index = self.merchant.iter().position(|x| x.user_id == owner_id).expect("the user is not in the list of users");

        let data = OfferObject {
            offer_id: self.offer_buy_id,
            owner_id: owner_id,
            asset: String::from(asset),
            exchange_rate: String::from(exchange_rate),
            amount: amount,
            remaining_amount: amount,
            min_limit: min_limit,
            max_limit: max_limit,
            payment_method: payment_method,
            fiat_method: fiat_method,
            is_merchant: self.merchant[index].is_merchant,
            time: time,
            terms_conditions: terms_conditions,
            status: 1,
        };
        self.offers_buy.push(data);
        env::log_str("Offer Created");
        self.offer_buy_id
    }

    #[payable]
    pub fn put_offers_buy(&mut self, offer_id: i128
        , asset: Option<String>
        , exchange_rate: Option<String>
        , remaining_amount: Option<f64>
        , min_limit: Option<f64>
        , max_limit: Option<f64>
        , payment_method: Option<Vec<PaymentMethodsOfferObject>>
        , fiat_method: Option<i128>
        , time: Option<i64>
        , terms_conditions: Option<String>
    ) -> OfferObject {
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= 1,
            "you have to deposit a minimum of one yoctoNear"
        );
        let offer = self.offers_buy.iter().position(|x| x.offer_id == offer_id && x.owner_id == env::signer_account_id()).expect("Offer not found");
        if asset.is_some() {
            self.offers_buy[offer].asset = asset.unwrap();
        }
        if exchange_rate.is_some() {
            self.offers_buy[offer].exchange_rate = exchange_rate.unwrap();
        }
        if remaining_amount.is_some() {
            if remaining_amount.unwrap() < self.offers_buy[offer].remaining_amount {
                let diff_return = self.offers_buy[offer].remaining_amount - remaining_amount.unwrap();

                #[warn(unused_assignments)]
                let contract_name: AccountId = AccountId::new_unchecked(self.contract_list.get(&self.offers_buy[offer].owner_id.clone()).expect("the user does not have a sub contract deployed").to_string());
                
                let contract_ft: Option<AccountId>;
                let fee_deducted: u128;
                let operation_amount: u128;
                if self.offers_buy[offer].asset == "USDC".to_string() {
                    contract_ft = Some(AccountId::new_unchecked(CONTRACT_USDC.to_string()));
                    fee_deducted = 0;
                    operation_amount = (diff_return as f64) as u128;
                } else {
                    contract_ft = None;
                    fee_deducted = 0;
                    operation_amount = (diff_return * YOCTO_NEAR as f64) as u128;
                }   
                
                ext_subcontract::transfer(
                    self.offers_buy[offer].owner_id.clone(),
                    operation_amount,
                    fee_deducted,
                    contract_ft,
                    contract_name,
                    0,
                    GAS_FOR_TRANSFER,
                );

            } else if remaining_amount.unwrap() > self.offers_buy[offer].remaining_amount {
                assert!(
                    remaining_amount.unwrap() <= self.offers_buy[offer].amount,
                    "the remaining amount is greater than the original amount of the offer, original amount {}, remaining amount {}.",
                    self.offers_buy[offer].amount, remaining_amount.unwrap()
                );
                let diff_pay = self.offers_buy[offer].remaining_amount - remaining_amount.unwrap();
                assert!(
                    (attached_deposit as f64 / YOCTO_NEAR as f64) as f64 >= diff_pay,
                    "the deposit attached is less than the remaining supplied : {}",
                    diff_pay
                );  
            }
            self.offers_buy[offer].remaining_amount = remaining_amount.unwrap();
        }
        if min_limit.is_some() {
            self.offers_buy[offer].min_limit = min_limit.unwrap();
        }
        if max_limit.is_some() {
            self.offers_buy[offer].max_limit = max_limit.unwrap();
        }
        if payment_method.is_some() {
            self.offers_buy[offer].payment_method = payment_method.unwrap().iter().map(|x| PaymentMethodsOfferObject {id: x.id, payment_method: x.payment_method.clone()}).collect();
        }
        if fiat_method.is_some() {
            self.offers_buy[offer].fiat_method = fiat_method.unwrap();
        }
        if time.is_some() {
            self.offers_buy[offer].time = time.unwrap();
        }
        if terms_conditions.is_some() {
            self.offers_buy[offer].terms_conditions = terms_conditions.unwrap();
        }
        
        env::log_str("Offer updated");
        OfferObject {
            offer_id: offer_id,
            owner_id: self.offers_buy[offer].owner_id.clone(),
            asset: String::from(self.offers_buy[offer].asset.clone()),
            exchange_rate: String::from(self.offers_buy[offer].exchange_rate.clone()),
            amount: self.offers_buy[offer].amount,
            remaining_amount: self.offers_buy[offer].remaining_amount,
            min_limit: self.offers_buy[offer].min_limit,
            max_limit: self.offers_buy[offer].max_limit,
            payment_method: self.offers_buy[offer].payment_method.iter().map(|x| PaymentMethodsOfferObject {id: x.id, payment_method: x.payment_method.clone()}).collect(),
            fiat_method: self.offers_buy[offer].fiat_method,
            is_merchant: self.offers_buy[offer].is_merchant,
            time: self.offers_buy[offer].time,
            terms_conditions: String::from(self.offers_buy[offer].terms_conditions.clone()),
            status: self.offers_buy[offer].status,
        }
    }
    

    pub fn delete_offers_buy(&mut self, offer_id: i128) {
        let offer = self.offers_buy.iter().position(|x| x.offer_id == offer_id && x.owner_id == env::signer_account_id()).expect("Offer not found");
        #[warn(unused_assignments)]
        let contract_name: AccountId = AccountId::new_unchecked(self.contract_list.get(&self.offers_buy[offer].owner_id.clone()).expect("the user does not have a sub contract deployed").to_string());
        
        let contract_ft: Option<AccountId>;
        let fee_deducted: u128;
        let operation_amount: u128;
        if self.offers_buy[offer].asset == "USDC".to_string() {
            contract_ft = Some(AccountId::new_unchecked(CONTRACT_USDC.to_string()));
            fee_deducted = 0;
            operation_amount = (self.offers_buy[offer].remaining_amount as f64) as u128;
        } else {
            contract_ft = None;
            fee_deducted = 0;
            operation_amount = (self.offers_buy[offer].remaining_amount * YOCTO_NEAR as f64) as u128;
        }   
        
        ext_subcontract::transfer(
            self.offers_buy[offer].owner_id.clone(),
            operation_amount,
            fee_deducted,
            contract_ft,
            contract_name,
            0,
            GAS_FOR_TRANSFER,
        );
        
        self.offers_buy.remove(offer);
        env::log_str("Offer Buy Delete");
    }

}