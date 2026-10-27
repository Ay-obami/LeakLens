import './style.css';
import { makeProposal, type Assessment, type Proposal, type Scenario } from './policy.ts';
import { reviewProposal, type ReviewSource } from './api.ts';

const icon = (name: 'arrow' | 'shield' | 'scan' | 'check' | 'info' | 'lock' | 'github' | 'chevron') => ({
  arrow: '<path d="M5 12h14m-6-6 6 6-6 6"/>',
  shield: '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z"/><path d="m9 12 2 2 4-4"/>',
  scan: '<path d="M4 9V5a1 1 0 0 1 1-1h4M15 4h4a1 1 0 0 1 1 1v4M20 15v4a1 1 0 0 1-1 1h-4M9 20H5a1 1 0 0 1-1-1v-4"/><circle cx="12" cy="12" r="3"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
  info: '<circle cx="12" cy="12" r="10"/><path d="M12 11v5m0-8h.01"/>',
  lock: '<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>',
  github: '<path d="M9 19c-4 1-4-2-6-2m12 4v-3.1a2.7 2.7 0 0 0-.8-2.1c2.7-.3 5.5-1.3 5.5-6a4.6 4.6 0 0 0-1.2-3.2 4.2 4.2 0 0 0-.1-3.1S17.4 3.2 15.1 5a13 13 0 0 0-6.2 0C6.6 3.2 5.6 3.5 5.6 3.5a4.2 4.2 0 0 0-.1 3.1 4.6 4.6 0 0 0-1.2 3.2c0 4.7 2.8 5.7 5.5 6a2.7 2.7 0 0 0-.8 2.1V21"/>',
  chevron: '<path d="m9 18 6-6-6-6"/>',
})[name];
const svg = (name: Parameters<typeof icon>[0], cls = '') => `<svg class="${cls}" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">${icon(name)}</svg>`;

type ScenarioInfo = { id: Scenario; name: string; tag: string; heading: string; description: string; flow: [string, string, string] };
const scenarios: ScenarioInfo[] = [
  { id: 'shielded', name: 'Shielded transfer', tag: 'SHIELDED PATH', heading: 'A private payment route', description: 'An Orchard output funded by shielded inputs. The implemented policy finds no public-disclosure signals in these supplied facts.', flow: ['Shielded input', 'Orchard pool', 'Shielded output'] },
  { id: 'transparent', name: 'Transparent payment', tag: 'PUBLIC PATH', heading: 'A payment with a public trail', description: 'Transparent inputs and outputs expose observable transaction history, destination and value.', flow: ['Public input', 'Transparent pool', 'Public output'] },
  { id: 'unified', name: 'Unified address', tag: 'RECEIVER SELECTION', heading: 'One address. Different outcomes.', description: 'A Unified Address can select a transparent receiver. Address format alone does not guarantee shielding.', flow: ['Unified address', 'Transparent receiver', 'Public output'] },
  { id: 'deshield', name: 'Deshielding', tag: 'CORRELATION RISK', heading: 'Where privacy can unwind', description: 'Shielded funds move toward a transparent output. Amount or timing may create a link, but do not prove identity.', flow: ['Shielded input', 'Pool boundary', 'Public output'] },
];
let scenario: Scenario = 'transparent';
let assessment: Assessment | null = null;
let reviewSource: ReviewSource = 'local_simulation';
let pending = false;

