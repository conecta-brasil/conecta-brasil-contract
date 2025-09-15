use crate::model::{Access, DataKey, Error, OrderRec, OrderSession, Package, Session};

use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token::Client as TokenClient, Address,
    Env, Symbol, Vec,
};


#[contract]
pub struct ConectaBrasil;


#[contractimpl]
impl ConectaBrasil {
    
    // -------------------- init / set_package (inalterados) --------------------
    pub fn init(env: Env, admin: Address, token_asset: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token_asset);        
        env.events()
            .publish((symbol_short!("init"), admin.clone()), token_asset);
    }
    

}
