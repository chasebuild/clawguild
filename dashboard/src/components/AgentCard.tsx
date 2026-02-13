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
    <div className="border rounded-2xl p-6 bg-card shadow-sm hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="text-3xl leading-none">{agent.emoji || '🧭'}</div>
          <div>
            <h3 className="text-lg font-semibold">{agent.name}</h3>
            <p className="text-xs uppercase tracking-wide text-muted-foreground">
              {agent.role} agent
            </p>
          </div>
        </div>
        <StatusIndicator status={agent.status} />
      </div>

      <div className="mt-4 space-y-2 text-sm text-muted-foreground">
        <div className="flex items-center gap-2">
          <span className="font-medium text-foreground">Focus</span>
          <span className="text-xs">•</span>
          <span>{agent.responsibility || 'Standing by for directives'}</span>
        </div>
      </div>
    </div>
  );
}
