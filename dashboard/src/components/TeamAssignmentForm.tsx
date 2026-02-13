'use client';

import { useMemo, useState } from 'react';
import { api, Agent, Team } from '@/lib/api';

interface TeamAssignmentFormProps {
  agents: Agent[];
  teams: Team[];
  onSuccess?: () => void;
}

export function TeamAssignmentForm({ agents, teams, onSuccess }: TeamAssignmentFormProps) {
  const [teamId, setTeamId] = useState('');
  const [agentId, setAgentId] = useState('');
  const [role, setRole] = useState<'master' | 'slave'>('slave');
  const [loading, setLoading] = useState(false);

  const selectableTeams = useMemo(() => teams, [teams]);
  const selectableAgents = useMemo(() => agents, [agents]);

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!teamId || !agentId) {
      alert('Select a team and agent.');
      return;
    }
    setLoading(true);

    try {
      await api.assignAgentToTeam(teamId, { agent_id: agentId, role });
      onSuccess?.();
    } catch (error) {
      console.error('Failed to assign agent:', error);
      alert('Failed to assign agent to team.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 border rounded-2xl p-6 bg-card shadow-sm">
      <div>
        <h3 className="text-lg font-semibold">Assign Agent to Team</h3>
        <p className="text-sm text-muted-foreground">
          Update team membership and sync agent roles.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <div>
          <label className="block text-sm font-medium mb-1">Team</label>
          <select
            value={teamId}
            onChange={(e) => setTeamId(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
          >
            <option value="">Select a team</option>
            {selectableTeams.map((team) => (
              <option key={team.id} value={team.id}>
                {team.name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Agent</label>
          <select
            value={agentId}
            onChange={(e) => setAgentId(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
          >
            <option value="">Select an agent</option>
            {selectableAgents.map((agent) => (
              <option key={agent.id} value={agent.id}>
                {agent.name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Role</label>
          <select
            value={role}
            onChange={(e) => setRole(e.target.value as 'master' | 'slave')}
            className="w-full px-3 py-2 border rounded-md"
          >
            <option value="master">Master</option>
            <option value="slave">Slave</option>
          </select>
        </div>
      </div>

      <button
        type="submit"
        disabled={loading}
        className="w-full px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
      >
        {loading ? 'Assigning...' : 'Assign Agent'}
      </button>
    </form>
  );
}
