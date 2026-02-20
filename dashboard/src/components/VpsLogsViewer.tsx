'use client';

import { useCallback, useEffect, useMemo, useState } from 'react';
import { RefreshCw, Terminal } from 'lucide-react';
import { api, DeploymentResponse } from '@/lib/api';

export function VpsLogsViewer() {
  const [deployments, setDeployments] = useState<DeploymentResponse[]>([]);
  const [selectedDeployment, setSelectedDeployment] = useState<string | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingLogs, setLoadingLogs] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [lines, setLines] = useState(100);
  const [filter, setFilter] = useState<'all' | 'errors' | 'warnings'>('all');

  const loadDeployments = useCallback(async () => {
    try {
      setRefreshing(true);
      const data = await api.listDeployments();
      setDeployments(data);
      setSelectedDeployment((previous) => previous || data[0]?.id || null);
    } catch (error) {
      console.error('Failed to load deployments:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, []);

  const loadLogs = useCallback(
    async (deploymentId: string) => {
      if (!deploymentId) return;

      try {
        setLoadingLogs(true);
        const logData = await api.getDeploymentLogs(deploymentId, lines);
        setLogs(logData);
      } catch (error) {
        console.error('Failed to load logs:', error);
        setLogs([`Error loading logs: ${error}`]);
      } finally {
        setLoadingLogs(false);
      }
    },
    [lines],
  );

  useEffect(() => {
    loadDeployments();
  }, [loadDeployments]);

  useEffect(() => {
    if (!selectedDeployment) {
      return;
    }

    void loadLogs(selectedDeployment);
    const interval = setInterval(() => {
      void loadLogs(selectedDeployment);
    }, 10000);

    return () => clearInterval(interval);
  }, [loadLogs, selectedDeployment]);

  const filteredLogs = useMemo(
    () =>
      logs.filter((log) => {
        if (filter === 'errors') return /error|failed|panic|exception/i.test(log);
        if (filter === 'warnings') return /warn|warning/i.test(log);
        return true;
      }),
    [filter, logs],
  );

  if (loading) {
    return (
      <section className="panel-surface p-5">
        <h3 className="text-lg font-semibold text-foreground">Debug Console</h3>
        <p className="mt-2 text-sm text-muted-foreground">Loading deployments...</p>
      </section>
    );
  }

  return (
    <section className="panel-surface space-y-4 p-5">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h3 className="text-lg font-semibold text-foreground">Debug Console</h3>
          <p className="text-sm text-muted-foreground">
            Trace errors and inspect deployment logs in real time.
          </p>
        </div>
        <button
          type="button"
          onClick={() => void loadDeployments()}
          disabled={refreshing}
          className="inline-flex items-center gap-2 rounded-md border border-border bg-panel-row px-3 py-2 text-sm text-foreground hover:bg-panel-hover disabled:opacity-50"
        >
          <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </header>

      <div className="grid gap-3 md:grid-cols-[1.6fr_0.5fr_0.6fr]">
        <label className="space-y-1.5">
          <span className="text-xs uppercase tracking-wide text-muted-foreground">Deployment</span>
          <select
            value={selectedDeployment || ''}
            onChange={(event) => setSelectedDeployment(event.target.value)}
            className="input-dark"
          >
            <option value="">Select deployment</option>
            {deployments.map((deployment) => (
              <option key={deployment.id} value={deployment.id}>
                {deployment.provider} · {deployment.status}
              </option>
            ))}
          </select>
        </label>

        <label className="space-y-1.5">
          <span className="text-xs uppercase tracking-wide text-muted-foreground">Lines</span>
          <select
            value={lines.toString()}
            onChange={(event) => setLines(parseInt(event.target.value, 10))}
            className="input-dark"
          >
            <option value="50">50</option>
            <option value="100">100</option>
            <option value="200">200</option>
            <option value="500">500</option>
          </select>
        </label>

        <label className="space-y-1.5">
          <span className="text-xs uppercase tracking-wide text-muted-foreground">Filter</span>
          <select
            value={filter}
            onChange={(event) => setFilter(event.target.value as typeof filter)}
            className="input-dark"
          >
            <option value="all">All logs</option>
            <option value="errors">Errors</option>
            <option value="warnings">Warnings</option>
          </select>
        </label>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Terminal className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium text-foreground">Live logs</span>
          </div>
          {selectedDeployment ? (
            <button
              type="button"
              onClick={() => void loadLogs(selectedDeployment)}
              disabled={loadingLogs}
              className="inline-flex items-center gap-2 rounded-md border border-border bg-panel-row px-3 py-1.5 text-xs text-foreground hover:bg-panel-hover disabled:opacity-50"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${loadingLogs ? 'animate-spin' : ''}`} />
              Reload
            </button>
          ) : null}
        </div>
        <div className="h-[28rem] overflow-auto rounded-xl border border-border bg-[#090d14] p-4 font-mono text-xs text-[#cbe8d5]">
          {loadingLogs ? (
            <div className="text-muted-foreground">Loading logs...</div>
          ) : filteredLogs.length === 0 ? (
            <div className="text-muted-foreground">No logs available</div>
          ) : (
            filteredLogs.map((log, index) => {
              const isError = /error|failed|panic|exception/i.test(log);
              const isWarn = /warn|warning/i.test(log);
              const colorClass = isError
                ? 'text-rose-300'
                : isWarn
                  ? 'text-amber-200'
                  : 'text-emerald-200';
              return (
                <div key={`${index}-${log.slice(0, 12)}`} className={colorClass}>
                  {log}
                </div>
              );
            })
          )}
        </div>
      </div>

      {deployments.length === 0 ? (
        <div className="rounded-md border border-border bg-panel-row px-4 py-3 text-sm text-muted-foreground">
          No deployments found. Deploy an agent to inspect logs.
        </div>
      ) : null}
    </section>
  );
}
