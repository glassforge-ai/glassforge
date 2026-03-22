<script lang="ts">
  import { onMount } from 'svelte';
  import { listScans, verify, type ScanResult, type VerifyResult, ApiError } from '$lib/api';

  let scans = $state<ScanResult[]>([]);
  let beforeId = $state('');
  let afterId = $state('');
  let comparing = $state(false);
  let result = $state<VerifyResult | null>(null);
  let error = $state('');
  let serverDown = $state(false);

  let completedScans = $derived(scans.filter((s) => s.status === 'completed'));

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

  async function doCompare() {
    if (!beforeId || !afterId) return;
    comparing = true;
    error = '';
    result = null;
    try {
      result = await verify(beforeId, afterId);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      comparing = false;
    }
  }

  function deltaColor(delta: number): string {
    if (delta > 0) return 'var(--success)';
    if (delta < 0) return 'var(--danger)';
    return 'var(--text-secondary)';
  }

  function deltaSign(delta: number): string {
    if (delta > 0) return `+${delta}`;
    return String(delta);
  }

  onMount(loadScans);
</script>

<svelte:head>
  <title>Verify - GlassForge</title>
</svelte:head>

<div class="page">
  <h1>Verify</h1>
  <p class="subtitle">Compare two scans to measure migration progress.</p>

  {#if serverDown}
    <div class="info-banner">Server not connected. Start the GlassForge backend first.</div>
  {/if}

  <div class="controls">
    <div class="select-group">
      <label for="before-select">Before</label>
      <select id="before-select" bind:value={beforeId} disabled={comparing}>
        <option value="">Select before scan...</option>
        {#each completedScans as scan}
          <option value={scan.id}>{scan.repo_path} (score: {scan.score ?? '?'})</option>
        {/each}
      </select>
    </div>

    <div class="select-group">
      <label for="after-select">After</label>
      <select id="after-select" bind:value={afterId} disabled={comparing}>
        <option value="">Select after scan...</option>
        {#each completedScans as scan}
          <option value={scan.id}>{scan.repo_path} (score: {scan.score ?? '?'})</option>
        {/each}
      </select>
    </div>

    <button onclick={doCompare} disabled={!beforeId || !afterId || comparing}>
      {#if comparing}
        <span class="spinner"></span> Comparing...
      {:else}
        Compare
      {/if}
    </button>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if result}
    <div class="result-grid">
      <div class="result-card">
        <h3>Before</h3>
        <div class="big-number">{result.before_score}</div>
        <span class="card-label">Score</span>
        <div class="card-detail">{result.before_findings} findings</div>
      </div>

      <div class="result-card delta-card">
        <h3>Delta</h3>
        <div class="big-number" style="color: {deltaColor(result.delta)}">
          {deltaSign(result.delta)}
        </div>
        <span class="card-label">Points</span>
        <div class="card-detail" style="color: {deltaColor(-result.resolved)}">
          {result.resolved} findings resolved
        </div>
      </div>

      <div class="result-card">
        <h3>After</h3>
        <div class="big-number">{result.after_score}</div>
        <span class="card-label">Score</span>
        <div class="card-detail">{result.after_findings} findings</div>
      </div>
    </div>

    <div class="verdict" style="color: {deltaColor(result.delta)}">
      {#if result.delta > 0}
        Migration improved readiness by {result.delta} points.
      {:else if result.delta < 0}
        Readiness regressed by {Math.abs(result.delta)} points. Review the migration.
      {:else}
        No change in readiness score.
      {/if}
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
  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #f87171;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }
  .controls {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
    align-items: flex-end;
  }
  .select-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    flex: 1;
    min-width: 200px;
  }
  .select-group label {
    font-size: 0.8rem;
    color: var(--text-secondary);
    font-weight: 500;
  }
  .controls select {
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
    height: fit-content;
  }
  .controls button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .controls button:hover:not(:disabled) {
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
  .result-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }
  .result-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.25rem;
    text-align: center;
  }
  .result-card h3 {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-secondary);
    margin: 0 0 0.5rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .big-number {
    font-size: 2.5rem;
    font-weight: 700;
    line-height: 1;
    margin-bottom: 0.25rem;
  }
  .card-label {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  .card-detail {
    margin-top: 0.5rem;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }
  .delta-card {
    border-color: var(--accent);
    background: rgba(16, 185, 129, 0.04);
  }
  .verdict {
    text-align: center;
    font-size: 1rem;
    font-weight: 600;
    padding: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  @media (max-width: 600px) {
    .result-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
