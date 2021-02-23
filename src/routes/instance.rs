use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::{
    get,
    web::{Payload, Query},
    HttpRequest, HttpResponse, Result,
};
use actix_web_actors::ws;
use serde::Deserialize;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
struct InstanceForm {
    id: String,
}

/// the websocket handler function
#[get("/instance")]
async fn ws_index(
    r: HttpRequest,
    form: Query<InstanceForm>,
    stream: Payload,
) -> Result<HttpResponse> {
    println!("{:?}", r);
    let res = ws::start(InstanceWS::new(form.into_inner().id), &r, stream);
    println!("{:?}", res);
    res
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct InstanceWS {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    //UUID of the instance
    id: String,
}

impl Actor for InstanceWS {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for InstanceWS {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl InstanceWS {
    fn new(id: String) -> Self {
        Self {
            id: id,
            hb: Instant::now(),
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}