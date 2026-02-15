# ClawGuild Justfile - Task runner for the project

# Default recipe - show available commands
default:
    @just --list

# Install Rust dependencies
install-rust:
    cargo build

# Install frontend dependencies
install-frontend:
    pnpm install

# Install all dependencies
install: install-rust install-frontend

# Run database migrations
migrate:
    @echo "Migrations run automatically on startup"

# Start Postgres (requires Docker Compose)
db-start:
    cd docker && docker compose --profile development up -d postgres

# Stop Postgres
db-stop:
    cd docker && docker compose stop postgres

# Start all development services (Postgres, API server, dashboard)
up:
    cd docker && docker compose --profile development up -d

# Start production services
up-prod:
    cd docker && docker compose --profile production up -d

# Stop all services
docker-down:
    cd docker && docker compose down

# View logs for development services
docker-logs:
    cd docker && docker compose --profile development logs -f

# View logs for production services
docker-logs-prod:
    cd docker && docker compose --profile production logs -f

# Start the API server
api-server:
    @echo "Starting API server..."
    cd api-server && cargo run

# Start the dashboard
dashboard:
    @echo "Starting dashboard..."
    cd dashboard && pnpm dev

# Start both API server and dashboard (in parallel)
dev: up
    @echo "Starting development environment..."
    just --jobs 2 api-server dashboard

# Build the API server
build-api-server:
    cd api-server && cargo build --release

# Build the engine library
build-engine:
    cd engine && cargo build --release

# Build the dashboard
build-dashboard:
    cd dashboard && pnpm build

# Build everything
build: build-engine build-api-server build-dashboard

# Run tests
test:
    cd engine && cargo test
    cd api-server && cargo test

# Run linter
lint:
    cd engine && cargo clippy -- -D warnings
    cd api-server && cargo clippy -- -D warnings
    cd dashboard && pnpm lint

# Format code (entire monorepo)
fmt:
    cargo fmt --all
    pnpm exec prettier --write .

# Backwards-compatible alias
format: fmt

# Clean build artifacts
clean:
    cd engine && cargo clean
    cd api-server && cargo clean
    cd dashboard && rm -rf .next out dist

# Check code (compile without building)
check:
    cd engine && cargo check
    cd api-server && cargo check
    cd dashboard && pnpm type-check || true

# Setup project (install dependencies and prepare environment)
setup: install
    @echo "Setting up ClawGuild..."
    @echo "Make sure to:"
    @echo "1. Copy .env.example to .env and configure it"
    @echo "2. Start services: just up (or cd docker && docker compose up -d)"
    @echo "3. Run the API server: just api-server"
    @echo "4. Run the dashboard: just dashboard"
