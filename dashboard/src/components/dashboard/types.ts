import { Agent, CreateAgentRequest } from '@/lib/api';

export type OrchestrationMode = 'single' | 'multi' | 'team' | 'assignment';

export type SpawnTemplateId = 'support' | 'research' | 'ops' | 'custom';

export type FieldErrors = Record<string, string>;

export interface CommandFilters {
  search: string;
  status: 'all' | Agent['status'];
  role: 'all' | Agent['role'];
  runtime: 'all' | Agent['runtime'];
}

export interface TelegramDraft {
  telegramEnabled: boolean;
  telegramBotToken: string;
  telegramDmPolicy: 'pairing' | 'allowlist' | 'open' | 'disabled';
  telegramAllowFrom: string;
  telegramGroupPolicy: 'open' | 'allowlist' | 'disabled';
  telegramGroupAllowFrom: string;
  telegramRequireMention: boolean;
}

export interface QuickSpawnDraft extends TelegramDraft {
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

export interface SpawnTemplate {
  id: SpawnTemplateId;
  label: string;
  description: string;
  defaults: Partial<QuickSpawnDraft> & {
    namePrefix: string;
  };
}

export interface NoticeState {
  tone: 'success' | 'error' | 'info';
  message: string;
}
