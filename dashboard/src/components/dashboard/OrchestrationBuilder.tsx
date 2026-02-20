'use client';

import { useMemo, useState } from 'react';
import { api, Agent, CreateAgentRequest, DiscordChannels, Team } from '@/lib/api';
import {
  buildOpenClawRuntimeConfig,
  buildTelegramSettings,
  validateTelegramDraft,
} from './telegram';
import { FieldErrors, NoticeState, OrchestrationMode, TelegramDraft } from './types';

interface OrchestrationBuilderProps {
  mode: OrchestrationMode;
  agents: Agent[];
  teams: Team[];
  onModeChange: (mode: OrchestrationMode) => void;
  onSuccess: () => Promise<void> | void;
}

interface SingleDraft extends TelegramDraft {
  name: string;
  responsibility: string;
  emoji: string;
  role: CreateAgentRequest['role'];
  provider: CreateAgentRequest['provider'];
  region: string;
  team_id: string;
  runtime: NonNullable<CreateAgentRequest['runtime']>;
  model_provider: CreateAgentRequest['model_provider'];
  model_api_key: string;
  model_endpoint: string;
  personality: string;
  skills: string;
  discord_bot_token: string;
  discord_channel_id: string;
}

interface MultiDraft extends TelegramDraft {
  selectedAgents: string[];
  provider: CreateAgentRequest['provider'];
  region: string;
}

interface TeamDraft extends TelegramDraft {
  name: string;
  masterId: string;
  slaveIds: string[];
  channels: DiscordChannels;
}

interface AssignmentDraft {
  teamId: string;
  agentId: string;
  role: 'master' | 'slave';
}

const emptyChannels: DiscordChannels = {
  coordination_logs: '',
  slave_communication: '',
  master_orders: '',
};

const modeOptions: Array<{ label: string; value: OrchestrationMode }> = [
  { label: 'Single Agent', value: 'single' },
  { label: 'Multi Deploy', value: 'multi' },
  { label: 'Create Team', value: 'team' },
  { label: 'Assign Agent', value: 'assignment' },
];

