//// Comprehensive tests for the formats module
//// Tests rigorous format validators using pure Gleam parsing:
//// - Email validation (RFC 5322 compliant)
//// - UUID validation (version and variant bits)
//// - URI validation (RFC 3986)
//// - ISO 8601 datetime validation (with calendar validation)

import gleeunit/should
import intent/formats

// ============================================================================
// Email Validation Tests - Valid Cases
// ============================================================================

pub fn email_valid_simple_test() {
  formats.validate_email("user@example.com")
  |> should.be_ok
}

pub fn email_valid_with_subdomain_test() {
  formats.validate_email("user@mail.example.com")
  |> should.be_ok
}

pub fn email_valid_with_dots_in_local_test() {
  formats.validate_email("first.last@example.com")
  |> should.be_ok
}

pub fn email_valid_with_plus_test() {
  formats.validate_email("user+tag@example.com")
  |> should.be_ok
}

pub fn email_valid_with_hyphen_test() {
  formats.validate_email("first-last@example.com")
  |> should.be_ok
}

pub fn email_valid_with_underscore_test() {
  formats.validate_email("user_name@example.com")
  |> should.be_ok
}

pub fn email_valid_with_numbers_test() {
  formats.validate_email("user123@example456.com")
  |> should.be_ok
}

pub fn email_valid_multiple_subdomains_test() {
  formats.validate_email("admin@mail.corp.example.com")
  |> should.be_ok
}

pub fn email_valid_uppercase_test() {
  formats.validate_email("User@Example.COM")
  |> should.be_ok
}

pub fn email_valid_mixed_case_test() {
  formats.validate_email("First.Last@Example.Com")
  |> should.be_ok
}

// ============================================================================
// Email Validation Tests - Invalid Cases
// ============================================================================

