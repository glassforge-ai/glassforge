<script lang="ts">
  import { onMount } from 'svelte';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import {
    listSessions,
    getSessionEvents,
    deleteSession,
    type Session,
    type SessionEvent,
    ApiError,
  } from '$lib/api';

  interface OutputBlock {
    kind: 'assistant' | 'tool_use' | 'tool_result' | 'thinking' | 'result';
    content: string;
  }

  function normalizeBlockKind(k: string): OutputBlock['kind'] {
    const s = (k ?? 'assistant').toString().toLowerCase();
    if (s === 'tooluse') return 'tool_use';
    if (s === 'toolresult') return 'tool_result';
    if (s === 'thinking' || s === 'result') return s;
    return 'assistant';
  }

  function renderMarkdown(raw: string): string {
    if (!raw?.trim()) return '';
    const html = marked.parse(raw, { async: false }) as string;
    return DOMPurify.sanitize(html);
  }

  let sessions = $state<Session[]>([]);
  let sessionsError = $state('');
  let serverDown = $state(false);
  let selectedId = $state<string | null>(null);
  let sessionOutput = $state<OutputBlock[]>([]);
  let outputLoading = $state(false);
  let deleting = $state<string | null>(null);

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

  async function loadSessions() {
    sessionsError = '';
    serverDown = false;
    try {
      sessions = await listSessions();
    } catch (e) {
      if (e instanceof TypeError || (e instanceof ApiError && (e.status === 0 || e.status >= 500))) {
        serverDown = true;
        sessionsError = 'Server not connected';
      } else {
        sessionsError = e instanceof Error ? e.message : String(e);
      }
    }
  }

  async function selectSession(id: string) {
    if (selectedId === id) {
      selectedId = null;
      sessionOutput = [];
      return;
    }
    selectedId = id;
    outputLoading = true;
    sessionOutput = [];
    try {
      const events = await getSessionEvents(id);
      const blocks: OutputBlock[] = [];
      for (const ev of events) {
        if (ev.event_type !== 'ProcessOutput') continue;
        let data: Record<string, unknown> = {};
        try {
          data = JSON.parse(ev.data_json);
        } catch {
          continue;
        }
        const kind = normalizeBlockKind((data.kind as string) ?? 'assistant');
        const content = typeof data.content === 'string' ? data.content : String(data.content ?? '');
        if (!content) continue;
        if (blocks.length > 0 && blocks[blocks.length - 1].kind === kind) {
          blocks[blocks.length - 1].content += content;
        } else {
          blocks.push({ kind, content });
        }
      }
      sessionOutput = blocks;
    } catch {
      sessionOutput = [];
    } finally {
      outputLoading = false;
    }
  }

  async function doDelete(id: string) {
    if (!confirm('Delete this session?')) return;
    deleting = id;
    try {
      await deleteSession(id);
      sessions = sessions.filter((s) => s.id !== id);
      if (selectedId === id) {
        selectedId = null;
        sessionOutput = [];
      }
    } catch (e) {
      alert(e instanceof Error ? e.message : String(e));
    } finally {
      deleting = null;
    }
  }

  onMount(loadSessions);
</script>

<svelte:head>
  <title>Sessions - GlassForge</title>
</svelte:head>

