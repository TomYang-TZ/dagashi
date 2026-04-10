// ============================================
// DAGASHI — Main App
// ============================================

// Tauri API — use window.__TAURI__ when available, mock for browser dev
const invoke = window.__TAURI__?.core?.invoke || (async (cmd, args) => {
  console.log(`[mock] invoke ${cmd}`, args);
  if (cmd === 'get_stats') return mockStats();
  if (cmd === 'get_config') return mockConfig();
  if (cmd === 'get_collection') return { pulls: [], unique_characters: {} };
  if (cmd === 'toggle_deaf_mode') return false;
  return null;
});

// ============================================
// ASCII RENDERER
// ============================================

// Default brightness ramp (fallback if user has < 10 unique chars)
const DEFAULT_RAMP = ' .`-_:,;^=+/|)\\!?0oOQ#%@';

function buildCharRamp(charFreqs) {
  // Sort user's chars by frequency (least → most), space-pad for brightness range
  const entries = Object.entries(charFreqs)
    .filter(([k]) => k.length === 1 && k !== ' ') // single printable chars only
    .sort((a, b) => a[1] - b[1]); // least frequent first = lightest

  if (entries.length < 10) return DEFAULT_RAMP;

  // Build ramp: space (lightest) + chars sorted by frequency
  return ' ' + entries.map(([ch]) => ch).join('');
}

function brightnessToChar(brightness, ramp) {
  const idx = Math.floor((brightness / 255) * (ramp.length - 1));
  return ramp[Math.min(idx, ramp.length - 1)];
}

function renderAsciiFrame(frame, ramp, mode) {
  // frame.pixels = [[{r,g,b,brightness}, ...], ...]
  const rows = frame.pixels;
  let html = '';

  for (const row of rows) {
    html += '<div class="ascii-row">';
    for (const px of row) {
      const ch = brightnessToChar(px.brightness, ramp);
      const escaped = ch === '<' ? '&lt;' : ch === '>' ? '&gt;' : ch === '&' ? '&amp;' : ch;

      if (ch === ' ') {
        html += ' ';
      } else if (mode === 'color') {
        html += `<span style="color:rgb(${px.r},${px.g},${px.b})">${escaped}</span>`;
      } else {
        html += escaped;
      }
    }
    html += '</div>';
  }
  return html;
}

// ============================================
// ASCII ANIMATION
// ============================================

class AsciiAnimator {
  constructor(container, frames, ramp) {
    this.container = container;
    this.frames = frames;
    this.ramp = ramp;
    this.mode = 'mono';
    this.currentFrame = 0;
    this.speed = 120;
    this.running = false;
    this.rafId = 0;
    this.lastTime = 0;
  }

  start() {
    this.running = true;
    this.render();
    this.rafId = requestAnimationFrame(t => this.loop(t));
  }

  stop() {
    this.running = false;
    cancelAnimationFrame(this.rafId);
  }

  setMode(mode) {
    this.mode = mode;
    this.render();
  }

  loop(now) {
    if (!this.running) return;
    if (now - this.lastTime > this.speed) {
      this.currentFrame = (this.currentFrame + 1) % this.frames.length;
      this.lastTime = now;
      this.render();
    }
    this.rafId = requestAnimationFrame(t => this.loop(t));
  }

  render() {
    const frame = this.frames[this.currentFrame];
    this.container.innerHTML = renderAsciiFrame(frame, this.ramp, this.mode);
  }
}

// ============================================
// STATE
// ============================================

let currentAnimator = null;
let appStats = null;
let appConfig = null;

// ============================================
// NAVIGATION
// ============================================

