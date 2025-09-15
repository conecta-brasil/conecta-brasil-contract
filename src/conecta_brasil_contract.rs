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

        pub fn set_package(env: Env, id: u32, price: i128, duration_secs: u32, name: Symbol, speed_message: Symbol, is_popular: bool) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
            .unwrap();
        admin.require_auth();
        let pkg = Package {
            price,
            duration_secs,
            name,
            speed_message,
            is_popular,
        };
        env.storage().instance().set(&DataKey::Package(id), &pkg);
        env.events()
            .publish((symbol_short!("pkg_set"), id), (price, duration_secs));
    }

    // Add this function to the contract implementation
    pub fn get_package(env: Env, package_id: u32) -> Package {
        env.storage()
            .instance()
            .get(&DataKey::Package(package_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::PackageNotFound))
    }


}
