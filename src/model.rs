use soroban_sdk::{contracterror, contracttype, Address, Symbol};

// -------------------------------------------------------------
// MODELO DE DADOS
// -------------------------------------------------------------

/// Um "pacote" de internet: preço (em unidades do token, ex.: XLM/SAC) e duração (segundos).
#[derive(Clone)]
#[contracttype]
pub struct Package {
    pub price: i128,        // preço em unidades do token (ex.: stroops se for XLM/SAC)
    pub duration_secs: u32, // duração total concedida ao comprar este pacote
    pub name: Symbol,       // nome descritivo do pacote (ex.: "Básico", "Premium")
    pub speed_message: Symbol, // mensagem sobre a velocidade (ex.: "Até 10 Mbps", "Velocidade máxima")
    pub is_popular: bool,   // indica se é o pacote mais popular/usado
}

/// Estado de sessão com "saldo de segundos" e marcador de início:
/// - Quando started_at == 0 -> pausado (saldo congelado)
/// - Quando started_at  > 0 -> consumindo desde 'started_at'
#[derive(Clone)]
#[contracttype]
pub struct Session {
    pub remaining_secs: u64, // saldo de segundos "congeláveis"
    pub started_at: u64,     // unix ts (ledger). 0 = pausado
}

/// Sessão específica por ordem/pacote
#[derive(Clone)]
#[contracttype]
pub struct OrderSession {
    pub order_id: u128,
    pub remaining_secs: u64, // saldo de segundos desta ordem específica
    pub started_at: u64,     // unix ts (ledger). 0 = pausado
}

/// Estrutura compatível com o modelo "expira em" caso você queira expor
/// um "expires_at virtual" quando a sessão está ativa.
#[derive(Clone)]
#[contracttype]
pub struct Access {
    pub owner: Address,
    pub expires_at: u64, // se pausado, 0; se ativo, started_at + remaining_secs
}

/// Registro de ordem de compra (paga on-chain, mas ainda não creditada).
/// Usado para separar COMPRA (buy_order) do CRÉDITO (grant) com idempotência.
#[derive(Clone)]
#[contracttype]
pub struct OrderRec {
    pub package_id: u32, // pacote comprado
    pub credited: bool,  // se os segundos já foram creditados na sessão
}

/// Chaves de armazenamento:
/// - Instance storage: Admin / Token / Package / NextOrder
///   (config/global + contador determinístico por dono)
/// - Persistent storage:
///     - Session(owner)            -> estado por usuário (vida longa)
///     - Order(owner, order_id)    -> ordem paga, pendente ou já creditada
#[contracttype]
pub enum DataKey {
    Admin,        // Address do administrador do catálogo
    Token,        // Address do contrato do token (SAC) usado na cobrança
    Package(u32), // id -> Package
    // contador sequencial por dono para gerar order_id determinístico
    NextOrder(Address),   // owner -> u128 (próximo order_id disponível)
    // lista de ordens por usuário
    UserOrders(Address),  // owner -> Vec<u128> (lista de order_ids do usuário)
    Session(Address),     // owner -> Session (mantido para compatibilidade)
    OrderSession(Address, u128), // (owner, order_id) -> OrderSession
    Order(Address, u128), // (owner, order_id) -> OrderRec
}

// -------------------------------------------------------------
// ERROS
// -------------------------------------------------------------
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    PackageNotFound = 4,
    InsufficientBalance = 5,

    // fluxo buy_order + grant
    OrderNotFound = 6,  // ordem não existe (ex.: order_id inválido)
    AlreadyGranted = 7, // ordem já foi creditada (idempotência no grant)
}
