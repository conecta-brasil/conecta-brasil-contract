use crate::model::{Access, DataKey, Error, OrderRec, OrderSession, Package, Session};

use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token::Client as TokenClient, Address,
    Env, Symbol, Vec,
};


#[contract]
pub struct ConectaBrasil;

    // -------------------------------------------------------------
    // HELPERS (tempo)
    // -------------------------------------------------------------
    fn remaining_at(env: &Env, s: &Session, now: u64) -> u64 {
        if s.started_at == 0 {
            s.remaining_secs
        } else {
            s.remaining_secs
                .saturating_sub(now.saturating_sub(s.started_at))
        }
    }

    fn load_session(env: &Env, owner: &Address) -> Session {
        env.storage()
            .persistent()
            .get::<_, Session>(&DataKey::Session(owner.clone()))
            .unwrap_or(Session {
                remaining_secs: 0,
                started_at: 0,
            })
    }

    fn save_session(env: &Env, owner: &Address, s: &Session) {
        env.storage()
            .persistent()
            .set(&DataKey::Session(owner.clone()), s);
    }

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

    // -------------------- FUNÇÕES HELPER PARA ORDER SESSION --------------------
    fn load_order_session(env: &Env, owner: &Address, order_id: u128) -> OrderSession {
        env.storage()
            .persistent()
            .get(&DataKey::OrderSession(owner.clone(), order_id))
            .unwrap_or(OrderSession {
                order_id,
                remaining_secs: 0,
                started_at: 0,
            })
    }

    fn save_order_session(env: &Env, owner: &Address, order_id: u128, session: &OrderSession) {
        env.storage()
            .persistent()
            .set(&DataKey::OrderSession(owner.clone(), order_id), session);
    }

    fn remaining_at_order(env: &Env, session: &OrderSession, now: u64) -> u64 {
        if session.started_at == 0 {
            session.remaining_secs
        } else {
            let elapsed = now.saturating_sub(session.started_at);
            session.remaining_secs.saturating_sub(elapsed)
        }
    }


    // -------------------------------------------------------------
    // EVENTOS
    // -------------------------------------------------------------

    fn emit_grant(env: &Env, owner: &Address, order_id: u128, new_remaining: u64) {
        env.events()
            .publish((symbol_short!("grant"), owner, order_id), new_remaining);
    }
    
