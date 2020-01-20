use futures::stream::{SplitSink};
use actix::{Addr, Recipient, Actor, Context, Running, Arbiter, Handler, Supervisor, AsyncContext, WrapFuture, ActorFuture, ContextFutureSpawner};
use crate::exchange_bot::{ExchangeBot, WsHandler, DefaultWsActor};
use crate::types::{Channel, Pair, LiveEventEnveloppe, LiveAggregatedOrderBook, LiveEvent};
use std::collections::{HashSet, HashMap};
use crate::coinnect::Credentials;
use std::rc::Rc;
use std::cell::RefCell;
use crate::error::*;
use std::time::Duration;
use actix::io::SinkWrite;
use awc::ws::{Message, Codec};
use actix_codec::Framed;
use awc::BoxedSocket;
use bytes::Bytes;
use bytes::Buf;
use crate::exchange::Exchange;
use super::models::*;

static WEBSOCKET_URL: &'static str = "wss://stream.binance.com:9443/ws";

pub struct BinanceBot {
    addr: Addr<DefaultWsActor>
}

impl ExchangeBot for BinanceBot {
    fn is_connected(&self) -> bool {
        unimplemented!()
    }
}

pub struct BinanceStreamingApi {
    books: Rc<RefCell<HashMap<Pair, LiveAggregatedOrderBook>>>,
    pub channels: HashMap<Channel, HashSet<Pair>>,
    pub recipients: Vec<Recipient<LiveEventEnveloppe>>,
}

impl BinanceStreamingApi {
    /// Create a new bittrex exchange bot, unavailable channels and currencies are ignored
    pub async fn new_bot<C: Credentials>(creds: Box<C>, channels: HashMap<Channel, HashSet<Pair>>, recipients: Vec<Recipient<LiveEventEnveloppe>>) -> Result<BinanceBot> {

        let mut map = channels.clone();
        let order_book_pairs: &HashSet<Pair> = map.entry(Channel::LiveFullOrderBook).or_default();
        let trade_pairs: &HashSet<Pair> = map.entry(Channel::LiveTrades).or_default();

        let api = BinanceStreamingApi {
            recipients,
            books: Rc::new(RefCell::new(HashMap::new())),
            channels,
        };

        let addr = DefaultWsActor::new(WEBSOCKET_URL, Box::new(api)).await?;

        return Ok(BinanceBot { addr });
    }
}

impl WsHandler for BinanceStreamingApi {
    fn handle_in(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, msg: Bytes) {
        let v : Result<Event> = serde_json::from_slice(msg.bytes()).map_err(|e| ErrorKind::Json(e).into());
        if v.is_err() {
            return trace!("Binance : error {:?} deserializing {:?}", v.err().unwrap(), msg);
        }

        let vec = self.recipients.clone();
        if vec.len() == 0 as usize {
            println!("{:?}", v);
        } else {
            let le : LiveEvent = v.unwrap().into();
            for r in &vec {
                let le : LiveEvent = le.clone();
                r.do_send(LiveEventEnveloppe(Exchange::Binance, le));
            }
        }
    }

    fn handle_started(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>) {
        let rc = self.books.clone();

        let mut books = rc.borrow_mut();

        for pair in self.channels.get(&Channel::LiveFullOrderBook).unwrap() {
            books.insert(pair.clone(), LiveAggregatedOrderBook::default(pair.clone()));
        }
        let mut i = 1;
        for (k, v) in &self.channels {
            for pair in v {
                let result = serde_json::to_string(&subscription(k.clone(), *super::utils::get_pair_string(&pair).unwrap(), i)).unwrap();
                debug!("Binance : connecting to {:?} for {:?} with {:?}", k, v, result);
                w.write(Message::Text(result));
                i += 1;
            }

        }
    }
}