<div class="page">
  <h1>Session History</h1>
  <p class="subtitle">View past migration agent sessions.</p>

  {#if serverDown}
    <div class="info-banner">Server not connected. Start the GlassForge backend to see sessions.</div>
  {/if}

  {#if sessionsError && !serverDown}
    <div class="error-banner">{sessionsError}</div>
  {/if}

  {#if sessions.length === 0 && !sessionsError}
    <p class="muted">No sessions yet. Run a migration to create sessions.</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>Agent</th>
            <th>Directory</th>
            <th>Status</th>
            <th>Created</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each sessions as session}
            <tr
              class:selected={selectedId === session.id}
              onclick={() => selectSession(session.id)}
              style="cursor: pointer;"
            >
              <td class="mono">{session.id.slice(0, 8)}...</td>
              <td>{session.agent_id.slice(0, 8)}...</td>
              <td class="mono">{session.directory || '--'}</td>
              <td>
                <span
                  class="status-badge"
                  class:completed={session.status === 'completed'}
                  class:running={session.status === 'running'}
                  class:failed={session.status === 'failed'}
                >
                  {session.status}
                </span>
              </td>
              <td class="muted">{formatDate(session.created_at)}</td>
              <td>
                <button
                  class="delete-btn"
                  onclick={(e) => { e.stopPropagation(); doDelete(session.id); }}
                  disabled={deleting === session.id}
                  title="Delete session"
                >
                  {deleting === session.id ? '...' : 'x'}
                </button>
              </td>
            </tr>

            {#if selectedId === session.id}
              <tr class="detail-row">
                <td colspan="6">
                  <div class="session-detail">
                    <h3>Output</h3>
                    {#if outputLoading}
                      <p class="muted">Loading...</p>
                    {:else if sessionOutput.length === 0}
                      <p class="muted">No output recorded for this session.</p>
                    {:else}
                      {#each sessionOutput as block}
                        {#if block.kind === 'assistant' || block.kind === 'result'}
                          <div class="block-assistant rendered">{@html renderMarkdown(block.content)}</div>
                        {:else if block.kind === 'tool_use'}
                          <details class="block-tool">
                            <summary>Tool Call</summary>
                            <pre><code>{block.content}</code></pre>
                          </details>
                        {:else if block.kind === 'tool_result'}
                          <details class="block-tool result">
                            <summary>Tool Result</summary>
                            <pre><code>{block.content}</code></pre>
                          </details>
                        {:else if block.kind === 'thinking'}
                          <details class="block-thinking">
                            <summary>Thinking...</summary>
                            <pre class="dimmed">{block.content}</pre>
                          </details>
                        {/if}
                      {/each}
                    {/if}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
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
  tr.selected td {
    background: rgba(16, 185, 129, 0.05);
    border-color: rgba(16, 185, 129, 0.2);
  }
  .detail-row td {
    padding: 0;
    background: var(--surface) !important;
  }
  .detail-row:hover td {
    background: var(--surface) !important;
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
  .delete-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0.15rem 0.4rem;
    font-size: 0.75rem;
    font-family: inherit;
  }
  .delete-btn:hover {
    color: var(--danger);
    border-color: var(--danger);
  }
  .delete-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .session-detail {
    padding: 1rem;
  }
  .session-detail h3 {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    font-weight: 600;
  }
  .block-assistant.rendered {
    font-size: 0.875rem;
    line-height: 1.5;
  }
  .block-assistant.rendered :global(h1) { font-size: 1.1rem; margin: 0.75rem 0 0.4rem; }
  .block-assistant.rendered :global(h2) { font-size: 1rem; margin: 0.5rem 0 0.3rem; }
  .block-assistant.rendered :global(p) { margin: 0.4rem 0; }
  .block-assistant.rendered :global(ul),
  .block-assistant.rendered :global(ol) { margin: 0.25rem 0; padding-left: 1.5rem; }
  .block-assistant.rendered :global(pre) {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.5rem;
    overflow-x: auto;
    margin: 0.5rem 0;
    font-size: 0.8rem;
  }
  .block-tool {
    border-left: 3px solid var(--accent);
    margin: 0.5rem 0;
    padding-left: 0.75rem;
  }
  .block-tool.result {
    border-left-color: var(--success);
  }
  .block-tool summary,
  .block-thinking summary {
    cursor: pointer;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  .block-tool pre,
  .block-thinking pre {
    margin: 0.25rem 0 0;
    font-size: 0.8rem;
    overflow-x: auto;
  }
  .block-thinking {
    border-left: 3px solid var(--border);
    margin: 0.5rem 0;
    padding-left: 0.75rem;
  }
  .block-thinking .dimmed {
    color: var(--text-secondary);
  }
  .muted {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }
</style>