const el = document.querySelector<HTMLDivElement>('#app');
if (!el) throw new Error('Missing app root');
const app = el;
app.innerHTML = `
  <div class="site-shell">
    <header class="site-header" id="top">
      <div class="container nav-shell">
        <a href="#top" class="wordmark" aria-label="LeakLens home"><span class="mark">${svg('scan')}</span><span>leak<span class="wordmark-light">lens</span><span class="wordmark-period">.</span></span></a>
        <nav class="top-nav" aria-label="Main navigation"><a href="#how">How it works</a><a href="#lab">Explore the lab</a><a href="#principles">Why LeakLens</a></nav>
        <a href="https://github.com/Ay-obami/LeakLens" target="_blank" rel="noopener noreferrer" class="github-link">${svg('github')}<span>View source</span>${svg('arrow', 'arrow-small')}</a>
      </div>
    </header>

    <main>
      <section class="hero" aria-labelledby="hero-title">
        <div class="container hero-grid">
          <div class="hero-content">
            <div class="eyebrow"><span class="eyebrow-dot"></span> THE PRE-SEND PRIVACY LAYER FOR ZCASH</div>
            <h1 id="hero-title">What if you could <em>see the leak</em> before you send?</h1>
            <p class="hero-description">A privacy-focused transaction review that surfaces public exposure and possible correlation risks <strong>before</strong> you authorize a Zcash payment.</p>
            <div class="hero-actions"><a href="#lab" class="btn btn-primary">Explore privacy risks ${svg('arrow')}</a><a href="#how" class="btn btn-text">How it works ${svg('chevron')}</a></div>
            <div class="hero-foot"><span class="tiny-shield">${svg('shield')}</span><span>Review before authorization</span><span class="hero-dot">·</span><span>No wallet connection required for the sandbox</span></div>
          </div>
          <div class="hero-art" aria-label="Illustration showing a transaction analyzed before broadcast" role="img">
            <div class="orb orb-1"></div><div class="orb orb-2"></div>
            <div class="art-grid"></div>
            <div class="art-node art-node-a"><span class="art-icon">${svg('lock')}</span><span>YOUR WALLET</span></div>
            <div class="art-beam"></div><div class="art-beam-glow"></div>
            <div class="lens-ring lens-ring-outer"><div class="lens-ring lens-ring-inner"><div class="lens-core">${svg('scan')}</div></div></div>
            <div class="art-node art-node-b"><span class="art-icon">${svg('arrow')}</span><span>RECIPIENT</span></div>
            <div class="floating-card"><span class="floating-mark">!</span><span><small>PRIVACY SIGNAL DETECTED</small><strong>Transparent output</strong><span>Recipient & value visible</span></span></div>
            <div class="art-caption">INSPECT THE PATH. UNDERSTAND THE EXPOSURE.</div>
          </div>
        </div>
      </section>

      <div class="trust-strip"><div class="container trust-inner"><span>BUILT FOR THE ZCASH ECOSYSTEM</span><span class="trust-divider"></span><span>REAL POLICY RULES</span><span class="trust-divider"></span><span>OPEN SOURCE</span><span class="trust-divider"></span><span>TESTNET VERIFIED</span></div></div>

      <section id="how" class="how-section section-pad"><div class="container"><div class="section-top"><div><div class="section-label"><span></span> THE IDEA</div><h2>Privacy isn't a switch.<br><span>It's a series of decisions.</span></h2></div><p>A payment can start shielded and still reveal data later. LeakLens brings those decisions into focus at the moment they matter.</p></div>
        <div class="steps-grid">
          <article class="step-card"><span class="step-number">01 /</span><div class="step-icon">${svg('scan')}</div><h3>Inspect the proposal</h3><p>Look at actual output pools, funding inputs, change routes, and the recipient type.</p></article>
          <article class="step-card"><span class="step-number">02 /</span><div class="step-icon">${svg('info')}</div><h3>Make exposure visible</h3><p>Distinguish facts anyone can see on-chain from correlations that are only possible.</p></article>
          <article class="step-card"><span class="step-number">03 /</span><div class="step-icon">${svg('shield')}</div><h3>Decide with context</h3><p>Surface the warning before wallet authorization, not after a transaction is irreversible.</p></article>
        </div>
      </div></section>

      <section id="lab" class="lab-section section-pad" aria-labelledby="lab-title">
        <div class="container">
          <div class="section-label"><span></span> INTERACTIVE SANDBOX</div>
          <div class="lab-heading"><div><h2 id="lab-title">Put privacy<br><span>under the lens.</span></h2><p>Explore how different transaction paths change what's visible on-chain.</p></div><div class="sandbox-label">${svg('info')} PUBLIC SIMULATION · NO REAL FUNDS</div></div>
          <div class="lab-layout">
            <div class="lab-left"><div class="lab-panel-head"><span class="lab-panel-title">01 &nbsp; SELECT A SCENARIO</span><span class="panel-subtitle">Synthetic examples</span></div><div class="scenarios" id="scenarios" role="group" aria-label="Privacy review scenarios"></div>
              <div class="simulation-note">${svg('lock')} <span>These are simplified, user-asserted proposal facts for education. This sandbox neither connects a wallet nor signs or broadcasts transactions.</span></div>
            </div>
            <div class="lab-right">
              <div class="transaction-card">
                <div class="transaction-top"><div><div class="lab-panel-title">02 &nbsp; REVIEW THE ROUTE</div><div class="scenario-tag" id="scenario-tag"></div></div><div class="testnet-pill"><span></span> TESTNET SIMULATION</div></div>
                <h3 id="scenario-heading"></h3><p id="scenario-description"></p>
                <div class="flow-visual"><div class="flow-piece"><span class="flow-index">SOURCE</span><div class="flow-symbol source-symbol">${svg('lock')}</div><span class="flow-text" id="flow-0"></span></div><div class="flow-connector"><span></span>${svg('arrow')}</div><div class="flow-piece"><span class="flow-index">ROUTE</span><div class="flow-symbol middle-symbol">${svg('scan')}</div><span class="flow-text" id="flow-1"></span></div><div class="flow-connector"><span></span>${svg('arrow')}</div><div class="flow-piece"><span class="flow-index">DESTINATION</span><div class="flow-symbol dest-symbol">${svg('arrow')}</div><span class="flow-text" id="flow-2"></span></div></div>
                <button class="btn btn-review" id="run-review" type="button">Run privacy review ${svg('arrow')}</button>
              </div>
              <div class="result-area" id="result-area" aria-live="polite" aria-atomic="true"></div>
            </div>
          </div>
        </div>
      </section>

      <section id="principles" class="principles-section section-pad"><div class="container principles-grid"><div><div class="section-label"><span></span> DESIGNED WITH RESTRAINT</div><h2>Know the difference between <span>evidence and inference.</span></h2><p>Privacy tooling should explain its limits as clearly as its warnings. LeakLens doesn't call a transaction anonymous just because it uses a shielded pool.</p><a href="https://github.com/Ay-obami/LeakLens/blob/main/docs/privacy-model.md" target="_blank" rel="noopener noreferrer" class="inline-link">Read the privacy model ${svg('arrow')}</a></div><div class="principle-list"><div><span class="principle-symbol">${svg('check')}</span><div><strong>Public disclosure</strong><p>What the transparent chain actually reveals.</p></div></div><div><span class="principle-symbol caution">${svg('info')}</span><div><strong>Possible correlation</strong><p>Patterns that could invite analysis, never a proven identity link.</p></div></div><div><span class="principle-symbol unknown">${svg('lock')}</span><div><strong>Incomplete evidence</strong><p>Unknown facts stay unknown. The review never silently passes them.</p></div></div></div></div></section>
    </main>
    <footer class="site-footer"><div class="container footer-main"><div><a href="#top" class="wordmark"><span class="mark">${svg('scan')}</span><span>leak<span class="wordmark-light">lens</span><span class="wordmark-period">.</span></span></a><p>See the risk. Own the decision.</p></div><div class="footer-links"><a href="#lab">Sandbox</a><a href="#how">How it works</a><a href="https://github.com/Ay-obami/LeakLens" target="_blank" rel="noopener noreferrer">GitHub ${svg('arrow', 'arrow-small')}</a></div></div><div class="container footer-bottom"><span>© 2026 LeakLens · Open-source Zcash privacy research</span><span>Hackathon prototype. Not financial advice or a production wallet.</span></div></footer>
  </div>`;

