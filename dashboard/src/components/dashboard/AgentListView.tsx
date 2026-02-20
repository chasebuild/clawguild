'use client';

import { Agent } from '@/lib/api';

interface AgentListViewProps {
  agents: Agent[];
  selectedAgentId: string | null;
  onSelectAgent: (agentId: string) => void;
  loading: boolean;
}

export function AgentListView({
  agents,
  selectedAgentId,
  onSelectAgent,
  loading,
}: AgentListViewProps) {
  if (loading) {
    return <div className="panel-surface p-4 text-sm text-muted-foreground">Loading agents...</div>;
  }

  if (agents.length === 0) {
    return (
      <div className="panel-surface p-4 text-sm text-muted-foreground">
        No agents yet. Use quick spawn to create your first one.
      </div>
    );
  }

  return (
    <section className="panel-surface overflow-hidden">
      <div className="grid grid-cols-[1.8fr_0.8fr_0.8fr_1.2fr] border-b border-border/80 px-4 py-2 text-xs uppercase tracking-wide text-muted-foreground">
        <span>Name</span>
        <span>Role</span>
        <span>Status</span>
        <span>Runtime</span>
      </div>
      <div className="max-h-[520px] overflow-y-auto">
        {agents.map((agent) => {
          const selected = selectedAgentId === agent.id;
          return (
            <button
              key={agent.id}
              type="button"
              onClick={() => onSelectAgent(agent.id)}
              className={
                selected
                  ? 'grid w-full grid-cols-[1.8fr_0.8fr_0.8fr_1.2fr] gap-2 border-b border-border/70 bg-panel-selected px-4 py-3 text-left text-sm text-foreground focus:outline-none'
                  : 'grid w-full grid-cols-[1.8fr_0.8fr_0.8fr_1.2fr] gap-2 border-b border-border/70 bg-panel-row px-4 py-3 text-left text-sm text-foreground hover:bg-panel-hover focus:bg-panel-hover focus:outline-none'
              }
            >
              <div className="min-w-0">
                <p className="truncate font-medium">
                  {agent.emoji ? `${agent.emoji} ` : ''}
                  {agent.name}
                </p>
                <p className="truncate text-xs text-muted-foreground">
                  {agent.responsibility || 'Standing by for directives'}
                </p>
              </div>
              <span className="text-xs uppercase tracking-wide text-muted-foreground">
                {agent.role}
              </span>
              <span className={`status-badge status-${agent.status}`}>{agent.status}</span>
              <span className="text-xs uppercase tracking-wide text-muted-foreground">
                {agent.runtime}
              </span>
            </button>
          );
        })}
      </div>
    </section>
  );
}