document.querySelectorAll('.nav-tab').forEach(tab => {
  tab.addEventListener('click', () => {
    document.querySelectorAll('.nav-tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
    tab.classList.add('active');
    document.getElementById(`page-${tab.dataset.page}`).classList.add('active');

    if (tab.dataset.page === 'gallery') renderGallery();
    if (tab.dataset.page === 'collection') renderCollection();
    if (tab.dataset.page === 'settings') renderSettings();
  });
});

// ============================================
// PULL PAGE
// ============================================

async function initPullPage() {
  appStats = await invoke('get_stats');
  appConfig = await invoke('get_config');

  const page = document.getElementById('page-pull');
  page.innerHTML = `
    <div class="pull-container">
      <div class="pull-stats-summary">
        TODAY: ${appStats.total.toLocaleString()} KEYS
        | ${Object.keys(appStats.chars).length} UNIQUE CHARS
        | ${appStats.categories?.letter || 0} LETTERS
        | ${appStats.backspace_count || 0} BACKSPACES
      </div>

      <button class="pull-btn" id="pull-btn">
        [ PULL ]
      </button>

      <div class="mode-toggle mt-8">
        <button class="mode-btn active" data-mode="mono">MONO</button>
        <button class="mode-btn" data-mode="color">COLOR</button>
      </div>

      <div id="pull-result" class="hidden">
        <div class="ascii-display">
          <div class="ascii-art" id="ascii-container"></div>
        </div>
        <div class="result-card" id="result-card"></div>
      </div>

      <div id="pull-status" class="text-center mt-16" style="font-size:8px;color:var(--text-dim)"></div>
    </div>
  `;

  document.getElementById('pull-btn').addEventListener('click', doPull);

  page.querySelectorAll('.mode-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      page.querySelectorAll('.mode-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      if (currentAnimator) currentAnimator.setMode(btn.dataset.mode);
    });
  });

  updateStatusBar();
}

async function doPull() {
  const btn = document.getElementById('pull-btn');
  const status = document.getElementById('pull-status');

  btn.classList.add('loading');
  btn.textContent = '[ PULLING... ]';
  status.textContent = 'ROLLING RARITY...';

  try {
    const meta = await invoke('do_pull');
    status.textContent = `GOT: ${meta.character} (${meta.rarity})`;

    // Load the frames
    const pipeline = await invoke('load_pull_frames', { date: meta.date });

    // Build char ramp from user's keystroke stats
    const ramp = buildCharRamp(appStats.chars);

    // Show result
    const resultDiv = document.getElementById('pull-result');
    resultDiv.classList.remove('hidden');

    const container = document.getElementById('ascii-container');
    if (currentAnimator) currentAnimator.stop();

    currentAnimator = new AsciiAnimator(container, pipeline.frames, ramp);
    const activeMode = document.querySelector('.mode-btn.active')?.dataset.mode || 'mono';
    currentAnimator.setMode(activeMode);
    currentAnimator.start();

    // Result card
    document.getElementById('result-card').innerHTML = `
      <span class="rarity-badge rarity-${meta.rarity}">${meta.rarity.toUpperCase()}</span>
      <div class="result-character">${meta.character}</div>
      <div class="result-flavor">"${meta.flavor_text}"</div>
      <div class="mt-8" style="font-size:7px;color:var(--text-dim)">
        SOURCE: ${meta.source} | FRAMES: ${meta.frame_count} | MODE: ${meta.color_mode}
      </div>
    `;

    btn.textContent = '[ PULL AGAIN ]';
  } catch (err) {
    status.textContent = `ERROR: ${err}`;
    btn.textContent = '[ PULL ]';
  }

  btn.classList.remove('loading');
}

// ============================================
// GALLERY PAGE
// ============================================

async function renderGallery() {
  const collection = await invoke('get_collection');
  const page = document.getElementById('page-gallery');

  if (collection.pulls.length === 0) {
    page.innerHTML = `
      <div class="text-center mt-16" style="font-size:8px;color:var(--text-dim)">
        NO PULLS YET. GO PULL SOMETHING.
      </div>
    `;
    return;
  }

  let html = '<div class="gallery-grid">';
  for (const pull of [...collection.pulls].reverse()) {
    html += `
      <div class="gallery-card" data-date="${pull.date}">
        <div class="gallery-card-date">${pull.date}</div>
        <span class="rarity-badge rarity-${pull.rarity}">${pull.rarity.toUpperCase()}</span>
        <div class="gallery-card-name">${pull.character}</div>
        <div class="gallery-card-preview">${pull.scene}</div>
      </div>
    `;
  }
  html += '</div>';
  page.innerHTML = html;

  // Click to view
  page.querySelectorAll('.gallery-card').forEach(card => {
    card.addEventListener('click', () => viewPull(card.dataset.date));
  });
}

