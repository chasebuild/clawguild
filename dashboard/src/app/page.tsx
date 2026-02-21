'use client';

import { useCallback, useEffect, useMemo, useState } from 'react';
import { TaskManager } from '@/components/TaskManager';
import { VpsLogsViewer } from '@/components/VpsLogsViewer';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  api,
  Agent,
  CreateAgentRequest,
  DeploymentResponse,
  ServerHealthResponse,
  ServerStatusResponse,
  Team,
} from '@/lib/api';
import { CommandWorkspace } from '@/components/dashboard/CommandWorkspace';
import { buildOpenClawRuntimeConfig, validateTelegramDraft } from '@/components/dashboard/telegram';
import {
  CommandFilters,
  FieldErrors,
  NoticeState,
  OrchestrationMode,
  QuickSpawnDraft,
  SpawnTemplate,
  SpawnTemplateId,
} from '@/components/dashboard/types';
import { OrchestrationBuilder } from '@/components/dashboard/OrchestrationBuilder';

const SUPPORTED_PROVIDER: CreateAgentRequest['provider'] = 'railway';

const spawnTemplates: SpawnTemplate[] = [
  {
    id: 'support',
    label: 'Support',
    description: 'Handle triage and customer-facing responses.',
    defaults: {
      namePrefix: 'support-agent',
      responsibility: 'Handle inbound support requests and issue triage',
      emoji: '🛟',
      personality: 'Helpful, calm, and concise',
      skills: 'triage,customer-support',
    },
  },
  {
    id: 'research',
    label: 'Research',
    description: 'Summarize information and validate sources.',
    defaults: {
      namePrefix: 'research-agent',
      responsibility: 'Collect references and produce concise synthesis',
      emoji: '🔎',
      personality: 'Analytical and detail-oriented',
      skills: 'research,synthesis',
    },
  },
  {
    id: 'ops',
    label: 'Ops',
    description: 'Automate deployment and maintenance workflows.',
    defaults: {
      namePrefix: 'ops-agent',
      responsibility: 'Automate runbooks and monitor deployment health',
      emoji: '🛠️',
      personality: 'Pragmatic and action-driven',
      skills: 'deployment,monitoring,automation',
    },
  },
  {
    id: 'custom',
    label: 'Custom',
    description: 'Start from a blank custom profile.',
    defaults: {
      namePrefix: 'custom-agent',
      responsibility: '',
      emoji: '🧭',
      personality: '',
      skills: '',
    },
  },
];

const defaultFilters: CommandFilters = {
  search: '',
  status: 'all',
  role: 'all',
  runtime: 'all',
};

