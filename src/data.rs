#![allow(unused_imports)]

// use super::schema::events;

use diesel;
use diesel::deserialize::Queryable;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

#[derive(Debug, Queryable)]
pub struct Event {
    pub id: Uuid,
    pub type_: String,
    pub correlation: String,
    pub causation: String,
    pub data: serde_json::Value,
    pub metadata: serde_json::Value,
    pub debug: bool,
    pub inserted_at: chrono::NaiveDateTime,
}

impl From<Event> for crate::grpc::Event {
    fn from(item: Event) -> Self {
        // Self { .. }
        unimplemented!()
    }
}
