use crate::coinnect::Credentials;
use crate::exchange_bot::{DefaultWsActor, WsHandler, ExchangeBot};
use crate::error::*;
use super::models::*;
use bytes::Bytes;
use bytes::Buf;
use futures::stream::{SplitSink};
use actix::{io::SinkWrite, Addr, Recipient};
use awc::{
    ws::{Codec, Message},  BoxedSocket
};
use actix_codec::{Framed};
use crate::types::{LiveEvent, Channel, LiveEventEnveloppe, Pair};
use crate::exchange::Exchange;
use std::collections::{HashMap, HashSet};

pub struct BitstampBot {
    addr: Addr<DefaultWsActor>
}

impl ExchangeBot for BitstampBot {
    fn is_connected(&self) -> bool {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct BitstampStreamingApi {
    api_key: String,
    api_secret: String,
    customer_id: String,
    pub recipients: Vec<Recipient<LiveEventEnveloppe>>,
    channels: HashMap<Channel, HashSet<Pair>>,
}

impl BitstampStreamingApi {
    pub async fn new_bot<C: Credentials>(creds: Box<C>, channels: HashMap<Channel, HashSet<Pair>>, recipients: Vec<Recipient<LiveEventEnveloppe>>) -> Result<BitstampBot> {
        let api = BitstampStreamingApi {
            api_key: creds.get("api_key").unwrap_or_default(),
            api_secret: creds.get("api_secret").unwrap_or_default(),
            customer_id: creds.get("customer_id").unwrap_or_default(),
            recipients,
            channels,
        };
        let addr = DefaultWsActor::new("wss://ws.bitstamp.net", Box::new(api)).await;
        Ok(BitstampBot { addr })
    }
}

impl WsHandler for BitstampStreamingApi {
    fn handle_in(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, msg: Bytes) {
        let v : Event = serde_json::from_slice(msg.bytes()).unwrap();
        match v {
            Event::ReconnectRequest(_) =>  {
                self.handle_started(w);
            },
            Event::SubSucceeded(_) => (),
            o => {
                let vec = self.recipients.clone();
                if vec.len() == 0 as usize {
                    println!("{:?}", o);
                } else {
                    let le : LiveEvent = o.into();
                    for r in &vec {
                        let le : LiveEvent = le.clone();
                        r.do_send(LiveEventEnveloppe(Exchange::Bitstamp, le));
                    }
                }
            },
        };
    }

    fn handle_started(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>) {
        for (k, v) in self.channels.clone() {
            for pair in v {
                let result = serde_json::to_string(&subscription(k.clone(), *super::utils::get_pair_string(&pair).unwrap())).unwrap();
                w.write(Message::Binary(result.into()));
            }

        }
    }
}
