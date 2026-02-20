'use client';

import { useCallback, useEffect, useState } from 'react';
import { api, TeamRosterResponse } from '@/lib/api';
import { StatusIndicator } from './StatusIndicator';

interface TeamRosterProps {
  teamId: string;
}

export function TeamRoster({ teamId }: TeamRosterProps) {
  const [roster, setRoster] = useState<TeamRosterResponse | null>(null);
  const [loading, setLoading] = useState(true);

  const loadRoster = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.getTeamRoster(teamId);
      setRoster(data);
    } catch (error) {
      console.error('Failed to load team roster:', error);
      setRoster(null);
    } finally {
      setLoading(false);
    }
  }, [teamId]);

  useEffect(() => {
    loadRoster();
  }, [loadRoster]);

  if (loading) {
    return (
      <div className="py-8 text-center text-sm text-muted-foreground">Loading team roster...</div>
    );
  }

  if (!roster) {
    return (
      <div className="py-8 text-center text-sm text-muted-foreground">
        No roster data available.
      </div>
    );
  }

  return (
    <div className="panel-surface p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">{roster.team_name}</h2>
          <p className="text-sm text-muted-foreground">Team roster and live status</p>
        </div>
      </div>
      <div className="space-y-3">
        {roster.members.map((member) => (
          <div
            key={member.id}
            className="flex items-start gap-3 rounded-xl border border-border bg-panel-row p-3 transition-colors hover:bg-panel-hover"
          >
            <div className="text-2xl leading-none">{member.emoji}</div>
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2">
                <span className="text-base font-semibold">{member.name}</span>
                <span className="text-xs uppercase tracking-wide text-muted-foreground">
                  {member.role}
                </span>
                <div className="ml-auto">
                  <StatusIndicator status={member.status} />
                </div>
              </div>
              <p className="text-sm leading-relaxed text-muted-foreground">
                {member.responsibility}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
