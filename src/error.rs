use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "abci")]
    #[error("ABCI Error: {0}")]
    ABCI(String),
    #[cfg(feature = "abci")]
    #[error(transparent)]
    ABCI2(#[from] abci2::Error),
    #[error("App Error: {0}")]
    App(String),
    #[error("Call Error: {0}")]
    Call(String),
    #[error("Client Error: {0}")]
    Client(String),
    #[error("Coins Error: {0}")]
    Coins(String),
    #[error(transparent)]
    Dalek(#[from] ed25519_dalek::ed25519::Error),
    #[error(transparent)]
    Decimal(#[from] rust_decimal::Error),
    #[error("Divide by Zero Error: Cannot divide by zero")]
    DivideByZero,
    #[error("Downcast Error: {0}")]
    Downcast(String),
    #[error(transparent)]
    Ed(#[from] ed::Error),
    #[error("Ibc Error: {0}")]
    Ibc(String),
    #[error("Invalid ID")]
    InvalidID,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[cfg(feature = "merk")]
    #[error(transparent)]
    Merk(#[from] merk::Error),
    #[error("Migration Error: {0}")]
    Migrate(String),
    #[error("Nonce Error: {0}")]
    Nonce(String),
    #[error("Overflow Error")]
    Overflow,
    #[error("Parse Int Error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Poison Error: {0}")]
    Poison(String),
    #[error("Tendermint Error: {0}")]
    Tendermint(String),
    #[cfg(feature = "abci")]
    #[error(transparent)]
    TendermintRPC(#[from] tendermint_rpc::Error),
    #[cfg(feature = "merk-full")]
    #[error(transparent)]
    RocksDB(#[from] merk::rocksdb::Error),
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("Signer Error: {0}")]
    Signer(String),
    #[error("Store Error: {0}")]
    Store(String),
    #[error("State Error: {0}")]
    State(String),
    #[error("Test Error: {0}")]
    Test(String),
    #[error("Query Error: {0}")]
    Query(String),
    #[cfg(feature = "abci")]
    #[error(transparent)]
    Upgrade(#[from] crate::upgrade::Error),
    #[error("Unknown Error")]
    Unknown,
}

/// A result type bound to the standard orga error type.
pub type Result<T> = std::result::Result<T, Error>;
