//! Use this module to interact with Binance through a Generic API.
//! This a more convenient and safe way to deal with the exchange since methods return a Result<>
//! but this generic API does not provide all the functionnality that Binance offers.

use crate::exchange::{ExchangeApi, Exchange};
use crate::binance::api::BinanceApi;
use crate::binance::utils;

use crate::error::*;
use crate::types::*;
use crate::helpers;
use async_trait::async_trait;
use binance::api::Binance;
use binance::market::Market;
use crate::utils::pair_or;
use bigdecimal::{BigDecimal, ToPrimitive};

#[async_trait]
impl ExchangeApi for BinanceApi {
    async fn ticker(&mut self, pair: Pair) -> Result<Ticker> {
        let market = self.market();

        let pair_str = pair_or(Exchange::Binance, &pair)?;
        let result = market.get_24h_price_stats(*pair_str)?;
        Ok(Ticker {
            timestamp: helpers::get_unix_timestamp_ms(),
            pair,
            last_trade_price: BigDecimal::from(result.last_price),
            lowest_ask: BigDecimal::from(result.low_price),
            highest_bid: BigDecimal::from(result.high_price),
            volume: Some(BigDecimal::from(result.volume)),
        })
    }

    async fn orderbook(&mut self, pair: Pair) -> Result<Orderbook> {
        let market = self.market();
        let pair_str = pair_or(Exchange::Binance, &pair)?;

        let book_ticker = market.get_depth(*pair_str)?;

        Ok(Orderbook {
            timestamp: helpers::get_unix_timestamp_ms(),
            pair: pair,
            asks: book_ticker.asks.into_iter().map(|a| (a.price.into(), a.qty.into())).collect(),
            bids: book_ticker.bids.into_iter().map(|a| (a.price.into(), a.qty.into())).collect(),
        })
    }

    async fn add_order(&mut self,
                       order_type: OrderType,
                       pair: Pair,
                       quantity: Volume,
                       price: Option<Price>)
                       -> Result<OrderInfo> {
        let pair_str = *pair_or(Exchange::Binance, &pair)?;
        let account = self.account();
        let quantity_f64 = quantity.as_f64()?;
        let result = match order_type {
            OrderType::BuyLimit => {
                if price.is_none() {
                    return Err(ErrorKind::MissingPrice.into());
                }
                account.limit_buy(pair_str, quantity_f64, price.unwrap().as_f64()?)
            }
            OrderType::BuyMarket => account.market_buy(pair_str, quantity_f64),
            OrderType::SellLimit => {
                if price.is_none() {
                    return Err(ErrorKind::MissingPrice.into());
                }
                account.limit_sell(pair_str, quantity_f64, price.unwrap().as_f64()?)
            }
            OrderType::SellMarket => account.market_sell(pair_str, quantity_f64),
        };

        Ok(OrderInfo {
            timestamp: helpers::get_unix_timestamp_ms(),
            identifier: vec![result?.client_order_id],
        })
    }

    /// Return the balances for each currency on the account
    async fn balances(&mut self) -> Result<Balances> {
        let result = self.account().get_account()?;

        let mut balances = Balances::new();

        for balance in result.balances {
            let currency = utils::get_currency_enum(balance.asset.as_str());

            match currency {
                Some(c) => {
                    balances.insert(c, balance.free.parse::<BigDecimal>()?);
                },
                _ => ()
            }
        }

        Ok(balances)
    }
}
