<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listScans,
    startMigration,
    wsUrl,
    type ScanResult,
    type MigrationPlan,
    type MigrationTask,
    ApiError,
  } from '$lib/api';

  let scans = $state<ScanResult[]>([]);
  let selectedScanId = $state('');
  let planning = $state(false);
  let executing = $state(false);
  let plan = $state<MigrationPlan | null>(null);
  let error = $state('');
  let serverDown = $state(false);
  let ws = $state<WebSocket | null>(null);

  // Swim-lane columns
  type TaskStatus = 'pending' | 'running' | 'completed' | 'failed';
  const COLUMNS: TaskStatus[] = ['pending', 'running', 'completed', 'failed'];
  const COLUMN_LABELS: Record<TaskStatus, string> = {
    pending: 'Pending',
    running: 'Running',
    completed: 'Completed',
    failed: 'Failed',
  };
  const COLUMN_COLORS: Record<TaskStatus, string> = {
    pending: '#71717a',
    running: '#3b82f6',
    completed: '#22c55e',
    failed: '#ef4444',
  };

  let tasksByStatus = $derived<Record<TaskStatus, MigrationTask[]>>(
    plan
      ? {
          pending: plan.tasks.filter((t) => t.status === 'pending' || t.status === 'queued'),
          running: plan.tasks.filter((t) => t.status === 'running'),
          completed: plan.tasks.filter((t) => t.status === 'completed'),
          failed: plan.tasks.filter((t) => t.status === 'failed'),
        }
      : { pending: [], running: [], completed: [], failed: [] }
  );

  let completedScans = $derived(scans.filter((s) => s.status === 'completed'));

  function priorityColor(priority: string): string {
    switch (priority.toLowerCase()) {
      case 'critical':
        return 'var(--danger)';
      case 'high':
        return '#f97316';
      case 'medium':
        return 'var(--warning)';
      default:
        return 'var(--text-secondary)';
    }
  }

  async function loadScans() {
    error = '';
    serverDown = false;
    try {
      scans = await listScans();
    } catch (e) {
      if (e instanceof TypeError || (e instanceof ApiError && (e.status === 0 || e.status >= 500))) {
        serverDown = true;
      } else {
        error = e instanceof Error ? e.message : String(e);
      }
    }
  }

  async function doPlan() {
    if (!selectedScanId) return;
    planning = true;
    error = '';
    plan = null;
    try {
      plan = await startMigration(selectedScanId, true);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      planning = false;
    }
  }

  async function doExecute() {
    if (!selectedScanId) return;
    executing = true;
    error = '';
    try {
      plan = await startMigration(selectedScanId, false);
      connectWs();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      executing = false;
    }
  }

  function connectWs() {
    if (ws?.readyState === WebSocket.OPEN) return;
    try {
      ws = new WebSocket(wsUrl());
      ws.onmessage = (event) => {
        try {
          const ev = JSON.parse(event.data);
          if (ev.type === 'TaskStatusChanged' && plan && ev.data?.task_id) {
            const task = plan.tasks.find((t) => t.id === ev.data.task_id);
            if (task) {
              task.status = ev.data.status;
              plan.tasks = [...plan.tasks];
              if (ev.data.status === 'completed') plan.completed_count++;
              if (ev.data.status === 'failed') plan.failed_count++;
            }
          }
        } catch {
          // ignore parse errors
        }
      };
      ws.onclose = () => {
        ws = null;
      };
    } catch {
      // ws not available
    }
  }

  onMount(loadScans);
</script>

<svelte:head>
  <title>Migrate - GlassForge</title>
</svelte:head>