const byId = <T extends HTMLElement = HTMLElement>(id: string): T => {
  const element = document.getElementById(id);
  if (!element) throw new Error(`Missing ${id}`);
  return element as T;
};

function selectScenario(next: Scenario) {
  scenario = next;
  assessment = null;
  const info = scenarios.find(s => s.id === scenario)!;
  byId('scenarios').innerHTML = scenarios.map(item => `<button class="scenario-choice ${item.id === scenario ? 'active' : ''}" type="button" data-scenario="${item.id}" aria-pressed="${item.id === scenario}"><span class="scenario-choice-orb"></span><span>${item.name}</span>${svg('arrow')}</button>`).join('');
  byId('scenario-tag').textContent = info.tag;
  byId('scenario-heading').textContent = info.heading;
  byId('scenario-description').textContent = info.description;
  info.flow.forEach((step, i) => { byId(`flow-${i}`).textContent = step; });
  byId('result-area').innerHTML = `<div class="empty-result">${svg('scan')}<span>Select <strong>Run privacy review</strong> to inspect this route.</span></div>`;
}

function renderAssessment(result: Assessment, source: ReviewSource) {
  const hasWarnings = result.findings.some(f => f.severity === 'warning');
  const state = !result.complete ? 'incomplete' : hasWarnings ? 'flagged' : result.findings.length ? 'caution' : 'clear';
  const headlines = { incomplete: 'Not enough information', flagged: 'Privacy exposure detected', caution: 'Correlation risk detected', clear: 'No implemented rule triggered' };
  const descriptions = {
    incomplete: 'Some proposal facts are missing or contradictory. This review cannot be treated as a pass.',
    flagged: 'This route exposes information publicly. Review the findings before choosing a payment path.',
    caution: 'This route could create observable correlations. A possible link is not proof of identity.',
    clear: 'The current rules found no listed exposure in these supplied facts. This is not proof of anonymity.',
  };
  const node = document.createElement('div');
  node.className = `assessment assessment-${state}`;
  node.innerHTML = `<div class="assessment-top"><span class="assessment-badge"></span><div><div class="lab-panel-title">PRIVACY REVIEW · POLICY V${result.policy_version}</div><h4></h4><p></p></div></div><div class="assessment-source"></div><div class="findings-list"></div><p class="assessment-footer">${svg('info')} Educational analysis only. Never use this page to authorize or broadcast a payment.</p>`;
  node.querySelector('h4')!.textContent = headlines[state];
  node.querySelector('.assessment-top p')!.textContent = descriptions[state];
  node.querySelector('.assessment-source')!.textContent = source === 'rust_api'
    ? 'Analyzed by Rust policy engine · user-provided simulation facts, not a validated wallet proposal'
    : source === 'backend_unavailable'
      ? 'BACKEND UNAVAILABLE · Results below are from the local browser simulation only, not the Rust backend.'
      : 'Local simulation · user-provided synthetic facts, not validated by a wallet';
  if (source === 'backend_unavailable') {
    node.classList.add('assessment-offline');
    const alert = document.createElement('div');
    alert.className = 'backend-alert';
    alert.setAttribute('role', 'alert');
    alert.textContent = 'Rust backend is offline or could not be reached. The results below are an educational local simulation, not a server review. Try again when service is restored.';
    node.prepend(alert);
  }
  const list = node.querySelector('.findings-list')!;
  if (result.findings.length) {
    result.findings.forEach(f => {
      const row = document.createElement('div');
      row.className = `finding finding-${f.severity}`;
      const heading = document.createElement('div');
      heading.className = 'finding-header';
      const code = document.createElement('strong');
      code.textContent = f.code;
      const certainty = document.createElement('span');
      certainty.textContent = f.certainty.replace(/_/g, ' ');
      heading.append(code, certainty);
      const message = document.createElement('p');
      message.textContent = f.message;
      row.append(heading, message);
      list.append(row);
    });
  } else {
    const p = document.createElement('p');
    p.className = 'no-findings';
    p.textContent = 'Zero findings across the implemented LL-001–LL-007 checks for these supplied facts.';
    list.append(p);
  }
  byId('result-area').replaceChildren(node);
}

byId('scenarios').addEventListener('click', e => {
  const button = (e.target as Element).closest<HTMLButtonElement>('[data-scenario]');
  if (button && !pending) selectScenario(button.dataset.scenario as Scenario);
});
byId<HTMLButtonElement>('run-review').addEventListener('click', async () => {
  if (pending) return;
  pending = true;
  const button = byId<HTMLButtonElement>('run-review');
  button.disabled = true;
  button.textContent = 'Reviewing proposal…';
  const chosen: Scenario = scenario;
  try {
    const proposal: Proposal = makeProposal(chosen);
    const result = await reviewProposal(proposal);
    if (scenario === chosen) { assessment = result.assessment; reviewSource = result.source; renderAssessment(assessment, reviewSource); }
  } catch {
    byId('result-area').textContent = 'Review unavailable. Please try again.';
  } finally {
    pending = false;
    button.disabled = false;
    button.innerHTML = `Run privacy review ${svg('arrow')}`;
  }
});
selectScenario(scenario);
