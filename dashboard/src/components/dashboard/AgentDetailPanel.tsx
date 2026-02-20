'use client';

import { Agent, DeploymentResponse, Team } from '@/lib/api';

interface AgentDetailPanelProps {
  agent: Agent | null;
  deployments: DeploymentResponse[];
  teams: Team[];
}

export function AgentDetailPanel({ agent, deployments, teams }: AgentDetailPanelProps) {
  if (!agent) {
    return (
      <aside className="panel-surface p-5 text-sm text-muted-foreground">
        Select an agent to inspect runtime, deployment, and team context.
      </aside>
    );
  }

  const latestDeployment = deployments
    .filter(
      (deployment) =>
        deployment.agent_id === agent.id ||
        (deployment.agent_ids ? deployment.agent_ids.includes(agent.id) : false),
    )
    .sort((a, b) => +new Date(b.created_at) - +new Date(a.created_at))[0];

  const team =
    teams.find((item) => item.master_id === agent.id) ||
    teams.find((item) => item.slave_ids.includes(agent.id));

  return (
    <aside className="panel-surface space-y-5 p-5">
      <div>
        <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Details</p>
        <h3 className="mt-2 text-lg font-semibold text-foreground">
          {agent.emoji ? `${agent.emoji} ` : ''}
          {agent.name}
        </h3>
        <p className="mt-1 text-sm text-muted-foreground">
          {agent.responsibility || 'Standing by for directives'}
        </p>
      </div>

      <dl className="grid grid-cols-2 gap-3 text-sm">
        <Meta label="Role" value={agent.role} />
        <Meta
          label="Status"
          value={agent.status}
          className={`status-badge status-${agent.status}`}
        />
        <Meta label="Runtime" value={agent.runtime} />
        <Meta label="Team" value={team?.name || 'Unassigned'} />
      </dl>

      <section className="space-y-3">
        <h4 className="text-xs uppercase tracking-wide text-muted-foreground">Deployment</h4>
        {latestDeployment ? (
          <div className="rounded-lg border border-border bg-panel-row p-3 text-sm">
            <p className="font-medium text-foreground">{latestDeployment.provider}</p>
            <p className="text-muted-foreground">Status: {latestDeployment.status}</p>
            <p className="text-muted-foreground">
              Endpoint: {latestDeployment.endpoint || latestDeployment.gateway_url || 'n/a'}
            </p>
            <p className="text-xs text-muted-foreground">
              Updated {new Date(latestDeployment.updated_at).toLocaleString()}
            </p>
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">No deployments recorded for this agent.</p>
        )}
      </section>

      <section className="space-y-3">
        <h4 className="text-xs uppercase tracking-wide text-muted-foreground">Team Channels</h4>
        {team ? (
          <div className="rounded-lg border border-border bg-panel-row p-3 text-sm text-muted-foreground">
            <p>Coordination: {team.discord_channel_id || 'n/a'}</p>
            <p>Master: {team.master_id}</p>
            <p>Slaves: {team.slave_ids.length}</p>
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">No team assignment yet.</p>
        )}
      </section>
    </aside>
  );
}

function Meta({ label, value, className }: { label: string; value: string; className?: string }) {
  return (
    <div className="rounded-md border border-border bg-panel-row px-3 py-2">
      <dt className="text-[11px] uppercase tracking-wide text-muted-foreground">{label}</dt>
      <dd className={className || 'mt-1 text-sm font-medium text-foreground'}>{value}</dd>
    </div>
  );
}
