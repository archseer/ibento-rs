%% -*- mode: erlang; tab-width: 4; indent-tabs-mode: 1; st-rulers: [70] -*-
%% vim: ts=4 sw=4 ft=erlang noet
-module(ibento_google_struct).

%% API
-export([decode/1]).
-export([decode_value/1]).
-export([encode/1]).
-export([encode_value/1]).

%% Macros
% See https://www.ecma-international.org/ecma-262/6.0/#sec-number.max_safe_integer
-define(MAX_SAFE_INTEGER, 9007199254740991).
% See https://www.ecma-international.org/ecma-262/6.0/#sec-number.min_safe_integer
-define(MIN_SAFE_INTEGER, -9007199254740991).

%%%===================================================================
%%% API functions
%%%===================================================================

decode(#{fields := Map}) when is_map(Map) ->
	maps:fold(fun do_decode_map/3, #{}, Map).

decode_value(#{kind := {null_value, 'NULL_VALUE'}}) ->
	nil;
decode_value(#{kind := {number_value, V}})
		when is_float(V)
		andalso V == round(V)
		andalso round(V) =< ?MAX_SAFE_INTEGER
		andalso round(V) >= ?MIN_SAFE_INTEGER ->
	round(V);
decode_value(#{kind := {number_value, V}}) when is_number(V) ->
	V;
decode_value(#{kind := {string_value, V}}) when is_binary(V) ->
	V;
decode_value(#{kind := {bool_value, V}}) when is_boolean(V) ->
	V;
decode_value(#{kind := {struct_value, V}}) when is_map(V) ->
	decode(V);
decode_value(#{kind := {list_value, #{values := Vs}}}) when is_list(Vs) ->
	[decode_value(V) || V <- Vs].

encode(Map) when is_map(Map) ->
	#{
		fields => maps:fold(fun do_encode_map/3, #{}, Map)
	}.

encode_value(nil) ->
	#{kind => {null_value, 'NULL_VALUE'}};
encode_value(V)
		when is_integer(V)
		andalso V =< ?MAX_SAFE_INTEGER
		andalso V >= ?MIN_SAFE_INTEGER ->
	#{kind => {number_value, V}};
encode_value(V) when is_float(V) ->
	#{kind => {number_value, V}};
encode_value(V) when is_binary(V) ->
	#{kind => {string_value, V}};
encode_value(V) when is_boolean(V) ->
	#{kind => {bool_value, V}};
encode_value(V) when is_map(V) ->
	#{kind => {struct_value, encode(V)}};
encode_value(V) when is_list(V) ->
	#{kind => {list_value, #{values => [encode_value(T) || T <- V]}}}.

%%%-------------------------------------------------------------------
%%% Internal functions
%%%-------------------------------------------------------------------

%% @private
do_decode_map(K, V, Acc) when is_binary(K) ->
	maps:put(K, decode_value(V), Acc).

%% @private
do_encode_map(K, V, Acc) when is_binary(K) ->
	maps:put(K, encode_value(V), Acc);
do_encode_map(K, V, Acc) when is_atom(K) ->
	maps:put(erlang:atom_to_binary(K, utf8), encode_value(V), Acc).
