% FFI module for reading from standard input
% Provides Gleam interface to Erlang io:read/1

-module(intent_ffi_stdin).

-export([read_line/0, read_line_trimmed/0]).

% Read a line from standard input (includes newline)
read_line() ->
  case io:get_line('') of
    eof -> {error, <<"EOF">>};
    {error, Reason} -> {error, atom_to_binary(Reason, utf8)};
    Line -> {ok, Line}
  end.

% Read a line from standard input and trim whitespace
read_line_trimmed() ->
  case io:get_line('') of
    eof -> {error, <<"EOF">>};
    {error, Reason} -> {error, atom_to_binary(Reason, utf8)};
    Line ->
      % Remove trailing newline/carriage return and convert to binary
      Trimmed = string:trim(Line, trailing, "\n\r"),
      {ok, Trimmed}
  end.
