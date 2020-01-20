use bidir_map::BidirMap;

use hmac::{Hmac, Mac};
use sha2::{Sha256};

use serde_json;
use serde_json::Value;
use serde_json::value::Map;

use crate::error::*;
use crate::helpers;
use crate::types::Currency;
use crate::types::Pair;
use crate::types::Pair::*;

const PAIRS_BYTES : &[u8] = include_bytes!("./PAIRS");

lazy_static! {
    //curl https://api.binance.com//api/v3/exchangeInfo | jq -r '.symbols | .[].baseAsset + "_" + .[].quoteAsset + "," '
    //Big warning, binance uses lower case for websockets, and upper case symbols for the rest api
    static ref ALL_BINANCE_PAIRS : Vec<(&'static str, String)> = std::str::from_utf8(PAIRS_BYTES).unwrap().lines().map(|s| {
        (s, s.to_string().replace("_", ""))
    }).collect();
}

lazy_static! {
    static ref PAIRS_STRING: BidirMap<Pair, &'static str> = {
        let mut m = BidirMap::new();
        for (pair, b_pair) in &*ALL_BINANCE_PAIRS {
            serde_json::from_str(format!("\"{}\"", pair).as_str()).map(|p_enum : Pair | {
                m.insert(p_enum, b_pair.as_str());
            });
        }
        m
    };
}

/// Return the name associated to pair used by Bitstamp
/// If the Pair is not supported, None is returned.
pub fn get_pair_string(pair: &Pair) -> Option<&&str> {
    PAIRS_STRING.get_by_first(pair)
}

/// Return the Pair enum associated to the string used by Bitstamp
/// If the Pair is not supported, None is returned.
pub fn get_pair_enum(pair: &str) -> Option<&Pair> {
    PAIRS_STRING.get_by_second(&pair)
}

/// Return the currency enum associated with the
/// string used by Bitstamp. If no currency is found,
/// return None
/// # Examples
///
/// ```
/// use crate::coinnect::gdax::utils::get_currency_enum;
/// use crate::coinnect::types::Currency;
///
/// let currency = get_currency_enum("usd_balance");
/// assert_eq!(Some(Currency::USD), currency);
/// ```
pub fn get_currency_enum(currency: &str) -> Option<Currency> {
    let c : Option<Currency> = serde_json::from_str(currency).ok();
    c
}

/// Return the currency string associated with the
/// enum used by Gdax. If no currency is found,
/// return None
/// # Examples
///
/// ```
/// use crate::coinnect::gdax::utils::get_currency_string;
/// use crate::coinnect::types::Currency;
///
/// let currency = get_currency_string(Currency::USD);
/// assert_eq!(currency, Some("USD".to_string()));
/// ```
pub fn get_currency_string(currency: Currency) -> Option<String> {
    serde_json::to_string(&currency).ok()
}