export default function Home() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [teams, setTeams] = useState<Team[]>([]);
  const [deployments, setDeployments] = useState<DeploymentResponse[]>([]);
  const [serverStatus, setServerStatus] = useState<ServerStatusResponse | null>(null);
  const [serverHealth, setServerHealth] = useState<ServerHealthResponse | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [filters, setFilters] = useState<CommandFilters>(defaultFilters);
  const [activeTemplate, setActiveTemplate] = useState<SpawnTemplateId>('support');
  const [quickSpawnDraft, setQuickSpawnDraft] = useState<QuickSpawnDraft>(() =>
    createQuickSpawnDraft(spawnTemplates[0]),
  );
  const [quickSpawnErrors, setQuickSpawnErrors] = useState<FieldErrors>({});
  const [quickSpawnSubmitting, setQuickSpawnSubmitting] = useState(false);
  const [quickSpawnNotice, setQuickSpawnNotice] = useState<NoticeState | null>(null);
  const [quickSpawnFocusSignal, setQuickSpawnFocusSignal] = useState(0);
  const [orchestrationMode, setOrchestrationMode] = useState<OrchestrationMode>('single');
  const [railwayApiKey, setRailwayApiKey] = useState('');
  const [loading, setLoading] = useState(true);

  const loadData = useCallback(async () => {
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
      setSelectedAgentId((previous) => previous || agentsData[0]?.id || null);
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const systemStats = useMemo(() => {
    const runningAgents = agents.filter((agent) => agent.status === 'running').length;
    const errorAgents = agents.filter((agent) => agent.status === 'error').length;
    const activeDeployments = deployments.filter(
      (deployment) => deployment.status.toLowerCase() === 'running',
    ).length;

    return {
      runningAgents,
      errorAgents,
      activeDeployments,
      uptimeHours: serverHealth ? Math.floor(serverHealth.uptime_seconds / 3600) : 0,
    };
  }, [agents, deployments, serverHealth]);
  const hasRailwayApiKey = railwayApiKey.trim().length > 0;

  const handleTemplateSelect = (template: SpawnTemplate) => {
    setActiveTemplate(template.id);
    const { namePrefix, ...defaults } = template.defaults;
    setQuickSpawnDraft((previous) => ({
      ...previous,
      ...defaults,
      name: namePrefix,
    }));
    setQuickSpawnFocusSignal((previous) => previous + 1);
  };

  const handleQuickSpawnSubmit = async () => {
    const errors = validateQuickSpawn(quickSpawnDraft, hasRailwayApiKey);
    setQuickSpawnErrors(errors);

    if (Object.keys(errors).length > 0) {
      setQuickSpawnNotice({
        tone: 'error',
        message: 'Fix the highlighted fields before creating.',
      });
      return;
    }

    const payload: CreateAgentRequest = {
      name: quickSpawnDraft.name.trim(),
      role: quickSpawnDraft.role,
      provider: SUPPORTED_PROVIDER,
      railway_api_key: railwayApiKey.trim() || undefined,
      region:
        SUPPORTED_PROVIDER === 'railway' ? undefined : quickSpawnDraft.region.trim() || undefined,
      team_id: quickSpawnDraft.team_id.trim() || undefined,
      runtime: quickSpawnDraft.runtime,
      model_provider: quickSpawnDraft.model_provider,
      model_api_key: quickSpawnDraft.model_api_key.trim() || undefined,
      model_endpoint: quickSpawnDraft.model_endpoint.trim() || undefined,
      personality: quickSpawnDraft.personality.trim() || undefined,
      skills: quickSpawnDraft.skills
        .split(',')
        .map((entry) => entry.trim())
        .filter(Boolean),
      discord_bot_token: quickSpawnDraft.discord_bot_token.trim() || undefined,
      discord_channel_id: quickSpawnDraft.discord_channel_id.trim() || undefined,
      responsibility: quickSpawnDraft.responsibility.trim() || undefined,
      emoji: quickSpawnDraft.emoji.trim() || undefined,
      runtime_config:
        quickSpawnDraft.runtime === 'openclaw'
          ? buildOpenClawRuntimeConfig(quickSpawnDraft)
          : undefined,
    };

    setQuickSpawnSubmitting(true);

    try {
      const created = await api.createAgent(payload);
      setAgents((previous) => [created, ...previous.filter((agent) => agent.id !== created.id)]);
      setSelectedAgentId(created.id);
      setQuickSpawnDraft((previous) => ({ ...previous, name: '' }));
      setQuickSpawnErrors({});
      setQuickSpawnNotice({
        tone: 'success',
        message: `Created ${created.name}. You can spawn another immediately.`,
      });
      setQuickSpawnFocusSignal((previous) => previous + 1);
      void loadData();
    } catch (error) {
      console.error('Failed to create agent:', error);
      setQuickSpawnNotice({
        tone: 'error',
        message: getErrorMessage(error, 'Failed to create agent. Please try again.'),
      });
    } finally {
      setQuickSpawnSubmitting(false);
    }
  };

  return (
    <main className="app-shell">
      <div className="mx-auto max-w-[1400px] px-4 py-6 md:px-6 md:py-8">
        <header className="panel-surface mb-5 space-y-4 p-5">
          <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
            <div>
              <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">
                Agent orchestration platform
              </p>
              <h1 className="mt-2 text-3xl font-semibold text-foreground md:text-4xl">
                ClawGuild Dashboard
              </h1>
              <p className="mt-2 text-sm text-muted-foreground">
                Concrete control center for provisioning, deployment, and runtime monitoring.
              </p>
            </div>

            <div className="flex flex-wrap gap-2">
              <Badge variant={serverStatus?.status === 'running' ? 'secondary' : 'destructive'}>
                API {serverStatus?.status || 'unknown'}
              </Badge>
              <Badge
                variant={serverStatus?.database_connected ? 'secondary' : 'destructive'}
                className="text-foreground"
              >
                DB {serverStatus?.database_connected ? 'connected' : 'offline'}
              </Badge>
              <Badge
                variant={hasRailwayApiKey ? 'secondary' : 'destructive'}
                className="text-foreground"
              >
                Railway token {hasRailwayApiKey ? 'loaded' : 'missing'}
              </Badge>
            </div>
          </div>

          <div className="grid gap-4 lg:grid-cols-[1.45fr_1fr]">
            <section className="rounded-xl border border-border bg-panel-row p-4">
              <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">
                Mission snapshot
              </p>
              <div className="mt-3 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
                <MetricCard label="Running agents" value={systemStats.runningAgents.toString()} />
                <MetricCard
                  label="Errors"
                  value={systemStats.errorAgents.toString()}
                  tone={systemStats.errorAgents > 0 ? 'danger' : 'default'}
                />
                <MetricCard
                  label="Live deployments"
                  value={systemStats.activeDeployments.toString()}
                />
                <MetricCard label="Uptime (hours)" value={systemStats.uptimeHours.toString()} />
              </div>
            </section>

            <section className="rounded-xl border border-border bg-panel-row p-4">
              <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">
                Deployment Credentials
              </p>
              <h2 className="mt-2 text-base font-semibold text-foreground">Railway API token</h2>
              <p className="mt-1 text-xs text-muted-foreground">
                Required for all Railway deployments. The token is sent from this client when you
                deploy.
              </p>
              <div className="mt-3 space-y-2">
                <input
                  type="password"
                  value={railwayApiKey}
                  onChange={(event) => setRailwayApiKey(event.target.value)}
                  className="input-dark"
                  placeholder="Enter Railway API token"
                />
                <div className="flex items-center justify-between gap-3 text-xs">
                  <span className={hasRailwayApiKey ? 'text-emerald-200' : 'text-rose-300'}>
                    {hasRailwayApiKey
                      ? 'Token loaded for deployments.'
                      : 'Token is required before deploy.'}
                  </span>
                  {hasRailwayApiKey ? (
                    <button
                      type="button"
                      onClick={() => setRailwayApiKey('')}
                      className="text-muted-foreground hover:text-foreground"
                    >
                      Clear
                    </button>
                  ) : null}
                </div>
              </div>
            </section>
          </div>
        </header>

        <Tabs defaultValue="command" className="space-y-4">
          <TabsList className="bg-panel-row h-auto rounded-lg border border-border/80 p-1">
            <TabsTrigger value="command">Command Center</TabsTrigger>
            <TabsTrigger value="orchestration">Orchestration</TabsTrigger>
            <TabsTrigger value="debug">Debug</TabsTrigger>
          </TabsList>

          <TabsContent value="command" className="mt-0">
            <CommandWorkspace
              agents={agents}
              deployments={deployments}
              teams={teams}
              loading={loading}
              filters={filters}
              selectedAgentId={selectedAgentId}
              quickSpawnDraft={quickSpawnDraft}
              quickSpawnErrors={quickSpawnErrors}
              quickSpawnSubmitting={quickSpawnSubmitting}
              quickSpawnNotice={quickSpawnNotice}
              hasRailwayApiKey={hasRailwayApiKey}
              activeTemplate={activeTemplate}
              templates={spawnTemplates}
              focusSignal={quickSpawnFocusSignal}
              onFiltersChange={setFilters}
              onSelectAgent={setSelectedAgentId}
              onDraftChange={setQuickSpawnDraft}
              onTemplateSelect={handleTemplateSelect}
              onSubmitQuickSpawn={handleQuickSpawnSubmit}
              onDismissQuickSpawnNotice={() => setQuickSpawnNotice(null)}
            />
          </TabsContent>

          <TabsContent value="orchestration" className="mt-0 space-y-4">
            <OrchestrationBuilder
              mode={orchestrationMode}
              agents={agents}
              teams={teams}
              railwayApiKey={railwayApiKey}
              onModeChange={setOrchestrationMode}
              onSuccess={loadData}
            />

            <section className="panel-surface p-5">
              <TaskManager agents={agents} />
            </section>
          </TabsContent>

          <TabsContent value="debug" className="mt-0">
            <VpsLogsViewer />
          </TabsContent>
        </Tabs>
      </div>
    </main>
  );
}

function MetricCard({
  label,
  value,
  tone = 'default',
}: {
  label: string;
  value: string;
  tone?: 'default' | 'danger';
}) {
  return (
    <div
      className={
        tone === 'danger'
          ? 'rounded-lg border border-rose-300/35 bg-rose-500/10 px-3 py-2'
          : 'rounded-lg border border-border/80 bg-background/40 px-3 py-2'
      }
    >
      <p className="text-[11px] uppercase tracking-wide text-muted-foreground">{label}</p>
      <p className="mt-1 text-2xl font-semibold text-foreground">{value}</p>
    </div>
  );
}

function createQuickSpawnDraft(template: SpawnTemplate): QuickSpawnDraft {
  return {
    name: template.defaults.namePrefix,
    responsibility: template.defaults.responsibility || '',
    emoji: template.defaults.emoji || '🧭',
    role: 'slave',
    provider: SUPPORTED_PROVIDER,
    region: '',
    team_id: '',
    runtime: 'openclaw',
    model_provider: 'openclaw',
    model_api_key: '',
    model_endpoint: '',
    personality: template.defaults.personality || '',
    skills: template.defaults.skills || '',
    discord_bot_token: '',
    discord_channel_id: '',
    telegramEnabled: false,
    telegramBotToken: '',
    telegramDmPolicy: 'pairing',
    telegramAllowFrom: '',
    telegramGroupPolicy: 'allowlist',
    telegramGroupAllowFrom: '',
    telegramRequireMention: true,
  };
}

function validateQuickSpawn(draft: QuickSpawnDraft, hasRailwayApiKey: boolean): FieldErrors {
  const errors: FieldErrors = {};

  if (!draft.name.trim()) {
    errors.name = 'Agent name is required.';
  }

  if (draft.model_provider !== 'openclaw' && !draft.model_api_key.trim()) {
    errors.model_api_key = 'Model API key is required for non-OpenClaw providers.';
  }

  if (!hasRailwayApiKey) {
    errors.railway_api_key = 'Railway API token is required before deployment.';
  }

  if (draft.runtime === 'openclaw') {
    Object.assign(errors, validateTelegramDraft(draft));
  }

  return errors;
}

function getErrorMessage(error: unknown, fallback: string): string {
  if (error && typeof error === 'object' && 'response' in error) {
    const response = (error as { response?: { data?: unknown } }).response;
    if (response?.data && typeof response.data === 'object' && 'error' in response.data) {
      const value = (response.data as { error?: unknown }).error;
      if (typeof value === 'string' && value.length > 0) {
        return value;
      }
    }
  }

  if (error instanceof Error && error.message) {
    return error.message;
  }

  return fallback;
}