// -------------------------------------------------------------
// CONTRATO
// -------------------------------------------------------------
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

    // -------------------- NOVO: grant (owner OU admin) ------------------------
    /// Credita os segundos do pacote na sessão do `owner` referentes à `order_id`.
    /// Pode ser chamado pelo **owner** (self-serve) OU pelo **admin**.
    /// - Idempotente: se já creditado, retorna erro `AlreadyGranted`.
    pub fn grant(env: Env, caller: Address, owner: Address, order_id: u128) {
        // autoriza: caller deve ser admin OU o próprio owner
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
            .unwrap();
        if caller != admin && caller != owner {
            panic_with_error!(&env, Error::Unauthorized);
        }
        caller.require_auth();

        // busca ordem; precisa existir e não ter sido creditada
        let mut ord = load_order(&env, &owner, order_id)
            .ok_or(Error::OrderNotFound)
            .unwrap();
        if ord.credited {
            panic_with_error!(&env, Error::AlreadyGranted);
        }

        // pega pacote vinculado à ordem
        let pkg: Package = env
            .storage()
            .instance()
            .get(&DataKey::Package(ord.package_id))
            .ok_or(Error::PackageNotFound)
            .unwrap();

        // credita tempo na sessão geral (mantém compatibilidade)
        let mut s = load_session(&env, &owner);
        s.remaining_secs = s.remaining_secs.saturating_add(pkg.duration_secs as u64);
        save_session(&env, &owner, &s);

        // credita tempo na sessão específica da ordem
        let mut order_session = load_order_session(&env, &owner, order_id);
        order_session.remaining_secs = order_session.remaining_secs.saturating_add(pkg.duration_secs as u64);
        save_order_session(&env, &owner, order_id, &order_session);

        // marca como creditado e emite evento
        ord.credited = true;
        save_order(&env, &owner, order_id, &ord);
        emit_grant(&env, &owner, order_id, s.remaining_secs);
    }

    // -------------------- start / pause / getters (inalterados) ---------------
    pub fn start(env: Env, owner: Address) {
        owner.require_auth();
        let now = env.ledger().timestamp();
        let mut s = load_session(&env, &owner);
        if remaining_at(&env, &s, now) == 0 {
            return;
        }
        if s.started_at == 0 {
            s.started_at = now;
            save_session(&env, &owner, &s);
            env.events().publish((symbol_short!("start"), owner), now);
        }
    }

    pub fn pause(env: Env, owner: Address) {
        owner.require_auth();
        let now = env.ledger().timestamp();
        let mut s = load_session(&env, &owner);
        if s.started_at > 0 {
            let gasto = now.saturating_sub(s.started_at);
            s.remaining_secs = s.remaining_secs.saturating_sub(gasto);
            s.started_at = 0;
            save_session(&env, &owner, &s);
            env.events()
                .publish((symbol_short!("pause"), owner), s.remaining_secs);
        }
    }

     /// Inicia uma sessão específica por order_id
    pub fn start_order(env: Env, owner: Address, order_id: u128) {
        owner.require_auth();
        let now = env.ledger().timestamp();
        
        // Verifica se a ordem existe e foi creditada
        let order = load_order(&env, &owner, order_id)
            .ok_or(Error::OrderNotFound)
            .unwrap();
        if !order.credited {
            panic_with_error!(&env, Error::AlreadyGranted);
        }
        
        // Carrega a sessão da ordem
        let mut order_session = load_order_session(&env, &owner, order_id);
        
        // Verifica se há tempo restante
        if remaining_at_order(&env, &order_session, now) == 0 {
            return;
        }
        
        // Inicia a sessão se não estiver ativa
        if order_session.started_at == 0 {
            order_session.started_at = now;
            save_order_session(&env, &owner, order_id, &order_session);
            env.events().publish(
                (Symbol::new(&env, "start_order"), owner),
                (order_id, now)
            );
        }
    }

    /// Pausa uma sessão específica por order_id
    pub fn pause_order(env: Env, owner: Address, order_id: u128) {
        owner.require_auth();
        let now = env.ledger().timestamp();
        
        // Carrega a sessão da ordem
        let mut order_session = load_order_session(&env, &owner, order_id);
        
        // Pausa apenas se estiver ativa
        if order_session.started_at > 0 {
            let elapsed = now.saturating_sub(order_session.started_at);
            order_session.remaining_secs = order_session.remaining_secs.saturating_sub(elapsed);
            order_session.started_at = 0;
            save_order_session(&env, &owner, order_id, &order_session);
            env.events().publish(
                (Symbol::new(&env, "pause_order"), owner),
                (order_id, order_session.remaining_secs)
            );
        }
    }

        /// Retorna a sessão específica de uma ordem
    pub fn get_order_session(env: Env, owner: Address, order_id: u128) -> OrderSession {
        load_order_session(&env, &owner, order_id)
    }

    /// Retorna o tempo restante de uma ordem específica
    pub fn remaining_by_order(env: Env, owner: Address, order_id: u128, now: u64) -> u64 {
        let order_session = load_order_session(&env, &owner, order_id);
        remaining_at_order(&env, &order_session, now)
    }

    /// Verifica se uma ordem específica está ativa
    pub fn is_order_active(env: Env, owner: Address, order_id: u128, now: u64) -> bool {
        let order_session = load_order_session(&env, &owner, order_id);
        order_session.started_at > 0 && remaining_at_order(&env, &order_session, now) > 0
    }

    /// Retorna lista de ordens ativas do usuário
    pub fn get_active_orders(env: Env, owner: Address, now: u64) -> Vec<u128> {
        let orders = get_user_orders_list(&env, &owner);
        let mut active_orders = Vec::new(&env);
        
        for order_id in orders.iter() {
            let order_session = load_order_session(&env, &owner, order_id);
            if order_session.started_at > 0 && remaining_at_order(&env, &order_session, now) > 0 {
                active_orders.push_back(order_id);
            }
        }
        
        active_orders
    }

    pub fn get_session(env: Env, owner: Address) -> Session {
        load_session(&env, &owner)
    }

    pub fn get_access(env: Env, owner: Address) -> Access {
        let s = load_session(&env, &owner);
        let ea = if s.started_at > 0 {
            s.started_at.saturating_add(s.remaining_secs)
        } else {
            0
        };
        Access {
            owner,
            expires_at: ea,
        }
    }


    pub fn is_active(env: Env, owner: Address, now: u64) -> bool {
        let s = load_session(&env, &owner);
        s.started_at > 0 && remaining_at(&env, &s, now) > 0
    }

    pub fn remaining(env: Env, owner: Address, now: u64) -> u64 {
        let s = load_session(&env, &owner);
        remaining_at(&env, &s, now)
    }

}
