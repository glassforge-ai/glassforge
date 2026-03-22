<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { wsUrl } from '$lib/api';

  interface Props {
    children: import('svelte').Snippet;
  }
  let { children }: Props = $props();

  let sidebarOpen = $state(false);
  let theme = $state<'dark' | 'light'>('dark');
  let wsConnected = $state(false);
  let wsRetryCount = $state(0);

  function toggleTheme() {
    theme = theme === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);
  }

  function connectWs() {
    try {
      const ws = new WebSocket(wsUrl());
      ws.onopen = () => {
        wsConnected = true;
        wsRetryCount = 0;
      };
      ws.onclose = () => {
        wsConnected = false;
        const delay = Math.min(1000 * Math.pow(2, wsRetryCount), 30000);
        wsRetryCount++;
        setTimeout(connectWs, delay);
      };
      ws.onerror = () => ws.close();
    } catch {
      wsConnected = false;
      setTimeout(connectWs, 5000);
    }
  }

  interface NavLink {
    href: string;
    text: string;
    icon: string;
  }

  const navLinks: NavLink[] = [
    { href: '/', text: 'Scan', icon: 'search' },
    { href: '/migrate', text: 'Migrate', icon: 'play' },
    { href: '/sessions', text: 'Sessions', icon: 'list' },
    { href: '/verify', text: 'Verify', icon: 'check' },
  ];

  const iconPaths: Record<string, string> = {
    search: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z',
    play: 'M5 3l14 9-14 9V3z',
    list: 'M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01',
    check: 'M5 13l4 4L19 7',
  };

  function isActive(href: string, pathname: string): boolean {
    if (href === '/') return pathname === '/';
    return pathname.startsWith(href);
  }

  $effect(() => {
    // close sidebar on navigation
    const _ = $page.url.pathname;
    sidebarOpen = false;
  });

  onMount(() => {
    const saved = localStorage.getItem('theme');
    if (saved === 'light' || saved === 'dark') {
      theme = saved;
    } else if (window.matchMedia('(prefers-color-scheme: light)').matches) {
      theme = 'light';
    }
    connectWs();
  });
</script>

