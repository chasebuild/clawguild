'use client';

import { useEffect, useMemo, useState } from 'react';
import { AgentCard } from '@/components/AgentCard';
import { DeploymentForm } from '@/components/DeploymentForm';
import { MultiDeploymentForm } from '@/components/MultiDeploymentForm';
import { TeamRoster } from '@/components/TeamRoster';
import { TeamForm } from '@/components/TeamForm';
import { TeamAssignmentForm } from '@/components/TeamAssignmentForm';
import { VpsLogsViewer } from '@/components/VpsLogsViewer';
import { TaskManager } from '@/components/TaskManager';
import {
  api,
  DeploymentResponse,
  ServerHealthResponse,
  ServerStatusResponse,
  Team,
} from '@/lib/api';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';

interface Agent {
  id: string;
  name: string;
  role: 'master' | 'slave';
  status: 'pending' | 'deploying' | 'running' | 'stopped' | 'error';
  responsibility?: string;
  emoji?: string;
}

export default function Home() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [teams, setTeams] = useState<Team[]>([]);
  const [deployments, setDeployments] = useState<DeploymentResponse[]>([]);
  const [serverStatus, setServerStatus] = useState<ServerStatusResponse | null>(null);
  const [serverHealth, setServerHealth] = useState<ServerHealthResponse | null>(null);
  const [selectedTeamId, setSelectedTeamId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [agentsData, teamsData, deploymentsData, statusData, healthData] = await Promise.all([
        api.listAgents(),
        api.listTeams(),
        api.listDeployments(),
        api.getServerStatus(),
        api.getServerHealth(),
      ]);
      setAgents(agentsData);
      setTeams(teamsData);
      setDeployments(deploymentsData);
      setServerStatus(statusData);
      setServerHealth(healthData);
      if (teamsData.length > 0 && !selectedTeamId) {
        setSelectedTeamId(teamsData[0].id);
      }
    } catch (error) {
      console.error('Failed to load data:', error);
    } finally {
      setLoading(false);
    }
  };

  const agentStats = useMemo(() => {
    const running = agents.filter((a) => a.status === 'running').length;
    const deploying = agents.filter((a) => a.status === 'deploying').length;
    const error = agents.filter((a) => a.status === 'error').length;
    const stopped = agents.filter((a) => a.status === 'stopped').length;
    return { running, deploying, error, stopped, total: agents.length };
  }, [agents]);

  const deploymentStats = useMemo(() => {
    const running = deployments.filter((d) => d.status.toLowerCase() === 'running').length;
    const failed = deployments.filter((d) => d.status.toLowerCase() === 'failed').length;
    return { running, failed, total: deployments.length };
  }, [deployments]);

  const errorAgents = useMemo(() => agents.filter((agent) => agent.status === 'error'), [agents]);

  const errorDeployments = useMemo(
    () => deployments.filter((deployment) => deployment.status.toLowerCase() === 'failed'),
    [deployments],
  );

  return (
    <main className="app-shell">
      <div className="container mx-auto px-6 py-10 space-y-10">
        <header className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.3em] text-muted-foreground">
              Agent Orchestration Platform
            </p>
            <h1 className="text-4xl md:text-5xl font-semibold mt-2">ClawGuild Command Center</h1>
            <p className="text-muted-foreground mt-3 max-w-xl">
              Monitor live agent health, investigate errors, and dispatch new work without context
              switching.
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <Badge variant={serverStatus?.status === 'running' ? 'default' : 'destructive'}>
              API {serverStatus?.status || 'unknown'}
            </Badge>
            <Badge variant={serverStatus?.database_connected ? 'secondary' : 'destructive'}>
              DB {serverStatus?.database_connected ? 'connected' : 'offline'}
            </Badge>
            <Badge variant="secondary">{agentStats.running} running agents</Badge>
            <Badge variant={agentStats.error > 0 ? 'destructive' : 'secondary'}>
              {agentStats.error} errors
            </Badge>
          </div>
        </header>

        <Tabs defaultValue="command" className="space-y-6">
          <TabsList className="bg-card/70 border rounded-full p-1">
            <TabsTrigger value="command">Command Center</TabsTrigger>
            <TabsTrigger value="orchestration">Orchestration</TabsTrigger>
            <TabsTrigger value="debug">Debugging</TabsTrigger>
          </TabsList>

          <TabsContent value="command" className="space-y-6">
            <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
              <div className="bg-card border rounded-2xl p-5 shadow-sm">
                <p className="text-xs uppercase tracking-wide text-muted-foreground">API Health</p>
                <div className="mt-3 flex items-end justify-between">
                  <div>
                    <p className="text-2xl font-semibold">{serverStatus?.status || 'unknown'}</p>
                    <p className="text-xs text-muted-foreground">
                      {serverStatus?.version ? `v${serverStatus.version}` : 'No version'}
                    </p>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {serverStatus?.timestamp
                      ? new Date(serverStatus.timestamp).toLocaleTimeString()
                      : '—'}
                  </div>
                </div>
              </div>
              <div className="bg-card border rounded-2xl p-5 shadow-sm">
                <p className="text-xs uppercase tracking-wide text-muted-foreground">Database</p>
                <div className="mt-3 flex items-end justify-between">
                  <div>
                    <p className="text-2xl font-semibold">
                      {serverStatus?.database_connected ? 'Connected' : 'Offline'}
                    </p>
                    <p className="text-xs text-muted-foreground">Postgres</p>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {serverHealth
                      ? `${Math.floor(serverHealth.uptime_seconds / 3600)}h uptime`
                      : '—'}
                  </div>
                </div>
              </div>
              <div className="bg-card border rounded-2xl p-5 shadow-sm">
                <p className="text-xs uppercase tracking-wide text-muted-foreground">Agents</p>
                <div className="mt-3 space-y-1">
                  <p className="text-2xl font-semibold">{agentStats.running} running</p>
                  <p className="text-xs text-muted-foreground">
                    {agentStats.deploying} deploying · {agentStats.error} errors ·{' '}
                    {agentStats.stopped} stopped
                  </p>
                </div>
              </div>
              <div className="bg-card border rounded-2xl p-5 shadow-sm">
                <p className="text-xs uppercase tracking-wide text-muted-foreground">Deployments</p>
                <div className="mt-3 space-y-1">
                  <p className="text-2xl font-semibold">{deploymentStats.running} live</p>
                  <p className="text-xs text-muted-foreground">
                    {deploymentStats.failed} failed · {deploymentStats.total} total
                  </p>
                </div>
              </div>
            </section>

            <section className="grid gap-6 lg:grid-cols-[1.2fr_0.8fr]">
              <div className="bg-card border rounded-2xl p-6 shadow-sm space-y-4">
                <div>
                  <h2 className="text-xl font-semibold">Active Alerts</h2>
                  <p className="text-sm text-muted-foreground">
                    Quick glance at agents or deployments that need attention.
                  </p>
                </div>
                {errorAgents.length === 0 && errorDeployments.length === 0 ? (
                  <div className="text-sm text-muted-foreground">
                    No active incidents. Everything is running smoothly.
                  </div>
                ) : (
                  <div className="space-y-3">
                    {errorAgents.map((agent) => (
                      <div
                        key={agent.id}
                        className="flex items-center justify-between rounded-xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm"
                      >
                        <span>{agent.name} is in error state</span>
                        <Badge variant="destructive">Agent</Badge>
                      </div>
                    ))}
                    {errorDeployments.map((deployment) => (
                      <div
                        key={deployment.id}
                        className="flex items-center justify-between rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm"
                      >
                        <span>Deployment {deployment.id.slice(0, 8)} failed</span>
                        <Badge variant="secondary">Deployment</Badge>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              <div className="space-y-4">
                {teams.length > 0 && selectedTeamId && (
                  <div>
                    <label className="text-sm font-medium mb-2 block">Active Team</label>
                    <select
                      value={selectedTeamId}
                      onChange={(e) => setSelectedTeamId(e.target.value)}
                      className="border rounded-md px-3 py-2 bg-background w-full"
                    >
                      {teams.map((team) => (
                        <option key={team.id} value={team.id}>
                          {team.name}
                        </option>
                      ))}
                    </select>
                  </div>
                )}
                {selectedTeamId ? (
                  <TeamRoster teamId={selectedTeamId} />
                ) : (
                  <div className="bg-card border rounded-2xl p-6 text-sm text-muted-foreground">
                    Create a team to see the live roster here.
                  </div>
                )}
              </div>
            </section>

            <section className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-xl font-semibold">Live Agents</h2>
                  <p className="text-sm text-muted-foreground">Track status across the swarm.</p>
                </div>
                <button
                  type="button"
                  onClick={loadData}
                  className="px-3 py-2 text-sm border rounded-md"
                >
                  Refresh
                </button>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
                {loading ? (
                  <div>Loading agents...</div>
                ) : agents.length === 0 ? (
                  <div className="col-span-full text-center text-muted-foreground">
                    No agents deployed yet. Create your first agent team below.
                  </div>
                ) : (
                  agents.map((agent) => <AgentCard key={agent.id} agent={agent} />)
                )}
              </div>
            </section>
          </TabsContent>

          <TabsContent value="orchestration" className="space-y-6">
            <div className="grid gap-6 lg:grid-cols-2">
              <DeploymentForm onSuccess={loadData} />
              <MultiDeploymentForm agents={agents} onSuccess={loadData} />
            </div>
            <div className="grid gap-6 lg:grid-cols-2">
              <TeamForm agents={agents} onSuccess={loadData} />
              <TeamAssignmentForm agents={agents} teams={teams} onSuccess={loadData} />
            </div>
            <TaskManager agents={agents} />
          </TabsContent>

          <TabsContent value="debug" className="space-y-6">
            <VpsLogsViewer />
          </TabsContent>
        </Tabs>
      </div>
    </main>
  );
}
