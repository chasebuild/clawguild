'use client';

import { useMemo, useState } from 'react';
import { api, Agent, DiscordChannels } from '@/lib/api';

interface TeamFormProps {
  agents: Agent[];
  onSuccess?: () => void;
}

const emptyChannels: DiscordChannels = {
  coordination_logs: '',
  slave_communication: '',
  master_orders: '',
};

export function TeamForm({ agents, onSuccess }: TeamFormProps) {
  const [name, setName] = useState('');
  const [masterId, setMasterId] = useState('');
  const [slaveIds, setSlaveIds] = useState<string[]>([]);
  const [channels, setChannels] = useState<DiscordChannels>(emptyChannels);
  const [loading, setLoading] = useState(false);

  const selectableAgents = useMemo(() => agents, [agents]);

  const toggleSlave = (agentId: string) => {
    setSlaveIds((prev) =>
      prev.includes(agentId) ? prev.filter((id) => id !== agentId) : [...prev, agentId],
    );
  };

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!masterId) {
      alert('Select a master agent.');
      return;
    }
    setLoading(true);

    try {
      await api.createTeam({
        name,
        master_id: masterId,
        slave_ids: slaveIds.filter((id) => id !== masterId),
        discord_channel_id: channels.coordination_logs,
        discord_channels: channels,
      });
      setName('');
      setMasterId('');
      setSlaveIds([]);
      setChannels(emptyChannels);
      onSuccess?.();
    } catch (error) {
      console.error('Failed to create team:', error);
      alert('Failed to create team.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 border rounded-2xl p-6 bg-card shadow-sm">
      <div>
        <h3 className="text-lg font-semibold">Create Team</h3>
        <p className="text-sm text-muted-foreground">
          Choose agents, then bind the Discord channels used for coordination.
        </p>
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Team Name</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
          className="w-full px-3 py-2 border rounded-md"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Master Agent</label>
        <select
          value={masterId}
          onChange={(e) => setMasterId(e.target.value)}
          className="w-full px-3 py-2 border rounded-md"
          required
        >
          <option value="">Select a master</option>
          {selectableAgents.map((agent) => (
            <option key={agent.id} value={agent.id}>
              {agent.name} ({agent.role})
            </option>
          ))}
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Slave Agents</label>
        <div className="grid gap-2 md:grid-cols-2">
          {selectableAgents.map((agent) => (
            <label key={agent.id} className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={slaveIds.includes(agent.id)}
                onChange={() => toggleSlave(agent.id)}
                disabled={agent.id === masterId}
              />
              <span>
                {agent.name} ({agent.role})
              </span>
            </label>
          ))}
          {selectableAgents.length === 0 && (
            <span className="text-sm text-muted-foreground">No agents available.</span>
          )}
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <div>
          <label className="block text-sm font-medium mb-1">Coordination Logs Channel</label>
          <input
            type="text"
            value={channels.coordination_logs}
            onChange={(e) => setChannels({ ...channels, coordination_logs: e.target.value })}
            required
            className="w-full px-3 py-2 border rounded-md"
          />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Slave Communication Channel</label>
          <input
            type="text"
            value={channels.slave_communication}
            onChange={(e) => setChannels({ ...channels, slave_communication: e.target.value })}
            required
            className="w-full px-3 py-2 border rounded-md"
          />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Master Orders Channel</label>
          <input
            type="text"
            value={channels.master_orders}
            onChange={(e) => setChannels({ ...channels, master_orders: e.target.value })}
            required
            className="w-full px-3 py-2 border rounded-md"
          />
        </div>
      </div>

      <button
        type="submit"
        disabled={loading}
        className="w-full px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
      >
        {loading ? 'Creating...' : 'Create Team'}
      </button>
    </form>
  );
}
