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
    pub data: serde_json::Value,
    pub metadata: serde_json::Value,
    pub debug: bool,
    pub inserted_at: chrono::NaiveDateTime,
}

impl From<Event> for crate::grpc::Event {
    fn from(event: Event) -> Self {
        Self {
            event_id: event.id.to_string().to_owned(),
            r#type: event.type_,
            correlation: event.correlation.unwrap_or(String::from("")),
            causation: event.causation.unwrap_or(String::from("")),
            debug: event.debug,
            inserted_at: Some(prost_types::Timestamp {
                seconds: event.inserted_at.timestamp(),
                nanos: event.inserted_at.timestamp_subsec_nanos() as i32,
            }),
            data: Some(event.data.into()),
            metadata: Some(event.metadata.into()),
        }
    }
}
