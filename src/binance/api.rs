//! Use this module to interact with Binance exchange.
//! Please see examples for more informations.


use hyper::{Client, Uri, Request, Body, Method};
use hyper::header::{CONTENT_TYPE,USER_AGENT};

use hyper_tls::HttpsConnector;

use serde_json::Value;
use serde_json::value::Map;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::coinnect::Credentials;
use crate::exchange::Exchange;

use crate::error::*;
use crate::helpers;
use crate::types::Pair;
use crate::binance::utils;
use crate::types::*;
use hyper::client::HttpConnector;
use futures::{TryFutureExt};
use bytes::buf::BufExt as _;
use crate::helpers::json;
use binance::api::Binance;
use binance::market::Market;
use binance::account::Account;

#[derive(Debug)]
pub struct BinanceApi {
    last_request: i64, // unix timestamp in ms, to avoid ban
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    customer_id: String,
    http_client: Client<HttpsConnector<HttpConnector>>,
    burst: bool,
}


impl BinanceApi {
    /// Create a new BinanceApi by providing an API key & API secret
    pub fn new<C: Credentials>(creds: C) -> Result<BinanceApi> {
        if creds.exchange() != Exchange::Binance {
            return Err(ErrorKind::InvalidConfigType(Exchange::Binance, creds.exchange()).into());
        }

        let connector = HttpsConnector::new();
        let ssl = Client::builder().build::<_, hyper::Body>(connector);
        let option = creds.get("api_key");
        let option1 = creds.get("api_secret");

        Ok(BinanceApi {
            last_request: 0,
            api_key: option,
            api_secret: option1,
            customer_id: creds.get("customer_id").unwrap_or_default(),
            http_client: ssl,
            burst: false, // No burst by default
        })
    }

    pub fn market(&self) -> Market {
        Binance::new(self.api_key.clone(), self.api_secret.clone())
    }

    pub fn account(&self) -> Account {
        Binance::new(self.api_key.clone(), self.api_secret.clone())
    }

    /// The number of calls in a given period is limited. In order to avoid a ban we limit
    /// by default the number of api requests.
    /// This function sets or removes the limitation.
    /// Burst false implies no block.
    /// Burst true implies there is a control over the number of calls allowed to the exchange
    pub fn set_burst(&mut self, burst: bool) {
        self.burst = burst
    }
}


#[cfg(test)]
mod binance_api_tests {
    use super::*;

//    #[test]
//    fn should_block_or_not_block_when_enabled_or_disabled() {
//        let mut api = BinanceApi {
//            last_request: helpers::get_unix_timestamp_ms(),
//            api_key: "".to_string(),
//            api_secret: "".to_string(),
//            customer_id: "".to_string(),
//            http_client: Client::new(),
//            burst: false,
//        };
//
//        let mut counter = 0;
//        loop {
//            api.set_burst(false);
//            let start = helpers::get_unix_timestamp_ms();
//            api.block_or_continue();
//            api.last_request = helpers::get_unix_timestamp_ms();
//
//            let difference = api.last_request - start;
//            assert!(difference >= 334);
//            assert!(difference < 10000);
//
//
//            api.set_burst(true);
//            let start = helpers::get_unix_timestamp_ms();
//            api.block_or_continue();
//            api.last_request = helpers::get_unix_timestamp_ms();
//
//            let difference = api.last_request - start;
//            assert!(difference < 10);
//
//            counter = counter + 1;
//            if counter >= 3 { break; }
//        }
//    }
}
