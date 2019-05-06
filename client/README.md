# Ibento.Client

**TODO: Add description**

## Installation

If [available in Hex](https://hex.pm/docs/publish), the package can be installed
by adding `ibento_client` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:ibento_client, git: "<..>", sparse: "client"}
  ]
end
```

Documentation can be generated with [ExDoc](https://github.com/elixir-lang/ex_doc)
and published on [HexDocs](https://hexdocs.pm). Once published, the docs can
be found at [https://hexdocs.pm/client](https://hexdocs.pm/client).

### Generate bindings

```elixir
rebar grpc gen
```

