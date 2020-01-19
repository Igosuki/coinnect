use crate::error::*;
use crate::exchange::Exchange;
use crate::types::Pair;

pub fn pair_fn(xchg: Exchange) -> fn(&Pair) -> Option<&&str> {
    match xchg {
        Exchange::Bittrex => crate::bittrex::utils::get_pair_string,
        Exchange::Bitstamp => crate::bitstamp::utils::get_pair_string,
        Exchange::Gdax => crate::gdax::utils::get_pair_string,
        Exchange::Kraken => crate::kraken::utils::get_pair_string,
        Exchange::Poloniex => crate::poloniex::utils::get_pair_string,
        Exchange::Binance => crate::poloniex::utils::get_pair_string,
    }
}

pub fn pair_or(xchg: Exchange, pair: &Pair) -> Result<&&str>{
    let pairs_fn : fn(&Pair) -> Option<&&str> = pair_fn(xchg);
    match pairs_fn(pair) {
        Some(name) => Ok(name),
        None => Err(ErrorKind::PairUnsupported.into()),
    }
}
