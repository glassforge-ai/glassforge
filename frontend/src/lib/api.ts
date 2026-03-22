/**
 * GlassForge REST API client.
 *
 * Endpoints may not exist yet — the UI handles 404/503 gracefully.
 */

const API_BASE = '/api/v1';

// --- Types ---

export interface ScanResult {
  id: string;
  repo_path: string;
  status: string;
  score: number | null;
  finding_count: number | null;
  created_at: string;
  completed_at: string | null;
}

export interface MigrationPlan {
  id: string;
  scan_id: string;
  status: string;
  task_count: number;
  completed_count: number;
  failed_count: number;
  tasks: MigrationTask[];
}

export interface MigrationTask {
  id: string;
  file_path: string;
  priority: string;
  status: string;
  persona_id: string | null;
  finding_count: number;
}

export interface Session {
  id: string;
  agent_id: string;
  directory: string;
  status: string;
  created_at: string;
}

export interface SessionEvent {
  id: string;
  event_type: string;
  data_json: string;
  timestamp: string;
}

export interface VerifyResult {
  before_score: number;
  after_score: number;
  delta: number;
  before_findings: number;
  after_findings: number;
  resolved: number;
}

// --- Helpers ---

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.text();
    let message = body;
    try {
      const j = JSON.parse(body);
      if (j?.error) message = j.error;
    } catch {
      // use body as-is
    }
    throw new ApiError(res.status, message || `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

// --- Scan ---

export async function triggerScan(repoPath: string): Promise<ScanResult> {
  const res = await fetch(`${API_BASE}/scans`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ repo_path: repoPath }),
  });
  return handleResponse<ScanResult>(res);
}

export async function listScans(): Promise<ScanResult[]> {
  const res = await fetch(`${API_BASE}/scans`);
  return handleResponse<ScanResult[]>(res);
}

export async function getScan(id: string): Promise<ScanResult> {
  const res = await fetch(`${API_BASE}/scans/${encodeURIComponent(id)}`);
  return handleResponse<ScanResult>(res);
}

// --- Migration ---

export async function startMigration(scanId: string, dryRun = false): Promise<MigrationPlan> {
  const res = await fetch(`${API_BASE}/migrations`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ scan_id: scanId, dry_run: dryRun }),
  });
  return handleResponse<MigrationPlan>(res);
}

export async function getMigration(id: string): Promise<MigrationPlan> {
  const res = await fetch(`${API_BASE}/migrations/${encodeURIComponent(id)}`);
  return handleResponse<MigrationPlan>(res);
}

// --- Sessions ---

export async function listSessions(): Promise<Session[]> {
  const res = await fetch(`${API_BASE}/sessions`);
  return handleResponse<Session[]>(res);
}

export async function getSessionEvents(id: string): Promise<SessionEvent[]> {
  const res = await fetch(`${API_BASE}/sessions/${encodeURIComponent(id)}/events`);
  return handleResponse<SessionEvent[]>(res);
}

export async function deleteSession(id: string): Promise<void> {
  const res = await fetch(`${API_BASE}/sessions/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  await handleResponse<void>(res);
}

// --- Verify ---

export async function verify(beforeId: string, afterId: string): Promise<VerifyResult> {
  const res = await fetch(`${API_BASE}/verify`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ before_scan_id: beforeId, after_scan_id: afterId }),
  });
  return handleResponse<VerifyResult>(res);
}

// --- WebSocket ---

export function wsUrl(path = '/api/v1/ws'): string {
  const proto = typeof location !== 'undefined' && location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${proto}//${typeof location !== 'undefined' ? location.host : 'localhost'}${path}`;
}
