'use client';

interface StatusIndicatorProps {
  status: 'pending' | 'deploying' | 'running' | 'stopped' | 'error';
}

export function StatusIndicator({ status }: StatusIndicatorProps) {
  return (
    <span className={`status-chip status-${status}`}>
      <span className="h-2 w-2 rounded-full bg-current opacity-80" />
      {status.replace('_', ' ')}
    </span>
  );
}
