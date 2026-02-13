'use client';

import { useEffect, useMemo, useState } from 'react';
import { api, Agent, Task } from '@/lib/api';

interface TaskManagerProps {
  agents: Agent[];
}

export function TaskManager({ agents }: TaskManagerProps) {
  const [selectedAgentId, setSelectedAgentId] = useState('');
  const [description, setDescription] = useState('');
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [resultDrafts, setResultDrafts] = useState<Record<string, string>>({});
  const [aggregate, setAggregate] = useState<Record<string, string>>({});

  const selectableAgents = useMemo(() => agents, [agents]);

  const loadTasks = async (agentId: string) => {
    if (!agentId) return;
    setLoading(true);
    try {
      const data = await api.getAgentTasks(agentId);
      setTasks(data);
    } catch (error) {
      console.error('Failed to load tasks:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (selectedAgentId) {
      loadTasks(selectedAgentId);
    } else {
      setTasks([]);
    }
    setAggregate({});
    setResultDrafts({});
  }, [selectedAgentId]);

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!selectedAgentId || !description.trim()) {
      alert('Select an agent and provide a task.');
      return;
    }
    setSubmitting(true);
    try {
      await api.sendTask(selectedAgentId, description.trim());
      setDescription('');
      await loadTasks(selectedAgentId);
    } catch (error) {
      console.error('Failed to send task:', error);
      alert('Failed to send task.');
    } finally {
      setSubmitting(false);
    }
  };

  const updateTaskStatus = async (taskId: string, status: Task['status']) => {
    const result = resultDrafts[taskId];
    try {
      await api.updateTask(taskId, { status, result });
      await loadTasks(selectedAgentId);
    } catch (error) {
      console.error('Failed to update task:', error);
      alert('Failed to update task.');
    }
  };

  const aggregateTask = async (taskId: string) => {
    try {
      const response = await api.aggregateTask(taskId);
      setAggregate((prev) => ({ ...prev, [taskId]: response.aggregated_result }));
    } catch (error) {
      console.error('Failed to aggregate task:', error);
      alert('Failed to aggregate task.');
    }
  };

  return (
    <div className="space-y-6">
      <form onSubmit={handleSubmit} className="space-y-4 border rounded-2xl p-6 bg-card shadow-sm">
        <div>
          <h3 className="text-lg font-semibold">Send Task</h3>
          <p className="text-sm text-muted-foreground">Assign work to an agent.</p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Agent</label>
          <select
            value={selectedAgentId}
            onChange={(e) => setSelectedAgentId(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
          >
            <option value="">Select an agent</option>
            {selectableAgents.map((agent) => (
              <option key={agent.id} value={agent.id}>
                {agent.name} ({agent.role})
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Task Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
            rows={3}
          />
        </div>

        <button
          type="submit"
          disabled={submitting}
          className="w-full px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
        >
          {submitting ? 'Sending...' : 'Send Task'}
        </button>
      </form>

      <div className="border rounded-2xl p-6 bg-card space-y-4 shadow-sm">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">Tasks</h3>
            <p className="text-sm text-muted-foreground">Update status or aggregate results.</p>
          </div>
          <button
            type="button"
            onClick={() => loadTasks(selectedAgentId)}
            className="px-3 py-2 border rounded-md text-sm"
            disabled={!selectedAgentId || loading}
          >
            {loading ? 'Refreshing...' : 'Refresh'}
          </button>
        </div>

        {tasks.length === 0 ? (
          <div className="text-sm text-muted-foreground">No tasks yet.</div>
        ) : (
          <div className="space-y-4">
            {tasks.map((task) => (
              <div key={task.id} className="border rounded-xl p-4 space-y-2 bg-muted/30">
                <div className="flex items-center justify-between gap-2">
                  <div className="font-medium">{task.description}</div>
                  <span className="text-xs uppercase text-muted-foreground">{task.status}</span>
                </div>
                {task.result && (
                  <div className="text-sm text-muted-foreground">Result: {task.result}</div>
                )}
                <div className="flex flex-col gap-2">
                  <input
                    type="text"
                    placeholder="Result (optional)"
                    value={resultDrafts[task.id] || ''}
                    onChange={(e) =>
                      setResultDrafts((prev) => ({ ...prev, [task.id]: e.target.value }))
                    }
                    className="w-full px-3 py-2 border rounded-md text-sm"
                  />
                  <div className="flex flex-wrap gap-2">
                    <button
                      type="button"
                      onClick={() => updateTaskStatus(task.id, 'completed')}
                      className="px-3 py-2 text-sm border rounded-md"
                    >
                      Mark Completed
                    </button>
                    <button
                      type="button"
                      onClick={() => updateTaskStatus(task.id, 'failed')}
                      className="px-3 py-2 text-sm border rounded-md"
                    >
                      Mark Failed
                    </button>
                    <button
                      type="button"
                      onClick={() => aggregateTask(task.id)}
                      className="px-3 py-2 text-sm border rounded-md"
                    >
                      Aggregate
                    </button>
                  </div>
                </div>
                {aggregate[task.id] && (
                  <div className="text-sm text-muted-foreground whitespace-pre-line">
                    {aggregate[task.id]}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
