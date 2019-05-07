%%%-------------------------------------------------------------------
%% @doc Client module for grpc service ibento.Ibento.
%% @end
%%%-------------------------------------------------------------------

%% this module was generated on 2019-05-07T06:11:46+00:00 and should not be modified manually

-module(ibento_ibento_client).

-compile(export_all).
-compile(nowarn_export_all).

-include_lib("grpcbox/include/grpcbox.hrl").

-define(is_ctx(Ctx), is_tuple(Ctx) andalso element(1, Ctx) =:= ctx).

-define(SERVICE, 'ibento.Ibento').
-define(PROTO_MODULE, 'ibento_ibento_pb').
-define(MARSHAL_FUN(T), fun(I) -> ?PROTO_MODULE:encode_msg(I, T) end).
-define(UNMARSHAL_FUN(T), fun(I) -> ?PROTO_MODULE:decode_msg(I, T) end).
-define(DEF(Input, Output, MessageType), #grpcbox_def{service=?SERVICE,
                                                      message_type=MessageType,
                                                      marshal_fun=?MARSHAL_FUN(Input),
                                                      unmarshal_fun=?UNMARSHAL_FUN(Output)}).

%% @doc 
-spec subscribe(ibento_ibento_pb:subscribe_request()) ->
    {ok, grpcbox_client:stream()} | grpcbox_stream:grpc_error_response().
subscribe(Input) ->
    subscribe(ctx:new(), Input, #{}).

-spec subscribe(ctx:t() | ibento_ibento_pb:subscribe_request(), ibento_ibento_pb:subscribe_request() | grpcbox_client:options()) ->
    {ok, grpcbox_client:stream()} | grpcbox_stream:grpc_error_response().
subscribe(Ctx, Input) when ?is_ctx(Ctx) ->
    subscribe(Ctx, Input, #{});
subscribe(Input, Options) ->
    subscribe(ctx:new(), Input, Options).

-spec subscribe(ctx:t(), ibento_ibento_pb:subscribe_request(), grpcbox_client:options()) ->
    {ok, grpcbox_client:stream()} | grpcbox_stream:grpc_error_response().
subscribe(Ctx, Input, Options) ->
    grpcbox_client:stream(Ctx, <<"/ibento.Ibento/Subscribe">>, Input, ?DEF(subscribe_request, event, <<"ibento.SubscribeRequest">>), Options).

