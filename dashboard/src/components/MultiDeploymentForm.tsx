'use client';

import { useState } from 'react';
import { api, Agent, TelegramSettings } from '@/lib/api';

interface MultiDeploymentFormProps {
  agents: Agent[];
  onSuccess?: () => void;
}

export function MultiDeploymentForm({ agents, onSuccess }: MultiDeploymentFormProps) {
  const [selectedAgents, setSelectedAgents] = useState<string[]>([]);
  const [provider, setProvider] = useState<'railway' | 'flyio' | 'aws'>('flyio');
  const [region, setRegion] = useState('');
  const [telegramEnabled, setTelegramEnabled] = useState(false);
  const [telegramBotToken, setTelegramBotToken] = useState('');
  const [telegramDmPolicy, setTelegramDmPolicy] =
    useState<TelegramSettings['dm_policy']>('pairing');
  const [telegramAllowFrom, setTelegramAllowFrom] = useState('');
  const [telegramGroupPolicy, setTelegramGroupPolicy] =
    useState<TelegramSettings['group_policy']>('allowlist');
  const [telegramGroupAllowFrom, setTelegramGroupAllowFrom] = useState('');
  const [telegramRequireMention, setTelegramRequireMention] = useState(true);
  const [loading, setLoading] = useState(false);

  const parseAllowList = (value: string) =>
    value
      .split(',')
      .map((entry) => entry.trim())
      .filter((entry) => entry.length > 0);

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
    if (telegramEnabled) {
      if (!telegramBotToken) {
        alert('Telegram bot token is required when Telegram is enabled.');
        return;
      }
      const allowFrom = parseAllowList(telegramAllowFrom);
      const groupAllowFrom = parseAllowList(telegramGroupAllowFrom);
      if (telegramDmPolicy === 'allowlist' && allowFrom.length === 0) {
        alert('DM allowlist is required when DM policy is allowlist.');
        return;
      }
      if (telegramDmPolicy === 'open' && !allowFrom.includes('*')) {
        alert('DM allowlist must include "*" when DM policy is open.');
        return;
      }
      if (
        telegramGroupPolicy === 'allowlist' &&
        allowFrom.length === 0 &&
        groupAllowFrom.length === 0
      ) {
        alert('Group allowlist is required when group policy is allowlist.');
        return;
      }
    }
    setLoading(true);
    try {
      const allowFrom = parseAllowList(telegramAllowFrom);
      const groupAllowFrom = parseAllowList(telegramGroupAllowFrom);
      await api.deployAgentsMulti({
        agent_ids: selectedAgents,
        provider,
        region: region || undefined,
        telegram_settings: telegramEnabled
          ? {
              enabled: telegramEnabled,
              bot_token: telegramBotToken || undefined,
              dm_policy: telegramDmPolicy,
              allow_from: allowFrom.length > 0 ? allowFrom : undefined,
              group_policy: telegramGroupPolicy,
              group_allow_from: groupAllowFrom.length > 0 ? groupAllowFrom : undefined,
              require_mention: telegramRequireMention,
            }
          : undefined,
      });
      setSelectedAgents([]);
      setRegion('');
      setTelegramEnabled(false);
      setTelegramBotToken('');
      setTelegramDmPolicy('pairing');
      setTelegramAllowFrom('');
      setTelegramGroupPolicy('allowlist');
      setTelegramGroupAllowFrom('');
      setTelegramRequireMention(true);
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
        <p className="text-sm text-muted-foreground">Launch a shared VPS for multiple agents.</p>
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

      <div className="space-y-3 border rounded-xl p-4">
        <div>
          <label className="flex items-center gap-2 text-sm font-medium">
            <input
              type="checkbox"
              checked={telegramEnabled}
              onChange={(e) => setTelegramEnabled(e.target.checked)}
            />
            Enable Telegram for OpenClaw agents
          </label>
          <p className="text-xs text-muted-foreground mt-1">
            Applies to selected agents using the OpenClaw runtime.
          </p>
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Telegram Bot Token</label>
          <input
            type="password"
            value={telegramBotToken}
            onChange={(e) => setTelegramBotToken(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
            placeholder="123456:ABC-DEF..."
          />
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          <div>
            <label className="block text-sm font-medium mb-1">DM Policy</label>
            <select
              value={telegramDmPolicy ?? 'pairing'}
              onChange={(e) =>
                setTelegramDmPolicy(e.target.value as 'pairing' | 'allowlist' | 'open' | 'disabled')
              }
              className="w-full px-3 py-2 border rounded-md"
            >
              <option value="pairing">Pairing</option>
              <option value="allowlist">Allowlist</option>
              <option value="open">Open</option>
              <option value="disabled">Disabled</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Group Policy</label>
            <select
              value={telegramGroupPolicy ?? 'allowlist'}
              onChange={(e) =>
                setTelegramGroupPolicy(e.target.value as 'open' | 'allowlist' | 'disabled')
              }
              className="w-full px-3 py-2 border rounded-md"
            >
              <option value="allowlist">Allowlist</option>
              <option value="open">Open</option>
              <option value="disabled">Disabled</option>
            </select>
          </div>
        </div>
        {(telegramDmPolicy === 'allowlist' || telegramDmPolicy === 'open') && (
          <div>
            <label className="block text-sm font-medium mb-1">DM Allowlist</label>
            <input
              type="text"
              value={telegramAllowFrom}
              onChange={(e) => setTelegramAllowFrom(e.target.value)}
              className="w-full px-3 py-2 border rounded-md"
              placeholder="telegram:12345, *"
            />
            <p className="text-xs text-muted-foreground mt-1">
              Comma-separated. Use `*` for open DM policy.
            </p>
          </div>
        )}
        {telegramGroupPolicy === 'allowlist' && (
          <div>
            <label className="block text-sm font-medium mb-1">Group Allowlist</label>
            <input
              type="text"
              value={telegramGroupAllowFrom}
              onChange={(e) => setTelegramGroupAllowFrom(e.target.value)}
              className="w-full px-3 py-2 border rounded-md"
              placeholder="telegram:12345"
            />
            <p className="text-xs text-muted-foreground mt-1">
              Leave blank to reuse the DM allowlist.
            </p>
          </div>
        )}
        <div>
          <label className="flex items-center gap-2 text-sm font-medium">
            <input
              type="checkbox"
              checked={telegramRequireMention}
              onChange={(e) => setTelegramRequireMention(e.target.checked)}
            />
            Require @mention in groups
          </label>
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
