'use client';

import { useEffect, useRef, useState } from 'react';
import { TemplateChips } from './TemplateChips';
import { FieldErrors, NoticeState, QuickSpawnDraft, SpawnTemplate, SpawnTemplateId } from './types';

interface QuickSpawnPanelProps {
  draft: QuickSpawnDraft;
  errors: FieldErrors;
  submitting: boolean;
  notice: NoticeState | null;
  activeTemplate: SpawnTemplateId;
  templates: SpawnTemplate[];
  focusSignal: number;
  onDraftChange: (next: QuickSpawnDraft) => void;
  onTemplateSelect: (template: SpawnTemplate) => void;
  onSubmit: () => void;
  onDismissNotice: () => void;
}

export function QuickSpawnPanel({
  draft,
  errors,
  submitting,
  notice,
  activeTemplate,
  templates,
  focusSignal,
  onDraftChange,
  onTemplateSelect,
  onSubmit,
  onDismissNotice,
}: QuickSpawnPanelProps) {
  const [showAdvanced, setShowAdvanced] = useState(false);
  const nameInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    nameInputRef.current?.focus();
  }, [focusSignal]);

  const setField = <T extends keyof QuickSpawnDraft>(field: T, value: QuickSpawnDraft[T]) => {
    onDraftChange({ ...draft, [field]: value });
  };

  return (
    <section id="quick-spawn" className="panel-surface space-y-4 p-5">
      <header>
        <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">No-code spawn</p>
        <h3 className="mt-1 text-lg font-semibold text-foreground">Quick Create Agent</h3>
        <p className="mt-1 text-sm text-muted-foreground">
          Pick a template, adjust fields, and launch immediately.
        </p>
      </header>

      <TemplateChips
        templates={templates}
        activeTemplate={activeTemplate}
        onSelect={onTemplateSelect}
      />

      {notice && (
        <div
          className={
            notice.tone === 'success'
              ? 'rounded-md border border-emerald-300/40 bg-emerald-500/15 px-3 py-2 text-sm text-emerald-100'
              : notice.tone === 'error'
                ? 'rounded-md border border-rose-300/40 bg-rose-500/15 px-3 py-2 text-sm text-rose-100'
                : 'rounded-md border border-border bg-panel-row px-3 py-2 text-sm text-foreground'
          }
        >
          <div className="flex items-center justify-between gap-3">
            <span>{notice.message}</span>
            <button
              type="button"
              onClick={onDismissNotice}
              className="text-xs uppercase tracking-wide text-muted-foreground hover:text-foreground"
            >
              dismiss
            </button>
          </div>
        </div>
      )}

      <form
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
        className="space-y-4"
      >
        <Field label="Agent name" required error={errors.name}>
          <input
            ref={nameInputRef}
            type="text"
            value={draft.name}
            onChange={(event) => setField('name', event.target.value)}
            className="input-dark"
            placeholder="support-agent-01"
            required
          />
        </Field>

        <div className="grid gap-3 md:grid-cols-2">
          <Field label="Responsibility" error={errors.responsibility}>
            <input
              type="text"
              value={draft.responsibility}
              onChange={(event) => setField('responsibility', event.target.value)}
              className="input-dark"
              placeholder="Handle user requests"
            />
          </Field>
          <Field label="Emoji" error={errors.emoji}>
            <input
              type="text"
              value={draft.emoji}
              onChange={(event) => setField('emoji', event.target.value)}
              className="input-dark"
              placeholder="🛟"
            />
          </Field>
        </div>

        <div className="grid gap-3 md:grid-cols-2">
          <Field label="Provider" error={errors.provider}>
            <select
              value={draft.provider}
              onChange={(event) =>
                setField('provider', event.target.value as QuickSpawnDraft['provider'])
              }
              className="input-dark"
            >
              <option value="flyio">Fly.io</option>
              <option value="railway">Railway</option>
              <option value="aws">AWS</option>
            </select>
          </Field>
          <Field label="Runtime" error={errors.runtime}>
            <select
              value={draft.runtime}
              onChange={(event) =>
                setField('runtime', event.target.value as QuickSpawnDraft['runtime'])
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
          <Field label="Model provider" error={errors.model_provider}>
            <select
              value={draft.model_provider}
              onChange={(event) =>
                setField('model_provider', event.target.value as QuickSpawnDraft['model_provider'])
              }
              className="input-dark"
            >
              <option value="openclaw">OpenClaw</option>
              <option value="anthropic">Anthropic</option>
              <option value="openai">OpenAI</option>
              <option value="byom">Bring your own model</option>
            </select>
          </Field>
          <Field label="Model API key" error={errors.model_api_key}>
            <input
              type="password"
              value={draft.model_api_key}
              onChange={(event) => setField('model_api_key', event.target.value)}
              className="input-dark"
              placeholder="sk-..."
            />
          </Field>
        </div>

        <Field label="Model endpoint" error={errors.model_endpoint}>
          <input
            type="text"
            value={draft.model_endpoint}
            onChange={(event) => setField('model_endpoint', event.target.value)}
            className="input-dark"
            placeholder="https://api.example.com/v1"
          />
        </Field>

        <div className="grid gap-3 md:grid-cols-2">
          <Field label="Discord bot token" error={errors.discord_bot_token}>
            <input
              type="password"
              value={draft.discord_bot_token}
              onChange={(event) => setField('discord_bot_token', event.target.value)}
              className="input-dark"
            />
          </Field>
          <Field label="Discord channel ID" error={errors.discord_channel_id}>
            <input
              type="text"
              value={draft.discord_channel_id}
              onChange={(event) => setField('discord_channel_id', event.target.value)}
              className="input-dark"
            />
          </Field>
        </div>

        <div className="space-y-3 rounded-lg border border-border bg-panel-row p-4">
          <label className="flex items-center gap-2 text-sm font-medium text-foreground">
            <input
              type="checkbox"
              checked={draft.telegramEnabled}
              onChange={(event) => setField('telegramEnabled', event.target.checked)}
            />
            Enable Telegram
          </label>
          <div className="grid gap-3 md:grid-cols-2">
            <Field label="Telegram bot token" error={errors.telegramBotToken}>
              <input
                type="password"
                value={draft.telegramBotToken}
                onChange={(event) => setField('telegramBotToken', event.target.value)}
                className="input-dark"
                placeholder="123456:ABC-DEF..."
              />
            </Field>
            <Field label="DM policy" error={errors.telegramDmPolicy}>
              <select
                value={draft.telegramDmPolicy}
                onChange={(event) =>
                  setField(
                    'telegramDmPolicy',
                    event.target.value as QuickSpawnDraft['telegramDmPolicy'],
                  )
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
                onChange={(event) => setField('telegramAllowFrom', event.target.value)}
                className="input-dark"
                placeholder="telegram:12345, *"
              />
            </Field>
            <Field label="Group policy" error={errors.telegramGroupPolicy}>
              <select
                value={draft.telegramGroupPolicy}
                onChange={(event) =>
                  setField(
                    'telegramGroupPolicy',
                    event.target.value as QuickSpawnDraft['telegramGroupPolicy'],
                  )
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
              onChange={(event) => setField('telegramGroupAllowFrom', event.target.value)}
              className="input-dark"
              placeholder="telegram:98765"
            />
          </Field>
          <label className="flex items-center gap-2 text-sm text-muted-foreground">
            <input
              type="checkbox"
              checked={draft.telegramRequireMention}
              onChange={(event) => setField('telegramRequireMention', event.target.checked)}
            />
            Require @mention in groups
          </label>
        </div>

        <button
          type="button"
          onClick={() => setShowAdvanced((prev) => !prev)}
          className="text-sm font-medium text-muted-foreground hover:text-foreground"
        >
          {showAdvanced ? 'Hide advanced fields' : 'Show advanced fields'}
        </button>

        {showAdvanced && (
          <div className="grid gap-3 rounded-lg border border-border bg-panel-row p-4 md:grid-cols-2">
            <Field label="Role" error={errors.role}>
              <select
                value={draft.role}
                onChange={(event) =>
                  setField('role', event.target.value as QuickSpawnDraft['role'])
                }
                className="input-dark"
              >
                <option value="slave">Slave</option>
                <option value="master">Master</option>
              </select>
            </Field>
            <Field label="Region" error={errors.region}>
              <input
                type="text"
                value={draft.region}
                onChange={(event) => setField('region', event.target.value)}
                className="input-dark"
                placeholder="iad"
              />
            </Field>
            <Field label="Team ID" error={errors.team_id}>
              <input
                type="text"
                value={draft.team_id}
                onChange={(event) => setField('team_id', event.target.value)}
                className="input-dark"
              />
            </Field>
            <Field label="Personality" error={errors.personality}>
              <input
                type="text"
                value={draft.personality}
                onChange={(event) => setField('personality', event.target.value)}
                className="input-dark"
                placeholder="Calm and concise"
              />
            </Field>
            <div className="md:col-span-2">
              <Field label="Skills (comma-separated)" error={errors.skills}>
                <input
                  type="text"
                  value={draft.skills}
                  onChange={(event) => setField('skills', event.target.value)}
                  className="input-dark"
                  placeholder="triage, docs, qa"
                />
              </Field>
            </div>
          </div>
        )}

        <button
          type="submit"
          disabled={submitting}
          className="w-full rounded-md border border-emerald-300/50 bg-emerald-500/20 px-4 py-2 text-sm font-medium text-emerald-100 hover:bg-emerald-500/30 disabled:opacity-60"
        >
          {submitting ? 'Creating...' : 'Create and deploy agent'}
        </button>
      </form>
    </section>
  );
}

function Field({
  label,
  error,
  required,
  children,
}: {
  label: string;
  error?: string;
  required?: boolean;
  children: React.ReactNode;
}) {
  return (
    <label className="space-y-1.5">
      <span className="text-xs uppercase tracking-wide text-muted-foreground">
        {label}
        {required ? ' *' : ''}
      </span>
      {children}
      {error ? <span className="text-xs text-rose-300">{error}</span> : null}
    </label>
  );
}