<div class="page">
  <h1>Migration Pipeline</h1>
  <p class="subtitle">Plan and execute AI-driven iOS 26 migrations.</p>

  {#if serverDown}
    <div class="info-banner">Server not connected. Start the GlassForge backend first.</div>
  {/if}

  <div class="controls">
    <select bind:value={selectedScanId} disabled={planning || executing}>
      <option value="">Select a completed scan...</option>
      {#each completedScans as scan}
        <option value={scan.id}>{scan.repo_path} (score: {scan.score ?? '?'})</option>
      {/each}
    </select>
    <button onclick={doPlan} disabled={!selectedScanId || planning || executing}>
      {#if planning}
        <span class="spinner"></span> Planning...
      {:else}
        Plan Migration
      {/if}
    </button>
    {#if plan}
      <button class="execute-btn" onclick={doExecute} disabled={executing || plan.status === 'running'}>
        {#if executing}
          <span class="spinner"></span> Executing...
        {:else}
          Execute
        {/if}
      </button>
    {/if}
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if plan}
    <div class="plan-summary">
      <span>Tasks: {plan.task_count}</span>
      <span class="completed-count">Completed: {plan.completed_count}</span>
      {#if plan.failed_count > 0}
        <span class="failed-count">Failed: {plan.failed_count}</span>
      {/if}
      <span class="plan-status">Status: {plan.status}</span>
    </div>

    <div class="swim-lanes">
      {#each COLUMNS as col}
        <div class="lane" style="--lane-color: {COLUMN_COLORS[col]}">
          <div class="lane-header">
            <span>{COLUMN_LABELS[col]}</span>
            <span class="lane-count">{tasksByStatus[col].length}</span>
          </div>
          <div class="lane-body">
            {#each tasksByStatus[col] as task (task.id)}
              <div class="task-card">
                <div class="task-file">{task.file_path}</div>
                <div class="task-meta">
                  <span class="task-priority" style="color: {priorityColor(task.priority)}">{task.priority}</span>
                  <span class="task-findings">{task.finding_count} findings</span>
                  {#if task.persona_id}
                    <span class="task-persona">{task.persona_id}</span>
                  {/if}
                </div>
              </div>
            {:else}
              <span class="muted lane-empty">None</span>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page h1 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0;
  }
  .subtitle {
    color: var(--text-secondary);
    margin: 0.25rem 0 1.5rem;
    font-size: 0.9rem;
  }
  .info-banner {
    background: rgba(234, 179, 8, 0.1);
    border: 1px solid rgba(234, 179, 8, 0.3);
    color: var(--warning);
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }
  .controls {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }
  .controls select {
    flex: 1;
    min-width: 200px;
    padding: 0.6rem 0.75rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-family: inherit;
    font-size: 0.85rem;
  }
  .controls select:focus {
    outline: none;
    border-color: var(--accent);
  }
  .controls button {
    padding: 0.6rem 1.25rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    white-space: nowrap;
  }
  .controls button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .controls button:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .execute-btn {
    background: #3b82f6 !important;
  }
  .execute-btn:hover:not(:disabled) {
    background: #60a5fa !important;
  }
  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #f87171;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }
  .plan-summary {
    display: flex;
    gap: 1.5rem;
    padding: 0.75rem 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }
  .completed-count {
    color: var(--success);
  }
  .failed-count {
    color: var(--danger);
  }
  .plan-status {
    margin-left: auto;
    color: var(--text-secondary);
  }
  .swim-lanes {
    display: flex;
    gap: 0.75rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
  }
  .lane {
    flex: 1;
    min-width: 14rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-top: 3px solid var(--lane-color, var(--border));
    border-radius: 8px;
    display: flex;
    flex-direction: column;
  }
  .lane-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    font-size: 0.85rem;
    font-weight: 600;
  }
  .lane-count {
    font-size: 0.75rem;
    padding: 0.1rem 0.4rem;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
    font-weight: 400;
  }
  .lane-body {
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    max-height: 30rem;
    overflow-y: auto;
  }
  .lane-empty {
    text-align: center;
    padding: 1rem 0;
    font-size: 0.8rem;
  }
  .task-card {
    padding: 0.5rem 0.65rem;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .task-file {
    font-family: var(--font-mono);
    font-size: 0.8rem;
    margin-bottom: 0.25rem;
    word-break: break-all;
  }
  .task-meta {
    display: flex;
    gap: 0.5rem;
    font-size: 0.75rem;
    flex-wrap: wrap;
  }
  .task-priority {
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .task-findings {
    color: var(--text-secondary);
  }
  .task-persona {
    color: var(--accent);
    font-size: 0.7rem;
  }
  .muted {
    color: var(--text-secondary);
  }
</style>
