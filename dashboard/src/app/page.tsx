'use client';

import { useEffect, useState } from 'react';
import { AgentCard } from '@/components/AgentCard';
import { DeploymentForm } from '@/components/DeploymentForm';
import { StatusIndicator } from '@/components/StatusIndicator';
import { api } from '@/lib/api';

interface Agent {
  id: string;
  name: string;
  role: 'master' | 'slave';
  status: 'pending' | 'deploying' | 'running' | 'stopped' | 'error';
}

export default function Home() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadAgents();
  }, []);

  const loadAgents = async () => {
    try {
      const data = await api.listAgents();
      setAgents(data);
    } catch (error) {
      console.error('Failed to load agents:', error);
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

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
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

      <div className="mt-8">
        <h2 className="text-2xl font-semibold mb-4">Deploy New Agent</h2>
        <DeploymentForm onSuccess={loadAgents} />
      </div>
    </main>
  );
}
