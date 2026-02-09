# ClawGuild Justfile - Task runner for the project

# Default recipe - show available commands
default:
    @just --list

# Install Rust dependencies
install-rust:
    cargo build

# Install frontend dependencies
install-frontend:
    cd dashboard && pnpm install

# Install all dependencies
install: install-rust install-frontend

# Run database migrations
migrate:
    @echo "Migrations run automatically on startup"

# Start SurrealDB (requires Docker)
db-start:
    docker run -d -p 8000:8000 --name surrealdb \
        surrealdb/surrealdb:latest start \
        --log trace \
        --user root \
        --pass root \
        memory

# Stop SurrealDB
db-stop:
    docker stop surrealdb || true
    docker rm surrealdb || true

# Start the orchestrator service
orchestrator:
    @echo "Starting orchestrator..."
    cd orchestrator && cargo run

# Start the dashboard
dashboard:
    @echo "Starting dashboard..."
    cd dashboard && pnpm dev

# Start both orchestrator and dashboard (in parallel)
dev: db-start
    @echo "Starting development environment..."
    just --jobs 2 orchestrator dashboard

# Build the orchestrator
build-orchestrator:
    cd orchestrator && cargo build --release

# Build the dashboard
build-dashboard:
    cd dashboard && pnpm build

# Build everything
build: build-orchestrator build-dashboard

# Run tests
test:
    cd orchestrator && cargo test

# Run linter
lint:
    cd orchestrator && cargo clippy -- -D warnings
    cd dashboard && pnpm lint

# Format code
format:
    cd orchestrator && cargo fmt
    cd dashboard && pnpm format || true

# Clean build artifacts
clean:
    cd orchestrator && cargo clean
    cd dashboard && rm -rf .next out dist

# Check code (compile without building)
check:
    cd orchestrator && cargo check
    cd dashboard && pnpm type-check || true

# Setup project (install dependencies and prepare environment)
setup: install
    @echo "Setting up ClawGuild..."
    @echo "Make sure to:"
    @echo "1. Copy .env.example to .env and configure it"
    @echo "2. Start SurrealDB: just db-start"
    @echo "3. Run the orchestrator: just orchestrator"
    @echo "4. Run the dashboard: just dashboard"
