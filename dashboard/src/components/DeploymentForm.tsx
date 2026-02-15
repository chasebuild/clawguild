'use client';

import { useState } from 'react';
import { api, CreateAgentRequest } from '@/lib/api';

interface DeploymentFormProps {
  onSuccess?: () => void;
}

export function DeploymentForm({ onSuccess }: DeploymentFormProps) {
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState<CreateAgentRequest>({
    name: '',
    role: 'slave',
    provider: 'flyio',
    runtime: 'openclaw',
    model_provider: 'openclaw',
    skills: [],
  });
  const [telegramEnabled, setTelegramEnabled] = useState(false);
  const [telegramBotToken, setTelegramBotToken] = useState('');
  const [telegramDmPolicy, setTelegramDmPolicy] = useState<
    'pairing' | 'allowlist' | 'open' | 'disabled'
  >('pairing');
  const [telegramAllowFrom, setTelegramAllowFrom] = useState('');
  const [telegramGroupPolicy, setTelegramGroupPolicy] = useState<'open' | 'allowlist' | 'disabled'>(
    'allowlist',
  );
  const [telegramGroupAllowFrom, setTelegramGroupAllowFrom] = useState('');
  const [telegramRequireMention, setTelegramRequireMention] = useState(true);

  const parseAllowList = (value: string) =>
    value
      .split(',')
      .map((entry) => entry.trim())
      .filter((entry) => entry.length > 0);

  const buildOpenClawRuntimeConfig = () => {
    const allowFrom = parseAllowList(telegramAllowFrom);
    const groupAllowFrom = parseAllowList(telegramGroupAllowFrom);
    return {
      channels: {
        telegram: {
          enabled: telegramEnabled,
          botToken: telegramBotToken || undefined,
          dmPolicy: telegramDmPolicy,
          allowFrom: allowFrom.length > 0 ? allowFrom : undefined,
          groupPolicy: telegramGroupPolicy,
          groupAllowFrom: groupAllowFrom.length > 0 ? groupAllowFrom : undefined,
          groups: { '*': { requireMention: telegramRequireMention } },
        },
      },
    };
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.runtime === 'openclaw' && telegramEnabled) {
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
      const payload: CreateAgentRequest = {
        ...formData,
        runtime_config: formData.runtime === 'openclaw' ? buildOpenClawRuntimeConfig() : undefined,
      };

      await api.createAgent(payload);
      if (onSuccess) {
        onSuccess();
      }
      // Reset form
      setFormData({
        name: '',
        role: 'slave',
        provider: 'flyio',
        runtime: 'openclaw',
        model_provider: 'openclaw',
        skills: [],
      });
      setTelegramEnabled(false);
      setTelegramBotToken('');
      setTelegramDmPolicy('pairing');
      setTelegramAllowFrom('');
      setTelegramGroupPolicy('allowlist');
      setTelegramGroupAllowFrom('');
      setTelegramRequireMention(true);
    } catch (error) {
      console.error('Failed to create agent:', error);
      alert('Failed to create agent. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 border rounded-2xl p-6 bg-card shadow-sm">
      <div>
        <h3 className="text-lg font-semibold">Single Agent Deployment</h3>
        <p className="text-sm text-muted-foreground">
          Provision an agent and bind it to a VPS with model credentials.
        </p>
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Agent Name</label>
        <input
          type="text"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          required
          className="w-full px-3 py-2 border rounded-md"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Role</label>
        <select
          value={formData.role}
          onChange={(e) => setFormData({ ...formData, role: e.target.value as 'master' | 'slave' })}
          className="w-full px-3 py-2 border rounded-md"
        >
          <option value="master">Master</option>
          <option value="slave">Slave</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">VPS Provider</label>
        <select
          value={formData.provider}
          onChange={(e) =>
            setFormData({ ...formData, provider: e.target.value as 'railway' | 'flyio' | 'aws' })
          }
          className="w-full px-3 py-2 border rounded-md"
        >
          <option value="railway">Railway</option>
          <option value="flyio">Fly.io</option>
          <option value="aws">AWS</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Runtime</label>
        <select
          value={formData.runtime ?? 'openclaw'}
          onChange={(e) =>
            setFormData({
              ...formData,
              runtime: e.target.value as 'openclaw' | 'zeroclaw' | 'picoclaw' | 'nanoclaw',
            })
          }
          className="w-full px-3 py-2 border rounded-md"
        >
          <option value="openclaw">OpenClaw</option>
          <option value="zeroclaw">ZeroClaw</option>
          <option value="picoclaw">PicoClaw</option>
          <option value="nanoclaw">NanoClaw</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Model Provider</label>
        <select
          value={formData.model_provider}
          onChange={(e) => setFormData({ ...formData, model_provider: e.target.value as any })}
          className="w-full px-3 py-2 border rounded-md"
        >
          <option value="openclaw">OpenClaw</option>
          <option value="anthropic">Anthropic</option>
          <option value="openai">OpenAI</option>
          <option value="byom">Bring Your Own Model</option>
        </select>
      </div>

      {formData.model_provider !== 'openclaw' && (
        <div>
          <label className="block text-sm font-medium mb-1">API Key</label>
          <input
            type="password"
            value={formData.model_api_key || ''}
            onChange={(e) => setFormData({ ...formData, model_api_key: e.target.value })}
            className="w-full px-3 py-2 border rounded-md"
          />
        </div>
      )}

      {formData.runtime === 'openclaw' && (
        <div className="space-y-3 border rounded-xl p-4">
          <div>
            <label className="flex items-center gap-2 text-sm font-medium">
              <input
                type="checkbox"
                checked={telegramEnabled}
                onChange={(e) => setTelegramEnabled(e.target.checked)}
              />
              Enable Telegram
            </label>
            <p className="text-xs text-muted-foreground mt-1">
              Configure DM and group behavior for the OpenClaw Telegram channel.
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
                value={telegramDmPolicy}
                onChange={(e) =>
                  setTelegramDmPolicy(
                    e.target.value as 'pairing' | 'allowlist' | 'open' | 'disabled',
                  )
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
                value={telegramGroupPolicy}
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
      )}

      <div>
        <label className="block text-sm font-medium mb-1">Discord Bot Token</label>
        <input
          type="password"
          value={formData.discord_bot_token || ''}
          onChange={(e) => setFormData({ ...formData, discord_bot_token: e.target.value })}
          className="w-full px-3 py-2 border rounded-md"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Discord Channel ID</label>
        <input
          type="text"
          value={formData.discord_channel_id || ''}
          onChange={(e) => setFormData({ ...formData, discord_channel_id: e.target.value })}
          className="w-full px-3 py-2 border rounded-md"
        />
      </div>

      <button
        type="submit"
        disabled={loading}
        className="w-full px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
      >
        {loading ? 'Deploying...' : 'Deploy Agent'}
      </button>
    </form>
  );
}
