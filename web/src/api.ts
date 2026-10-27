import { assessLocally, type Assessment, type Proposal } from './policy.ts';

export type ReviewSource = 'rust_api' | 'local_simulation' | 'backend_unavailable';
const configuredApi = (import.meta.env?.VITE_LEAKLENS_API_URL ?? '').replace(/\/$/, '');

/** If no backend is configured, the public education sandbox remains usable. */
export async function reviewWithApi(
  proposal: Proposal,
  origin: string,
  fetcher: typeof fetch = fetch,
): Promise<{ assessment: Assessment; source: ReviewSource }> {
  if (!origin) return { assessment: assessLocally(proposal), source: 'local_simulation' };
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 7000);
  try {
    const response = await fetcher(`${origin}/api/v1/assess`, {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(proposal), signal: controller.signal,
    });
    if (!response.ok) throw new Error(`API returned ${response.status}`);
    const assessment: Assessment = await response.json();
    if (!assessment || !Array.isArray(assessment.findings) || typeof assessment.complete !== 'boolean') throw new Error('Unexpected response');
    return { assessment, source: 'rust_api' };
  } catch {
    return { assessment: assessLocally(proposal), source: 'backend_unavailable' };
  } finally { clearTimeout(timeout); }
}

export function reviewProposal(proposal: Proposal) {
  return reviewWithApi(proposal, configuredApi);
}
