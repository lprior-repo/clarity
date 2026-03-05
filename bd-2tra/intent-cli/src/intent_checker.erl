-module(intent_checker).
-export([get_or_compile_regex/1]).

%% Regex pattern cache using ETS
%% Initialize cache table on first use
-define(CACHE_TABLE, intent_regex_cache).

%% Initialize ETS table if it doesn't exist
init_cache() ->
    case ets:info(?CACHE_TABLE) of
        undefined ->
            ets:new(?CACHE_TABLE, [named_table, public, {read_concurrency, true}]);
        _ ->
            ok
    end.

%% Get or compile a regex pattern with caching
%% Returns {ok, Regexp} or {error, nil}
get_or_compile_regex(Pattern) when is_binary(Pattern) ->
    init_cache(),
    case ets:lookup(?CACHE_TABLE, Pattern) of
        [{Pattern, CompiledRegex}] ->
            {ok, CompiledRegex};
        [] ->
            %% Pattern not cached, compile it using gleam_regexp_ffi
            case gleam_regexp_ffi:compile(Pattern) of
                {ok, Regex} ->
                    %% Cache the compiled regex
                    ets:insert(?CACHE_TABLE, {Pattern, Regex}),
                    {ok, Regex};
                {error, _} ->
                    {error, nil}
            end
    end.