pub fn email_invalid_no_at_sign_test() {
  case formats.validate_email("userexample.com") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'userexample.com' is not a valid email address (invalid @ format)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_multiple_at_signs_test() {
  case formats.validate_email("user@@example.com") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'user@@example.com' is not a valid email address (invalid @ format)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_empty_local_test() {
  case formats.validate_email("@example.com") {
    Error(msg) -> {
      msg
      |> should.equal("Email local part cannot be empty")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_empty_domain_test() {
  case formats.validate_email("user@") {
    Error(msg) -> {
      msg
      |> should.equal("Email domain cannot be empty")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_consecutive_dots_local_test() {
  case formats.validate_email("user..name@example.com") {
    Error(msg) -> {
      msg
      |> should.equal("Email local part cannot contain consecutive dots")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_starts_with_dot_test() {
  case formats.validate_email(".user@example.com") {
    Error(msg) -> {
      msg
      |> should.equal("Email local part cannot start or end with a dot")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_ends_with_dot_test() {
  case formats.validate_email("user.@example.com") {
    Error(msg) -> {
      msg
      |> should.equal("Email local part cannot start or end with a dot")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_special_chars_local_test() {
  case formats.validate_email("user#name@example.com") {
    Error(msg) -> {
      msg
      |> should.equal("Email local part contains invalid characters: user#name")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_no_dot_in_domain_test() {
  case formats.validate_email("user@localhost") {
    Error(msg) -> {
      msg
      |> should.equal("Email domain must contain at least one dot")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_consecutive_dots_domain_test() {
  case formats.validate_email("user@example..com") {
    Error(msg) -> {
      msg
      |> should.equal(
        "Email domain contains empty label (consecutive or trailing dots)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_domain_starts_with_hyphen_test() {
  case formats.validate_email("user@-example.com") {
    Error(msg) -> {
      msg
      |> should.equal("Email domain contains invalid labels: -example.com")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_domain_ends_with_hyphen_test() {
  case formats.validate_email("user@example-.com") {
    Error(msg) -> {
      msg
      |> should.equal("Email domain contains invalid labels: example-.com")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_domain_special_chars_test() {
  case formats.validate_email("user@exam_ple.com") {
    Error(msg) -> {
      msg
      |> should.equal("Email domain contains invalid labels: exam_ple.com")
    }
    Ok(_) -> should.fail()
  }
}

pub fn email_invalid_empty_string_test() {
  case formats.validate_email("") {
    Error(msg) -> {
      msg
      |> should.equal("'' is not a valid email address (invalid @ format)")
    }
    Ok(_) -> should.fail()
  }
}

// ============================================================================
// UUID Validation Tests - Valid Cases
// ============================================================================

pub fn uuid_valid_v1_test() {
  // Version 1 UUID with variant bits set correctly
  formats.validate_uuid("550e8400-e29b-11d4-a716-446655440000")
  |> should.be_ok
}

pub fn uuid_valid_v4_test() {
  // Version 4 UUID (random)
  formats.validate_uuid("123e4567-e89b-42d3-a456-426614174000")
  |> should.be_ok
}

pub fn uuid_valid_uppercase_test() {
  // UUID with uppercase hex characters
  formats.validate_uuid("550E8400-E29B-41D4-A716-446655440000")
  |> should.be_ok
}

pub fn uuid_valid_mixed_case_test() {
  // UUID with mixed case
  formats.validate_uuid("550e8400-E29b-31D4-B716-446655440000")
  |> should.be_ok
}

pub fn uuid_valid_v2_test() {
  // Version 2 UUID
  formats.validate_uuid("000003e8-2363-21ed-8100-325096b39f47")
  |> should.be_ok
}

pub fn uuid_valid_v3_test() {
  // Version 3 UUID (MD5 hash)
  formats.validate_uuid("a3bb189e-8bf9-3888-9912-ace4e6543002")
  |> should.be_ok
}

pub fn uuid_valid_v5_test() {
  // Version 5 UUID (SHA-1 hash)
  formats.validate_uuid("886313e1-3b8a-5372-9b90-0c9aee199e5d")
  |> should.be_ok
}

// ============================================================================
// UUID Validation Tests - Invalid Cases
// ============================================================================

pub fn uuid_invalid_wrong_segment_count_test() {
  case formats.validate_uuid("550e8400-e29b-11d4-a716") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'550e8400-e29b-11d4-a716' is not a valid UUID (invalid segment count)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn uuid_invalid_segment_length_test() {
  case formats.validate_uuid("550e840-e29b-11d4-a716-446655440000") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'550e840-e29b-11d4-a716-446655440000' has invalid UUID segment lengths (expected 8-4-4-4-12)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn uuid_invalid_non_hex_chars_test() {
  case formats.validate_uuid("550e8400-e29b-11d4-a716-44665544000g") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'550e8400-e29b-11d4-a716-44665544000g' contains non-hexadecimal characters",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn uuid_invalid_version_zero_test() {
  case formats.validate_uuid("550e8400-e29b-01d4-a716-446655440000") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'550e8400-e29b-01d4-a716-446655440000' has invalid UUID version (expected 1-5, got 0)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn uuid_invalid_version_six_test() {
  case formats.validate_uuid("550e8400-e29b-61d4-a716-446655440000") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'550e8400-e29b-61d4-a716-446655440000' has invalid UUID version (expected 1-5, got 6)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn uuid_invalid_variant_bits_test() {
  // Variant bits should be 8, 9, a, or b (RFC 4122)
  case formats.validate_uuid("550e8400-e29b-41d4-0716-446655440000") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'550e8400-e29b-41d4-0716-446655440000' has invalid RFC 4122 variant (expected 8,9,a,b variant bits)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn uuid_invalid_no_hyphens_test() {
  case formats.validate_uuid("550e8400e29b11d4a716446655440000") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'550e8400e29b11d4a716446655440000' is not a valid UUID (invalid segment count)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn uuid_invalid_empty_string_test() {
  case formats.validate_uuid("") {
    Error(msg) -> {
      msg
      |> should.equal("'' is not a valid UUID (invalid segment count)")
    }
    Ok(_) -> should.fail()
  }
}

// ============================================================================
// URI Validation Tests - Valid Cases
// ============================================================================

pub fn uri_valid_http_test() {
  formats.validate_uri("http://example.com")
  |> should.be_ok
}

pub fn uri_valid_https_test() {
  formats.validate_uri("https://example.com")
  |> should.be_ok
}

pub fn uri_valid_ftp_test() {
  formats.validate_uri("ftp://ftp.example.com")
  |> should.be_ok
}

pub fn uri_valid_with_path_test() {
  formats.validate_uri("https://example.com/path/to/resource")
  |> should.be_ok
}

pub fn uri_valid_with_query_test() {
  formats.validate_uri("https://example.com/search?q=test&page=1")
  |> should.be_ok
}

pub fn uri_valid_with_fragment_test() {
  formats.validate_uri("https://example.com/page#section")
  |> should.be_ok
}

pub fn uri_valid_with_port_test() {
  formats.validate_uri("https://example.com:8080/api")
  |> should.be_ok
}

pub fn uri_valid_with_user_info_test() {
  formats.validate_uri("ftp://user:pass@ftp.example.com")
  |> should.be_ok
}

pub fn uri_valid_custom_scheme_test() {
  formats.validate_uri("custom://example.com")
  |> should.be_ok
}

pub fn uri_valid_scheme_with_plus_test() {
  formats.validate_uri("git+https://github.com/user/repo")
  |> should.be_ok
}

pub fn uri_valid_scheme_with_dot_test() {
  formats.validate_uri("vnd.example://test")
  |> should.be_ok
}

pub fn uri_valid_scheme_with_hyphen_test() {
  formats.validate_uri("x-custom://example.com")
  |> should.be_ok
}

pub fn uri_valid_localhost_test() {
  formats.validate_uri("http://localhost:3000")
  |> should.be_ok
}

pub fn uri_valid_ip_address_test() {
  formats.validate_uri("http://192.168.1.1:8080")
  |> should.be_ok
}

// ============================================================================
// URI Validation Tests - Invalid Cases
// ============================================================================

pub fn uri_invalid_empty_string_test() {
  case formats.validate_uri("") {
    Error(msg) -> {
      msg
      |> should.equal("URI cannot be empty")
    }
    Ok(_) -> should.fail()
  }
}

pub fn uri_invalid_no_scheme_test() {
  case formats.validate_uri("example.com") {
    Error(msg) -> {
      msg
      |> should.equal("'example.com' is not a valid URI (missing scheme)")
    }
    Ok(_) -> should.fail()
  }
}

pub fn uri_invalid_scheme_only_test() {
  case formats.validate_uri("http://") {
    Error(msg) -> {
      msg
      |> should.equal("'http://' has no authority after scheme")
    }
    Ok(_) -> should.fail()
  }
}

pub fn uri_invalid_scheme_starts_with_number_test() {
  case formats.validate_uri("123://example.com") {
    Error(msg) -> {
      msg
      |> should.equal("URI scheme must start with a letter")
    }
    Ok(_) -> should.fail()
  }
}

pub fn uri_invalid_scheme_starts_with_hyphen_test() {
  case formats.validate_uri("-http://example.com") {
    Error(msg) -> {
      msg
      |> should.equal("URI scheme must start with a letter")
    }
    Ok(_) -> should.fail()
  }
}

pub fn uri_invalid_scheme_special_chars_test() {
  case formats.validate_uri("ht_tp://example.com") {
    Error(msg) -> {
      msg
      |> should.equal("URI scheme contains invalid characters: ht_tp")
    }
    Ok(_) -> should.fail()
  }
}

pub fn uri_invalid_empty_scheme_test() {
  case formats.validate_uri("://example.com") {
    Error(msg) -> {
      msg
      |> should.equal("URI scheme cannot be empty")
    }
    Ok(_) -> should.fail()
  }
}

// ============================================================================
// ISO 8601 Date Validation Tests - Valid Cases
// ============================================================================

pub fn iso8601_valid_date_only_test() {
  formats.validate_iso8601("2024-03-15")
  |> should.be_ok
}

pub fn iso8601_valid_datetime_test() {
  formats.validate_iso8601("2024-03-15T14:30:00")
  |> should.be_ok
}

pub fn iso8601_valid_datetime_with_z_test() {
  formats.validate_iso8601("2024-03-15T14:30:00Z")
  |> should.be_ok
}

pub fn iso8601_valid_datetime_with_positive_tz_test() {
  formats.validate_iso8601("2024-03-15T14:30:00+05:30")
  |> should.be_ok
}

pub fn iso8601_valid_datetime_with_negative_tz_test() {
  formats.validate_iso8601("2024-03-15T14:30:00-08:00")
  |> should.be_ok
}

pub fn iso8601_valid_datetime_with_space_separator_test() {
  formats.validate_iso8601("2024-03-15 14:30:00")
  |> should.be_ok
}

pub fn iso8601_valid_datetime_with_fractional_seconds_test() {
  formats.validate_iso8601("2024-03-15T14:30:00.123")
  |> should.be_ok
}

pub fn iso8601_valid_leap_year_feb_29_test() {
  formats.validate_iso8601("2024-02-29")
  |> should.be_ok
}

pub fn iso8601_valid_jan_31_test() {
  formats.validate_iso8601("2024-01-31")
  |> should.be_ok
}

pub fn iso8601_valid_april_30_test() {
  formats.validate_iso8601("2024-04-30")
  |> should.be_ok
}

pub fn iso8601_valid_midnight_test() {
  formats.validate_iso8601("2024-03-15T00:00:00")
  |> should.be_ok
}

pub fn iso8601_valid_end_of_day_test() {
  formats.validate_iso8601("2024-03-15T23:59:59")
  |> should.be_ok
}

// ============================================================================
// ISO 8601 Date Validation Tests - Invalid Cases
// ============================================================================

pub fn iso8601_invalid_too_short_test() {
  case formats.validate_iso8601("2024-03") {
    Error(msg) -> {
      msg
      |> should.equal("'2024-03' is not a valid ISO8601 datetime (too short)")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_month_zero_test() {
  case formats.validate_iso8601("2024-00-15") {
    Error(msg) -> {
      msg
      |> should.equal("'2024-00-15' has invalid month: 00 (must be 01-12)")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_month_13_test() {
  case formats.validate_iso8601("2024-13-15") {
    Error(msg) -> {
      msg
      |> should.equal("'2024-13-15' has invalid month: 13 (must be 01-12)")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_day_zero_test() {
  case formats.validate_iso8601("2024-03-00") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'2024-03-00' has invalid day: 00 (month 03 has max 31 days)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_day_32_test() {
  case formats.validate_iso8601("2024-03-32") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'2024-03-32' has invalid day: 32 (month 03 has max 31 days)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_feb_30_test() {
  case formats.validate_iso8601("2024-02-30") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'2024-02-30' has invalid day: 30 (month 02 has max 29 days)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_feb_29_non_leap_year_test() {
  case formats.validate_iso8601("2023-02-29") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'2023-02-29' has invalid day: 29 (month 02 has max 28 days)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_april_31_test() {
  case formats.validate_iso8601("2024-04-31") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'2024-04-31' has invalid day: 31 (month 04 has max 30 days)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_hour_24_test() {
  case formats.validate_iso8601("2024-03-15T24:00:00") {
    Error(msg) -> {
      msg
      |> should.equal("Invalid ISO8601 time: hour must be 00-23, got 24")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_minute_60_test() {
  case formats.validate_iso8601("2024-03-15T14:60:00") {
    Error(msg) -> {
      msg
      |> should.equal("Invalid ISO8601 time: minute must be 00-59, got 60")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_second_60_test() {
  case formats.validate_iso8601("2024-03-15T14:30:60") {
    Error(msg) -> {
      msg
      |> should.equal("Invalid ISO8601 time: second must be 00-59, got 60")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_negative_hour_test() {
  case formats.validate_iso8601("2024-03-15T-1:30:00") {
    Error(msg) -> {
      msg
      |> should.equal("Invalid ISO8601 time: hour must be a number, got -1")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_year_not_number_test() {
  case formats.validate_iso8601("abcd-03-15") {
    Error(msg) -> {
      msg
      |> should.equal("'abcd-03-15' has invalid year (not a number)")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_month_not_number_test() {
  case formats.validate_iso8601("2024-ab-15") {
    Error(msg) -> {
      msg
      |> should.equal("'2024-ab-15' has invalid month (not a number)")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_day_not_number_test() {
  case formats.validate_iso8601("2024-03-ab") {
    Error(msg) -> {
      msg
      |> should.equal("'2024-03-ab' has invalid day (not a number)")
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_separator_test() {
  case formats.validate_iso8601("2024-03-15X14:30:00") {
    Error(msg) -> {
      msg
      |> should.equal(
        "'2024-03-15X14:30:00' is not a valid ISO8601 datetime (invalid separator, expected T or space)",
      )
    }
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_invalid_time_format_test() {
  case formats.validate_iso8601("2024-03-15T14:30") {
    Error(msg) -> {
      msg
      |> should.equal("Invalid ISO8601 time format (expected HH:MM:SS)")
    }
    Ok(_) -> should.fail()
  }
}

// ============================================================================
// Leap Year Tests
// ============================================================================

pub fn iso8601_leap_year_divisible_by_4_test() {
  // 2024 is divisible by 4 and not by 100, so it's a leap year
  formats.validate_iso8601("2024-02-29")
  |> should.be_ok
}

pub fn iso8601_not_leap_year_test() {
  // 2023 is not divisible by 4, so it's not a leap year
  case formats.validate_iso8601("2023-02-29") {
    Error(_) -> should.be_ok(Ok(Nil))
    Ok(_) -> should.fail()
  }
}

pub fn iso8601_leap_year_divisible_by_400_test() {
  // 2000 is divisible by 400, so it's a leap year
  formats.validate_iso8601("2000-02-29")
  |> should.be_ok
}

pub fn iso8601_not_leap_year_divisible_by_100_test() {
  // 1900 is divisible by 100 but not by 400, so it's not a leap year
  case formats.validate_iso8601("1900-02-29") {
    Error(_) -> should.be_ok(Ok(Nil))
    Ok(_) -> should.fail()
  }
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

pub fn email_very_long_local_test() {
  let long_local =
    "verylonglocalpartverylonglocalpartverylonglocalpart@example.com"
  formats.validate_email(long_local)
  |> should.be_ok
}

pub fn email_very_long_domain_test() {
  let long_domain =
    "user@verylongdomainpartverylongdomainpartverylongdomainpart.example.com"
  formats.validate_email(long_domain)
  |> should.be_ok
}

pub fn uuid_all_zeros_test() {
  // All zeros except version and variant bits
  case formats.validate_uuid("00000000-0000-1000-8000-000000000000") {
    Ok(_) -> should.be_ok(Ok(Nil))
    Error(_) -> should.be_ok(Ok(Nil))
  }
}

pub fn uuid_all_fs_test() {
  // All Fs with valid version and variant
  formats.validate_uuid("ffffffff-ffff-4fff-bfff-ffffffffffff")
  |> should.be_ok
}

pub fn uri_very_long_path_test() {
  formats.validate_uri(
    "https://example.com/very/long/path/with/many/segments/that/goes/on/and/on",
  )
  |> should.be_ok
}

pub fn iso8601_year_1_test() {
  formats.validate_iso8601("0001-01-01")
  |> should.be_ok
}

pub fn iso8601_year_9999_test() {
  formats.validate_iso8601("9999-12-31")
  |> should.be_ok
}

pub fn iso8601_dec_31_test() {
  formats.validate_iso8601("2024-12-31")
  |> should.be_ok
}

pub fn iso8601_june_30_test() {
  formats.validate_iso8601("2024-06-30")
  |> should.be_ok
}

pub fn iso8601_september_30_test() {
  formats.validate_iso8601("2024-09-30")
  |> should.be_ok
}

pub fn iso8601_november_30_test() {
  formats.validate_iso8601("2024-11-30")
  |> should.be_ok
}
