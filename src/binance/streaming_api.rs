use futures::stream::{SplitSink, FuturesUnordered};
use crate::exchange_bot::{ExchangeBot, WsHandler, DefaultWsActor};
use crate::types::{Channel, Pair, LiveEventEnveloppe, LiveAggregatedOrderBook, LiveEvent, Orderbook};
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
use crate::exchange::{Exchange, ExchangeApi};
use super::models::*;
use crate::binance::BinanceApi;
use futures::Future;
use std::task::Poll;
use futures_util::{FutureExt, TryFutureExt, StreamExt};
use std::sync::{Arc, RwLock};
use std::borrow::Borrow;
use tokio::sync::mpsc::{self, Receiver};

use std::{thread, ptr};
use actix::{Addr, Recipient, Context, Actor, AsyncContext};
use actix_web::error::Canceled;
use tokio::runtime::Runtime;
use async_trait::async_trait;
use bigdecimal::BigDecimal;

static WEBSOCKET_URL: &'static str = "wss://stream.binance.com:9443/ws";

pub struct BinanceBot {
    addr: Addr<DefaultWsActor>
}

impl ExchangeBot for BinanceBot {
    fn is_connected(&self) -> bool {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct BinanceStreamingApi {
    books: Arc<RefCell<HashMap<Pair, LiveAggregatedOrderBook>>>,
    pub channels: HashMap<Channel, HashSet<Pair>>,
    pub recipients: Vec<Recipient<LiveEventEnveloppe>>,
    api: Arc<BinanceApi>,
}

impl BinanceStreamingApi {
    /// Create a new bittrex exchange bot, unavailable channels and currencies are ignored
    pub async fn new_bot<C: Credentials>(creds: Box<C>, channels: HashMap<Channel, HashSet<Pair>>, recipients: Vec<Recipient<LiveEventEnveloppe>>) -> Result<BinanceBot> {
        let mut map = channels.clone();
        let order_book_pairs: &HashSet<Pair> = map.entry(Channel::LiveFullOrderBook).or_default();
        let trade_pairs: &HashSet<Pair> = map.entry(Channel::LiveTrades).or_default();
        let api = BinanceStreamingApi {
            recipients,
            books: Arc::new(RefCell::new(HashMap::new())),
            channels,
            api: Arc::new(BinanceApi::new(*creds).unwrap()),
        };
        api.refresh_order_books().await;
        let addr = DefaultWsActor::new("BinanceStream", WEBSOCKET_URL, Some(Duration::from_secs(30)), Box::new(api)).await?;

        return Ok(BinanceBot { addr });
    }

    async fn refresh_order_books(&self) {
            let mut orderbooks_futs: Vec<Receiver<Result<Orderbook>>> = Vec::new();
            let mut arc3 = &mut &self.api.as_ref().clone();
            for &pair in self.channels.get(&Channel::LiveFullOrderBook).unwrap() {
                let mut arc = arc3.clone();
                let (mut tx, mut rx) = mpsc::channel::<Result<Orderbook>>(100);
                orderbooks_futs.push(rx);
                tokio::spawn(async move {
                    let r = arc.orderbook(pair.clone()).await;
                    let r = r.map_err(|e| {
                        info!("Binance : error fetching order book for {:?} : {:?}", pair, e);
                        e
                    });
                    tx.send(r).await;
                });
            }

            let ret: FuturesUnordered<_> = orderbooks_futs.iter_mut().map(|rx| rx.recv()).collect();
            let ret2 : Vec<Option<Result<Orderbook>>> = ret.collect().await;
            for ob in ret2.into_iter().filter(|&ref o| o.as_ref().and_then(|r| r.as_ref().ok()).is_some()) {
                let ob = ob.unwrap().unwrap();
                let mut books = self.books.borrow_mut();
                let default_book = LiveAggregatedOrderBook::default(ob.pair);
                let mut agg = books.entry(ob.pair).or_insert(default_book);
                agg.reset_asks(ob.asks.into_iter());
                agg.reset_bids(ob.bids.into_iter());
                let latest_order_book: Orderbook = agg.order_book();
                self.broadcast(LiveEvent::LiveOrderbook(latest_order_book.clone()));
            }
    }

    fn broadcast(&self, v: LiveEvent) {
        let vec = self.recipients.clone();
        if vec.len() == 0 as usize {
            println!("{:?}", v);
        } else {
            for r in &vec {
                let le: LiveEvent = v.clone();
                r.do_send(LiveEventEnveloppe(Exchange::Binance, le));
            }
        }
    }
}

#[async_trait]
impl WsHandler for BinanceStreamingApi {
    #[cfg_attr(feature = "flame_it", flame)]
    fn handle_in(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, msg: Bytes) {
        let v: Result<Event> = serde_json::from_slice(msg.bytes()).map_err(|e| ErrorKind::Json(e).into());
        if v.is_err() {
            return trace!("Binance : error {:?} deserializing {:?}", v.err().unwrap(), msg);
        }
        match v {
            Ok(Event::DepthOrderBook(ob)) => {
                let pair = super::utils::get_pair_enum(ob.symbol.as_str());
                if pair.is_none() {
                    return;
                }
                let current_pair = *pair.unwrap();
                let mut books = self.books.borrow_mut();
                let default_book = LiveAggregatedOrderBook::default(current_pair);
                let mut agg = books.entry(current_pair).or_insert(default_book);
                agg.update_asks(ob.asks.into_iter().map(|a| (BigDecimal::from(a.price), BigDecimal::from(a.qty))));
                agg.update_bids(ob.bids.into_iter().map(|a| (BigDecimal::from(a.price), BigDecimal::from(a.qty))));
                agg.latest_order_book().map(|ob| self.broadcast(LiveEvent::LiveOrderbook(ob)));
            }
            Ok(Event::Trade(t)) => {
                let le : LiveEvent = Event::Trade(t).into();
                self.broadcast(le)
            },
            _ => return
        }
    }

    fn handle_started(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>) {
        let rc = self.books.clone();

        let mut books = rc.borrow_mut();

        let x = self.channels.get(&Channel::LiveFullOrderBook).unwrap();
        for &pair in x {
            books.insert(pair.clone(), LiveAggregatedOrderBook::default(pair.clone()));
        }
//        ctx.spawn(self.clone().refresh_order_books());

        for (k, v) in &self.channels {
            let pairs = v.into_iter().map(|pair| *super::utils::get_pair_string(&pair).unwrap()).collect();
            info!("Binance : connecting to {:?} for {:?}", k, &pairs);
            let result = serde_json::to_string(&subscription(k.clone(), pairs, 1)).unwrap();
            w.write(Message::Text(result));
        }
    }
}
