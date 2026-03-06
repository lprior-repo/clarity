-module(pokemon_handler).
-export([do/1]).

-include_lib("inets/include/httpd.hrl").

do(ModData) ->
    Method = ModData#mod.method,
    Path = ModData#mod.request_uri,
    Body = get_body(ModData),
    Store = persistent_term:get(pokemon_store),

    %% Call the Gleam handler
    {Status, ResponseBody} = pokemon_api:handle_request(
        list_to_binary(Method),
        list_to_binary(Path),
        list_to_binary(Body),
        Store
    ),

    %% Build response
    Headers = [
        {content_type, "application/json"},
        {content_length, integer_to_list(byte_size(ResponseBody))}
    ],

    {proceed, [
        {response, {response, [{code, Status} | Headers], binary_to_list(ResponseBody)}}
    ]}.

get_body(ModData) ->
    case ModData#mod.entity_body of
        undefined -> "";
        Body -> Body
    end.
