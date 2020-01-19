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

lazy_static! {
    static ref PAIRS_STRING: BidirMap<Pair, &'static str> = {
        let mut m = BidirMap::new();
        m.insert(BCH_USD, "bch-usd");
        m.insert(LTC_EUR, "ltc-eur");
        m.insert(LTC_USD, "ltc-usd");
        m.insert(LTC_BTC, "ltc-btc");
        m.insert(ETH_EUR, "eth-eur");
        m.insert(ETH_USD, "eth-usd");
        m.insert(ETH_BTC, "eth-btc");
        m.insert(BTC_GBP, "btc-gbp");
        m.insert(BTC_EUR, "btc-eur");
        m.insert(BTC_USD, "btc-usd");
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
    match currency {
        "btc_balance" => Some(Currency::BTC),
        "eur_balance" => Some(Currency::EUR),
        "ltc_balance" => Some(Currency::LTC),
        "gbp_balance" => Some(Currency::GBP),
        "usd_balance" => Some(Currency::USD),
        "eth_balance" => Some(Currency::ETH),
        "bch_balance" => Some(Currency::BCH),
        _ => None,
    }
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
    match currency {
        Currency::BTC => Some("BTC".to_string()),
        Currency::EUR => Some("EUR".to_string()),
        Currency::LTC => Some("LTC".to_string()),
        Currency::GBP => Some("GBP".to_string()),
        Currency::USD => Some("USD".to_string()),
        Currency::ETH => Some("ETH".to_string()),
        Currency::BCH => Some("BCH".to_string()),
        _ => None,
    }
}
