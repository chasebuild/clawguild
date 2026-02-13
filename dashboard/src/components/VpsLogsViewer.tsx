'use client';

import { useEffect, useState } from 'react';
import { api, DeploymentResponse } from '@/lib/api';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { RefreshCw, Terminal } from 'lucide-react';
import { Badge } from '@/components/ui/badge';

export function VpsLogsViewer() {
  const [deployments, setDeployments] = useState<DeploymentResponse[]>([]);
  const [selectedDeployment, setSelectedDeployment] = useState<string | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingLogs, setLoadingLogs] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [lines, setLines] = useState(100);
  const [filter, setFilter] = useState<'all' | 'errors' | 'warnings'>('all');

  const loadDeployments = async () => {
    try {
      setRefreshing(true);
      const data = await api.listDeployments();
      setDeployments(data);
      if (data.length > 0 && !selectedDeployment) {
        setSelectedDeployment(data[0].id);
      }
    } catch (error) {
      console.error('Failed to load deployments:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  const loadLogs = async (deploymentId: string) => {
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
  };

  useEffect(() => {
    loadDeployments();
  }, []);

  useEffect(() => {
    if (selectedDeployment) {
      loadLogs(selectedDeployment);
      const interval = setInterval(() => loadLogs(selectedDeployment), 10000); // Refresh every 10 seconds
      return () => clearInterval(interval);
    }
  }, [selectedDeployment, lines]);

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'running':
        return 'default';
      case 'creating':
      case 'pending':
        return 'secondary';
      case 'failed':
      case 'stopped':
        return 'destructive';
      default:
        return 'outline';
    }
  };

  const filteredLogs = logs.filter((log) => {
    if (filter === 'errors') {
      return /error|failed|panic|exception/i.test(log);
    }
    if (filter === 'warnings') {
      return /warn|warning/i.test(log);
    }
    return true;
  });

  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Debug Console</CardTitle>
        </CardHeader>
        <CardContent>
          <div>Loading deployments...</div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Debug Console</CardTitle>
            <CardDescription>Trace errors and inspect deployment logs in real time</CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={loadDeployments}
            disabled={refreshing}
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${refreshing ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-4">
          <div className="flex-1">
            <label className="text-sm font-medium mb-2 block">Select Deployment</label>
            <Select
              value={selectedDeployment || ''}
              onValueChange={setSelectedDeployment}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select a deployment" />
              </SelectTrigger>
              <SelectContent>
                {deployments.map((deployment) => (
                  <SelectItem key={deployment.id} value={deployment.id}>
                    <div className="flex items-center gap-2">
                      <span>{deployment.provider}</span>
                      <Badge variant={getStatusColor(deployment.status)} className="text-xs">
                        {deployment.status}
                      </Badge>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="w-32">
            <label className="text-sm font-medium mb-2 block">Lines</label>
            <Select
              value={lines.toString()}
              onValueChange={(v) => setLines(parseInt(v))}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="50">50</SelectItem>
                <SelectItem value="100">100</SelectItem>
                <SelectItem value="200">200</SelectItem>
                <SelectItem value="500">500</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="w-40">
            <label className="text-sm font-medium mb-2 block">Filter</label>
            <Select value={filter} onValueChange={(v) => setFilter(v as typeof filter)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Logs</SelectItem>
                <SelectItem value="errors">Errors</SelectItem>
                <SelectItem value="warnings">Warnings</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {selectedDeployment && (
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Terminal className="h-4 w-4" />
                <span className="text-sm font-medium">Logs</span>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => loadLogs(selectedDeployment)}
                disabled={loadingLogs}
              >
                <RefreshCw className={`h-4 w-4 mr-2 ${loadingLogs ? 'animate-spin' : ''}`} />
                Reload
              </Button>
            </div>
            <div className="bg-slate-950 text-emerald-300 font-mono text-xs p-4 rounded-xl h-96 overflow-auto border border-slate-800">
              {loadingLogs ? (
                <div className="text-gray-500">Loading logs...</div>
              ) : filteredLogs.length === 0 ? (
                <div className="text-gray-500">No logs available</div>
              ) : (
                filteredLogs.map((log, index) => {
                  const isError = /error|failed|panic|exception/i.test(log);
                  const isWarn = /warn|warning/i.test(log);
                  const color = isError
                    ? 'text-rose-300'
                    : isWarn
                      ? 'text-amber-200'
                      : 'text-emerald-300';
                  return (
                    <div key={index} className={`mb-1 ${color}`}>
                      {log}
                    </div>
                  );
                })
              )}
            </div>
          </div>
        )}

        {deployments.length === 0 && (
          <div className="text-center text-muted-foreground py-8">
            No deployments found. Deploy an agent to see logs.
          </div>
        )}
      </CardContent>
    </Card>
  );
}