async function viewPull(date) {
  try {
    const pipeline = await invoke('load_pull_frames', { date });
    const stats = await invoke('get_stats');
    const ramp = buildCharRamp(stats.chars);

    // Switch to pull page and show the art
    document.querySelectorAll('.nav-tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
    document.querySelector('[data-page="pull"]').classList.add('active');
    document.getElementById('page-pull').classList.add('active');

    const resultDiv = document.getElementById('pull-result');
    if (resultDiv) {
      resultDiv.classList.remove('hidden');
      const container = document.getElementById('ascii-container');
      if (currentAnimator) currentAnimator.stop();
      currentAnimator = new AsciiAnimator(container, pipeline.frames, ramp);
      currentAnimator.start();
    }
  } catch (err) {
    console.error('Failed to load pull:', err);
  }
}

// ============================================
// COLLECTION PAGE
// ============================================

const ROSTER = [
  { name: 'Justaway', tier: 'common' },
  { name: 'Elizabeth', tier: 'common' },
  { name: 'Edo Citizen', tier: 'common' },
  { name: 'Shinpachi Shimura', tier: 'uncommon' },
  { name: 'Otae Shimura', tier: 'uncommon' },
  { name: 'Hasegawa (MADAO)', tier: 'uncommon' },
  { name: 'Kagura', tier: 'rare' },
  { name: 'Sadaharu', tier: 'rare' },
  { name: 'Isao Kondo', tier: 'rare' },
  { name: 'Otose', tier: 'rare' },
  { name: 'Gintoki Sakata', tier: 'epic' },
  { name: 'Toshiro Hijikata', tier: 'epic' },
  { name: 'Sougo Okita', tier: 'epic' },
  { name: 'Shinsuke Takasugi', tier: 'epic' },
  { name: 'Shiroyasha', tier: 'legendary' },
  { name: 'Yato Kagura', tier: 'legendary' },
  { name: 'Gintoki vs Takasugi', tier: 'legendary' },
];

async function renderCollection() {
  const collection = await invoke('get_collection');
  const page = document.getElementById('page-collection');

  const collected = collection.unique_characters || {};
  const collectedCount = Object.keys(collected).length;
  const totalCount = ROSTER.length;
  const pct = totalCount > 0 ? Math.round((collectedCount / totalCount) * 100) : 0;

  let html = `
    <div class="roster-progress">
      COLLECTED: ${collectedCount} / ${totalCount} (${pct}%)
      <div class="progress-bar">
        <div class="progress-fill" style="width:${pct}%"></div>
      </div>
    </div>
    <div class="roster-grid">
  `;

  for (const entry of ROSTER) {
    const count = collected[entry.name] || 0;
    const cls = count > 0 ? 'collected' : 'uncollected';
    html += `
      <div class="roster-card ${cls}">
        <span class="rarity-badge rarity-${entry.tier}">${entry.tier.toUpperCase()}</span>
        <div class="roster-name">${count > 0 ? entry.name : '???'}</div>
        <div class="roster-count">${count > 0 ? `x${count}` : ''}</div>
      </div>
    `;
  }

  html += '</div>';
  page.innerHTML = html;
}

// ============================================
// SETTINGS PAGE
// ============================================

async function renderSettings() {
  const config = await invoke('get_config');
  const page = document.getElementById('page-settings');

  page.innerHTML = `
    <div class="settings-section">
      <div class="settings-label">GIPHY API KEY</div>
      <input class="settings-input" id="set-giphy-key"
        value="${config.giphy_api_key || ''}"
        placeholder="LEAVE BLANK FOR DEMO KEY">
      <div class="settings-hint">FREE KEY FROM DEVELOPERS.GIPHY.COM/DASHBOARD</div>
    </div>

    <div class="settings-section">
      <div class="settings-label">PULL TRIGGER</div>
      <div class="settings-row">
        <button class="mode-btn ${config.pull_trigger.mode === 'manual' ? 'active' : ''}"
          data-trigger="manual">MANUAL</button>
        <button class="mode-btn ${config.pull_trigger.mode === 'scheduled' ? 'active' : ''}"
          data-trigger="scheduled">SCHEDULED</button>
        <input class="settings-input" id="set-schedule-time"
          value="${config.pull_trigger.scheduled_time}" style="width:80px">
      </div>
    </div>

    <div class="settings-section">
      <div class="settings-label">ASCII COLUMNS: ${config.ascii.columns}</div>
      <input type="range" id="set-cols" min="40" max="150" value="${config.ascii.columns}"
        style="width:300px">
    </div>

    <div class="settings-section">
      <div class="settings-label">COLOR PROBABILITY: ${Math.round(config.ascii.color_probability * 100)}%</div>
      <input type="range" id="set-color-prob" min="0" max="100"
        value="${Math.round(config.ascii.color_probability * 100)}" style="width:300px">
    </div>

    <div class="settings-section">
      <div class="settings-label">LLM MODEL</div>
      <div class="settings-row">
        <button class="mode-btn ${config.llm.cli_model === 'haiku' ? 'active' : ''}"
          data-model="haiku">HAIKU</button>
        <button class="mode-btn ${config.llm.cli_model === 'sonnet' ? 'active' : ''}"
          data-model="sonnet">SONNET</button>
      </div>
    </div>

    <button class="settings-btn mt-16" id="save-settings">SAVE</button>
    <span id="settings-status" style="font-size:7px;color:var(--text-dim);margin-left:8px"></span>
  `;

  // Wire up save
  document.getElementById('save-settings').addEventListener('click', async () => {
    const triggerBtn = page.querySelector('[data-trigger].active');
    const modelBtn = page.querySelector('[data-model].active');

    config.giphy_api_key = document.getElementById('set-giphy-key').value || null;
    config.pull_trigger.mode = triggerBtn?.dataset.trigger || 'manual';
    config.pull_trigger.scheduled_time = document.getElementById('set-schedule-time').value;
    config.ascii.columns = parseInt(document.getElementById('set-cols').value);
    config.ascii.color_probability = parseInt(document.getElementById('set-color-prob').value) / 100;
    config.llm.cli_model = modelBtn?.dataset.model || 'haiku';

    try {
      await invoke('save_config_cmd', { newConfig: config });
      document.getElementById('settings-status').textContent = 'SAVED!';
    } catch (err) {
      document.getElementById('settings-status').textContent = `ERROR: ${err}`;
    }
  });

  // Toggle buttons
  page.querySelectorAll('[data-trigger]').forEach(btn => {
    btn.addEventListener('click', () => {
      page.querySelectorAll('[data-trigger]').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
    });
  });
  page.querySelectorAll('[data-model]').forEach(btn => {
    btn.addEventListener('click', () => {
      page.querySelectorAll('[data-model]').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
    });
  });
}

// ============================================
// DEAF MODE
// ============================================

document.getElementById('deaf-toggle').addEventListener('click', async () => {
  const isDeaf = await invoke('toggle_deaf_mode');
  const btn = document.getElementById('deaf-toggle');
  const icon = document.getElementById('deaf-icon');
  const label = document.getElementById('deaf-label');
  btn.classList.toggle('deaf', isDeaf);
  icon.textContent = isDeaf ? '\u{1F6AB}' : '\u{1F442}';
  label.textContent = isDeaf ? 'DEAF' : 'LISTENING';
  document.getElementById('status-deaf').textContent = isDeaf ? '| DEAF' : '';
});

// ============================================
// STATUS BAR
// ============================================

async function updateStatusBar() {
  try {
    const stats = await invoke('get_stats');
    document.getElementById('status-keys').textContent = `KEYS: ${stats.total.toLocaleString()}`;
  } catch {}

  const now = new Date();
  document.getElementById('status-time').textContent =
    now.toLocaleTimeString('en-US', { hour12: false });
}

setInterval(updateStatusBar, 5000);

// ============================================
// MOCK DATA (for browser dev without Tauri)
// ============================================

function mockStats() {
  return {
    date: new Date().toISOString().split('T')[0],
    total: 4281,
    chars: { e: 890, t: 720, a: 680, o: 590, i: 510, n: 480, s: 440, r: 410, h: 320, l: 280 },
    categories: { letter: 3800, number: 200, symbol: 181, modifier: 100 },
    backspace_count: 142,
    shift_count: 89,
    capslock_count: 0,
    hourly_volume: new Array(24).fill(0),
    regions: { left_hand: 2100, right_hand: 2181, home_row: 1800 },
  };
}

function mockConfig() {
  return {
    keystroke_capture: { enabled: true, deaf_mode: false, deaf_mode_shortcut: 'Cmd+Shift+D' },
    pull_trigger: { mode: 'manual', scheduled_time: '18:00' },
    rarity_thresholds: { uncommon: 10000, rare: 30000, epic: 60000, legendary: 100000 },
    llm: { mode: 'cli', cli_model: 'haiku', cli_effort: 'low', api_key: null, api_temperature: 0.99 },
    ascii: { columns: 100, color_probability: 0.5 },
    giphy_api_key: null,
  };
}

// ============================================
// INIT
// ============================================

initPullPage();
