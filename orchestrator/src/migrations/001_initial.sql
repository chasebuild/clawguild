-- Enable UUID extension
-- SQLite doesn't have native UUID support, so we'll use TEXT

-- Agents table
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('master', 'slave')),
    status TEXT NOT NULL CHECK(status IN ('pending', 'deploying', 'running', 'stopped', 'error')),
    deployment_id TEXT,
    team_id TEXT,
    discord_bot_token TEXT,
    discord_channel_id TEXT,
    model_provider TEXT NOT NULL CHECK(model_provider IN ('openclaw', 'anthropic', 'openai', 'byom')),
    model_api_key TEXT,
    model_endpoint TEXT,
    personality TEXT,
    skills TEXT, -- JSON array of strings
    workspace_dir TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Deployments table
CREATE TABLE IF NOT EXISTS deployments (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    provider TEXT NOT NULL CHECK(provider IN ('railway', 'flyio', 'aws')),
    region TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending', 'creating', 'running', 'stopped', 'failed')),
    endpoint TEXT,
    gateway_url TEXT,
    volume_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Teams table
CREATE TABLE IF NOT EXISTS teams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    master_id TEXT NOT NULL,
    slave_ids TEXT NOT NULL, -- JSON array of UUIDs
    discord_channel_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (master_id) REFERENCES agents(id)
);

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL,
    assigned_to TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending', 'in_progress', 'completed', 'failed')),
    description TEXT NOT NULL,
    result TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (team_id) REFERENCES teams(id),
    FOREIGN KEY (assigned_to) REFERENCES agents(id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_agents_team_id ON agents(team_id);
CREATE INDEX IF NOT EXISTS idx_agents_deployment_id ON agents(deployment_id);
CREATE INDEX IF NOT EXISTS idx_deployments_agent_id ON deployments(agent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_team_id ON tasks(team_id);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned_to ON tasks(assigned_to);
