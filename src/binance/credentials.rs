//! Contains the Binance credentials.

use serde_json;
use serde_json::Value;

use crate::coinnect::Credentials;
use crate::exchange::Exchange;
use crate::helpers;
use crate::error::*;

use std::collections::HashMap;
use std::str::FromStr;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BinanceCreds {
    exchange: Exchange,
    name: String,
    data: HashMap<String, String>,
}

impl BinanceCreds {
    /// Create a new `BinanceCreds` from a json configuration file. This file must follow this
    /// structure:
    ///
    /// ```json
    /// {
    ///     "account_binance": {
    ///         "exchange"  : "binance",
    ///         "api_key"   : "123456789ABCDEF",
    ///         "api_secret": "ABC&EF?abcdef"
    ///     }
    /// }
    /// ```
    /// For this example, you could use load your Binance account with
    /// `BinanceAPI::new(BinanceCreds::new_from_file("account_gdax", Path::new("/keys.json")))`
    pub fn new_from_file(name: &str, path: PathBuf) -> Result<Self> {
        let mut f = File::open(&path)?;
        let mut buffer = String::new();
        f.read_to_string(&mut buffer)?;

        let data: Value = serde_json::from_str(&buffer)?;
        let json_obj = data.as_object()
            .ok_or_else(|| ErrorKind::BadParse)?
            .get(name)
            .ok_or_else(|| ErrorKind::MissingField(name.to_string()))?;

        let api_key = helpers::get_json_string(json_obj, "api_key")?;
        let api_secret = helpers::get_json_string(json_obj, "api_secret")?;
        let exchange = {
            let exchange_str = helpers::get_json_string(json_obj, "exchange")?;
            Exchange::from_str(exchange_str)
                .chain_err(|| ErrorKind::InvalidFieldValue("exchange".to_string()))?
        };

        if exchange != Exchange::Binance {
            return Err(ErrorKind::InvalidConfigType(Exchange::Binance, exchange).into());
        }

        Ok(BinanceCreds::new(name, api_key, api_secret))
    }


    /// Create a new `BinanceCreds` from arguments.
    pub fn new(name: &str, api_key: &str, api_secret: &str) -> Self {
        let mut creds = BinanceCreds {
            data: HashMap::new(),
            exchange: Exchange::Binance,
            name: if name.is_empty() {
                "BinanceClient".to_string()
            } else {
                name.to_string()
            },
        };


        //if api_key.is_empty() {
        //warning!("No API key set for the Binance client");
        //}
        creds
            .data
            .insert("api_key".to_string(), api_key.to_string());

        //if api_secret.is_empty() {
        //warning!("No API secret set for the Binance client");
        //}
        creds
            .data
            .insert("api_secret".to_string(), api_secret.to_string());

        //if api_secret.is_empty() {
        //warning!("No API customer ID set for the Binance client");
        //}
        creds
    }
}

impl Credentials for BinanceCreds {
    /// Return a value from the credentials.
    fn get(&self, key: &str) -> Option<String> {
        if let Some(res) = self.data.get(key) {
            Some(res.clone())
        } else {
            None
        }
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn exchange(&self) -> Exchange {
        self.exchange
    }
}
