'use client';

interface StatusIndicatorProps {
  status: 'pending' | 'deploying' | 'running' | 'stopped' | 'error';
}

export function StatusIndicator({ status }: StatusIndicatorProps) {
  const statusColors = {
    pending: 'bg-yellow-500',
    deploying: 'bg-blue-500',
    running: 'bg-green-500',
    stopped: 'bg-gray-500',
    error: 'bg-red-500',
  };

  return (
    <div className="flex items-center gap-2">
      <div className={`w-3 h-3 rounded-full ${statusColors[status]}`} />
      <span className="text-sm capitalize">{status}</span>
    </div>
  );
}
