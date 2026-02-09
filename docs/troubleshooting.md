# Troubleshooting Guide

This guide helps you diagnose and resolve common issues when working with the Clarity project.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Database Issues](#database-issues)
- [Build & Compilation Issues](#build--compilation-issues)
- [Testing Issues](#testing-issues)
- [Runtime Issues](#runtime-issues)
- [Development Environment Issues](#development-environment-issues)
- [Performance Issues](#performance-issues)
- [CI/CD Issues](#cicd-issues)
- [Getting Help](#getting-help)

## Quick Diagnostics

When you encounter an issue, run these commands first to gather diagnostic information:

```bash
# Check your environment
rustc --version
cargo --version
moon --version
psql --version
sqlx --version

# Check database connectivity
psql -l | grep clarity

# Check build status
moon run :quick

# Check for common issues
moon run :check
```

Save this output - it's often needed when asking for help.

## Database Issues

### "Connection refused" when connecting to PostgreSQL

**Symptoms:**
```
Error: connection to server at "localhost" (::1), port 5432 failed: Connection refused
```

**Diagnosis:**
```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql  # Linux
brew services list                 # macOS
```

**Solutions:**

1. **Start PostgreSQL:**
   ```bash
   # Linux (Arch/Manjaro)
   sudo systemctl start postgresql

   # macOS
   brew services start postgresql@14

   # Ubuntu/Debian
   sudo systemctl start postgresql
   ```

2. **Verify DATABASE_URL:**
   ```bash
   echo $DATABASE_URL
   # Should output something like:
   # postgresql://username:password@localhost/clarity
   ```

3. **Test connection manually:**
   ```bash
   psql -U postgres -d clarity -c "SELECT 1;"
   ```

### "Database 'clarity' does not exist"

**Symptoms:**
```
Error: database "clarity" does not exist
```

**Solution:**
```bash
# Create the database
createdb clarity

# Verify it exists
psql -l | grep clarity
```

### Migration failures

**Symptoms:**
```
Error: Database migration failed: migration XYZ was not applied
```

**Diagnosis:**
```bash
# Check migration status
sqlx migrate info
```

**Solutions:**

1. **Re-run migrations:**
   ```bash
   moon run :db-migrate
   ```

2. **If migrations are out of sync:**
   ```bash
   # WARNING: This deletes all data
   dropdb clarity
   createdb clarity
   moon run :db-migrate
   ```

3. **Check for duplicate migrations:**
   ```bash
   ls -la clarity-core/migrations/
   # Look for duplicate version numbers
   ```

### "Permission denied for database clarity"

**Symptoms:**
```
Error: permission denied for database clarity
```

**Solution:**
```bash
# Grant permissions to your user
psql -c "GRANT ALL PRIVILEGES ON DATABASE clarity TO YOUR_USERNAME;"
```

### Connection pool exhaustion

**Symptoms:**
```
Error: pool exhausted - connection timeout
```

**Diagnosis:**
```bash
# Check active connections
psql -d clarity -c "SELECT count(*) FROM pg_stat_activity WHERE datname = 'clarity';"
```

**Solution:**
```bash
# Terminate idle connections
psql -d clarity -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'clarity' AND state = 'idle';"
```

## Build & Compilation Issues

### "Moon command not found"

**Symptoms:**
```
bash: moon: command not found
```

**Solution:**
```bash
# Install Moon
curl -fsSL https://moonrepo.dev/install/setup.sh | bash

# Restart your terminal or source your profile
source ~/.bashrc  # or ~/.zshrc
```

### Cargo compilation errors

**Symptoms:**
```
error[E0432]: unresolved import `clarity_core::Something`
```

**Diagnosis:**
```bash
# Check Rust version
rustc --version

# Update Rust if needed
rustup update stable
```

**Solutions:**

1. **Clean and rebuild:**
   ```bash
   cargo clean
   moon run :build
   ```

2. **Update dependencies:**
   ```bash
   cargo update
   moon run :build
   ```

3. **Check for circular dependencies:**
   ```bash
   cargo tree --duplicates
   ```

### Clippy warnings/errors

**Symptoms:**
```
error: deny(clippy::unwrap_used)
   --> src/file.rs:10:5
    |
10  |     let value = option.unwrap();
    |                  ^^^^^^^^^^^^^^
```

**Important:** NEVER modify clippy configuration to fix warnings.

**Solutions:**

1. **Fix the zero-unwrap violations:**
   ```rust
   // ❌ WRONG
   let value = option.unwrap();

   // ✅ CORRECT
   let value = option.ok_or_else(|| Error::NotFound)?;
   ```

2. **See the [Zero-Unwrap Philosophy](./zero-unwrap-philosophy.md) for patterns**

3. **Run clippy to see all issues:**
   ```bash
   moon run :clippy
   ```

### "Cannot find -lssl" or OpenSSL errors

**Symptoms:**
```
error: linking with `cc` failed: exit code: 1
  = note: /usr/bin/ld: cannot find -lssl
```

**Solution:**
```bash
# Install OpenSSL development libraries
# Linux (Arch/Manjaro)
sudo pacman -S openssl

# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# macOS
brew install openssl
```

### Target compilation issues

**Symptoms:**
```
error: failed to run custom build command for `openssl-sys`
```

**Solution:**
```bash
# Set OpenSSL environment variables (macOS)
export OPENSSL_DIR=$(brew --prefix openssl)
export OPENSSL_LIB_DIR=$OPENSSL_DIR/lib
export OPENSSL_INCLUDE_DIR=$OPENSSL_DIR/include
```

## Testing Issues

### Tests fail with "database error"

**Symptoms:**
```
test database_tests::test_create_user ... FAILED
Error: Database error: connection refused
```

**Diagnosis:**
```bash
# Check DATABASE_URL
echo $DATABASE_URL

# Check database exists
psql -l | grep clarity
```

**Solution:**
```bash
# Ensure DATABASE_URL is set
export DATABASE_URL="postgresql://localhost/clarity"

# Run migrations
moon run :db-migrate

# Re-run tests
moon run :test
```

### "Test panicked" errors

**Symptoms:**
```
test something::test_case ... FAILED
panic: 'called Result::unwrap() on an Err value'
```

**Important:** Panics in tests violate the zero-panic policy.

**Solution:**
```rust
// ❌ WRONG
#[test]
fn test_something() {
    let result = operation();
    let value = result.unwrap();  // PANICS
    assert_eq!(value.field, expected);
}

// ✅ CORRECT
#[test]
fn test_something() {
    let result = operation();
    assert!(result.is_ok(), "Operation should succeed: {:?}", result);
    let value = result.unwrap();  // Safe - we just checked
    assert_eq!(value.field, expected);
}
```

### Flaky tests (intermittent failures)

**Symptoms:**
Tests sometimes pass, sometimes fail.

**Diagnosis:**
```bash
# Run tests multiple times
for i in {1..10}; do
  echo "Run $i:"
  moon run :test
done
```

**Common causes:**

1. **Race conditions:** Add proper synchronization
2. **Shared state:** Use isolated test data
3. **Timing dependencies:** Use explicit waits/synchronization
4. **Resource cleanup:** Ensure tests clean up after themselves

**Solution:**
```rust
#[tokio::test]
async fn test_with_cleanup() -> Result<(), Error> {
    let pool = create_test_pool()?;
    let test_id = Uuid::new_v4();

    // Create test data
    create_test_data(&pool, test_id).await?;

    // Test logic here
    let result = get_data(&pool, test_id).await?;
    assert!(result.is_some());

    // Clean up
    cleanup_test_data(&pool, test_id).await?;

    Ok(())
}
```

### Tests timeout

**Symptoms:**
```
test test_name ... FAILED (timeout after 60s)
```

**Solution:**
```rust
// Increase timeout for specific tests
#[tokio::test]
#[timeout(120)] // 120 seconds
async fn test_slow_operation() {
    // ...
}
```

## Runtime Issues

### "Address already in use" (port 3000)

**Symptoms:**
```
Error: Os { code: 98, kind: AddrInUse, message: "Address already in use" }
```

**Diagnosis:**
```bash
# Check what's using the port
lsof -i :3000
# or
netstat -tulpn | grep 3000
```

**Solutions:**

1. **Kill the existing process:**
   ```bash
   kill -9 $(lsof -t -i:3000)
   ```

2. **Use a different port:**
   ```bash
   # Set PORT environment variable
   export PORT=3001
   moon run :server
   ```

### WebSocket connection failures

**Symptoms:**
```
WebSocket connection to 'ws://localhost:3000/ws' failed
```

**Diagnosis:**
```bash
# Check if server is running
curl http://localhost:3000/health

# Check browser console for WebSocket errors
```

**Solutions:**

1. **Verify server is running:**
   ```bash
   moon run :server
   ```

2. **Check CORS configuration:**
   ```rust
   // Ensure CORS allows your frontend origin
   .layer(cors_layer)
   ```

3. **Verify WebSocket route:**
   ```bash
   # Test WebSocket upgrade
   curl -i -N \
     -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Version: 13" \
     -H "Sec-WebSocket-Key: test" \
     http://localhost:3000/ws
   ```

### Memory leaks

**Symptoms:**
- Memory usage grows over time
- Application becomes sluggish
- OOM (Out of Memory) crashes

**Diagnosis:**
```bash
# Monitor memory usage
watch -n 1 'ps aux | grep clarity-server'

# Use valgrind for detailed analysis
valgrind --leak-check=full moon run :server
```

**Common causes:**

1. **Unreleased database connections:** Ensure connection pool is configured
2. **Infinite loops:** Check for loops without proper exit conditions
3. **Recursive calls:** Ensure base cases are correct
4. **Unclosed resources:** Use RAII patterns (Drop trait)

### Slow response times

**Diagnosis:**
```bash
# Check database query performance
psql -d clarity -c "EXPLAIN ANALYZE SELECT * FROM users;"

# Check server logs for slow queries
tail -f /path/to/server.log | grep "slow query"
```

**Solutions:**

1. **Add database indexes:**
   ```sql
   CREATE INDEX idx_users_email ON users(email);
   ```

2. **Use connection pooling:**
   ```rust
   // Configure pool size in clarity-core
   let pool = SqlPoolOptions::new()
       .max_connections(10)
       .connect(&DATABASE_URL)
       .await?;
   ```

3. **Enable query logging:**
   ```rust
   .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(1))
   ```

## Development Environment Issues

### "Command not found: cargo"

**Symptoms:**
```
bash: cargo: command not found
```

**Solution:**
```bash
# Add Rust to PATH
source $HOME/.cargo/env

# Make it permanent
echo 'source $HOME/.cargo/env' >> ~/.bashrc  # or ~/.zshrc
```

### SQLx CLI not found

**Symptoms:**
```
bash: sqlx: command not found
```

**Solution:**
```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### Moon cache issues

**Symptoms:**
- Moon tasks always run from scratch
- Cache not working as expected

**Diagnosis:**
```bash
# Check Moon version
moon --version

# Check cache directory
ls -la ~/.moon/cache/
```

**Solutions:**

1. **Clear Moon cache:**
   ```bash
   moon cache clean
   ```

2. **Verify cache is enabled:**
   ```bash
   moon run :check --verbose
   ```

3. **Check Moon configuration:**
   ```bash
   cat .moon/workspace.yml
   ```

### Editor/IDE integration issues

**VS Code issues:**

1. **rust-analyzer not working:**
   ```bash
   # Ensure rust-analyzer is installed
   rustup component add rust-analyzer

   # Reload VS Code window
   Ctrl+Shift+P -> "Developer: Reload Window"
   ```

2. **Cannot find workspace members:**
   - Check `.vscode/settings.json` has correct cargo path
   - Restart rust-analyzer: `Ctrl+Shift+P -> "rust-analyzer: Restart server"`

## Performance Issues

### Slow compile times

**Diagnosis:**
```bash
# Check build time
time moon run :build
```

**Solutions:**

1. **Use Moon's cached tasks:**
   ```bash
   moon run :quick  # Much faster than cargo
   ```

2. **Use link-time optimization (LTO) selectively:**
   ```toml
   [profile.release]
   lto = "thin"
   ```

3. **Consider using `sccache` for distributed compilation:**
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```

### Slow test execution

**Solutions:**

1. **Run tests in parallel:**
   ```bash
   moon run :test -- --test-threads=4
   ```

2. **Run only specific tests:**
   ```bash
   moon run :test -- user::tests::test_create_user
   ```

3. **Use release mode for faster tests:**
   ```bash
   moon run :test --release
   ```

## CI/CD Issues

### CI fails but local passes

**Symptoms:**
- Tests pass locally but fail in CI

**Diagnosis:**
```bash
# Check CI logs
# Look for differences in:
# - Rust version
# - Environment variables
# - Database setup
```

**Solutions:**

1. **Match CI environment locally:**
   ```bash
   # Check CI Rust version
   rustc --version

   # Use same versions locally
   rustup default stable
   ```

2. **Check environment variables:**
   ```bash
   # CI uses different DATABASE_URL
   export DATABASE_URL="postgresql://postgres:@localhost/clarity"
   ```

3. **Run full CI pipeline locally:**
   ```bash
   moon run :ci
   ```

### "No space left on device" in CI

**Symptoms:**
```
error: No space left on device
```

**Solution:**
```bash
# Clean build artifacts
cargo clean

# Clear Moon cache
moon cache clean

# Clear Docker images (if using Docker)
docker system prune -a
```

## Getting Help

### Before asking for help

1. **Run diagnostics:**
   ```bash
   rustc --version
   cargo --version
   moon --version
   psql --version
   moon run :quick
   ```

2. **Search existing issues:**
   - Check [GitHub Issues](../../issues)
   - Search error messages

3. **Check documentation:**
   - [README.md](../README.md)
   - [AGENTS.md](../AGENTS.md)
   - [Zero-Unwrap Philosophy](./zero-unwrap-philosophy.md)

### When creating an issue

Include:

1. **Environment information:**
   ```bash
   rustc --version
   cargo --version
   moon --version
   psql --version
   ```

2. **Full error message:**
   ```bash
   # Include complete error output
   moon run :build 2>&1 | tee build-error.log
   ```

3. **Steps to reproduce:**
   - What you did
   - What you expected
   - What actually happened

4. **What you've tried:**
   - List of attempted solutions
   - Results of those attempts

5. **Relevant code:**
   - Minimal reproduction case
   - Configuration files (if relevant)

### Community resources

- **GitHub Issues:** Report bugs and request features
- **Documentation:** Check the docs first
- **Code Examples:** See `clarity-core/tests/` for examples

## Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `connection refused` | PostgreSQL not running | Start PostgreSQL service |
| `database "clarity" does not exist` | Database not created | Run `createdb clarity` |
| `unwrap() called on Err value` | Zero-unwrap violation | Use `?` operator or Result combinators |
| `moon: command not found` | Moon not installed | Install MoonRepo |
| `pool exhausted` | Too many connections | Close idle connections or increase pool size |
| `AddrInUse` | Port already in use | Kill process or use different port |
| `permission denied for database` | Insufficient permissions | Grant permissions to user |
| `cannot find -lssl` | OpenSSL missing | Install OpenSSL development libraries |

## Prevention

### Regular maintenance

```bash
# Keep dependencies updated
cargo update

# Run full checks regularly
moon run :ci

# Monitor database connections
psql -d clarity -c "SELECT count(*) FROM pg_stat_activity WHERE datname = 'clarity';"
```

### Best practices

1. **Always use Moon commands** instead of raw cargo
2. **Set DATABASE_URL** in your shell profile
3. **Run migrations** after pulling changes
4. **Keep dependencies updated** with `cargo update`
5. **Run `moon run :quick`** before committing
6. **Run `moon run :ci`** before pushing

## Debug Mode

Enable debug logging for more information:

```bash
# Set RUST_LOG environment variable
export RUST_LOG=debug

# Run server with debug logging
moon run :server

# Run tests with debug logging
RUST_LOG=debug moon run :test
```

For SQLx query logging:
```bash
export SQLX_LOG=info
```

---

**Still having issues?** Create a GitHub issue with the diagnostic information gathered from the [Quick Diagnostics](#quick-diagnostics) section.
