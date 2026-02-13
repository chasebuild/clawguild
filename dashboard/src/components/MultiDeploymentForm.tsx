'use client';

import { useState } from 'react';
import { api, Agent } from '@/lib/api';

interface MultiDeploymentFormProps {
  agents: Agent[];
  onSuccess?: () => void;
}

export function MultiDeploymentForm({ agents, onSuccess }: MultiDeploymentFormProps) {
  const [selectedAgents, setSelectedAgents] = useState<string[]>([]);
  const [provider, setProvider] = useState<'railway' | 'flyio' | 'aws'>('flyio');
  const [region, setRegion] = useState('');
  const [loading, setLoading] = useState(false);

  const toggleAgent = (agentId: string) => {
    setSelectedAgents((prev) =>
      prev.includes(agentId) ? prev.filter((id) => id !== agentId) : [...prev, agentId],
    );
  };

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (selectedAgents.length === 0) {
      alert('Select at least one agent.');
      return;
    }
    setLoading(true);
    try {
      await api.deployAgentsMulti({
        agent_ids: selectedAgents,
        provider,
        region: region || undefined,
      });
      setSelectedAgents([]);
      setRegion('');
      onSuccess?.();
    } catch (error) {
      console.error('Failed to deploy multiple agents:', error);
      alert('Failed to deploy multi-agent setup.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 border rounded-2xl p-6 bg-card shadow-sm">
      <div>
        <h3 className="text-lg font-semibold">Deploy Multiple Agents</h3>
        <p className="text-sm text-muted-foreground">
          Launch a shared VPS for multiple agents.
        </p>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Agents</label>
        <div className="grid gap-2 md:grid-cols-2">
          {agents.map((agent) => (
            <label key={agent.id} className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={selectedAgents.includes(agent.id)}
                onChange={() => toggleAgent(agent.id)}
              />
              <span>{agent.name}</span>
            </label>
          ))}
          {agents.length === 0 && (
            <span className="text-sm text-muted-foreground">No agents available.</span>
          )}
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div>
          <label className="block text-sm font-medium mb-1">VPS Provider</label>
          <select
            value={provider}
            onChange={(e) => setProvider(e.target.value as 'railway' | 'flyio' | 'aws')}
            className="w-full px-3 py-2 border rounded-md"
          >
            <option value="railway">Railway</option>
            <option value="flyio">Fly.io</option>
            <option value="aws">AWS</option>
          </select>
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Region (optional)</label>
          <input
            type="text"
            value={region}
            onChange={(e) => setRegion(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
          />
        </div>
      </div>

      <button
        type="submit"
        disabled={loading}
        className="w-full px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
      >
        {loading ? 'Deploying...' : 'Deploy Multi-Agent'}
      </button>
    </form>
  );
}