export function OrchestrationBuilder({
  mode,
  agents,
  teams,
  onModeChange,
  onSuccess,
}: OrchestrationBuilderProps) {
  const [singleDraft, setSingleDraft] = useState<SingleDraft>(createSingleDraft);
  const [multiDraft, setMultiDraft] = useState<MultiDraft>(createMultiDraft);
  const [teamDraft, setTeamDraft] = useState<TeamDraft>(createTeamDraft);
  const [assignmentDraft, setAssignmentDraft] = useState<AssignmentDraft>({
    teamId: '',
    agentId: '',
    role: 'slave',
  });

  const [singleErrors, setSingleErrors] = useState<FieldErrors>({});
  const [multiErrors, setMultiErrors] = useState<FieldErrors>({});
  const [teamErrors, setTeamErrors] = useState<FieldErrors>({});
  const [assignmentErrors, setAssignmentErrors] = useState<FieldErrors>({});
  const [notice, setNotice] = useState<NoticeState | null>(null);
  const [loading, setLoading] = useState(false);

  const selectableAgents = useMemo(() => agents, [agents]);
  const selectableTeams = useMemo(() => teams, [teams]);

  const resetNotice = () => setNotice(null);

  const submitSingle = async () => {
    const errors = validateSingle(singleDraft);
    setSingleErrors(errors);
    if (Object.keys(errors).length > 0) {
      setNotice({ tone: 'error', message: 'Fix field errors before deploying.' });
      return;
    }

    const payload: CreateAgentRequest = {
      name: singleDraft.name.trim(),
      role: singleDraft.role,
      provider: singleDraft.provider,
      region: singleDraft.region.trim() || undefined,
      team_id: singleDraft.team_id.trim() || undefined,
      runtime: singleDraft.runtime,
      model_provider: singleDraft.model_provider,
      model_api_key: singleDraft.model_api_key.trim() || undefined,
      model_endpoint: singleDraft.model_endpoint.trim() || undefined,
      personality: singleDraft.personality.trim() || undefined,
      skills: singleDraft.skills
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean),
      discord_bot_token: singleDraft.discord_bot_token.trim() || undefined,
      discord_channel_id: singleDraft.discord_channel_id.trim() || undefined,
      responsibility: singleDraft.responsibility.trim() || undefined,
      emoji: singleDraft.emoji.trim() || undefined,
      runtime_config:
        singleDraft.runtime === 'openclaw' ? buildOpenClawRuntimeConfig(singleDraft) : undefined,
    };

    setLoading(true);
    try {
      await api.createAgent(payload);
      await onSuccess();
      setSingleDraft(createSingleDraft());
      setSingleErrors({});
      setNotice({ tone: 'success', message: 'Agent deployed successfully.' });
    } catch (error) {
      console.error('Failed to create agent:', error);
      setNotice({ tone: 'error', message: 'Failed to deploy agent.' });
    } finally {
      setLoading(false);
    }
  };

  const submitMulti = async () => {
    const errors = validateMulti(multiDraft);
    setMultiErrors(errors);
    if (Object.keys(errors).length > 0) {
      setNotice({ tone: 'error', message: 'Fix field errors before multi-deploy.' });
      return;
    }

    setLoading(true);
    try {
      await api.deployAgentsMulti({
        agent_ids: multiDraft.selectedAgents,
        provider: multiDraft.provider,
        region: multiDraft.region.trim() || undefined,
        telegram_settings: buildTelegramSettings(multiDraft),
      });
      await onSuccess();
      setMultiDraft(createMultiDraft());
      setMultiErrors({});
      setNotice({ tone: 'success', message: 'Multi-agent deployment started.' });
    } catch (error) {
      console.error('Failed to deploy multiple agents:', error);
      setNotice({ tone: 'error', message: 'Failed to deploy selected agents.' });
    } finally {
      setLoading(false);
    }
  };

  const submitTeam = async () => {
    const errors = validateTeam(teamDraft);
    setTeamErrors(errors);
    if (Object.keys(errors).length > 0) {
      setNotice({ tone: 'error', message: 'Fix field errors before creating the team.' });
      return;
    }

    setLoading(true);
    try {
      await api.createTeam({
        name: teamDraft.name.trim(),
        master_id: teamDraft.masterId,
        slave_ids: teamDraft.slaveIds.filter((id) => id !== teamDraft.masterId),
        discord_channel_id: teamDraft.channels.coordination_logs,
        discord_channels: teamDraft.channels,
        telegram_settings: buildTelegramSettings(teamDraft),
      });
      await onSuccess();
      setTeamDraft(createTeamDraft());
      setTeamErrors({});
      setNotice({ tone: 'success', message: 'Team created successfully.' });
    } catch (error) {
      console.error('Failed to create team:', error);
      setNotice({ tone: 'error', message: 'Failed to create team.' });
    } finally {
      setLoading(false);
    }
  };

  const submitAssignment = async () => {
    const errors = validateAssignment(assignmentDraft);
    setAssignmentErrors(errors);
    if (Object.keys(errors).length > 0) {
      setNotice({ tone: 'error', message: 'Select both team and agent first.' });
      return;
    }

    setLoading(true);
    try {
      await api.assignAgentToTeam(assignmentDraft.teamId, {
        agent_id: assignmentDraft.agentId,
        role: assignmentDraft.role,
      });
      await onSuccess();
      setAssignmentDraft({ teamId: '', agentId: '', role: 'slave' });
      setAssignmentErrors({});
      setNotice({ tone: 'success', message: 'Agent assignment updated.' });
    } catch (error) {
      console.error('Failed to assign agent:', error);
      setNotice({ tone: 'error', message: 'Failed to assign agent to team.' });
    } finally {
      setLoading(false);
    }
  };

  return (
    <section className="panel-surface space-y-5 p-5">
      <header>
        <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Orchestration</p>
        <h3 className="mt-1 text-lg font-semibold text-foreground">Unified Builder</h3>
        <p className="mt-1 text-sm text-muted-foreground">
          Manage single deployment, multi deployment, team creation, and assignment from one place.
        </p>
      </header>

      <div className="flex flex-wrap gap-2">
        {modeOptions.map((option) => (
          <button
            key={option.value}
            type="button"
            onClick={() => {
              resetNotice();
              onModeChange(option.value);
            }}
            className={
              option.value === mode
                ? 'rounded-md border border-emerald-300/50 bg-emerald-500/20 px-3 py-1.5 text-xs font-medium text-emerald-100'
                : 'rounded-md border border-border bg-panel-row px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-panel-hover hover:text-foreground'
            }
          >
            {option.label}
          </button>
        ))}
      </div>

      {notice && (
        <div
          className={
            notice.tone === 'success'
              ? 'rounded-md border border-emerald-300/40 bg-emerald-500/15 px-3 py-2 text-sm text-emerald-100'
              : 'rounded-md border border-rose-300/40 bg-rose-500/15 px-3 py-2 text-sm text-rose-100'
          }
        >
          {notice.message}
        </div>
      )}

      {mode === 'single' ? (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            submitSingle();
          }}
          className="space-y-4"
        >
          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Agent name" error={singleErrors.name}>
              <input
                type="text"
                value={singleDraft.name}
                onChange={(event) => setSingleDraft({ ...singleDraft, name: event.target.value })}
                className="input-dark"
                required
              />
            </Field>
            <Field label="Role" error={singleErrors.role}>
              <select
                value={singleDraft.role}
                onChange={(event) =>
                  setSingleDraft({
                    ...singleDraft,
                    role: event.target.value as SingleDraft['role'],
                  })
                }
                className="input-dark"
              >
                <option value="slave">Slave</option>
                <option value="master">Master</option>
              </select>
            </Field>
          </div>
          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Provider" error={singleErrors.provider}>
              <select
                value={singleDraft.provider}
                onChange={(event) =>
                  setSingleDraft({
                    ...singleDraft,
                    provider: event.target.value as SingleDraft['provider'],
                  })
                }
                className="input-dark"
              >
                <option value="flyio">Fly.io</option>
                <option value="railway">Railway</option>
                <option value="aws">AWS</option>
              </select>
            </Field>
            <Field label="Runtime" error={singleErrors.runtime}>
              <select
                value={singleDraft.runtime}
                onChange={(event) =>
                  setSingleDraft({
                    ...singleDraft,
                    runtime: event.target.value as SingleDraft['runtime'],
                  })
                }
                className="input-dark"
              >
                <option value="openclaw">OpenClaw</option>
                <option value="zeroclaw">ZeroClaw</option>
                <option value="picoclaw">PicoClaw</option>
                <option value="nanoclaw">NanoClaw</option>
              </select>
            </Field>
          </div>

          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Model provider" error={singleErrors.model_provider}>
              <select
                value={singleDraft.model_provider}
                onChange={(event) =>
                  setSingleDraft({
                    ...singleDraft,
                    model_provider: event.target.value as SingleDraft['model_provider'],
                  })
                }
                className="input-dark"
              >
                <option value="openclaw">OpenClaw</option>
                <option value="anthropic">Anthropic</option>
                <option value="openai">OpenAI</option>
                <option value="byom">BYOM</option>
              </select>
            </Field>
            <Field label="Model API key" error={singleErrors.model_api_key}>
              <input
                type="password"
                value={singleDraft.model_api_key}
                onChange={(event) =>
                  setSingleDraft({ ...singleDraft, model_api_key: event.target.value })
                }
                className="input-dark"
              />
            </Field>
          </div>

          <Field label="Model endpoint" error={singleErrors.model_endpoint}>
            <input
              type="text"
              value={singleDraft.model_endpoint}
              onChange={(event) =>
                setSingleDraft({ ...singleDraft, model_endpoint: event.target.value })
              }
              className="input-dark"
            />
          </Field>

          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Discord bot token" error={singleErrors.discord_bot_token}>
              <input
                type="password"
                value={singleDraft.discord_bot_token}
                onChange={(event) =>
                  setSingleDraft({ ...singleDraft, discord_bot_token: event.target.value })
                }
                className="input-dark"
              />
            </Field>
            <Field label="Discord channel ID" error={singleErrors.discord_channel_id}>
              <input
                type="text"
                value={singleDraft.discord_channel_id}
                onChange={(event) =>
                  setSingleDraft({ ...singleDraft, discord_channel_id: event.target.value })
                }
                className="input-dark"
              />
            </Field>
          </div>

          <TelegramSettingsFields
            draft={singleDraft}
            errors={singleErrors}
            onChange={(next) => setSingleDraft({ ...singleDraft, ...next })}
          />

          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Region" error={singleErrors.region}>
              <input
                type="text"
                value={singleDraft.region}
                onChange={(event) => setSingleDraft({ ...singleDraft, region: event.target.value })}
                className="input-dark"
              />
            </Field>
            <Field label="Team ID" error={singleErrors.team_id}>
              <input
                type="text"
                value={singleDraft.team_id}
                onChange={(event) =>
                  setSingleDraft({ ...singleDraft, team_id: event.target.value })
                }
                className="input-dark"
              />
            </Field>
          </div>

          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Responsibility" error={singleErrors.responsibility}>
              <input
                type="text"
                value={singleDraft.responsibility}
                onChange={(event) =>
                  setSingleDraft({ ...singleDraft, responsibility: event.target.value })
                }
                className="input-dark"
              />
            </Field>
            <Field label="Emoji" error={singleErrors.emoji}>
              <input
                type="text"
                value={singleDraft.emoji}
                onChange={(event) => setSingleDraft({ ...singleDraft, emoji: event.target.value })}
                className="input-dark"
              />
            </Field>
          </div>

          <Field label="Personality" error={singleErrors.personality}>
            <input
              type="text"
              value={singleDraft.personality}
              onChange={(event) =>
                setSingleDraft({ ...singleDraft, personality: event.target.value })
              }
              className="input-dark"
            />
          </Field>
          <Field label="Skills (comma-separated)" error={singleErrors.skills}>
            <input
              type="text"
              value={singleDraft.skills}
              onChange={(event) => setSingleDraft({ ...singleDraft, skills: event.target.value })}
              className="input-dark"
            />
          </Field>

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-md border border-emerald-300/50 bg-emerald-500/20 px-4 py-2 text-sm font-medium text-emerald-100 hover:bg-emerald-500/30 disabled:opacity-60"
          >
            {loading ? 'Deploying...' : 'Deploy single agent'}
          </button>
        </form>
      ) : null}

      {mode === 'multi' ? (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            submitMulti();
          }}
          className="space-y-4"
        >
          <Field label="Agents" error={multiErrors.selectedAgents}>
            <div className="grid gap-2 rounded-lg border border-border bg-panel-row p-3 md:grid-cols-2">
              {selectableAgents.map((agent) => (
                <label key={agent.id} className="flex items-center gap-2 text-sm text-foreground">
                  <input
                    type="checkbox"
                    checked={multiDraft.selectedAgents.includes(agent.id)}
                    onChange={() =>
                      setMultiDraft((prev) => ({
                        ...prev,
                        selectedAgents: prev.selectedAgents.includes(agent.id)
                          ? prev.selectedAgents.filter((id) => id !== agent.id)
                          : [...prev.selectedAgents, agent.id],
                      }))
                    }
                  />
                  {agent.name}
                </label>
              ))}
              {selectableAgents.length === 0 ? (
                <p className="text-sm text-muted-foreground">No agents available.</p>
              ) : null}
            </div>
          </Field>

          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Provider" error={multiErrors.provider}>
              <select
                value={multiDraft.provider}
                onChange={(event) =>
                  setMultiDraft({
                    ...multiDraft,
                    provider: event.target.value as MultiDraft['provider'],
                  })
                }
                className="input-dark"
              >
                <option value="flyio">Fly.io</option>
                <option value="railway">Railway</option>
                <option value="aws">AWS</option>
              </select>
            </Field>
            <Field label="Region" error={multiErrors.region}>
              <input
                type="text"
                value={multiDraft.region}
                onChange={(event) => setMultiDraft({ ...multiDraft, region: event.target.value })}
                className="input-dark"
              />
            </Field>
          </div>

          <TelegramSettingsFields
            draft={multiDraft}
            errors={multiErrors}
            onChange={(next) => setMultiDraft({ ...multiDraft, ...next })}
          />

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-md border border-emerald-300/50 bg-emerald-500/20 px-4 py-2 text-sm font-medium text-emerald-100 hover:bg-emerald-500/30 disabled:opacity-60"
          >
            {loading ? 'Starting...' : 'Deploy selected agents'}
          </button>
        </form>
      ) : null}

      {mode === 'team' ? (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            submitTeam();
          }}
          className="space-y-4"
        >
          <Field label="Team name" error={teamErrors.name}>
            <input
              type="text"
              value={teamDraft.name}
              onChange={(event) => setTeamDraft({ ...teamDraft, name: event.target.value })}
              className="input-dark"
              required
            />
          </Field>

          <Field label="Master agent" error={teamErrors.masterId}>
            <select
              value={teamDraft.masterId}
              onChange={(event) => setTeamDraft({ ...teamDraft, masterId: event.target.value })}
              className="input-dark"
            >
              <option value="">Select master</option>
              {selectableAgents.map((agent) => (
                <option key={agent.id} value={agent.id}>
                  {agent.name} ({agent.role})
                </option>
              ))}
            </select>
          </Field>

          <Field label="Slave agents" error={teamErrors.slaveIds}>
            <div className="grid gap-2 rounded-lg border border-border bg-panel-row p-3 md:grid-cols-2">
              {selectableAgents.map((agent) => (
                <label key={agent.id} className="flex items-center gap-2 text-sm text-foreground">
                  <input
                    type="checkbox"
                    checked={teamDraft.slaveIds.includes(agent.id)}
                    disabled={teamDraft.masterId === agent.id}
                    onChange={() =>
                      setTeamDraft((prev) => ({
                        ...prev,
                        slaveIds: prev.slaveIds.includes(agent.id)
                          ? prev.slaveIds.filter((id) => id !== agent.id)
                          : [...prev.slaveIds, agent.id],
                      }))
                    }
                  />
                  {agent.name}
                </label>
              ))}
            </div>
          </Field>

          <div className="grid gap-3 md:grid-cols-3">
            <Field label="Coordination logs channel" error={teamErrors.coordination_logs}>
              <input
                type="text"
                value={teamDraft.channels.coordination_logs}
                onChange={(event) =>
                  setTeamDraft({
                    ...teamDraft,
                    channels: { ...teamDraft.channels, coordination_logs: event.target.value },
                  })
                }
                className="input-dark"
                required
              />
            </Field>
            <Field label="Slave communication channel" error={teamErrors.slave_communication}>
              <input
                type="text"
                value={teamDraft.channels.slave_communication}
                onChange={(event) =>
                  setTeamDraft({
                    ...teamDraft,
                    channels: { ...teamDraft.channels, slave_communication: event.target.value },
                  })
                }
                className="input-dark"
                required
              />
            </Field>
            <Field label="Master orders channel" error={teamErrors.master_orders}>
              <input
                type="text"
                value={teamDraft.channels.master_orders}
                onChange={(event) =>
                  setTeamDraft({
                    ...teamDraft,
                    channels: { ...teamDraft.channels, master_orders: event.target.value },
                  })
                }
                className="input-dark"
                required
              />
            </Field>
          </div>

          <TelegramSettingsFields
            draft={teamDraft}
            errors={teamErrors}
            onChange={(next) => setTeamDraft({ ...teamDraft, ...next })}
          />

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-md border border-emerald-300/50 bg-emerald-500/20 px-4 py-2 text-sm font-medium text-emerald-100 hover:bg-emerald-500/30 disabled:opacity-60"
          >
            {loading ? 'Creating...' : 'Create team'}
          </button>
        </form>
      ) : null}

      {mode === 'assignment' ? (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            submitAssignment();
          }}
          className="space-y-4"
        >
          <div className="grid gap-3 md:grid-cols-3">
            <Field label="Team" error={assignmentErrors.teamId}>
              <select
                value={assignmentDraft.teamId}
                onChange={(event) =>
                  setAssignmentDraft({ ...assignmentDraft, teamId: event.target.value })
                }
                className="input-dark"
              >
                <option value="">Select team</option>
                {selectableTeams.map((team) => (
                  <option key={team.id} value={team.id}>
                    {team.name}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Agent" error={assignmentErrors.agentId}>
              <select
                value={assignmentDraft.agentId}
                onChange={(event) =>
                  setAssignmentDraft({ ...assignmentDraft, agentId: event.target.value })
                }
                className="input-dark"
              >
                <option value="">Select agent</option>
                {selectableAgents.map((agent) => (
                  <option key={agent.id} value={agent.id}>
                    {agent.name}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Role" error={assignmentErrors.role}>
              <select
                value={assignmentDraft.role}
                onChange={(event) =>
                  setAssignmentDraft({
                    ...assignmentDraft,
                    role: event.target.value as AssignmentDraft['role'],
                  })
                }
                className="input-dark"
              >
                <option value="master">Master</option>
                <option value="slave">Slave</option>
              </select>
            </Field>
          </div>

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-md border border-emerald-300/50 bg-emerald-500/20 px-4 py-2 text-sm font-medium text-emerald-100 hover:bg-emerald-500/30 disabled:opacity-60"
          >
            {loading ? 'Assigning...' : 'Assign agent'}
          </button>
        </form>
      ) : null}
    </section>
  );
}

function TelegramSettingsFields({
  draft,
  errors,
  onChange,
}: {
  draft: TelegramDraft;
  errors: FieldErrors;
  onChange: (next: Partial<TelegramDraft>) => void;
}) {
  return (
    <div className="space-y-3 rounded-lg border border-border bg-panel-row p-4">
      <label className="flex items-center gap-2 text-sm font-medium text-foreground">
        <input
          type="checkbox"
          checked={draft.telegramEnabled}
          onChange={(event) => onChange({ telegramEnabled: event.target.checked })}
        />
        Enable Telegram
      </label>
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="Telegram bot token" error={errors.telegramBotToken}>
          <input
            type="password"
            value={draft.telegramBotToken}
            onChange={(event) => onChange({ telegramBotToken: event.target.value })}
            className="input-dark"
          />
        </Field>
        <Field label="DM policy" error={errors.telegramDmPolicy}>
          <select
            value={draft.telegramDmPolicy}
            onChange={(event) =>
              onChange({
                telegramDmPolicy: event.target.value as TelegramDraft['telegramDmPolicy'],
              })
            }
            className="input-dark"
          >
            <option value="pairing">Pairing</option>
            <option value="allowlist">Allowlist</option>
            <option value="open">Open</option>
            <option value="disabled">Disabled</option>
          </select>
        </Field>
      </div>
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="DM allowlist" error={errors.telegramAllowFrom}>
          <input
            type="text"
            value={draft.telegramAllowFrom}
            onChange={(event) => onChange({ telegramAllowFrom: event.target.value })}
            className="input-dark"
            placeholder="telegram:12345, *"
          />
        </Field>
        <Field label="Group policy" error={errors.telegramGroupPolicy}>
          <select
            value={draft.telegramGroupPolicy}
            onChange={(event) =>
              onChange({
                telegramGroupPolicy: event.target.value as TelegramDraft['telegramGroupPolicy'],
              })
            }
            className="input-dark"
          >
            <option value="allowlist">Allowlist</option>
            <option value="open">Open</option>
            <option value="disabled">Disabled</option>
          </select>
        </Field>
      </div>
      <Field label="Group allowlist" error={errors.telegramGroupAllowFrom}>
        <input
          type="text"
          value={draft.telegramGroupAllowFrom}
          onChange={(event) => onChange({ telegramGroupAllowFrom: event.target.value })}
          className="input-dark"
        />
      </Field>
      <label className="flex items-center gap-2 text-sm text-muted-foreground">
        <input
          type="checkbox"
          checked={draft.telegramRequireMention}
          onChange={(event) => onChange({ telegramRequireMention: event.target.checked })}
        />
        Require @mention in groups
      </label>
    </div>
  );
}

function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: React.ReactNode;
}) {
  return (
    <label className="space-y-1.5">
      <span className="text-xs uppercase tracking-wide text-muted-foreground">{label}</span>
      {children}
      {error ? <span className="text-xs text-rose-300">{error}</span> : null}
    </label>
  );
}

