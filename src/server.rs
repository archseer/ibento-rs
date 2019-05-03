#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(async_await, await_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate dotenv;

// mod data;
pub mod ibento {
    include!(concat!(env!("OUT_DIR"), "/ibento.rs"));
}
use crate::ibento::{server, Event, SubscribeRequest};

use dotenv::dotenv;

use futures01::sync::mpsc;
use futures01::{future, Future, Sink, Stream};
use tokio::executor::DefaultExecutor;
use tokio::net::TcpListener;
use tower_grpc::{Request, Response};
use tower_hyper::server::{Http, Server};

use futures::{
  compat::*,
  future::{FutureExt, TryFutureExt},
  io::AsyncWriteExt,
  stream::StreamExt,
  sink::SinkExt,
};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::Connection;
use diesel::PgConnection;
use r2d2::{CustomizeConnection, Pool, PooledConnection};
use tokio_threadpool::blocking;
// use bb8::Pool;
// use bb8_postgres::PostgresConnectionManager;

use std::sync::Arc;
// use std::time::Instant;

#[derive(Clone)]
struct IBento {
    state: Arc<State>,
}

struct State {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl ibento::server::IBento for IBento {
    // type GetFeatureFuture = future::FutureResult<Response<Feature>, tower_grpc::Status>;

    // /// returns the feature at the given point.
    // fn get_feature(&mut self, request: Request<Point>) -> Self::GetFeatureFuture {
    //     println!("GetFeature = {:?}", request);

    //     for feature in &self.state.features[..] {
    //         if feature.location.as_ref() == Some(request.get_ref()) {
    //             return future::ok(Response::new(feature.clone()));
    //         }
    //     }

    //     // Otherwise, return some other feature?
    //     let response = Response::new(Feature {
    //         name: "".to_string(),
    //         location: None,
    //     });

    //     future::ok(response)
    // }

    type SubscribeStream = Box<Stream<Item = Event, Error = tower_grpc::Status> + Send>;
    type SubscribeFuture =
        future::FutureResult<Response<Self::SubscribeStream>, tower_grpc::Status>;

    /// Lists all features contained within the given bounding Rectangle.
    fn subscribe(&mut self, request: Request<SubscribeRequest>) -> Self::SubscribeFuture {
        use std::thread;

        println!("Subscribe = {:?}", request);

        let (tx, rx) = mpsc::channel(4);

        let state = self.state.clone();

        thread::spawn(move || {
            let mut tx = tx.wait();

            // for feature in &state.features[..] {
            //     if in_range(feature.location.as_ref().unwrap(), request.get_ref()) {
            //         println!("  => send {:?}", feature);
            //         tx.send(feature.clone()).unwrap();
            //     }
            // }
            tx.send(Event {
                event_id: "abc".to_owned(),
                r#type: "VehicleEvent".to_owned(),
                correlation: "a".to_owned(),
                causation: "b".to_owned(),
                data: None,
                metadata: None,
                inserted_at: 1,
                debug: false,
            })
            .unwrap();

            println!(" /// done sending");
        });

        let rx = rx.map_err(|_| unimplemented!());
        future::ok(Response::new(Box::new(rx)))
    }

    //type RecordRouteFuture =
    //    Box<Future<Item = Response<RouteSummary>, Error = tower_grpc::Status> + Send>;

    ///// Records a route composited of a sequence of points.
    /////
    ///// It gets a stream of points, and responds with statistics about the
    ///// "trip": number of points,  number of known features visited, total
    ///// distance traveled, and total time spent.
    //fn record_route(&mut self, request: Request<Streaming<Point>>) -> Self::RecordRouteFuture {
    //    println!("RecordRoute = {:?}", request);

    //    let now = Instant::now();
    //    let state = self.state.clone();

    //    let response = request
    //        .into_inner()
    //        .map_err(|e| {
    //            println!("  !!! err={:?}", e);
    //            e
    //        })
    //        // Iterate over all points, building up the route summary
    //        .fold(
    //            (RouteSummary::default(), None),
    //            move |(mut summary, last_point), point| {
    //                println!("  ==> Point = {:?}", point);

    //                // Increment the point count
    //                summary.point_count += 1;

    //                // Find features
    //                for feature in &state.features[..] {
    //                    if feature.location.as_ref() == Some(&point) {
    //                        summary.feature_count += 1;
    //                    }
    //                }

    //                // Calculate the distance
    //                if let Some(ref last_point) = last_point {
    //                    summary.distance += calc_distance(last_point, &point);
    //                }

    //                Ok::<_, tower_grpc::Status>((summary, Some(point)))
    //            },
    //        )
    //        // Map the route summary to a gRPC response
    //        .map(move |(mut summary, _)| {
    //            println!("  => Done = {:?}", summary);

    //            summary.elapsed_time = now.elapsed().as_secs() as i32;
    //            Response::new(summary)
    //        });

    //    Box::new(response)
    //}

    //type RouteChatStream = Box<Stream<Item = RouteNote, Error = tower_grpc::Status> + Send>;
    //type RouteChatFuture =
    //    future::FutureResult<Response<Self::RouteChatStream>, tower_grpc::Status>;

    //// Receives a stream of message/location pairs, and responds with a stream
    //// of all previous messages at each of those locations.
    //fn route_chat(&mut self, request: Request<Streaming<RouteNote>>) -> Self::RouteChatFuture {
    //    println!("RouteChat = {:?}", request);

    //    let state = self.state.clone();

    //    let response = request
    //        .into_inner()
    //        .map(move |note| {
    //            let location = note.location.clone().unwrap();
    //            let mut notes = state.notes.lock().unwrap();
    //            let notes = notes.entry(location).or_insert(vec![]);

    //            notes.push(note);

    //            stream::iter_ok(notes.clone())
    //        })
    //        .flatten();

    //    future::ok(Response::new(Box::new(response)))
    //}
}

#[runtime::main(runtime_tokio::Tokio)]
pub async fn main() -> std::io::Result<()> {
    let _ = ::env_logger::init();

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let manager = ConnectionManager::new(database_url);
    let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("could not initiate test db pool");
    // let manager = PostgresConnectionManager::new(
    //     "postgresql://postgres:postgres@localhost:5432/ibento_dev",
    //     tokio_postgres::tls::NoTls,
    // );

    // let pool = await!(
    //     Pool::builder()
    //         .build(manager)
	    // .compat()
    //         .map_err(|e| bb8::RunError::User(e))
    // ).unwrap();

    let handler = IBento {
        state: Arc::new(State { pool }),
    };

    let new_service = server::IBentoServer::new(handler);

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
	    runtime::spawn(serve.map_err(|e| error!("h2 error: {:?}", e)).compat());

	    Ok::<(), std::io::Error>(())
	});
 
    await!(serve.compat())?;

    Ok(())
}
