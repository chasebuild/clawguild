# ClawGuild - Claw Runtime Orchestrator

A Rust-based orchestrator for deploying and managing multiple Claw runtimes across VPS providers (Railway, Fly.io, AWS) with Discord-based coordination.

## Features

- **Multi-VPS Deployment**: Deploy agents to Railway, Fly.io, or AWS using adapter pattern
- **Master-Slave Coordination**: Master agent delegates tasks to specialized slave agents via Discord
- **One-Click Dashboard**: Web-based dashboard for managing agent teams and monitoring status
- **Discord Integration**: All agents coordinate through Discord channels
- **Multi-Runtime Support**: OpenClaw, ZeroClaw, PicoClaw, and NanoClaw runtimes
- **BYOM Support**: Use OpenClaw API or bring your own model (BYOM)

## Architecture

- **Engine (Rust)**: Core orchestration logic, storage, deployments, and Discord coordination
- **API Server (Rust)**: Axum HTTP API exposing engine functionality
- **VPS Adapter Layer**: Abstract interface for different VPS providers
- **Runtime Layer**: Pluggable runtime crates for each Claw family
- **Web Dashboard**: Next.js frontend with shadcn/ui components

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 22+
- pnpm
- PostgreSQL (for database)
- Docker (optional, for local development)

### Setup

1. Clone the repository
2. Install dependencies:

   ```bash
   # Rust dependencies
   cargo build

   # Frontend dependencies
   cd dashboard
   pnpm install
   ```

3. Start PostgreSQL:

   ```bash
   # Using Docker
   docker run -d -p 5432:5432 \\
     -e POSTGRES_USER=postgres \\
     -e POSTGRES_PASSWORD=postgres \\
     -e POSTGRES_DB=clawguild \\
     postgres:16-alpine
   ```

4. Configure environment variables (see `.env.example`):
   - Set `DATABASE_URL=postgres://postgres:postgres@localhost:5432/clawguild`
   - Optional: set `API_KEY` to require `x-api-key` on API requests

5. Start the API server (it will run migrations automatically):

   ```bash
   cd api-server
   cargo run
   ```

6. Start the dashboard:

   ```bash
   cd dashboard
   pnpm install
   pnpm dev
   ```

   If `API_KEY` is set for the server, also set `NEXT_PUBLIC_API_KEY` in the dashboard environment.

## Project Structure

```
clawguild/
├── engine/           # Core Rust engine library
├── api-server/       # Rust API server (Axum)
├── dashboard/        # Next.js web dashboard
├── claws/            # Runtime crates (OpenClaw, ZeroClaw, PicoClaw, NanoClaw)
├── docs/             # Architecture and compatibility docs
├── docker/           # Docker configurations
└── scripts/          # Deployment scripts
```

## Runtime Compatibility

See `docs/runtime-compat.md` for runtime-specific configuration, overrides, and support notes.

## License

MIT
