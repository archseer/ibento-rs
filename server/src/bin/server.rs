#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(async_await, await_macro)]

use ibento::grpc::{server, Event, SubscribeRequest};

use dotenv::dotenv;

use futures01::{Future, Stream};
use tokio::executor::DefaultExecutor;
use tokio::net::TcpListener;
use tower_grpc::{Request, Response};
use tower_hyper::server::{Http, Server};

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

#[derive(Clone)]
struct Ibento {
    state: Arc<State>,
}

struct State {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl ibento::grpc::server::Ibento for Ibento {
    type SubscribeStream = Box<Stream<Item = Event, Error = tower_grpc::Status> + Send>;
    type SubscribeFuture =
        futures01::future::FutureResult<Response<Self::SubscribeStream>, tower_grpc::Status>;

    /// Lists all features contained within the given bounding Rectangle.
    fn subscribe(&mut self, request: Request<SubscribeRequest>) -> Self::SubscribeFuture {
        println!("Subscribe = {:?}", request);

        let request = request.into_inner();
        // TODO: I wish I could mark the field as prost optional, then I can do request.limit.or(5)
        let limit = if request.limit != 0 { request.limit } else { 5 };
        let after = if request.after != "" { Some(request.after.clone()) } else { None };

        let (mut tx, rx) = mpsc::channel::<Result<Event, tower_grpc::Status>>(4);

        let state = self.state.clone();

        runtime::spawn(async move {
            // TODO error handling
            let connection = state.pool.get().unwrap();
            let data = await!(blocking_fn(move || { 
                use schema::{events, streams, stream_events};

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
            }));

            for event in data {
                // println!("Event = {:?}", event);
                await!(tx.send(Ok(event.into()))).unwrap();
            }
        });

        futures01::future::ok(Response::new(Box::new(rx.compat())))
    }
}

fn blocking_fn<F, T>(mut f: F) -> impl futures::future::Future<Output = T>
where F: FnMut() -> T {
    futures::future::poll_fn(move |_| {
        match tokio_threadpool::blocking(|| {
            f()
        }).expect("the threadpool shut down") {
            futures01::Async::Ready(n) => futures::task::Poll::Ready(n),
            futures01::Async::NotReady => futures::task::Poll::Pending
        }
    })
}

#[runtime::main(runtime_tokio::Tokio)]
pub async fn main() -> std::io::Result<()> {
    let _ = ::env_logger::init();

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::new(database_url);

    let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("could not initiate test db pool");

    let handler = Ibento {
        state: Arc::new(State { pool }),
    };

    let new_service = server::IbentoServer::new(handler);

    let mut server = Server::new(new_service);
    let http = Http::new().http2_only(true).clone();
    let http = http.with_executor(DefaultExecutor::current());

    let addr = "127.0.0.1:5600".parse().unwrap();
    let bind = TcpListener::bind(&addr).expect("bind");

    println!("Listening on {:?}", addr);

    let serve = bind
	.incoming()
	.for_each(move |sock| {
	    if let Err(e) = sock.set_nodelay(true) {
		return Err(e);
	    }

	    let serve = server.serve_with(sock, http.clone());
	    runtime::spawn(serve.map_err(|e| panic!("h2 error: {:?}", e)).compat());

	    Ok::<(), std::io::Error>(())
	});
 
    await!(serve.compat())?;

    Ok(())
}
