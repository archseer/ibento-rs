defmodule Ibento.Client do
  @moduledoc """
  """
  require Logger

  # <<(:erlang.system_time(:millisecond) - :timer.minutes(5))::unsigned-big-integer-unit(1)-size(48), 0::80>>`
  # Or if you wanted it to be `before` a certain time non-inclusive:
  # `<<(:erlang.system_time(:millisecond) - :timer.minutes(5))::unsigned-big-integer-unit(1)-size(48), (:erlang.bsl(1, 80) - 1)::80>>

  @spec subscribe(%{topics: [String.t()]}) :: {:ok, stream :: term()} | {:error, reason :: term()}
  def subscribe(request) do
    {:ok, stream} = :ibento_ibento_client.subscribe(request)
    {:ok, _headers} = :grpcbox_client.recv_headers(stream, 5000)
    stream
  end

  def loop(stream, cursor, mod, time) do
    case :grpcbox_client.recv_data(stream, 5000) do
      {:ok, data} ->
        event = :ibento_event.decode(data)
        :ok = mod.perform(event)

        new_cursor = event.ingest_id
        loop(stream, new_cursor, mod, time)
      {:error, :closed} ->
        # :ok = :h2_stream.stop(stream.stream_pid)
        # :grpcbox_client_stream.close_and_flush(stream)
        IO.puts "timing: #{System.convert_time_unit(:timer.now_diff(:erlang.timestamp(), time), :microsecond, :millisecond)}ms"
         consume(mod, cursor)
      :stream_finished ->
        IO.puts "ok"
        consume(mod, cursor)
    end
  end

  def consume(mod, cursor \\ nil) do
    config = mod.config()
    {:ok, cursor} = if cursor, do: {:ok, cursor}, else: mod.fetch_cursor()

    config
    |> Map.put(:after, cursor)
    |> subscribe()
    |> loop(cursor, mod, :erlang.timestamp())
  end

  @type cursor :: String.t()
  @type event :: term
  # @type event :: %Ibento.Client.Event{}

  defmodule Consumer do
    @callback config() :: map()
    @callback fetch_cursor() :: {:ok, Ibento.Client.cursor}
    @callback perform(Ibento.Client.event) :: :ok
  end

  defmodule A do
    @behaviour Consumer

    def config do
      %{
        topics: ["Organization:b0a089cc-0bcf-4f3c-9cab-cf7327938f91"],
        limit: 10
      }
    end

    def fetch_cursor do
      {:ok, "01667f19-9e88-0000-0000-000000000002"}
    end

    def perform(event) do
      # IO.inspect(event)
      IO.inspect(event.ingest_id)
      :ok
    end
  end

  # TODO: implement a wrapper that takes a fn (plus a calculate_cursor fn) and
  # streams responses back into the callback. If a batch completes, keep
  # fetching. If the connection breaks, reopen, but first recalculate the
  # cursor. If no data or connection issues, use exponential backoff.
  #
  # This type of consumer/connection wrapper is nice since it implements most of
  # the boilerplate for us. In the future we could also allow an extra config
  # setting that would enable parallel processing via a pool, without any
  # breaking changes to the API.

  # defmodule Connection do
  #   @behavior :gen_statem

  #   defstruct [:backoff, :callback]

  #   def start_link(callback) do
  #     :gen_statem.start_link(__MODULE__, callback)
  #   end

  #   # def request(pid, request) do
  #   #   :gen_statem.call(pid, {:request, request})
  #   # end

  #   ## :gen_statem callbacks

  #   @impl true
  #   def callback_mode(), do: [:handle_event_function, :state_enter]

  #   @impl true
  #   def init(callback) do
  #     data = %__MODULE__{callback: callback}
  #     actions = [{:next_event, :internal, :connect}]
  #     {:next_state, :disconnected, data, actions}
  #   end

  #   ## Disconnected state

  #   def handle_event(:enter, _, :disconnected, _data), do: :keep_state_and_data

  #   def handle_event(:enter, :connected, :disconnected, data) do
  #     Logger.error("Connection closed")

  #     Enum.each(data.requests, fn {_id, from} ->
  #       :gen_statem.reply(from, {:error, :disconnected})
  #     end)

  #     data = %{data | socket: nil, requests: %{}}

  #     actions = [{:timeout, 500, :reconnect}]
  #     {:keep_state, data, actions}
  #   end

  #   def handle_event(:internal, :connect, :disconnected, data) do
  #     spawn_monitor(fn ->
  #       # outer:
  #       # open_stream()
  #       # loop {
  #       #   fetch_msg do
  #       #     {:ok, msg} -> process
  #       #     {:error, :closed} -> break, refetch cursor
  #       #     {:error, :stream_finished} -> break 'outer
  #       #   end
  #       # }
  #     end)

  #     case :ibento_ibento_client.subscribe(request) do
  #       {:ok, socket} ->
  #         {:next_state, :connected, %{data | socket: socket}}

  #       {:error, error} ->
  #         Logger.error("Connection failed: #{:inet.format_error(error)}")
  #         :keep_state_and_data
  #     end
  #   end

  #   def handle_event(:timeout, :reconnect, :disconnected, data) do
  #     actions = [{:next_event, :internal, :connect}]
  #     {:keep_state, data, actions}
  #   end

  #   def handle_event({:call, from}, {:request, request}, :disconnected, data) do
  #     actions = [{:reply, from, {:error, :disconnected}}]
  #     {:keep_state_and_data, actions}
  #   end

  #   ## Connected state

  #   def handle_event(:enter, _old_state, :connected, _data), do: :keep_state_and_data

  #   def handle_event(:info, {:tcp_closed, socket}, :connected, %{socket: socket} = data) do
  #     {:next_state, :disconnected, data}
  #   end

  #   def handle_event({:call, from}, {:request, request}, :connected, data) do
  #     case :gen_tcp.send(data.socket, encode_request(request)) do
  #       :ok ->
  #         data = %{data | requests: Map.put(data.requests, request.id, from)}
  #         {:keep_state, data}

  #       {:error, _reason} ->
  #         :ok = :gen_tcp.close(socket)
  #         {:next_state, :disconnected, data}
  #     end
  #   end

  #   def handle_event(:info, {:tcp, socket, packet}, :connected, %{socket: socket} = data) do
  #     response = decode_response(packet)
  #     {from, requests} = Map.pop(data.requests, response.id)

  #     :gen_statem.reply(from, {:ok, response})

  #     {:keep_state, %{data | requests: requests}}
  #   end
  # end
end
