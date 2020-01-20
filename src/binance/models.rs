use crate::types::{Channel, LiveEvent, TradeType};
use crate::types;
use bigdecimal::BigDecimal;

pub static OUTBOUND_ACCOUNT_INFO: &'static str = "outboundAccountInfo";
pub static EXECUTION_REPORT: &'static str = "executionReport";

pub static KLINE: &'static str = "kline";
pub static AGGREGATED_TRADE: &'static str = "aggTrade";
pub static DEPTH_ORDERBOOK : &'static str = "depthUpdate";
pub static PARTIAL_ORDERBOOK : &'static str = "lastUpdateId";

pub static DAYTICKER: &'static str = "24hrTicker";

#[derive(Debug, Serialize, Deserialize, Message)]
#[serde(tag = "e")]
#[rtype(result = "()")]
pub enum Event {
    #[serde(alias = "outboundAccountInfo")]
    AccountUpdate(AccountUpdateEvent),
    #[serde(alias = "executionReport")]
    OrderTrade(OrderTradeEvent),
    #[serde(alias = "aggTrade")]
    AggTrade(TradesEvent),
    #[serde(alias = "trade")]
    Trade(TradesEvent),
    #[serde(alias = "lastUpdateId")]
    OrderBook(OrderBook),
    #[serde(alias = "24hrTicker")]
    DayTicker(Vec<DayTickerEvent>),
    #[serde(alias = "kline")]
    Kline(KlineEvent),
    #[serde(alias = "depthUpdate")]
    DepthOrderBook(DepthOrderBookEvent),
}

