use actix::{Addr, Recipient, Actor, Context, Running, Arbiter, Handler, Supervisor, AsyncContext, WrapFuture};
use crate::exchange_bot::ExchangeBot;
use crate::types::{Channel, Pair, LiveEventEnveloppe, LiveAggregatedOrderBook};
use std::collections::{HashSet, HashMap};
use crate::coinnect::Credentials;
use binance::websockets::{WebSockets, WebsocketEvent};
use std::rc::Rc;
use std::cell::RefCell;
use crate::error::*;

pub struct BinanceBot {
    addr: Addr<BinanceStreamingApi>
}

impl ExchangeBot for BinanceBot {
    fn is_connected(&self) -> bool {
        unimplemented!()
    }
}

pub struct BinanceStreamingApi {
    web_socket: WebSockets<'static>,
    books: Rc<RefCell<HashMap<Pair, LiveAggregatedOrderBook>>>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ConnQuery {
    channel: String,
    pair: String,
}

impl BinanceStreamingApi {
    /// Create a new bittrex exchange bot, unavailable channels and currencies are ignored
    pub async fn new_bot<C: Credentials>(creds: Box<C>, channels: HashMap<Channel, HashSet<Pair>>, recipients: Vec<Recipient<LiveEventEnveloppe>>) -> Result<BinanceBot> {
        let client: WebSockets = WebSockets::new(BinanceStreamingApi::handler);
        let api = BinanceStreamingApi {
            web_socket: client,
            books: Rc::new(RefCell::new(HashMap::new())),
        };

        let rc = api.books.clone();

        let mut books = rc.borrow_mut();
        let order_book_pairs = channels.get(&Channel::LiveFullOrderBook);
        order_book_pairs.map(|pairs| {
            for pair in pairs {
                books.insert(pair.clone(), LiveAggregatedOrderBook::default(pair.clone()));
            }
        });

        let addr = Supervisor::start(move |ctx| {
            api
        });

        order_book_pairs.map(|pairs| {
            for pair in pairs {
                let currency = *super::utils::get_pair_string(pair).unwrap();
                addr.do_send(ConnQuery { pair: currency.to_string(), channel: "aggTrade".to_string() });
            }
        });

        return Ok(BinanceBot { addr });
    }

    pub fn handler(event: WebsocketEvent) -> binance::errors::Result<()> {
        match event {
            WebsocketEvent::Trade(trade) => {
                println!(
                    "Symbol: {}, price: {}, qty: {}",
                    trade.symbol, trade.price, trade.qty
                );
            },
            WebsocketEvent::DepthOrderBook(depth_order_book) => {
                println!(
                    "Symbol: {}, Bids: {:?}, Ask: {:?}",
                    depth_order_book.symbol, depth_order_book.bids, depth_order_book.asks
                );
            },
            WebsocketEvent::OrderBook(order_book) => {
                println!(
                    "last_update_id: {}, Bids: {:?}, Ask: {:?}",
                    order_book.last_update_id, order_book.bids, order_book.asks
                );
            },
            _ => (),
        };

        Ok(())
    }
}

impl actix::Supervised for BinanceStreamingApi {
    fn restarting(&mut self, ctx: &mut <Self as Actor>::Context) {
    }
}

impl Actor for BinanceStreamingApi {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");
    }
}

impl Handler<ConnQuery> for BinanceStreamingApi {
    type Result = ();

    fn handle(&mut self, msg: ConnQuery, ctx: &mut Self::Context) -> Self::Result {
        match self.web_socket.connect(&format!("{}@{}", msg.pair, msg.channel)) {
            Ok(_) => (),
            Err(e) => debug!("Error connecting {}", e)
        }
    }
}
