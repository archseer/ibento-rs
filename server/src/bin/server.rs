#![feature(async_await)]

use ibento::grpc::{server, Event, SubscribeRequest};

use dotenv::dotenv;

use futures01::{Future, Stream};
use tokio::net::TcpListener;
use tower_grpc::{Request, Response};
use tower_hyper::server::{Http, Server};
use tokio_trace::{field, Level};
use tokio_trace_futures::Instrument;

use futures::{
  compat::*,
  future::{FutureExt, TryFutureExt},
  io::AsyncWriteExt,
  stream::{StreamExt, TryStreamExt},
  sink::SinkExt,
  channel::mpsc,
};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::Connection;
use diesel::PgConnection;
use r2d2::{CustomizeConnection, Pool, PooledConnection};

use std::sync::Arc;

use ibento::{schema, data};

#[macro_use]
extern crate tokio_trace;

#[derive(Clone)]
struct Ibento {
    state: Arc<State>,
}

type DbPool = Pool<ConnectionManager<PgConnection>>;

struct State {
    pool: DbPool,
}

fn fetch_events(pool: DbPool, request: SubscribeRequest) -> Vec<data::Event> {
    let connection = pool.get().unwrap();
    use schema::{events, streams, stream_events};

    // TODO: I wish I could mark the field as prost optional, then I can do request.limit.or(5)
    let limit = if request.limit != 0 { request.limit } else { 5 };
    let after = if request.after != "" { Some(request.after.clone()) } else { None };
    let topics = if request.topics.is_empty() { vec![String::from("$all")] } else { request.topics.clone() };

    let mut query = events::table
        .limit(limit as i64)
        .order_by(events::ingest_id.asc())
        .into_boxed();

    if let Some(after) = &after {
        // TODO: extract parsing, return tower_grpc::Status::new(Code::InvalidArgument, "message here")
        let after = uuid::Uuid::parse_str(after).expect("Invalid after ulid");
        query = query.filter(events::ingest_id.gt(after));
    }

    use diesel::pg::expression::dsl::any;

    query
        .left_join(stream_events::table.left_join(streams::table))
        .filter(streams::source.eq(any(topics)))
        .load::<crate::data::Event>(&connection)
        .expect("Error loading events")
}

impl ibento::grpc::server::Ibento for Ibento {
    type SubscribeStream = Box<Stream<Item = Event, Error = tower_grpc::Status> + Send>;
    type SubscribeFuture =
        futures01::future::FutureResult<Response<Self::SubscribeStream>, tower_grpc::Status>;

    /// Lists all features contained within the given bounding Rectangle.
    fn subscribe(&mut self, request: Request<SubscribeRequest>) -> Self::SubscribeFuture {
        println!("Subscribe = {:?}", request);

        let request = request.into_inner();

        let (mut tx, rx) = mpsc::channel::<Result<Event, tower_grpc::Status>>(4);

        let pool = self.state.pool.clone();

        tokio::spawn(async move {
            // TODO error handling
            /*let data = blocking_fn(|| {*/
            /*}).await;*/
            let data = fetch_events(pool, request);

            // for event in data {
            //     println!("Event = {:?}", event);
            //     // tx.send(Ok(event.into())).await?
            // }
            // send_all might be better
            let mut stream = futures::stream::iter(data.into_iter().map(|v| Ok(v.into())));
            tx.send_all(&mut stream).await; //?;

            // tx.send(Err(tower_grpc::Status::new(tower_grpc::Code::Ok, ""))).await.unwrap();
            // Ok::<(), mpsc::SendError>(())
        }.unit_error().boxed().compat());

        futures01::future::ok(Response::new(Box::new(rx.compat())))
    }
}

fn blocking_fn<F, T>(f: F) -> impl futures::future::Future<Output = T>
where F: FnOnce() -> T {
    // this f.take() trick is from https://github.com/gotham-rs/gotham/blob/431f1ca0da26c89e7e94079e89f6a9ca39c69090/middleware/diesel/src/repo.rs#L145-L148
    let mut f = Some(f);
    futures::future::poll_fn(move |_| {
        match tokio_threadpool::blocking(|| {
            f.take().unwrap()()
        }).expect("the threadpool shut down") {
            futures01::Async::Ready(n) => futures::task::Poll::Ready(n),
            futures01::Async::NotReady => futures::task::Poll::Pending
        }
    })
}

// #[runtime::main(runtime_tokio::Tokio)]
// pub async fn main() -> std::io::Result<()> {
pub fn main() {
    let _ = ::env_logger::init();

    dotenv().ok();

    let subscriber = tokio_trace_fmt::FmtSubscriber::builder()
        // .with_filter(tokio_trace_fmt::filter::EnvFilter::from(
        //     "tower_h2_server=trace",
        // ))
        .full()
        .finish();

    tokio_trace::subscriber::with_default(subscriber, || {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::new(database_url);

        let pool = r2d2::Pool::builder()
                .max_size(32)
                .build(manager)
                .expect("could not initiate test db pool");

        let handler = Ibento {
            state: Arc::new(State { pool }),
        };

        let addr: std::net::SocketAddr = "127.0.0.1:5600".parse().unwrap();

       // let serve_span = span!(
       //      Level::INFO,
       //      "serve",
       //      local_ip = field::debug(addr.ip()),
       //      local_port = addr.port() as u64
       //  );

        let new_service = server::IbentoServer::new(handler);

        // let new_service =
        //     tokio_trace_tower_http::InstrumentedMakeService::new::<http::Request<tower_hyper::Body>>>(new_service, serve_span.clone());

        let mut server = Server::new(new_service);
        let http = Http::new().http2_only(true).clone();

        let bind = TcpListener::bind(&addr).expect("bind");

        println!("Listening on {:?}", addr);

        let serve = bind
            .incoming()
            .for_each(move |sock| {
                if let Err(e) = sock.set_nodelay(true) {
                    return Err(e);
                }
                let addr = sock.peer_addr().expect("can't get addr");
                let conn_span = span!(
                    Level::ERROR,
                    "conn",
                    remote_ip = field::debug(addr.ip()),
                    remote_port = addr.port() as u64
                );
                let conn_span2 = conn_span.clone();
                conn_span.enter(|| {
                    let serve = server.serve_with(sock, http.clone())
                            .map_err(|e| println!("error {:?}", e))
                            .and_then(|_| {
                                debug!("response finished");
                                futures01::future::ok(())
                            })
                        .instrument(conn_span2);
                    /*runtime*/tokio::spawn(serve.map_err(|e| println!("h2 error: {:?}", e)));
                });

                Ok::<(), std::io::Error>(())
            })
            .map_err(|e| eprintln!("accept error: {}", e));

        tokio::run(serve);
        // serve.compat().await; // TODO: ?
    });
}