<button class="hamburger" onclick={() => sidebarOpen = !sidebarOpen} aria-label="Toggle navigation">
  {#if sidebarOpen}
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
  {:else}
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12h18M3 6h18M3 18h18"/></svg>
  {/if}
</button>

{#if sidebarOpen}
  <div class="sidebar-overlay" onclick={() => sidebarOpen = false} role="presentation"></div>
{/if}

<div class="app">
  <aside class="sidebar" class:open={sidebarOpen}>
    <nav class="nav">
      <a class="brand" href="/">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>
        GlassForge
      </a>

      <div class="nav-group">
        {#each navLinks as link}
          <a
            class="link"
            class:active={isActive(link.href, $page.url.pathname)}
            href={link.href}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d={iconPaths[link.icon]} />
            </svg>
            {link.text}
          </a>
        {/each}
      </div>

      <div class="nav-divider"></div>

      <button class="theme-toggle" onclick={toggleTheme} aria-label="Toggle theme">
        {#if theme === 'dark'}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
          <span>Light mode</span>
        {:else}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/></svg>
          <span>Dark mode</span>
        {/if}
      </button>
    </nav>
  </aside>

  <main class="main" id="main-content">
    {@render children()}
  </main>

  <footer class="statusbar" role="status">
    <span>v0.1.0</span>
    <span class="ws-status" class:connected={wsConnected} title={wsConnected ? 'Connected' : 'Disconnected'}>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        {#if wsConnected}
          <path d="M5 12.55a11 11 0 0114.08 0M1.42 9a16 16 0 0121.16 0M8.53 16.11a6 6 0 016.95 0M12 20h.01"/>
        {:else}
          <path d="M1 1l22 22M16.72 11.06A10.94 10.94 0 0119 12.55M5 12.55a10.94 10.94 0 015.17-2.39M10.71 5.05A16 16 0 0122.58 9M1.42 9a15.91 15.91 0 014.7-2.88M8.53 16.11a6 6 0 016.95 0M12 20h.01"/>
        {/if}
      </svg>
    </span>
    <span class="statusbar-note">iOS migration platform</span>
  </footer>
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(:root),
  :global([data-theme='dark']) {
    --bg: #0a0a0a;
    --surface: #141414;
    --surface-hover: #1a1a1a;
    --border: #262626;
    --text: #fafafa;
    --text-secondary: #a1a1aa;
    --accent: #10b981;
    --accent-hover: #34d399;
    --danger: #ef4444;
    --success: #22c55e;
    --warning: #eab308;
    --font-sans: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif;
    --font-mono: 'SF Mono', ui-monospace, 'Fira Code', monospace;
    --transition: 0.15s ease;
  }

  :global([data-theme='light']) {
    --bg: #fafafa;
    --surface: #ffffff;
    --surface-hover: #f4f4f5;
    --border: #e4e4e7;
    --text: #09090b;
    --text-secondary: #71717a;
    --accent: #059669;
    --accent-hover: #10b981;
    --danger: #dc2626;
    --success: #16a34a;
    --warning: #ca8a04;
  }

  :global(body) {
    font-family: var(--font-sans);
    background: var(--bg);
    color: var(--text);
    line-height: 1.6;
    -webkit-font-smoothing: antialiased;
  }

  :global(code) {
    font-family: var(--font-mono);
    font-size: 0.85em;
    background: var(--surface);
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
  }

  :global(pre) {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.75rem;
    overflow-x: auto;
    font-size: 0.85rem;
    font-family: var(--font-mono);
  }

  .app {
    display: grid;
    grid-template-columns: 13rem 1fr;
    grid-template-rows: 1fr auto;
    min-height: 100vh;
  }

  .sidebar {
    grid-row: 1 / -1;
    background: var(--surface);
    border-right: 1px solid var(--border);
    padding: 0.75rem 0;
    display: flex;
    flex-direction: column;
  }

  .nav {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    margin-bottom: 0.5rem;
    font-weight: 700;
    font-size: 1rem;
    color: var(--accent);
    text-decoration: none;
  }

  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .link {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.45rem 1.25rem;
    color: var(--text-secondary);
    text-decoration: none;
    font-size: 0.875rem;
    transition: all var(--transition);
    border-radius: 0;
  }

  .link:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .link.active {
    color: var(--accent);
    background: rgba(16, 185, 129, 0.08);
    font-weight: 500;
  }

  .nav-divider {
    height: 1px;
    background: var(--border);
    margin: 0.5rem 0.75rem;
    opacity: 0.5;
  }

  .theme-toggle {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.4rem 1.25rem;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 0.875rem;
    font-family: inherit;
    cursor: pointer;
    transition: all var(--transition);
  }

  .theme-toggle:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .main {
    padding: 1.5rem 2rem;
    overflow-y: auto;
    max-width: 1200px;
  }

  .statusbar {
    grid-column: 2;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 2rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
    border-top: 1px solid var(--border);
    background: var(--surface);
  }

  .ws-status {
    display: flex;
    align-items: center;
    color: var(--danger);
    opacity: 0.7;
  }

  .ws-status.connected {
    color: var(--success);
  }

  .statusbar-note {
    margin-left: auto;
  }

  .hamburger {
    display: none;
    position: fixed;
    top: 0.75rem;
    left: 0.75rem;
    z-index: 110;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.35rem;
    color: var(--text);
    cursor: pointer;
  }

  .sidebar-overlay {
    display: none;
  }

  @media (max-width: 768px) {
    .app {
      grid-template-columns: 1fr;
    }

    .sidebar {
      position: fixed;
      top: 0;
      left: 0;
      bottom: 0;
      width: 14rem;
      z-index: 100;
      transform: translateX(-100%);
      transition: transform 0.2s ease;
    }

    .sidebar.open {
      transform: translateX(0);
    }

    .hamburger {
      display: flex;
    }

    .sidebar-overlay {
      display: block;
      position: fixed;
      inset: 0;
      z-index: 99;
      background: rgba(0, 0, 0, 0.5);
    }

    .main {
      padding: 1rem;
      padding-top: 3rem;
    }

    .statusbar {
      grid-column: 1;
      padding: 0.35rem 1rem;
    }
  }
</style>
