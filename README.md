# ibento-rs / lighthouse

![bento](./bento.png)

tower(_grpc) + ibents => lighthouse

Ibento implements a gRPC server that provides event streaming capabilities
(event bus) similar to Kafka, but allows filtering and replaying historic data.

gRPC was chosen over GraphQL because the event model is quite flat, and because
it's easy to generate bindings for it in various programming languages.

# Build

```
cd server
cargo run --bin ibento-server
cargo run --bin ibento-client
cd client
iex -S mix
iex> Ibento.Client.subscribe(%{topics: ["a", "b"]})
```

