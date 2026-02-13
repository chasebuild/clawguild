'use client';

import { useEffect, useState } from 'react';
import { api, TeamRosterResponse } from '@/lib/api';
import { StatusIndicator } from './StatusIndicator';

interface TeamRosterProps {
  teamId: string;
}

export function TeamRoster({ teamId }: TeamRosterProps) {
  const [roster, setRoster] = useState<TeamRosterResponse | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadRoster();
  }, [teamId]);

  const loadRoster = async () => {
    try {
      const data = await api.getTeamRoster(teamId);
      setRoster(data);
    } catch (error) {
      console.error('Failed to load team roster:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="text-center py-8">Loading team roster...</div>;
  }

  if (!roster) {
    return <div className="text-center py-8 text-muted-foreground">No roster data available</div>;
  }

  return (
    <div className="bg-card border rounded-2xl p-6 shadow-sm">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-semibold">{roster.team_name}</h2>
          <p className="text-sm text-muted-foreground">Team roster and live status</p>
        </div>
      </div>
      <div className="space-y-3">
        {roster.members.map((member) => (
          <div
            key={member.id}
            className="flex items-start gap-3 p-3 rounded-xl border border-transparent hover:border-border hover:bg-muted/30 transition-colors"
          >
            <div className="text-2xl flex-shrink-0 leading-none">{member.emoji}</div>
            <div className="flex-1 min-w-0">
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-semibold text-base">{member.name}</span>
                <span className="text-xs uppercase tracking-wide text-muted-foreground">
                  {member.role}
                </span>
                <div className="ml-auto">
                  <StatusIndicator status={member.status} />
                </div>
              </div>
              <p className="text-sm text-muted-foreground leading-relaxed">{member.responsibility}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
