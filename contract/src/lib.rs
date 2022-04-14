
// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,setup_alloc,
    serde::{Deserialize, Serialize},
};

setup_alloc!();

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Funding {
    funding_value: i8,
    funding_account_id: String,
    partial_funding_value: i8

}
impl Funding {
    pub fn inc_funding_value(&mut self,amount: i8){
        self.partial_funding_value += amount;
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    funding_memo: LookupMap<String, Funding>
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            funding_memo:LookupMap::new(b"memo".to_vec())
        }
    }
}
#[near_bindgen]
impl Contract {

    pub fn create_funding(&mut self, amount:i8 ,account_id: Option<AccountId>){

        let storage_account_id = account_id
            //convert the valid account ID into an account ID
            .map(|a| a.into())
            //if we didn't specify an account ID, we simply use the caller of the function
            .unwrap_or_else(env::signer_account_id);
        let funding = Funding { partial_funding_value: 0, funding_value: amount, funding_account_id: storage_account_id.to_string() };

        self.funding_memo.insert(&storage_account_id.to_string(),&funding);
    }
    pub fn transfer_money(&mut self, account_id: AccountId,amount:Balance){
        Promise::new(account_id).transfer(amount);

    }
    #[payable]
    pub fn donate_to_funding(&mut self, amount:i8, funding_account_id: AccountId) {
        let mut record= self.funding_memo.get(&funding_account_id.to_string()).unwrap_or_else(|| env::panic_str("Funding not found."));
        record.inc_funding_value(amount);
        self.funding_memo.insert(&funding_account_id.to_string(),&record);
        if record.partial_funding_value + amount == record.funding_value {
            self.transfer_money(funding_account_id,near_sdk::env::attached_deposit());
        }
    }

    pub fn get_funding(self, user: AccountId) -> Funding {
        match self.funding_memo.get(&user.to_string()){
            Some(x)=>x,
            None => panic!("Didnt find funding"),
        }
    }
}


#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use std::convert::TryInto;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("bob.near".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    fn test_create_funding() {
        let key = "bob_near";

        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::default();
        contract.create_funding(10,Some(key.parse().unwrap()));

        assert_eq!( contract.funding_memo.get(&key.to_string()).unwrap().funding_value,10);

    }

    #[test]
    fn test_donate_funding() {
        let key1 = "bob_near";

        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::default();
        contract.create_funding(10,Some(key1.parse().unwrap()));
        let mut record = contract.funding_memo.get(&key1.to_string()).unwrap_or_else(|| env::panic_str("Record not found."));

        record.inc_funding_value(5);
        contract.funding_memo.insert(&key1.to_string(), &record);

        assert_eq!( contract.funding_memo.get(&key1.to_string()).unwrap().partial_funding_value,5)

    }


}