# Getting Started with Clarity

Welcome to Clarity! This guide will help you get up and running quickly, whether you're a seasoned Rust developer or just getting started with the language.

## What is Clarity?

Clarity is a modern fullstack application built with Rust that demonstrates best practices in:
- **Functional Programming**: Writing clean, maintainable code
- **Test-Driven Development**: Building reliable software with confidence
- **Type Safety**: Catching errors at compile time, not runtime
- **Zero-Panic Architecture**: No unexpected crashes, ever

## Before You Begin

### System Requirements

- **Operating System**: Linux, macOS, or Windows (with WSL2)
- **Memory**: At least 4GB RAM (8GB recommended)
- **Disk Space**: 2GB free space for dependencies

### Prerequisites Checklist

You'll need these tools installed before starting:

- [ ] Rust (latest stable)
- [ ] PostgreSQL (version 12 or higher)
- [ ] MoonRepo (build system)
- [ ] SQLx CLI (database tool)

Don't have these installed? Don't worry! We'll walk through each installation below.

## Installation Guide

### Step 1: Install Rust

Rust is the programming language Clarity is built with.

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:**
Download and run the installer from [rustup.rs](https://rustup.rs/)

**Verify installation:**
```bash
rustc --version
cargo --version
```

### Step 2: Install PostgreSQL

PostgreSQL is our database.

**Linux (Arch/Manjaro):**
```bash
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**macOS:**
```bash
brew install postgresql@14
brew services start postgresql@14
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows:**
Download from [postgresql.org](https://www.postgresql.org/download/windows/)

**Verify installation:**
```bash
psql --version
```

### Step 3: Install MoonRepo

MoonRepo is our build system that makes development faster.

```bash
curl -fsSL https://moonrepo.dev/install/setup.sh | bash
```

**Verify installation:**
```bash
moon --version
```

### Step 4: Install SQLx CLI

SQLx CLI helps manage database migrations.

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

**Verify installation:**
```bash
sqlx --version
```

## Setting Up Your Development Environment

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/clarity.git
cd clarity
```

### 2. Set Up the Database

Create the database:
```bash
createdb clarity
```

Set up the database connection:
```bash
export DATABASE_URL="postgresql://localhost/clarity"
```

*Note: On macOS or Linux, you may want to add this to your `~/.bashrc` or `~/.zshrc`:*
```bash
echo 'export DATABASE_URL="postgresql://localhost/clarity"' >> ~/.bashrc
source ~/.bashrc
```

Run database migrations:
```bash
moon run :db-migrate
```

### 3. Install Dependencies

Moon will automatically download and install Rust dependencies:
```bash
moon run :check
```

This may take a few minutes on the first run as it downloads dependencies.

### 4. Verify Your Setup

Run a quick check to make sure everything is working:
```bash
moon run :quick
moon run :test
```

## Your First Run

Let's get the application running!

### Start the Backend Server

In your terminal:
```bash
moon run :server
```

You should see output like:
```
Server running on http://localhost:3000
```

### Start the Frontend (in a new terminal)

Open a new terminal window, navigate to the project directory, and run:
```bash
moon run :client
```

The frontend will typically run on `http://localhost:8080`

### Visit the Application

Open your web browser and go to `http://localhost:8080`

Congratulations! You now have Clarity running locally!

## Understanding the Project Structure

Clarity is organized into three main parts:

```
clarity/
├── clarity-client/     # Frontend (what users see in browser)
├── clarity-core/       # Shared business logic and database
└── clarity-server/     # Backend (API and server logic)
```

### What Each Part Does

- **clarity-client**: The user interface built with Dioxus (like React, but for Rust)
- **clarity-core**: Shared code that both frontend and backend use
- **clarity-server**: The server that handles API requests and business logic

## Common Development Tasks

### Running Tests

```bash
# Run all tests
moon run :test

# Run only unit tests
moon run :test-unit

# Run documentation tests
moon run :test-doc
```

### Building the Project

```bash
# Quick type check (fast)
moon run :check

# Full build
moon run :build

# Build for production
moon run :release
```

### Code Quality

```bash
# Format code
moon run :fmt-fix

# Run linter
moon run :clippy

# Run full CI pipeline
moon run :ci
```

## Learning Resources

### New to Rust?

Start here:
- [Rust Book](https://doc.rust-lang.org/book/) - Comprehensive guide
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by doing
- [Rustlings](https://rustlings.cool/) - Interactive exercises

### Functional Programming Concepts

Clarity uses functional programming principles:
- **Immutability**: Variables don't change after being set
- **Pure Functions**: Same input always produces same output
- **Error Handling**: Using `Result<T, E>` instead of exceptions

Learn more in our [Zero-Unwrap Philosophy Guide](zero-unwrap-philosophy.md)

### Test-Driven Development

We write tests BEFORE writing code:
1. Write a test that describes what we want
2. Run the test (it will fail - this is expected!)
3. Write the minimum code to make it pass
4. Improve the code while keeping tests green

This is called the **RED-GREEN-REFACTOR** cycle.

Read more in the main [README.md](../README.md#testing-philosophy)

## Getting Help

### Encountering an Issue?

1. **Check the logs**: Error messages often contain helpful information
2. **Search existing issues**: Someone may have already solved it
3. **Ask for help**: We're friendly! Create a GitHub issue

### Common Problems

**Database connection fails:**
- Make sure PostgreSQL is running: `sudo systemctl status postgresql`
- Check DATABASE_URL is set: `echo $DATABASE_URL`

**Build fails:**
- Update Rust: `rustup update stable`
- Clean and rebuild: `cargo clean && moon run :build`

**Tests fail:**
- Make sure database is set up: `moon run :db-migrate`
- Check DATABASE_URL is set correctly

## Next Steps

Now that you're set up, here's what you can do:

1. **Explore the code**: Start with `clarity-core/src/lib.rs`
2. **Make a change**: Try adding a new feature or fixing a bug
3. **Run the tests**: Make sure everything still works
4. **Contribute**: See our contributing guidelines in the main README

## Development Tips

### Quick Iteration Loop

When making changes:
```bash
# Edit code...
moon run :quick    # Fast check (6-7ms with cache)
```

### Before Committing

Always run:
```bash
moon run :fmt-fix  # Format code
moon run :ci       # Full test suite
```

### Understanding MoonRepo

**Why use Moon instead of cargo?**

- **Faster**: Cached tasks take 6-7ms vs 450ms for cargo
- **Parallel**: Builds multiple crates at once
- **Consistent**: Same environment for everyone

**Pro tip**: Never use raw `cargo` commands - always use `moon run :task-name`

## Architecture Overview

### Three-Crate Structure

```
┌─────────────────┐
│  clarity-client │  ← Frontend (Dioxus)
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│  clarity-core   │  ← Shared logic (types, validation, database)
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│ clarity-server  │  ← Backend (Axum web server)
└─────────────────┘
         │
         ↓
    PostgreSQL
```

### Design Principles

1. **Immutability**: Data doesn't change unexpectedly
2. **Type Safety**: Errors caught at compile time
3. **Explicit Error Handling**: No hidden exceptions
4. **Function Composition**: Build complex features from simple parts

## Testing Philosophy

### Why Test-First?

- **Catches bugs early**: Find issues before they reach production
- **Documents behavior**: Tests show how code should work
- **Enables refactoring**: Change code with confidence

### The ATDD Cycle

```
1. READ  - Understand what to build
2. WRITE - Write a test describing it
3. RED   - Run test (it fails - this is OK!)
4. GREEN - Write minimum code to pass
5. REFACTOR - Improve while keeping tests green
```

### What to Test

✅ **Test these:**
- Edge cases (empty input, maximum values)
- Error conditions (network failures, invalid data)
- Concurrent access (multiple users)
- Resource limits (out of memory, etc.)

❌ **Don't just test:**
- Happy paths (obvious behavior)
- Trivial getters/setters

## Contributing

We welcome contributions! Here's how to get started:

### For Your First Contribution

1. Find an issue labeled "good first issue"
2. Comment that you'd like to work on it
3. Follow the development workflow
4. Submit a pull request

### Development Workflow

1. **Create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Write tests first**:
   ```bash
   # Create test file describing your feature
   moon run :test  # Tests fail (expected!)
   ```

3. **Implement feature**:
   ```bash
   # Write minimum code to pass tests
   moon run :test  # Tests pass!
   ```

4. **Run quality checks**:
   ```bash
   moon run :fmt-fix
   moon run :ci
   ```

5. **Commit and push**:
   ```bash
   git add .
   git commit -m "feat: description of your feature"
   git push origin feature/your-feature-name
   ```

### Commit Message Style

We use conventional commits:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding tests

Example:
```
feat: add user authentication
fix: handle database connection timeout
docs: update API documentation
```

## Additional Resources

### Project Documentation

- [Main README](../README.md) - Comprehensive project documentation
- [Zero-Unwrap Philosophy](zero-unwrap-philosophy.md) - Functional error handling
- [Troubleshooting Guide](troubleshooting.md) - Common issues and solutions
- [REST API Reference](rest-api-reference.md) - API documentation

### External Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Axum Guide](https://docs.rs/axum/)
- [Dioxus Guide](https://dioxuslabs.com/learn/0.7/)
- [SQLx Guide](https://docs.rs/sqlx/)
- [MoonRepo Documentation](https://moonrepo.dev/docs)

## Quick Reference

### Essential Commands

```bash
# Setup
moon run :db-migrate         # Set up database
moon run :check              # Verify dependencies

# Development
moon run :quick              # Fast format + lint check
moon run :server             # Start backend server
moon run :client             # Start frontend client

# Testing
moon run :test               # Run all tests
moon run :test-unit          # Run unit tests only

# Quality
moon run :fmt-fix            # Auto-format code
moon run :clippy             # Run linter
moon run :ci                 # Full CI pipeline
```

### File Locations

```
clarity/
├── clarity-client/src/      # Frontend code
├── clarity-core/src/        # Shared business logic
├── clarity-server/src/      # Backend code
├── migrations/              # Database migrations
└── docs/                   # Additional documentation
```

## What's Next?

Now that you're set up and running, explore:

1. **The codebase**: Start reading from `clarity-core/src/lib.rs`
2. **The tests**: See how we test in `clarity-core/tests/`
3. **The documentation**: Read more in the `docs/` directory

Happy coding! Welcome to the Clarity community.

---

**Need help?** Check the [troubleshooting guide](troubleshooting.md) or [create an issue](https://github.com/yourusername/clarity/issues/new)

**Want to contribute?** See the main [README.md](../README.md#contributing) for detailed guidelines.
