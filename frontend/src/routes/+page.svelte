<script lang="ts">
  import { onMount } from 'svelte';
  import { triggerScan, listScans, type ScanResult, ApiError } from '$lib/api';

  let repoPath = $state('');
  let scanning = $state(false);
  let scanError = $state('');
  let currentScan = $state<ScanResult | null>(null);
  let recentScans = $state<ScanResult[]>([]);
  let loadError = $state('');
  let serverDown = $state(false);

  async function doScan() {
    if (!repoPath.trim()) return;
    scanning = true;
    scanError = '';
    currentScan = null;
    serverDown = false;
    try {
      currentScan = await triggerScan(repoPath.trim());
      await loadScans();
    } catch (e) {
      if (e instanceof ApiError && (e.status === 0 || e.status >= 500)) {
        serverDown = true;
        scanError = 'Server not connected. Start the GlassForge backend first.';
      } else {
        scanError = e instanceof Error ? e.message : String(e);
      }
    } finally {
      scanning = false;
    }
  }

  async function loadScans() {
    loadError = '';
    serverDown = false;
    try {
      recentScans = await listScans();
    } catch (e) {
      if (e instanceof TypeError || (e instanceof ApiError && (e.status === 0 || e.status >= 500))) {
        serverDown = true;
        loadError = 'Server not connected';
      } else {
        loadError = e instanceof Error ? e.message : String(e);
      }
    }
  }

  function scoreColor(score: number | null): string {
    if (score === null) return 'var(--text-secondary)';
    if (score >= 70) return 'var(--success)';
    if (score >= 40) return 'var(--warning)';
    return 'var(--danger)';
  }

  function scoreBand(score: number | null): string {
    if (score === null) return 'N/A';
    if (score >= 70) return 'Ready';
    if (score >= 40) return 'Needs Work';
    return 'High Risk';
  }

  function formatDate(iso: string): string {
    try {
      return new Date(iso).toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return iso;
    }
  }

  onMount(loadScans);
</script>

<svelte:head>
  <title>Scan - GlassForge</title>
</svelte:head>

<div class="page">
  <h1>Scan Dashboard</h1>
  <p class="subtitle">Analyze your iOS codebase for iOS 26 readiness.</p>

  <div class="scan-form">
    <input
      type="text"
      bind:value={repoPath}
      placeholder="/path/to/your/ios-project"
      disabled={scanning}
      onkeydown={(e) => { if (e.key === 'Enter') doScan(); }}
    />
    <button onclick={doScan} disabled={scanning || !repoPath.trim()}>
      {#if scanning}
        <span class="spinner"></span>
        Scanning...
      {:else}
        Scan
      {/if}
    </button>
  </div>

  {#if scanError}
    <div class="error-banner">{scanError}</div>
  {/if}

  {#if serverDown && !scanError}
    <div class="info-banner">Server not connected. Start the GlassForge backend to see scan results.</div>
  {/if}

  {#if currentScan}
    <section class="result-card">
      <h2>Scan Result</h2>
      <div class="score-display">
        <div class="score-circle" style="--score-color: {scoreColor(currentScan.score)}">
          <span class="score-value">{currentScan.score ?? '...'}</span>
          <span class="score-label">/100</span>
        </div>
        <div class="score-meta">
          <span class="score-band" style="color: {scoreColor(currentScan.score)}">{scoreBand(currentScan.score)}</span>
          <span class="score-path">{currentScan.repo_path}</span>
          <span class="score-status">Status: {currentScan.status}</span>
        </div>
      </div>

      {#if currentScan.finding_count !== null}
        <div class="findings-summary">
          <div class="finding-item">
            <span class="finding-count">{currentScan.finding_count}</span>
            <span class="finding-label">Total Findings</span>
          </div>
        </div>
      {/if}
    </section>
  {/if}

  <section class="recent-section">
    <h2>Recent Scans</h2>
    {#if loadError && !serverDown}
      <p class="error-text">{loadError}</p>
    {:else if recentScans.length === 0}
      <p class="muted">No scans yet. Enter a repo path above to get started.</p>
    {:else}
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Repo</th>
              <th>Score</th>
              <th>Findings</th>
              <th>Status</th>
              <th>Date</th>
            </tr>
          </thead>
          <tbody>
            {#each recentScans as scan}
              <tr>
                <td class="mono">{scan.repo_path}</td>
                <td>
                  <span style="color: {scoreColor(scan.score)}; font-weight: 600;">
                    {scan.score ?? '--'}
                  </span>
                </td>
                <td>{scan.finding_count ?? '--'}</td>
                <td>
                  <span class="status-badge" class:completed={scan.status === 'completed'} class:running={scan.status === 'running'} class:failed={scan.status === 'failed'}>
                    {scan.status}
                  </span>
                </td>
                <td class="muted">{formatDate(scan.created_at)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>
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
  .scan-form {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }
  .scan-form input {
    flex: 1;
    padding: 0.6rem 0.75rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 0.875rem;
  }
  .scan-form input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .scan-form button {
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
  .scan-form button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .scan-form button:hover:not(:disabled) {
    background: var(--accent-hover);
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
  .info-banner {
    background: rgba(234, 179, 8, 0.1);
    border: 1px solid rgba(234, 179, 8, 0.3);
    color: var(--warning);
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }
  .result-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1.5rem;
  }
  .result-card h2 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 1rem;
  }
  .score-display {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }
  .score-circle {
    width: 100px;
    height: 100px;
    border-radius: 50%;
    border: 4px solid var(--score-color, var(--border));
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .score-value {
    font-size: 1.75rem;
    font-weight: 700;
    line-height: 1;
  }
  .score-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }
  .score-meta {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .score-band {
    font-weight: 600;
    font-size: 1.1rem;
  }
  .score-path {
    font-family: var(--font-mono);
    font-size: 0.85rem;
    color: var(--text-secondary);
  }
  .score-status {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  .findings-summary {
    display: flex;
    gap: 1.5rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }
  .finding-item {
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .finding-count {
    font-size: 1.5rem;
    font-weight: 700;
  }
  .finding-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .recent-section {
    margin-top: 1rem;
  }
  .recent-section h2 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 0.75rem;
  }
  .table-wrap {
    overflow-x: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }
  th {
    text-align: left;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    color: var(--text-secondary);
    font-weight: 500;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }
  tr:hover td {
    background: var(--surface-hover);
  }
  .mono {
    font-family: var(--font-mono);
    font-size: 0.82rem;
  }
  .status-badge {
    font-size: 0.7rem;
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    background: var(--surface);
    color: var(--text-secondary);
  }
  .status-badge.completed {
    background: rgba(34, 197, 94, 0.15);
    color: var(--success);
  }
  .status-badge.running {
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
  }
  .status-badge.failed {
    background: rgba(239, 68, 68, 0.15);
    color: var(--danger);
  }
  .muted {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }
  .error-text {
    color: var(--danger);
    font-size: 0.85rem;
  }
</style>
