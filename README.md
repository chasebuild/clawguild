# ClawGuild - OpenClaw Agent Swarm Orchestrator

A Rust-based orchestrator for deploying and managing multiple OpenClaw agent instances across VPS providers (Railway, Fly.io, AWS) with Discord-based coordination.

## Features

- **Multi-VPS Deployment**: Deploy agents to Railway, Fly.io, or AWS using adapter pattern
- **Master-Slave Coordination**: Master agent delegates tasks to specialized slave agents via Discord
- **One-Click Dashboard**: Web-based dashboard for managing agent teams and monitoring status
- **Discord Integration**: All agents coordinate through Discord channels
- **BYOM Support**: Use OpenClaw API or bring your own model (BYOM)

## Architecture

- **Orchestrator Service** (Rust): Manages deployments, configurations, and agent lifecycle
- **VPS Adapter Layer**: Abstract interface for different VPS providers
- **Agent Coordinator**: Handles master-slave coordination via Discord
- **Web Dashboard**: Next.js frontend with shadcn/ui components

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 22+
- pnpm
- SurrealDB (for database)
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

3. Start SurrealDB:
   ```bash
   # Using Docker
   docker run -d -p 8000:8000 surrealdb/surrealdb:latest start --log trace --user root --pass root memory
   ```

4. Configure environment variables (see `.env.example`):
   - Set `DATABASE_URL=ws://localhost:8000` (or your SurrealDB URL)

5. Start the orchestrator service (it will run migrations automatically):
   ```bash
   cd orchestrator
   cargo run
   ```

6. Start the dashboard:
   ```bash
   cd dashboard
   pnpm install
   pnpm dev
   ```

## Project Structure

```
clawguild/
├── orchestrator/     # Rust orchestrator service
├── dashboard/        # Next.js web dashboard
├── docker/           # Docker configurations
└── scripts/          # Deployment scripts
```

## License

MIT