function createTelegramDraft(): TelegramDraft {
  return {
    telegramEnabled: false,
    telegramBotToken: '',
    telegramDmPolicy: 'pairing',
    telegramAllowFrom: '',
    telegramGroupPolicy: 'allowlist',
    telegramGroupAllowFrom: '',
    telegramRequireMention: true,
  };
}

function createSingleDraft(): SingleDraft {
  return {
    name: '',
    responsibility: '',
    emoji: '',
    role: 'slave',
    provider: 'flyio',
    region: '',
    team_id: '',
    runtime: 'openclaw',
    model_provider: 'openclaw',
    model_api_key: '',
    model_endpoint: '',
    personality: '',
    skills: '',
    discord_bot_token: '',
    discord_channel_id: '',
    ...createTelegramDraft(),
  };
}

function createMultiDraft(): MultiDraft {
  return {
    selectedAgents: [],
    provider: 'flyio',
    region: '',
    ...createTelegramDraft(),
  };
}

function createTeamDraft(): TeamDraft {
  return {
    name: '',
    masterId: '',
    slaveIds: [],
    channels: emptyChannels,
    ...createTelegramDraft(),
  };
}

function validateSingle(draft: SingleDraft): FieldErrors {
  const errors: FieldErrors = {};

  if (!draft.name.trim()) {
    errors.name = 'Agent name is required.';
  }

  if (draft.model_provider !== 'openclaw' && !draft.model_api_key.trim()) {
    errors.model_api_key = 'Model API key is required for non-OpenClaw providers.';
  }

  if (draft.runtime === 'openclaw') {
    Object.assign(errors, validateTelegramDraft(draft));
  }

  return errors;
}

