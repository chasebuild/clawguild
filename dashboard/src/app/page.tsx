'use client';

import { useEffect, useState } from 'react';
import { AgentCard } from '@/components/AgentCard';
import { DeploymentForm } from '@/components/DeploymentForm';
import { TeamRoster } from '@/components/TeamRoster';
import { ServerManagement } from '@/components/ServerManagement';
import { VpsLogsViewer } from '@/components/VpsLogsViewer';
import { api, Team } from '@/lib/api';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

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
  const [selectedTeamId, setSelectedTeamId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [agentsData, teamsData] = await Promise.all([
        api.listAgents(),
        api.listTeams(),
      ]);
      setAgents(agentsData);
      setTeams(teamsData);
      if (teamsData.length > 0 && !selectedTeamId) {
        setSelectedTeamId(teamsData[0].id);
      }
    } catch (error) {
      console.error('Failed to load data:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="container mx-auto p-8">
      <div className="mb-8">
        <h1 className="text-4xl font-bold mb-2">ClawGuild</h1>
        <p className="text-muted-foreground">
          OpenClaw Agent Swarm Orchestrator
        </p>
      </div>

      <Tabs defaultValue="agents" className="space-y-6">
        <TabsList>
          <TabsTrigger value="agents">Agents</TabsTrigger>
          <TabsTrigger value="server">Server Management</TabsTrigger>
          <TabsTrigger value="logs">VPS Logs</TabsTrigger>
        </TabsList>

        <TabsContent value="agents" className="space-y-6">
          {teams.length > 0 && selectedTeamId && (
            <div>
              <div className="mb-4">
                <label className="text-sm font-medium mb-2 block">Select Team:</label>
                <select
                  value={selectedTeamId}
                  onChange={(e) => setSelectedTeamId(e.target.value)}
                  className="border rounded px-3 py-2 bg-background"
                >
                  {teams.map((team) => (
                    <option key={team.id} value={team.id}>
                      {team.name}
                    </option>
                  ))}
                </select>
              </div>
              <TeamRoster teamId={selectedTeamId} />
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {loading ? (
              <div>Loading agents...</div>
            ) : agents.length === 0 ? (
              <div className="col-span-full text-center text-muted-foreground">
                No agents deployed yet. Create your first agent team below.
              </div>
            ) : (
              agents.map((agent) => (
                <AgentCard key={agent.id} agent={agent} />
              ))
            )}
          </div>

          <div>
            <h2 className="text-2xl font-semibold mb-4">Deploy New Agent</h2>
            <DeploymentForm onSuccess={loadData} />
          </div>
        </TabsContent>

        <TabsContent value="server">
          <ServerManagement />
        </TabsContent>

        <TabsContent value="logs">
          <VpsLogsViewer />
        </TabsContent>
      </Tabs>
    </main>
  );
}
