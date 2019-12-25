use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct LiveTrade {
    microtimestamp: String,
    amount: f32,
    buy_order_id: i64,
    sell_order_id: i64,
    amount_str: String,
    price_str: String,
    timestamp: String,
    price: f32,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    ty: i64,
    id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiveOrderBook {
    microtimestamp: String,
    timestamp: String,
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiveOrder {
    id: i64,
    amount: f32,
    amount_str: String,
    price: f32,
    price_str: String,
    order_type: i64,
    datetime: String,
    microtimestamp: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Enveloppe<T> {
    data: T,
    channel: String,
}

pub type PlainEvent = Enveloppe<serde_json::Value>;

#[derive(Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum Event {
    #[serde(alias = "bts:subscription_succeeded")]
    SubSucceeded(PlainEvent),
    #[serde(alias = "bts:request_reconnect")]
    ReconnectRequest(PlainEvent),
    #[serde(alias = "trade")]
    LiveTrade(Enveloppe<LiveTrade>),
    #[serde(alias = "data")]
    LiveFullOrderBook(Enveloppe<LiveOrderBook>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    channel: String
}

#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")]
pub struct Subscription {
    event: String,
    data: Data,
}

pub enum Channel {
    LiveTrades,
    LiveOrders,
    LiveOrderBook,
    LiveDetailOrderBook,
    LiveFullOrderBook,
}

impl Channel {
    pub fn subscription(c: Channel, currency_pair: &str) -> Subscription {
        let channel_str = match c {
            Channel::LiveTrades => "live_trades",
            Channel::LiveOrders => "live_orders",
            Channel::LiveOrderBook => "order_book",
            Channel::LiveDetailOrderBook => "detail_order_book",
            Channel::LiveFullOrderBook => "diff_order_book",
        };
        Subscription {
            event: String::from("bts:subscribe"),
            data: Data {
                channel: format!("{}_{}", channel_str, currency_pair)
            },
        }
    }
}

lazy_static! {
    static ref PAIRS : HashSet<&'static str> = vec!["btcusd", "btceur", "eurusd", "xrpusd", "xrpeur", "xrpbtc", "ltcusd", "ltceur", "ltcbtc", "ethusd", "etheur", "ethbtc", "bchusd", "bcheur", "bchbtc"].into_iter().collect();
}

#[cfg(test)]
mod model_tests {
    use super::*;

    #[test]
    fn deserialize_live_trade() {
        let _v: Event = serde_json::from_slice(b"{\"data\": {\"microtimestamp\": \"1577146143220559\", \"amount\": 0.00434678, \"buy_order_id\": 4481152330, \"sell_order_id\": 4481152280, \"amount_str\": \"0.00434678\", \"price_str\": \"7312.91\", \"timestamp\": \"1577146143\", \"price\": 7312.91, \"type\": 0, \"id\": 102177815}, \"event\": \"trade\", \"channel\": \"live_trades_btcusd\"}").unwrap();
    }

    #[test]
    fn deserialize_sub_succeeded() {
        let _v: Event = serde_json::from_slice(b"{\"data\": {\"microtimestamp\": \"1577146143220559\", \"amount\": 0.00434678, \"buy_order_id\": 4481152330, \"sell_order_id\": 4481152280, \"amount_str\": \"0.00434678\", \"price_str\": \"7312.91\", \"timestamp\": \"1577146143\", \"price\": 7312.91, \"type\": 0, \"id\": 102177815}, \"event\": \"trade\", \"channel\": \"live_trades_btcusd\"}").unwrap();
    }
}
