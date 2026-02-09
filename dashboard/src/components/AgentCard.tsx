'use client';

import { StatusIndicator } from './StatusIndicator';

interface AgentCardProps {
  agent: {
    id: string;
    name: string;
    role: 'master' | 'slave';
    status: 'pending' | 'deploying' | 'running' | 'stopped' | 'error';
    responsibility?: string;
    emoji?: string;
  };
}

export function AgentCard({ agent }: AgentCardProps) {
  return (
    <div className="border rounded-lg p-6 bg-card">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">{agent.name}</h3>
        <StatusIndicator status={agent.status} />
      </div>
      <div className="space-y-2">
        {agent.emoji && (
          <div className="text-2xl mb-2">{agent.emoji}</div>
        )}
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Role:</span>
          <span className="text-sm font-medium capitalize">{agent.role}</span>
        </div>
        {agent.responsibility && (
          <p className="text-sm text-muted-foreground">{agent.responsibility}</p>
        )}
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Status:</span>
          <span className="text-sm font-medium capitalize">{agent.status}</span>
        </div>
      </div>
    </div>
  );
}
