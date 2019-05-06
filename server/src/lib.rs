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

pub mod grpc {
    include!(concat!(env!("OUT_DIR"), "/ibento.rs"));
}
pub mod schema;

pub mod data;
