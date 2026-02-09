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
    model_provider: 'openclaw',
    skills: [],
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      await api.createAgent(formData);
      if (onSuccess) {
        onSuccess();
      }
      // Reset form
      setFormData({
        name: '',
        role: 'slave',
        provider: 'flyio',
        model_provider: 'openclaw',
        skills: [],
      });
    } catch (error) {
      console.error('Failed to create agent:', error);
      alert('Failed to create agent. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 border rounded-lg p-6">
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
          onChange={(e) => setFormData({ ...formData, provider: e.target.value as 'railway' | 'flyio' | 'aws' })}
          className="w-full px-3 py-2 border rounded-md"
        >
          <option value="railway">Railway</option>
          <option value="flyio">Fly.io</option>
          <option value="aws">AWS</option>
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
