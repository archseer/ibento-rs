%% -*- mode: erlang; tab-width: 4; indent-tabs-mode: 1; st-rulers: [70] -*-
%% vim: ts=4 sw=4 ft=erlang noet
-module(ibento_event).

%% Elixir API
-export(['__struct__'/0]).
-export(['__struct__'/1]).
%% API
-export([new/1]).
-export([decode/1]).
-export([encode/1]).

%%%===================================================================
%%% Elixir API functions
%%%===================================================================

'__struct__'() ->
	#{
		'__struct__' => 'Elixir.Ibento.Client.Event',
		event_id => nil,
		ingest_id => nil,
		type => nil,
		causation => nil,
		correlation => nil,
		data => nil,
		metadata => nil,
		debug => nil,
		inserted_at => nil
	}.

'__struct__'(List) when is_list(List) ->
	'__struct__'(maps:from_list(List));
'__struct__'(Map) when is_map(Map) ->
	maps:fold(fun do_struct/3, '__struct__'(), Map).

%%%===================================================================
%%% API functions
%%%===================================================================

new(Struct = #{'__struct__' := ?MODULE}) ->
	Struct;
new(Input) ->
	'__struct__'(Input).

decode(Input) when is_map(Input) ->
	maps:fold(fun do_decode/3, '__struct__'(), Input).

encode(Struct = #{'__struct__' := ?MODULE}) ->
	maps:fold(fun do_encode/3, maps:new(), maps:remove('__struct__', Struct)).

%%%-------------------------------------------------------------------
%%% Internal functions
%%%-------------------------------------------------------------------

%% @private
do_decode(K = data, V, A) when is_map(V) ->
	Value = ibento_google_struct:decode_value(V),
	maps:update(K, Value, A);
do_decode(K = metadata, V, A) when is_map(V) ->
	Value = ibento_google_struct:decode_value(V),
	maps:update(K, Value, A);
do_decode(K = inserted_at, #{seconds := Seconds, nanos := Nanos}, A) ->
	% super annoying but easiest way to ensure Elixir constructs nanosecond
	% {_, 6} precision field.
	V = 'Elixir.System':convert_time_unit(Seconds, second, nanosecond),
	{ok, Value} = 'Elixir.DateTime':from_unix(V, nanosecond),
	Value1 = 'Elixir.DateTime':add(Value, Nanos, nanosecond),
	maps:update(K, Value1, A);
do_decode(K, V, A) when K =/= '__struct__' ->
	maps:update(K, V, A).

%% @private
do_encode(K = data, V, A) when is_map(V) ->
	case map_size(V) of
		0 ->
			A;
		_ ->
			Value = ibento_google_struct:encode_value(V),
			maps:put(K, Value, A)
	end;
do_encode(K = metadata, V, A) when is_map(V) ->
	case map_size(V) of
		0 ->
			A;
		_ ->
			Value = ibento_google_struct:encode_value(V),
			maps:put(K, Value, A)
	end;
do_encode(_, nil, A) ->
	A;
do_encode(K, V, A) ->
  maps:put(K, V, A).

%% @private
do_struct(K = data, V, A) when is_map(V) ->
	maps:update(K, V, A);
do_struct(K = metadata, V, A) when is_map(V) ->
	maps:update(K, V, A);
do_struct(K, V, A) when K =/= '__struct__' ->
	maps:update(K, V, A).
