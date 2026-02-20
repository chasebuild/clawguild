import { TelegramSettings } from '@/lib/api';
import { FieldErrors, TelegramDraft } from './types';

export function parseAllowList(value: string): string[] {
  return value
    .split(',')
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
}

export function validateTelegramDraft(draft: TelegramDraft): FieldErrors {
  const errors: FieldErrors = {};

  if (!draft.telegramEnabled) {
    return errors;
  }

  if (!draft.telegramBotToken.trim()) {
    errors.telegramBotToken = 'Telegram bot token is required when Telegram is enabled.';
  }

  const allowFrom = parseAllowList(draft.telegramAllowFrom);
  const groupAllowFrom = parseAllowList(draft.telegramGroupAllowFrom);

  if (draft.telegramDmPolicy === 'allowlist' && allowFrom.length === 0) {
    errors.telegramAllowFrom = 'DM allowlist is required when DM policy is allowlist.';
  }

  if (draft.telegramDmPolicy === 'open' && !allowFrom.includes('*')) {
    errors.telegramAllowFrom = 'DM allowlist must include "*" when DM policy is open.';
  }

  if (
    draft.telegramGroupPolicy === 'allowlist' &&
    allowFrom.length === 0 &&
    groupAllowFrom.length === 0
  ) {
    errors.telegramGroupAllowFrom = 'Group allowlist is required when group policy is allowlist.';
  }

  return errors;
}

export function buildOpenClawRuntimeConfig(draft: TelegramDraft): Record<string, unknown> {
  const allowFrom = parseAllowList(draft.telegramAllowFrom);
  const groupAllowFrom = parseAllowList(draft.telegramGroupAllowFrom);

  return {
    channels: {
      telegram: {
        enabled: draft.telegramEnabled,
        botToken: draft.telegramBotToken || undefined,
        dmPolicy: draft.telegramDmPolicy,
        allowFrom: allowFrom.length > 0 ? allowFrom : undefined,
        groupPolicy: draft.telegramGroupPolicy,
        groupAllowFrom: groupAllowFrom.length > 0 ? groupAllowFrom : undefined,
        groups: { '*': { requireMention: draft.telegramRequireMention } },
      },
    },
  };
}

export function buildTelegramSettings(draft: TelegramDraft): TelegramSettings | undefined {
  if (!draft.telegramEnabled) {
    return undefined;
  }

  const allowFrom = parseAllowList(draft.telegramAllowFrom);
  const groupAllowFrom = parseAllowList(draft.telegramGroupAllowFrom);

  return {
    enabled: draft.telegramEnabled,
    bot_token: draft.telegramBotToken || undefined,
    dm_policy: draft.telegramDmPolicy,
    allow_from: allowFrom.length > 0 ? allowFrom : undefined,
    group_policy: draft.telegramGroupPolicy,
    group_allow_from: groupAllowFrom.length > 0 ? groupAllowFrom : undefined,
    require_mention: draft.telegramRequireMention,
  };
}
