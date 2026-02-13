'use client';

interface StatusIndicatorProps {
  status: 'pending' | 'deploying' | 'running' | 'stopped' | 'error';
}

export function StatusIndicator({ status }: StatusIndicatorProps) {
  const statusColors = {
    pending: 'bg-amber-400 text-amber-950',
    deploying: 'bg-sky-400 text-sky-950',
    running: 'bg-emerald-400 text-emerald-950',
    stopped: 'bg-slate-300 text-slate-700',
    error: 'bg-rose-500 text-rose-50',
  };

  return (
    <span className={`status-chip ${statusColors[status]}`}>
      <span className="h-2 w-2 rounded-full bg-current opacity-80" />
      {status.replace('_', ' ')}
    </span>
  );
}
