'use client';

import { useMemo } from 'react';
import { Agent, DeploymentResponse, Team } from '@/lib/api';
import { AgentDetailPanel } from './AgentDetailPanel';
import { AgentListView } from './AgentListView';
import { CommandToolbar } from './CommandToolbar';
import { QuickSpawnPanel } from './QuickSpawnPanel';
import {
  CommandFilters,
  FieldErrors,
  NoticeState,
  QuickSpawnDraft,
  SpawnTemplate,
  SpawnTemplateId,
} from './types';

interface CommandWorkspaceProps {
  agents: Agent[];
  deployments: DeploymentResponse[];
  teams: Team[];
  loading: boolean;
  filters: CommandFilters;
  selectedAgentId: string | null;
  quickSpawnDraft: QuickSpawnDraft;
  quickSpawnErrors: FieldErrors;
  quickSpawnSubmitting: boolean;
  quickSpawnNotice: NoticeState | null;
  activeTemplate: SpawnTemplateId;
  templates: SpawnTemplate[];
  focusSignal: number;
  onFiltersChange: (next: CommandFilters) => void;
  onSelectAgent: (agentId: string) => void;
  onDraftChange: (next: QuickSpawnDraft) => void;
  onTemplateSelect: (template: SpawnTemplate) => void;
  onSubmitQuickSpawn: () => void;
  onDismissQuickSpawnNotice: () => void;
}

export function CommandWorkspace({
  agents,
  deployments,
  teams,
  loading,
  filters,
  selectedAgentId,
  quickSpawnDraft,
  quickSpawnErrors,
  quickSpawnSubmitting,
  quickSpawnNotice,
  activeTemplate,
  templates,
  focusSignal,
  onFiltersChange,
  onSelectAgent,
  onDraftChange,
  onTemplateSelect,
  onSubmitQuickSpawn,
  onDismissQuickSpawnNotice,
}: CommandWorkspaceProps) {
  const filteredAgents = useMemo(() => {
    const query = filters.search.trim().toLowerCase();

    return agents.filter((agent) => {
      const matchesSearch =
        query.length === 0 ||
        agent.name.toLowerCase().includes(query) ||
        (agent.responsibility || '').toLowerCase().includes(query);
      const matchesStatus = filters.status === 'all' || agent.status === filters.status;
      const matchesRole = filters.role === 'all' || agent.role === filters.role;
      const matchesRuntime = filters.runtime === 'all' || agent.runtime === filters.runtime;

      return matchesSearch && matchesStatus && matchesRole && matchesRuntime;
    });
  }, [agents, filters]);

  const selectedAgent = useMemo(
    () => agents.find((agent) => agent.id === selectedAgentId) || null,
    [agents, selectedAgentId],
  );

  const openQuickSpawn = () => {
    const panel = document.getElementById('quick-spawn');
    panel?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  };

  return (
    <div className="space-y-4">
      <CommandToolbar
        filters={filters}
        onFiltersChange={onFiltersChange}
        onOpenQuickSpawn={openQuickSpawn}
      />

      <section className="grid gap-4 xl:grid-cols-[1.45fr_0.95fr]">
        <AgentListView
          agents={filteredAgents}
          selectedAgentId={selectedAgentId}
          onSelectAgent={onSelectAgent}
          loading={loading}
        />

        <div className="space-y-4">
          <AgentDetailPanel agent={selectedAgent} deployments={deployments} teams={teams} />
          <QuickSpawnPanel
            draft={quickSpawnDraft}
            errors={quickSpawnErrors}
            submitting={quickSpawnSubmitting}
            notice={quickSpawnNotice}
            activeTemplate={activeTemplate}
            templates={templates}
            focusSignal={focusSignal}
            onDraftChange={onDraftChange}
            onTemplateSelect={onTemplateSelect}
            onSubmit={onSubmitQuickSpawn}
            onDismissNotice={onDismissQuickSpawnNotice}
          />
        </div>
      </section>
    </div>
  );
}
