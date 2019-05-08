#![allow(dead_code)]
#![allow(unused_variables)]

use futures01::{Future, Stream};
use hyper::client::connect::{Destination, HttpConnector};
use std::time::Instant;

use tower_grpc::Request;
use tower_hyper::{client, util};
use tower_util::MakeService;

use ibento::grpc::SubscribeRequest;

use ibento::data;

pub fn main() {
    let _ = ::env_logger::init();

    let uri: http::Uri = format!("http://localhost:5600").parse().unwrap();

    let dst = Destination::try_from_uri(uri.clone()).unwrap();
    let connector = util::Connector::new(HttpConnector::new(4));
    let settings = client::Builder::new().http2_only(true).clone();
    let mut make_client = client::Connect::new(connector, settings);

    let rg = make_client
        .make_service(dst)
        .map_err(|e| {
            panic!("HTTP/2 connection failed; err={:?}", e);
        })
        .map(move |conn| {
            use ibento::grpc::client::Ibento;

            let conn = tower_request_modifier::Builder::new()
                .set_origin(uri)
                .build(conn)
                .unwrap();

            Ibento::new(conn)
        })
        .and_then(|mut client| {
            let start = Instant::now();
            let r_feature = client
                .subscribe(Request::new(SubscribeRequest {
                    after: "".to_owned(),
                    limit: 5,
                    topics: vec!["a".to_owned(), "b".to_owned()],
                }))
                .map_err(|e| eprintln!("Subscribe request failed; err={:?}", e))
                .and_then(|response| {
                    let inbound = response.into_inner();
                    inbound
                        .for_each(|event| {
                            println!("EVENT = {:?}", event);
                            Ok(())
                        })
                        .map_err(|e| eprintln!("gRPC inbound stream error: {:?}", e))
                });
            // let outbound = Interval::new_interval(Duration::from_secs(1))
            //     .map(move |t| {
            //         let elapsed = t.duration_since(start);
            //         RouteNote {
            //             location: Some(Point {
            //                 latitude: 409146138 + elapsed.as_secs() as i32,
            //                 longitude: -746188906,
            //             }),
            //             message: format!("at {:?}", elapsed),
            //         }
            //     })
            //     .map_err(|e| panic!("timer error; err={:?}", e));
            // let r_chat = client
            //     .route_chat(Request::new(outbound))
            //     .map_err(|e| {
            //         eprintln!("RouteChat request failed; err={:?}", e);
            //     })
            //     .and_then(|response| {
            //         let inbound = response.into_inner();
            //         inbound
            //             .for_each(|note| {
            //                 println!("NOTE = {:?}", note);
            //                 Ok(())
            //             })
            //             .map_err(|e| eprintln!("gRPC inbound stream error: {:?}", e))
            //     });
            tokio::spawn(r_feature /*.and_then(|()| r_chat)*/)
        });

    tokio::run(rg);
}
