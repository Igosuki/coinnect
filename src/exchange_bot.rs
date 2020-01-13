use actix::{Context, io::SinkWrite, Actor, Handler, StreamHandler, AsyncContext, ActorContext, Addr};
use awc::{error::WsProtocolError, ws::{Codec, Frame, Message}, BoxedSocket};
use actix_codec::{Framed};
use std::time::{Duration, Instant};
use bytes::Bytes;
use futures::stream::{SplitSink, StreamExt};
use crate::helpers;

pub struct DefaultWsActor {
    inner: SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>,
    handler: Box<dyn WsHandler>,
    hb: Instant
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
    fn restarting(&mut self, _: &mut Context<DefaultWsActor>) {
        println!("restarting exchange bot...");
    }
}

impl DefaultWsActor
{
    pub async fn new(wss_url: &str, handler: Box<dyn WsHandler>) -> Addr<DefaultWsActor> {
        let c = helpers::new_ws_client(wss_url).await;
        let (sink, stream) = c.split();
        actix::Supervisor::start(|ctx| {
            DefaultWsActor::add_stream(stream, ctx);
            DefaultWsActor { inner: SinkWrite::new(sink, ctx), handler, hb: Instant::now() }
        })
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
impl StreamHandler<Result<Frame, WsProtocolError>> for DefaultWsActor
{
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
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

