'use client';

import { useMemo, useState } from 'react';
import { api, Agent, DiscordChannels, TelegramSettings } from '@/lib/api';

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

  const selectableAgents = useMemo(() => agents, [agents]);
  const parseAllowList = (value: string) =>
    value
      .split(',')
      .map((entry) => entry.trim())
      .filter((entry) => entry.length > 0);

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
      await api.createTeam({
        name,
        master_id: masterId,
        slave_ids: slaveIds.filter((id) => id !== masterId),
        discord_channel_id: channels.coordination_logs,
        discord_channels: channels,
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
      setName('');
      setMasterId('');
      setSlaveIds([]);
      setChannels(emptyChannels);
      setTelegramEnabled(false);
      setTelegramBotToken('');
      setTelegramDmPolicy('pairing');
      setTelegramAllowFrom('');
      setTelegramGroupPolicy('allowlist');
      setTelegramGroupAllowFrom('');
      setTelegramRequireMention(true);
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
            Applies to team agents using the OpenClaw runtime.
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
        {loading ? 'Creating...' : 'Create Team'}
      </button>
    </form>
  );
}
