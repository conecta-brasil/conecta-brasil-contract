use crate::model::{Access, DataKey, Error, OrderRec, OrderSession, Package, Session};

use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token::Client as TokenClient, Address,
    Env, Symbol, Vec,
};


#[contract]
pub struct ConectaBrasil;


    // -------------------------------------------------------------
    // TIPOS AUXILIARES (somente neste arquivo)
    // -------------------------------------------------------------

    // storage helpers p/ OrderRec (persistent)
    fn load_order(env: &Env, owner: &Address, order_id: u128) -> Option<OrderRec> {
        env.storage()
            .persistent()
            .get::<_, OrderRec>(&DataKey::Order(owner.clone(), order_id))
    }
    fn save_order(env: &Env, owner: &Address, order_id: u128, rec: &OrderRec) {
        env.storage()
            .persistent()
            .set(&DataKey::Order(owner.clone(), order_id), rec);
    }
    
    fn get_user_orders_list(env: &Env, owner: &Address) -> Vec<u128> {
    env.storage()
        .persistent()
        .get(&DataKey::UserOrders(owner.clone()))
        .unwrap_or(Vec::new(env))
    }

    // helper para gerenciar lista de ordens do usuário
    fn add_user_order(env: &Env, owner: &Address, order_id: u128) {
        let key = DataKey::UserOrders(owner.clone());
        let mut orders: Vec<u128> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
        orders.push_back(order_id);
        env.storage().persistent().set(&key, &orders);
    }

    // helper p/ contador determinístico (instance): NextOrder(owner) -> u128
    fn next_order_id(env: &Env, owner: &Address) -> u128 {
        let key = DataKey::NextOrder(owner.clone());
        let current: u128 = env.storage().instance().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&key, &next);
        next
    }


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

    // Retorna informações completas dos pacotes creditados de um usuário
    pub fn get_user_packages(env: Env, owner: Address) -> Vec<(u128, u32, bool)> {
        let orders = get_user_orders_list(&env, &owner);
        let mut packages = Vec::new(&env);
        
        for order_id in orders.iter() {
            if let Some(order_rec) = load_order(&env, &owner, order_id) {
                packages.push_back((order_id, order_rec.package_id, order_rec.credited));
            }
        }
        
        packages
    }

    /// Retorna todos os pacotes cadastrados
    pub fn get_all_packages(env: Env) -> Vec<(u32, Package)> {
        let mut packages = Vec::new(&env);
        
        // Itera através de possíveis IDs de pacotes (máximo 10 pacotes)
        for package_id in 1..=10u32 {
            if let Some(package) = env.storage().instance().get(&DataKey::Package(package_id)) {
                packages.push_back((package_id, package));
            }
        }
        
        packages
    }

    // -------------------- NOVO: buy_order (compra sem crédito) ----------------

    fn dbg(env: &Env, step: &str) {
        env.events()
            .publish((Symbol::new(env, "dbg"), Symbol::new(env, step)), ());
    }

    pub fn buy_order(env: Env, owner: Address, package_id: u32) -> u128 {
        owner.require_auth();
        Self::dbg(&env, "start");

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| {
                Self::dbg(&env, "err_no_admin");
                panic_with_error!(&env, Error::NotInitialized)
            });

        let token_id: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .unwrap_or_else(|| {
                Self::dbg(&env, "err_no_token");
                panic_with_error!(&env, Error::NotInitialized)
            });

        let pkg: Package = env
            .storage()
            .instance()
            .get(&DataKey::Package(package_id))
            .unwrap_or_else(|| {
                Self::dbg(&env, "err_pkg_nf");
                panic_with_error!(&env, Error::PackageNotFound)
            });

        Self::dbg(&env, "before_transfer");
        let token = TokenClient::new(&env, &token_id);
        token.transfer(&owner, &admin, &pkg.price); // se der erro do SAC, diagnostics mostram

        Self::dbg(&env, "after_transfer");

        // >>>>> ALTERAÇÃO: gerar order_id determinístico pelo contador <<<<<
        let order_id: u128 = next_order_id(&env, &owner);

        save_order(
            &env,
            &owner,
            order_id,
            &OrderRec {
                package_id,
                credited: false,
            },
        );

        // NOVO: adicionar order_id à lista do usuário
        add_user_order(&env, &owner, order_id);

        env.events().publish(
            (Symbol::new(&env, "purchase"), Symbol::new(&env, "created")),
            (owner, package_id, order_id, pkg.price),
        );
        Self::dbg(&env, "done");
        order_id
    }




}
