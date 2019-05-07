%%%-------------------------------------------------------------------
%% @doc Behaviour to implement for grpc service ibento.Ibento.
%% @end
%%%-------------------------------------------------------------------

%% this module was generated on 2019-05-07T06:11:46+00:00 and should not be modified manually

-module(ibento_ibento_bhvr).

%% @doc 
-callback subscribe(ibento_ibento_pb:subscribe_request(), grpcbox_stream:t()) ->
    ok | grpcbox_stream:grpc_error_response().