impl Into<LiveEvent> for Event {
    fn into(self) -> LiveEvent {
        match self {
            Event::Trade(e) => LiveEvent::LiveTrade(types::LiveTrade{
                amount: e.qty.parse::<f32>().unwrap(),
                event_ms: e.event_time as i64,
                price: e.price.parse::<BigDecimal>().unwrap(),
                tt: TradeType::Sell,
                pair: e.symbol,
            }),
            Event::DepthOrderBook(e) => {
                let asks = e.asks;
                let bids = e.bids;
                LiveEvent::LiveOrderbook(types::Orderbook {
                    asks: asks.into_iter().map(|a| {
                        (a.price.into(), a.qty.into())
                    }).collect(),
                    bids: bids.into_iter().map(|a| {
                        (a.price.into(), a.qty.into())
                    }).collect(),
                    timestamp: e.event_time as i64,
                    pair: *super::utils::get_pair_enum(e.symbol.as_str()).unwrap(),
                })
            },
            _ => LiveEvent::Noop
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")]
pub struct Subscription {
    method: String,
    params: Vec<String>,
    id: i32,
}

pub fn subscription(c: Channel, currency_pairs: Vec<&str>, id: i32) -> Subscription {
    let channel_str = match c {
        Channel::LiveTrades => "trade",
        Channel::LiveOrders => "orders",
        Channel::LiveOrderBook => "depth@100ms",
        Channel::LiveDetailOrderBook => "depth@100ms",
        Channel::LiveFullOrderBook => "depth@100ms",
    };
    Subscription {
        method: String::from("SUBSCRIBE"),
        params: currency_pairs.into_iter().map(|cp| format!("{}@{}", cp.to_lowercase(), channel_str)).collect(),
        id,
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformation {
    pub timezone: String,
    pub server_time: u64,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub limit: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub base_asset_precision: u64,
    pub quote_asset:String,
    pub quote_precision: u64,
    pub order_types: Vec<String>,
    pub iceberg_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub filters: Vec<Filters>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "filterType")]
pub enum Filters {
    #[serde(rename = "PRICE_FILTER")]
    #[serde(rename_all = "camelCase")]
    PriceFilter { min_price: String, max_price: String, tick_size: String },
    #[serde(rename = "PERCENT_PRICE")]
    #[serde(rename_all = "camelCase")]
    PercentPrice { multiplier_up: String, multiplier_down: String, avg_price_mins: f64 },
    #[serde(rename = "LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    LotSize { min_qty: String, max_qty: String, step_size: String },
    #[serde(rename = "MIN_NOTIONAL")]
    #[serde(rename_all = "camelCase")]
    MinNotional { min_notional: String, apply_to_market: bool, avg_price_mins: f64 },
    #[serde(rename = "ICEBERG_PARTS")]
    #[serde(rename_all = "camelCase")]
    IcebergParts { limit: u16 },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumAlgoOrders { max_num_algo_orders: u16 },
    #[serde(rename = "MARKET_LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    MarketLotSize { min_qty: String, max_qty: String, step_size: String }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformation {
    pub maker_commission: f32,
    pub taker_commission: f32,
    pub buyer_commission: f32,
    pub seller_commission: f32,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub balances: Vec<Balance>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub symbol: String,
    pub order_id: u64,
    pub client_order_id: String,
    #[serde(with = "string_or_float")] pub price: f64,
    pub orig_qty: String,
    pub executed_qty: String,
    pub status: String,
    pub time_in_force: String,
    #[serde(rename = "type")] pub type_name: String,
    pub side: String,
    #[serde(with = "string_or_float")] pub stop_price: f64,
    pub iceberg_qty: String,
    pub time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderCanceled {
    pub symbol: String,
    pub orig_client_order_id: String,
    pub order_id: u64,
    pub client_order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub symbol: String,
    pub order_id: u64,
    pub client_order_id: String,
    pub transact_time: u64,
}

/// Response to a test order (endpoint /api/v3/order/test).
///
/// Currently, the API responds {} on a successfull test transaction,
/// hence this struct has no fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestResponse {
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub last_update_id: u64,
    pub bids: Vec<Bids>,
    pub asks: Vec<Asks>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bids {
    #[serde(with = "string_or_float")] pub price: f64,
    #[serde(with = "string_or_float")] pub qty: f64,

    // Never serialized.
    #[serde(skip)]
    ignore: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asks {
    #[serde(with = "string_or_float")] pub price: f64,
    #[serde(with = "string_or_float")] pub qty: f64,

    // Never serialized.
    #[serde(skip)]
    ignore: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDataStream {
    pub listen_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Success {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum Prices {
    AllPrices(Vec<SymbolPrice>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SymbolPrice {
    pub symbol: String,
    #[serde(with = "string_or_float")] pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AveragePrice {
    pub mins: u64,
    #[serde(with = "string_or_float")]
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum BookTickers {
    AllBookTickers(Vec<Tickers>),
}

#[derive(Debug, Clone)]
pub enum KlineSummaries {
    AllKlineSummaries(Vec<KlineSummary>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tickers {
    pub symbol: String,
    #[serde(with = "string_or_float")] pub bid_price: f64,
    #[serde(with = "string_or_float")] pub bid_qty: f64,
    #[serde(with = "string_or_float")] pub ask_price: f64,
    #[serde(with = "string_or_float")] pub ask_qty: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradeHistory {
    pub id: u64,
    #[serde(with = "string_or_float")] pub price: f64,
    #[serde(with = "string_or_float")] pub qty: f64,
    pub commission: String,
    pub commission_asset: String,
    pub time: u64,
    pub is_buyer: bool,
    pub is_maker: bool,
    pub is_best_match: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceStats {
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    #[serde(with = "string_or_float")] pub prev_close_price: f64,
    #[serde(with = "string_or_float")] pub last_price: f64,
    #[serde(with = "string_or_float")] pub bid_price: f64,
    #[serde(with = "string_or_float")] pub ask_price: f64,
    #[serde(with = "string_or_float")] pub open_price: f64,
    #[serde(with = "string_or_float")] pub high_price: f64,
    #[serde(with = "string_or_float")] pub low_price: f64,
    #[serde(with = "string_or_float")] pub volume: f64,
    pub open_time: u64,
    pub close_time: u64,
    pub first_id: u64,
    pub last_id: u64,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateEvent {

    #[serde(rename = "E")] pub event_time: u64,

    m: u64,
    t: u64,
    b: u64,
    s: u64,

    #[serde(rename = "T")] t_ignore: bool,
    #[serde(rename = "W")] w_ignore: bool,
    #[serde(rename = "D")] d_ignore: bool,

    #[serde(rename = "B")] pub balance: Vec<EventBalance>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventBalance {
    #[serde(rename = "a")] pub asset: String,
    #[serde(rename = "f")] pub free: String,
    #[serde(rename = "l")] pub locked: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderTradeEvent {

    #[serde(rename = "E")] pub event_time: u64,

    #[serde(rename = "s")] pub symbol: String,

    #[serde(rename = "c")] pub new_client_order_id: String,

    #[serde(rename = "S")] pub side: String,

    #[serde(rename = "o")] pub order_type: String,

    #[serde(rename = "f")] pub time_in_force: String,

    #[serde(rename = "q")] pub qty: String,

    #[serde(rename = "p")] pub price: String,

    #[serde(skip, rename = "P")] pub p_ignore: String,

    #[serde(skip, rename = "F")] pub f_ignore: String,

    #[serde(skip)] pub g: i32,

    #[serde(skip, rename = "C")] pub c_ignore: Option<String>,

    #[serde(rename = "x")] pub execution_type: String,

    #[serde(rename = "X")] pub order_status: String,

    #[serde(rename = "r")] pub order_reject_reason: String,

    #[serde(rename = "i")] pub order_id: u64,

    #[serde(rename = "l")] pub qty_last_filled_trade: String,

    #[serde(rename = "z")] pub accumulated_qty_filled_trades: String,

    #[serde(rename = "L")] pub price_last_filled_trade: String,

    #[serde(rename = "n")] pub commission: String,

    #[serde(skip, rename = "N")] pub asset_commisioned: Option<String>,

    #[serde(rename = "T")] pub trade_order_time: u64,

    #[serde(rename = "t")] pub trade_id: i64,

    #[serde(skip, rename = "I")] pub i_ignore: u64,

    #[serde(skip)] pub w: bool,

    #[serde(rename = "m")] pub is_buyer_maker: bool,

    #[serde(skip, rename = "M")] pub m_ignore: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradesEvent {

    #[serde(rename = "E")] pub event_time: u64,

    #[serde(rename = "s")] pub symbol: String,

    #[serde(rename = "a")] pub aggregated_trade_id: u64,

    #[serde(rename = "p")] pub price: String,

    #[serde(rename = "q")] pub qty: String,

    #[serde(rename = "f")] pub first_break_trade_id: u64,

    #[serde(rename = "l")] pub last_break_trade_id: u64,

    #[serde(rename = "T")] pub trade_order_time: u64,

    #[serde(rename = "m")] pub is_buyer_maker: bool,

    #[serde(skip, rename = "M")] pub m_ignore: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DayTickerEvent {

    #[serde(rename = "E")] pub event_time: u64,

    #[serde(rename = "s")] pub symbol: String,

    #[serde(rename = "p")] pub price_change: String,

    #[serde(rename = "P")] pub price_change_percent: String,

    #[serde(rename = "w")] pub average_price: String,

    #[serde(rename = "x")] pub prev_close: String,

    #[serde(rename = "c")] pub current_close: String,

    #[serde(rename = "Q")] pub current_close_qty: String,

    #[serde(rename = "b")] pub best_bid: String,

    #[serde(rename = "B")] pub best_bid_qty: String,

    #[serde(rename = "a")] pub best_ask: String,

    #[serde(rename = "A")] pub best_ask_qty: String,

    #[serde(rename = "o")] pub open: String,

    #[serde(rename = "h")] pub high: String,

    #[serde(rename = "l")] pub low: String,

    #[serde(rename = "v")] pub volume: String,

    #[serde(rename = "q")] pub quote_volume: String,

    #[serde(rename = "O")] pub open_time: u64,

    #[serde(rename = "C")] pub close_time: u64,

    #[serde(rename = "F")] pub first_trade_id: i64,

    #[serde(rename = "L")] pub last_trade_id: i64,

    #[serde(rename = "n")] pub num_trades: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KlineEvent {

    #[serde(rename = "E")] pub event_time: u64,

    #[serde(rename = "s")] pub symbol: String,

    #[serde(rename = "k")] pub kline: Kline,
}

#[derive(Debug, Clone)]
pub struct KlineSummary {
    pub open_time: i64,

    pub open: f64,

    pub high: f64,

    pub low: f64,

    pub close: f64,

    pub volume: f64,

    pub close_time: i64,

    pub quote_asset_volume: f64,

    pub number_of_trades: i64,

    pub taker_buy_base_asset_volume: f64,

    pub taker_buy_quote_asset_volume: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Kline {
    #[serde(rename = "t")] pub start_time: i64,

    #[serde(rename = "T")] pub end_time: i64,

    #[serde(rename = "s")] pub symbol: String,

    #[serde(rename = "i")] pub interval: String,

    #[serde(rename = "f")] pub first_trade_id: i32,

    #[serde(rename = "L")] pub last_trade_id: i32,

    #[serde(rename = "o")] pub open: String,

    #[serde(rename = "c")] pub close: String,

    #[serde(rename = "h")] pub high: String,

    #[serde(rename = "l")] pub low: String,

    #[serde(rename = "v")] pub volume: String,

    #[serde(rename = "n")] pub number_of_trades: i32,

    #[serde(rename = "x")] pub is_final_bar: bool,

    #[serde(rename = "q")] pub quote_volume: String,

    #[serde(rename = "V")] pub active_buy_volume: String,

    #[serde(rename = "Q")] pub active_volume_buy_quote: String,

    #[serde(skip, rename = "B")] pub ignore_me: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepthOrderBookEvent {

    #[serde(rename = "E")] pub event_time: u64,

    #[serde(rename = "s")] pub symbol: String,

    #[serde(rename = "U")] pub first_update_id: u64,

    #[serde(rename = "u")] pub final_update_id: u64,

    #[serde(rename = "b")] pub bids: Vec<Bids>,

    #[serde(rename = "a")] pub asks: Vec<Asks>
}

mod string_or_float {
    use std::fmt;

    use serde::{de, Serializer, Deserialize, Deserializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where T: fmt::Display,
              S: Serializer
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
        where D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::String(s) => s.parse().map_err(de::Error::custom),
            StringOrFloat::Float(i) => Ok(i),
        }
    }
}
