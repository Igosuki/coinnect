use actix::{Context, io::SinkWrite, Actor, Handler, StreamHandler, AsyncContext, ActorContext, Addr, ActorFuture, WrapFuture, ContextFutureSpawner, Supervisor};
use awc::{error::WsProtocolError, ws::{Codec, Frame, Message}, BoxedSocket};
use actix_codec::{Framed};
use std::time::{Duration, Instant};
use bytes::Bytes;
use futures::stream::{SplitSink, StreamExt};
use crate::helpers;
use crate::error::*;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;

pub struct DefaultWsActor {
    inner: SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>,
    handler: Box<dyn WsHandler>,
    hb: Instant,
    conn_backoff: ExponentialBackoff,
    pub url: String
}

pub trait WsHandler {
    /// Handle incoming messages
    fn handle_in(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, msg: Bytes);
    fn handle_started(&mut self, w: &mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>);
}

#[derive(Message)]
#[rtype(result = "()")]
struct ClientCommand(String);

impl Actor for DefaultWsActor
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");
    }
}

impl actix::Supervised for DefaultWsActor {
    fn restarting(&mut self, ctx: &mut <Self as Actor>::Context) {
        let url = self.url.clone();
        let client1 = helpers::new_ws_client(url.clone());
        client1
            .into_actor(self)
            .map(move |res, act, ctx| match res {
                Ok(client) => {
                    let (sink, stream) = client.split();
                    DefaultWsActor::add_stream(stream, ctx);
                    act.conn_backoff.reset();
                    act.inner = SinkWrite::new(sink, ctx);
                }
                Err(err) => {
                    error!("Can not connect to websocket {} : {}", url, err);
                    // re-connect with backoff time.
                    // we stop current context, supervisor will restart it.
                    if let Some(timeout) = act.conn_backoff.next_backoff() {
                        ctx.run_later(timeout, |_, ctx| ctx.stop());
                    }
                }
            }).wait(ctx);
    }
}

impl DefaultWsActor
{
    pub async fn new(wss_url: &str, handler: Box<dyn WsHandler>) -> Result<Addr<DefaultWsActor>> {
        let url = wss_url.to_string();
        let c = helpers::new_ws_client(url.clone()).await?;
        let (sink, stream) = c.split();
        let mut conn_backoff = ExponentialBackoff::default();
        conn_backoff.max_elapsed_time = None;
        Ok(Supervisor::start(move |ctx| {
            DefaultWsActor::add_stream(stream, ctx);
            DefaultWsActor { inner: SinkWrite::new(sink, ctx), handler, hb: Instant::now(), url: url.clone(), conn_backoff }
        }))
    }
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(30, 0), |act, ctx| {
            act.inner.write(Message::Ping(Bytes::from_static(b""))).unwrap();
            act.hb(ctx);
            // client should also check for a timeout here, similar to the
            // server code
        });
    }
}

/// Handle stdin commands
impl Handler<ClientCommand> for DefaultWsActor
{
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _ctx: &mut Context<Self>) {
        self.inner.write(Message::Text(msg.0)).unwrap();
    }
}

/// Handle server websocket messages
impl StreamHandler<std::result::Result<Frame, WsProtocolError>> for DefaultWsActor
{
    fn handle(&mut self, msg: std::result::Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        match msg {
            Ok(Frame::Ping(msg)) => {
                self.hb = Instant::now();
                self.inner.write(Message::Pong(Bytes::copy_from_slice(&msg)));
            }
            Ok(Frame::Text(txt)) => {
                self.handler.handle_in(&mut self.inner, txt);
            }
            _ => {
                ();
            }
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Connected");
        self.handler.handle_started(&mut self.inner);
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

impl actix::io::WriteHandler<WsProtocolError> for DefaultWsActor
{}

pub trait ExchangeBot {
    /// Returns the address of the exchange actor
    fn is_connected(&self) -> bool;
}

