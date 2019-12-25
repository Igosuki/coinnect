use crate::coinnect::Credentials;
use crate::exchange_bot::{ExchangeBot, ExchangeBotHandler};
use crate::error::*;
use super::models::*;
use bytes::Bytes;
use bytes::Buf;
use serde_json::Value;
use futures::stream::{SplitSink, StreamExt};
use actix::{Context, io::SinkWrite, Actor, Handler, StreamHandler, AsyncContext, ActorContext, Addr};
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame, Message},
    Client, BoxedSocket
};
use actix_codec::{AsyncRead, AsyncWrite, Framed};
use actix_rt::{System, Arbiter};

#[derive(Debug)]
pub struct BitstampStreamingApi {
    api_key: String,
    api_secret: String,
    customer_id: String,
    currency_pair: String,
}

impl BitstampStreamingApi {
    pub async fn new_bot<C: Credentials>(creds: C, currency_pair: String) -> Result<Addr<ExchangeBot>> {
        let api = BitstampStreamingApi {
            api_key: creds.get("api_key").unwrap_or_default(),
            api_secret: creds.get("api_secret").unwrap_or_default(),
            customer_id: creds.get("customer_id").unwrap_or_default(),
            currency_pair,
        };
        let addr = ExchangeBot::new("wss://ws.bitstamp.net", Box::new(api)).await;
        Ok(addr)
    }
}

impl ExchangeBotHandler for BitstampStreamingApi {
    fn handle_in(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, msg: Bytes) {
        let m : Value = serde_json::from_slice(msg.bytes()).unwrap();
        println!("Server: {:?}", m);
        let v : Event = serde_json::from_slice(msg.bytes()).unwrap();
        println!("Server: {:?}", msg);
        match v {
            Event::ReconnectRequest(_) =>  {
                let result = serde_json::to_string(&Channel::subscription(Channel::LiveFullOrderBook, self.currency_pair.as_str())).unwrap();
                w.write(Message::Binary(result.into())).unwrap();
            }
            _ => ()
        };
    }

    fn handle_started(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>) {
        let result = serde_json::to_string(&Channel::subscription(Channel::LiveFullOrderBook, self.currency_pair.as_str())).unwrap();
        w.write(Message::Binary(result.into())).unwrap();
//        let result = serde_json::to_string(&Channel::subscription(Channel::LiveDetailOrderBook, self.currency_pair.as_str())).unwrap();
//        w.write(Message::Binary(result.into())).unwrap();
//        let result = serde_json::to_string(&Channel::subscription(Channel::LiveOrderBook, self.currency_pair.as_str())).unwrap();
//        w.write(Message::Binary(result.into())).unwrap();
    }
}
