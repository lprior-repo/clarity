-module(pokemon_server).
-export([start/2]).

%% Start an HTTP server using Erlang's inets httpd
start(Port, Store) ->
    application:ensure_all_started(inets),

    %% Store the Gleam store in a persistent term for handler access
    persistent_term:put(pokemon_store, Store),

    %% Start the HTTP server
    {ok, _Pid} = inets:start(httpd, [
        {port, Port},
        {server_name, "pokemon_api"},
        {server_root, "."},
        {document_root, "."},
        {modules, [pokemon_handler]},
        {bind_address, any}
    ]),
    {ok, nil}.
