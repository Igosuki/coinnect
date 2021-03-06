//! This module contains Exchange enum.

use std::fmt::Debug;
use std::convert::Into;
use std::str::FromStr;

use crate::error::*;
use crate::types::*;
use futures::{Future};
use async_trait::async_trait;
use serde::{Deserializer, Deserialize};
use serde::de;

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, Deserialize, Serialize)]
#[serde(from = "String")]
pub enum Exchange {
    Bitstamp,
    Kraken,
    Poloniex,
    Bittrex,
    Gdax,
    Binance,
}

pub trait DeserializeWith: Sized {
    fn deserialize_with<'de, D>(de: D) -> ::std::result::Result<Self, D::Error>
        where D: Deserializer<'de>;
}

impl DeserializeWith for Exchange {
    fn deserialize_with<'de, D>(dez: D) -> ::std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(dez)?;

        let e : Self = Exchange::from_str(s.as_str()).map_err(|e| de::Error::custom(format!("{:?}", e)))?;
        Ok(e)
    }
}

impl Into<String> for Exchange {
    fn into(self) -> String {
        match self {
            Exchange::Bitstamp => "Bitstamp".to_string(),
            Exchange::Kraken => "Kraken".to_string(),
            Exchange::Poloniex => "Poloniex".to_string(),
            Exchange::Bittrex => "Bittrex".to_string(),
            Exchange::Gdax => "Gdax".to_string(),
            Exchange::Binance => "Binance".to_string(),
        }
    }
}

impl From<String> for Exchange {
    fn from(s: String) -> Self {
        Self::from_str(s.as_ref()).unwrap()
    }
}

impl FromStr for Exchange {
    type Err = Error;

    fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "bitstamp" => Ok(Exchange::Bitstamp),
            "kraken" => Ok(Exchange::Kraken),
            "poloniex" => Ok(Exchange::Poloniex),
            "bittrex" => Ok(Exchange::Bittrex),
            "gdax" => Ok(Exchange::Gdax),
            "binance" => Ok(Exchange::Binance),
            _ => Err(ErrorKind::InvalidExchange(input.to_string()).into()),
        }
    }
}

pub type FResult<T> = dyn Future<Output = Result<T>>;

#[async_trait]
pub trait ExchangeApi: Debug {
    /// Return a Ticker for the Pair specified.
    async fn ticker(&mut self, pair: Pair) -> Result<Ticker>;

    /// Return an Orderbook for the specified Pair.
    async fn orderbook(&mut self, pair: Pair) -> Result<Orderbook>;

    /// Place an order directly to the exchange.
    /// Quantity is in quote currency. So if you want to buy 1 Bitcoin for X€ (pair BTC_EUR),
    /// base currency (right member in the pair) is BTC and quote/counter currency is BTC (left
    /// member in the pair).
    /// So quantity = 1.
    ///
    /// A good practice is to store the return type (OrderInfo) somewhere since it can later be used
    /// to modify or cancel the order.
    async fn add_order(&mut self,
                 order_type: OrderType,
                 pair: Pair,
                 quantity: Volume,
                 price: Option<Price>)
                 -> Result<OrderInfo>;

    /// Retrieve the current amounts of all the currencies that the account holds
    /// The amounts returned are available (not used to open an order)
    async fn balances(&mut self) -> Result<Balances>;
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeedSettings {
    pub symbols: Vec<Pair>
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExchangeSettings {
    pub orderbook: Option<FeedSettings>,
    pub trades: Option<FeedSettings>,
}
