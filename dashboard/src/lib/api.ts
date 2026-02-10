import axios from 'axios';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

const client = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

export interface Agent {
  id: string;
  name: string;
  role: 'master' | 'slave';
  status: 'pending' | 'deploying' | 'running' | 'stopped' | 'error';
  responsibility?: string;
  emoji?: string;
}

export interface CreateAgentRequest {
  name: string;
  role: 'master' | 'slave';
  provider: 'railway' | 'flyio' | 'aws';
  region?: string;
  discord_bot_token?: string;
  discord_channel_id?: string;
  model_provider: 'openclaw' | 'anthropic' | 'openai' | 'byom';
  model_api_key?: string;
  model_endpoint?: string;
  personality?: string;
  skills: string[];
  responsibility?: string;
  emoji?: string;
}

export interface Team {
  id: string;
  name: string;
  master_id: string;
  slave_ids: string[];
  discord_channel_id: string;
}

export interface CreateTeamRequest {
  name: string;
  master_id: string;
  slave_ids: string[];
  discord_channel_id: string;
}

export const api = {
  async listAgents(): Promise<Agent[]> {
    const response = await client.get<Agent[]>('/api/agents');
    return response.data;
  },

  async createAgent(data: CreateAgentRequest): Promise<Agent> {
    const response = await client.post<Agent>('/api/agents', data);
    return response.data;
  },

  async getAgentStatus(id: string): Promise<Agent['status']> {
    const response = await client.get<Agent['status']>(`/api/agents/${id}/status`);
    return response.data;
  },

  async destroyAgent(id: string): Promise<void> {
    await client.delete(`/api/agents/${id}`);
  },

  async listTeams(): Promise<Team[]> {
    const response = await client.get<Team[]>('/api/teams');
    return response.data;
  },

  async createTeam(data: CreateTeamRequest): Promise<Team> {
    const response = await client.post<Team>('/api/teams', data);
    return response.data;
  },

  async getTeamRoster(teamId: string): Promise<TeamRosterResponse> {
    const response = await client.get<TeamRosterResponse>(`/api/teams/${teamId}/roster`);
    return response.data;
  },

  async getServerHealth(): Promise<ServerHealthResponse> {
    const response = await client.get<ServerHealthResponse>('/api/server/health');
    return response.data;
  },

  async getServerStatus(): Promise<ServerStatusResponse> {
    const response = await client.get<ServerStatusResponse>('/api/server/status');
    return response.data;
  },

  async listDeployments(): Promise<DeploymentResponse[]> {
    const response = await client.get<DeploymentResponse[]>('/api/deployments');
    return response.data;
  },

  async getDeployment(id: string): Promise<DeploymentResponse> {
    const response = await client.get<DeploymentResponse>(`/api/deployments/${id}`);
    return response.data;
  },

  async getDeploymentLogs(id: string, lines?: number): Promise<string[]> {
    const params = lines ? { lines: lines.toString() } : {};
    const response = await client.get<string[]>(`/api/deployments/${id}/logs`, { params });
    return response.data;
  },
};

export interface TeamRosterMember {
  id: string;
  name: string;
  role: string;
  responsibility: string;
  emoji: string;
  status: 'pending' | 'deploying' | 'running' | 'stopped' | 'error';
}

export interface TeamRosterResponse {
  team_id: string;
  team_name: string;
  members: TeamRosterMember[];
}

export interface ServerHealthResponse {
  status: string;
  timestamp: string;
  uptime_seconds: number;
}

export interface ServerStatusResponse {
  status: string;
  version: string;
  database_connected: boolean;
  timestamp: string;
}

export interface DeploymentResponse {
  id: string;
  agent_id: string;
  provider: string;
  region: string | null;
  status: string;
  endpoint: string | null;
  gateway_url: string | null;
  created_at: string;
  updated_at: string;
}
