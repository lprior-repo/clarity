#!/bin/bash
set -e

# Script to run database integration tests for clarity-core

echo "🧪 Clarity Core Database Integration Tests"
echo "=========================================="

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
  echo "⚠️  DATABASE_URL not set. Using default test database."
  export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/clarity_test"
fi

echo "📊 Database URL: $DATABASE_URL"
echo ""

# Check if database is accessible
echo "🔍 Checking database connection..."
if ! psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
  echo "❌ Cannot connect to database!"
  echo ""
  echo "💡 To start a test database using Docker:"
  echo "   docker-compose -f docker-compose.test.yml up -d"
  echo ""
  echo "💡 Or use your local PostgreSQL:"
  echo "   createdb clarity_test"
  echo "   export DATABASE_URL=postgresql://user@localhost/clarity_test"
  exit 1
fi

echo "✅ Database connection successful"
echo ""

# Run tests
echo "🚀 Running integration tests..."
echo ""

# Set SQLX offline mode for faster compilation
export SQLX_OFFLINE=false

# Run the integration tests
cargo test --package clarity-core --features integration-tests --test integration_test "$@"

echo ""
echo "✅ Integration tests completed!"
