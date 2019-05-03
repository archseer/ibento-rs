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
    pub correlation: Option<String>,
    pub causation: Option<String>,
    // pub data: serde_json::Value,
    // pub metadata: serde_json::Value,
    pub debug: bool,
    pub inserted_at: chrono::NaiveDateTime,
}

// impl From<Event> for crate::grpc::Event {
//     fn from(event: Event) -> Self {
//         Self {
//             event_id: event.id.to_string().to_owned(),
//             r#type: event.type_,
//             correlation: event.correlation,
//             causation: event.causation,
//             debug: event.debug,
//             // TODO use iso 8601
//             inserted_at: event.inserted_at.timestamp_nanos() as u64,
//             data: None,
//             metadata: None,
//             // TODO: cross encode data and metadata
//         }
//     }
// }
