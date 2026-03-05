import gleam/dict
import gleam/json
import gleam/option
import gleam/string
import gleeunit/should
import intent/http_client
import intent/interpolate
import intent/types

/// Test SSRF protection blocks localhost
pub fn ssrf_blocks_localhost_test() {
  let config =
    types.Config(
      base_url: "http://localhost:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/admin",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "localhost") |> should.be_true()
      string.contains(msg, "127.x") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks 127.0.0.1
pub fn ssrf_blocks_127_0_0_1_test() {
  let config =
    types.Config(
      base_url: "http://127.0.0.1:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/admin",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "localhost") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks 127.x range
pub fn ssrf_blocks_127_subnet_test() {
  let config =
    types.Config(
      base_url: "http://127.1.2.3:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(_)) -> Nil
    // Test passes
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks 10.x private range
pub fn ssrf_blocks_10_network_test() {
  let config =
    types.Config(
      base_url: "http://10.0.0.1",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/internal",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "private IP") |> should.be_true()
      string.contains(msg, "10.x") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks 192.168.x private range
pub fn ssrf_blocks_192_168_network_test() {
  let config =
    types.Config(
      base_url: "http://192.168.1.1",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/router",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "192.168.x") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks 172.16.x-172.31.x private range
pub fn ssrf_blocks_172_16_network_test() {
  let config =
    types.Config(
      base_url: "http://172.16.0.1",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/internal",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "172.16-31.x") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks 172.31.x (upper boundary)
pub fn ssrf_blocks_172_31_network_test() {
  let config =
    types.Config(
      base_url: "http://172.31.255.255",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(_)) -> Nil
    // Test passes
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks AWS metadata endpoint
pub fn ssrf_blocks_aws_metadata_test() {
  let config =
    types.Config(
      base_url: "http://169.254.169.254",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/latest/meta-data",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "AWS metadata") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks .local domains
pub fn ssrf_blocks_local_domain_test() {
  let config =
    types.Config(
      base_url: "http://internal.local",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "internal domain") |> should.be_true()
      string.contains(msg, ".local") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks .internal domains
pub fn ssrf_blocks_internal_domain_test() {
  let config =
    types.Config(
      base_url: "http://api.internal",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, ".internal") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks GCP metadata endpoint
pub fn ssrf_blocks_gcp_metadata_test() {
  let config =
    types.Config(
      base_url: "http://metadata.google.internal",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/computeMetadata/v1/",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "metadata.google.internal") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks IPv6 localhost ::1
pub fn ssrf_blocks_ipv6_localhost_test() {
  let config =
    types.Config(
      base_url: "http://[::1]:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "IPv6") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks IPv6 link-local fe80::
pub fn ssrf_blocks_ipv6_link_local_test() {
  let config =
    types.Config(
      base_url: "http://[fe80::1]:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "IPv6") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks IPv6 unique local fc00::
pub fn ssrf_blocks_ipv6_unique_local_fc_test() {
  let config =
    types.Config(
      base_url: "http://[fc00::1]:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "IPv6") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks IPv6 unique local fd00::
pub fn ssrf_blocks_ipv6_unique_local_fd_test() {
  let config =
    types.Config(
      base_url: "http://[fd00::1]:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/test",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "IPv6") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection blocks file:// protocol
pub fn ssrf_blocks_file_protocol_test() {
  let config =
    types.Config(
      base_url: "file:///etc/passwd",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  case result {
    Error(http_client.SSRFBlocked(msg)) -> {
      string.contains(msg, "file") |> should.be_true()
      string.contains(msg, "http://") |> should.be_true()
    }
    _ -> should.fail()
  }
}

/// Test SSRF protection allows valid public domains
pub fn ssrf_allows_public_domain_test() {
  let config =
    types.Config(
      base_url: "https://api.github.com",
      timeout_ms: 5000,
      headers: dict.new(),
    )

  let request =
    types.Request(
      method: types.Get,
      path: "/users/octocat",
      headers: dict.new(),
      query: dict.new(),
      body: json.null(),
    )

  let ctx =
    interpolate.Context(
      variables: dict.new(),
      request_body: option.None,
      response_body: option.None,
    )
  let result = http_client.execute_request(config, request, ctx)

  // Should either succeed or fail with a network error, but NOT SSRFBlocked
  case result {
    Error(http_client.SSRFBlocked(_)) -> should.fail()
    _ -> Nil
    // Test passes - not blocked
  }
}
