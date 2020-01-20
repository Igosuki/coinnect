//! Use this module to interact with Bitstamp exchange.

pub mod api;
pub mod credentials;
pub mod utils;
pub mod generic_api;
pub mod streaming_api;
pub mod models;

pub use self::credentials::BinanceCreds;
pub use self::api::BinanceApi;
