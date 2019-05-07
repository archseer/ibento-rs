// generated via `diesel print-schema`'s output.

table! {
    events (id) {
        id -> Uuid,
        ingest_id -> Uuid,
        #[sql_name = "type"]
        type_ -> Text,
        correlation -> Nullable<Text>,
        causation -> Nullable<Text>,
        data -> Jsonb,
        metadata -> Jsonb,
        debug -> Bool,
        inserted_at -> Timestamp,
        // vclock -> Int8,
    }
}

table! {
    schema_migrations (version) {
        version -> Int8,
        inserted_at -> Nullable<Timestamp>,
    }
}

table! {
    spatial_ref_sys (srid) {
        srid -> Int4,
        auth_name -> Nullable<Varchar>,
        auth_srid -> Nullable<Int4>,
        srtext -> Nullable<Varchar>,
        proj4text -> Nullable<Varchar>,
    }
}

table! {
    stream_events (event_id, stream_id) {
        event_id -> Uuid,
        stream_id -> Int8,
    }
}

table! {
    streams (id) {
        id -> Int8,
        source -> Text,
        inserted_at -> Timestamp,
    }
}

joinable!(stream_events -> events (event_id));
joinable!(stream_events -> streams (stream_id));

allow_tables_to_appear_in_same_query!(
    events,
    schema_migrations,
    spatial_ref_sys,
    stream_events,
    streams,
);
