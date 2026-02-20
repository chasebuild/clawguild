'use client';

import { Agent } from '@/lib/api';
import { CommandFilters } from './types';

interface CommandToolbarProps {
  filters: CommandFilters;
  onFiltersChange: (next: CommandFilters) => void;
  onOpenQuickSpawn: () => void;
}

const statusOptions: Array<{ label: string; value: CommandFilters['status'] }> = [
  { label: 'All status', value: 'all' },
  { label: 'Running', value: 'running' },
  { label: 'Deploying', value: 'deploying' },
  { label: 'Pending', value: 'pending' },
  { label: 'Stopped', value: 'stopped' },
  { label: 'Error', value: 'error' },
];

const roleOptions: Array<{ label: string; value: CommandFilters['role'] }> = [
  { label: 'All roles', value: 'all' },
  { label: 'Master', value: 'master' },
  { label: 'Slave', value: 'slave' },
];

const runtimeOptions: Array<{ label: string; value: CommandFilters['runtime'] }> = [
  { label: 'All runtimes', value: 'all' },
  { label: 'OpenClaw', value: 'openclaw' },
  { label: 'ZeroClaw', value: 'zeroclaw' },
  { label: 'PicoClaw', value: 'picoclaw' },
  { label: 'NanoClaw', value: 'nanoclaw' },
];

export function CommandToolbar({
  filters,
  onFiltersChange,
  onOpenQuickSpawn,
}: CommandToolbarProps) {
  return (
    <section className="panel-surface space-y-4 p-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Views</p>
          <h2 className="mt-1 text-xl font-semibold text-foreground">Agent Command Center</h2>
        </div>
        <button
          type="button"
          onClick={onOpenQuickSpawn}
          className="rounded-md border border-emerald-300/50 bg-emerald-500/20 px-4 py-2 text-sm font-medium text-emerald-100 hover:bg-emerald-500/30"
        >
          + New agent
        </button>
      </div>
      <div className="grid gap-2 md:grid-cols-[1.4fr_1fr_1fr_1fr]">
        <input
          type="text"
          value={filters.search}
          onChange={(event) => onFiltersChange({ ...filters, search: event.target.value })}
          placeholder="Search agent name or responsibility"
          className="h-10 rounded-md border border-border bg-panel-row px-3 text-sm text-foreground placeholder:text-muted-foreground"
        />
        <SelectFilter<CommandFilters['status']>
          label="Status"
          value={filters.status}
          options={statusOptions}
          onChange={(value) => onFiltersChange({ ...filters, status: value })}
        />
        <SelectFilter<CommandFilters['role']>
          label="Role"
          value={filters.role}
          options={roleOptions}
          onChange={(value) => onFiltersChange({ ...filters, role: value })}
        />
        <SelectFilter<CommandFilters['runtime']>
          label="Runtime"
          value={filters.runtime}
          options={runtimeOptions}
          onChange={(value) => onFiltersChange({ ...filters, runtime: value })}
        />
      </div>
    </section>
  );
}

interface SelectFilterProps<T extends string> {
  label: string;
  value: T;
  options: Array<{ label: string; value: T }>;
  onChange: (value: T) => void;
}

function SelectFilter<T extends string>({ label, value, options, onChange }: SelectFilterProps<T>) {
  return (
    <label className="space-y-1">
      <span className="text-xs uppercase tracking-wide text-muted-foreground">{label}</span>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value as T)}
        className="h-10 w-full rounded-md border border-border bg-panel-row px-3 text-sm text-foreground"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}