function validateMulti(draft: MultiDraft): FieldErrors {
  const errors: FieldErrors = {};

  if (draft.selectedAgents.length === 0) {
    errors.selectedAgents = 'Select at least one agent.';
  }

  Object.assign(errors, validateTelegramDraft(draft));

  return errors;
}

function validateTeam(draft: TeamDraft): FieldErrors {
  const errors: FieldErrors = {};

  if (!draft.name.trim()) {
    errors.name = 'Team name is required.';
  }

  if (!draft.masterId) {
    errors.masterId = 'Master agent is required.';
  }

  if (!draft.channels.coordination_logs.trim()) {
    errors.coordination_logs = 'Coordination logs channel is required.';
  }

  if (!draft.channels.slave_communication.trim()) {
    errors.slave_communication = 'Slave communication channel is required.';
  }

  if (!draft.channels.master_orders.trim()) {
    errors.master_orders = 'Master orders channel is required.';
  }

  Object.assign(errors, validateTelegramDraft(draft));

  return errors;
}

function validateAssignment(draft: AssignmentDraft): FieldErrors {
  const errors: FieldErrors = {};

  if (!draft.teamId) {
    errors.teamId = 'Team is required.';
  }

  if (!draft.agentId) {
    errors.agentId = 'Agent is required.';
  }

  return errors;
}
