'use client';

import { useEffect, useState } from 'react';
import { api, ServerHealthResponse, ServerStatusResponse } from '@/lib/api';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { RefreshCw, Server, Database, CheckCircle2, XCircle } from 'lucide-react';

export function ServerManagement() {
  const [health, setHealth] = useState<ServerHealthResponse | null>(null);
  const [status, setStatus] = useState<ServerStatusResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  const loadData = async () => {
    try {
      setRefreshing(true);
      const [healthData, statusData] = await Promise.all([
        api.getServerHealth(),
        api.getServerStatus(),
      ]);
      setHealth(healthData);
      setStatus(statusData);
    } catch (error) {
      console.error('Failed to load server data:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 30000); // Refresh every 30 seconds
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>System Readiness</CardTitle>
        </CardHeader>
        <CardContent>
          <div>Loading server status...</div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <div>
            <CardTitle className="text-2xl font-bold">System Readiness</CardTitle>
            <CardDescription>Live heartbeat for the API and database</CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={loadData}
            disabled={refreshing}
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${refreshing ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Server className="h-5 w-5" />
                <span className="font-medium">API Status</span>
              </div>
              <Badge variant={status?.status === 'running' ? 'default' : 'destructive'}>
                {status?.status || 'unknown'}
              </Badge>
            </div>

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Database className="h-5 w-5" />
                <span className="font-medium">Database</span>
              </div>
              {status?.database_connected ? (
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                  <span className="text-sm text-muted-foreground">Connected</span>
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  <XCircle className="h-4 w-4 text-red-500" />
                  <span className="text-sm text-muted-foreground">Disconnected</span>
                </div>
              )}
            </div>

            <div className="flex items-center justify-between">
              <span className="font-medium">Version</span>
              <span className="text-sm text-muted-foreground">{status?.version || 'N/A'}</span>
            </div>

            {health && (
              <div className="flex items-center justify-between">
                <span className="font-medium">Uptime</span>
                <span className="text-sm text-muted-foreground">
                  {Math.floor((health.uptime_seconds || 0) / 3600)}h{' '}
                  {Math.floor(((health.uptime_seconds || 0) % 3600) / 60)}m
                </span>
              </div>
            )}

            {status && (
              <div className="flex items-center justify-between">
                <span className="font-medium">Last Updated</span>
                <span className="text-sm text-muted-foreground">
                  {new Date(status.timestamp).toLocaleTimeString()}
                </span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
