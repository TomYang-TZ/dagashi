// ============================================
// DAGASHI — Main App
// ============================================

// Tauri API — use window.__TAURI__ when available, mock for browser dev
var hasTauri = !!(window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke);
console.log('[dagashi] __TAURI__ available:', hasTauri);

async function invoke(cmd, args) {
  if (hasTauri) {
    try {
      console.log('[dagashi] invoke ' + cmd, args);
      var result = await window.__TAURI__.core.invoke(cmd, args);
      console.log('[dagashi] ' + cmd + ' result:', result);
      return result;
    } catch (err) {
      console.error('[dagashi] ' + cmd + ' error:', err);
      throw err;
    }
  } else {
    console.log('[mock] invoke ' + cmd, args);
    if (cmd === 'get_stats') return mockStats();
    if (cmd === 'get_config') return mockConfig();
    if (cmd === 'get_collection') return { pulls: [], unique_characters: {} };
    if (cmd === 'toggle_deaf_mode') return false;
    return null;
  }
}

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
        // Mono mode: vary opacity based on brightness for depth
        var alpha = (px.brightness / 255 * 0.8 + 0.2).toFixed(2);
        html += `<span style="color:rgba(196,163,90,${alpha})">${escaped}</span>`;
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
    this.mode = 'color';
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

function currentPullKey() {
  var now = new Date();
  var date = now.toISOString().split('T')[0];
  var hour = (now.getHours() < 10 ? '0' : '') + now.getHours();
  return date + '-' + hour;
}

function formatCountdown() {
  var now = new Date();
  var nextHour = new Date(now);
  nextHour.setMinutes(0, 0, 0);
  nextHour.setHours(nextHour.getHours() + 1);
  var diff = nextHour - now;
  if (diff <= 0) return null;
  var m = Math.floor(diff / 60000);
  var s = Math.floor((diff % 60000) / 1000);
  return (m < 10 ? '0' : '') + m + ':' + (s < 10 ? '0' : '') + s;
}

async function initPullPage() {
  var page = document.getElementById('page-pull');

  page.innerHTML =
    '<div class="pull-container">' +
      '<div class="pull-stats-summary" id="stats-summary">LOADING STATS...</div>' +
      '<div id="pull-countdown" style="font-size:20px;color:var(--text-bright);text-align:center;margin:20px 0;font-family:var(--pixel-font)"></div>' +
      '<div id="pull-countdown-label" style="font-size:7px;color:var(--text-dim);text-align:center;margin-bottom:16px"></div>' +
      '<div class="mode-toggle mt-8">' +
        '<button class="mode-btn active" data-mode="color">COLOR</button>' +
        '<button class="mode-btn" data-mode="mono">MONO</button>' +
      '</div>' +
      '<div id="pull-result" class="hidden">' +
        '<div class="ascii-display">' +
          '<div class="ascii-art" id="ascii-container"></div>' +
        '</div>' +
        '<div class="result-card" id="result-card"></div>' +
      '</div>' +
      '<div id="pull-status" class="text-center mt-16" style="font-size:8px;color:var(--text-dim)"></div>' +
    '</div>';

  // Fetch data and update UI
  try {
    appStats = await invoke('get_stats');
    appConfig = await invoke('get_config');
    document.getElementById('stats-summary').textContent =
      'TODAY: ' + appStats.total.toLocaleString() + ' KEYS | ' +
      Object.keys(appStats.chars).length + ' UNIQUE CHARS | ' +
      (appStats.categories ? appStats.categories.letter : 0) + ' LETTERS | ' +
      (appStats.backspace_count || 0) + ' BACKSPACES';
  } catch (err) {
    console.error('[dagashi] Failed to load stats:', err);
    document.getElementById('stats-summary').textContent = 'STATS ERROR: ' + err;
  }

  page.querySelectorAll('.mode-btn').forEach(function(btn) {
    btn.addEventListener('click', function() {
      page.querySelectorAll('.mode-btn').forEach(function(b) { b.classList.remove('active'); });
      btn.classList.add('active');
      if (currentAnimator) currentAnimator.setMode(btn.dataset.mode);
    });
  });

  // Check if this hour's pull already exists
  var pullKey = currentPullKey();
  try {
    var hourMeta = await invoke('load_pull_meta', { date: pullKey });
    if (hourMeta) {
      viewPull(pullKey);
    }
  } catch (e) {
    // No pull this hour
  }

  // Always show countdown to next hour
  function updateCountdown() {
    var cd = formatCountdown();
    var el = document.getElementById('pull-countdown');
    var label = document.getElementById('pull-countdown-label');
    if (!el) return;

    var newKey = currentPullKey();
    if (newKey !== pullKey) {
      el.textContent = '[ PULLING... ]';
      label.textContent = 'AUTO-PULL TRIGGERED';
      clearInterval(countdownInterval);
      pullKey = newKey;
      doPull();
    } else if (cd !== null) {
      el.textContent = cd;
      label.textContent = 'NEXT PULL IN';
    }
  }
  updateCountdown();
  var countdownInterval = setInterval(updateCountdown, 1000);

  updateStatusBar();
}

// Loading messages for auto-pull
var LOADING_MESSAGES = [
  'ROLLING RARITY DICE...',
  'CONSULTING THE ORACLE...',
  'SEARCHING FOR ANIME GIFS...',
  'CONVERTING TO ASCII...',
  'ALMOST THERE...',
];

async function doPull() {
  var countdown = document.getElementById('pull-countdown');
  var status = document.getElementById('pull-status');

  // Animate countdown area while pulling
  var frame = 0;
  var msgIdx = 0;
  var spinChars = ['◰', '◳', '◲', '◱'];
  var loadingInterval = setInterval(function() {
    if (countdown) countdown.textContent = spinChars[frame % spinChars.length] + ' PULLING...';
    frame++;
    if (frame % 8 === 0) {
      msgIdx = Math.min(msgIdx + 1, LOADING_MESSAGES.length - 1);
    }
    if (status) status.textContent = LOADING_MESSAGES[msgIdx];
  }, 200);

  try {
    var meta = await invoke('do_pull');
    clearInterval(loadingInterval);
    status.textContent = `GOT: ${meta.character} (${meta.rarity})`;

    // Hide pull button
    var pullBtn = document.querySelector('.pull-now-btn');
    if (pullBtn) pullBtn.style.display = 'none';

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
    var mode = 'color';
    document.querySelectorAll('.mode-btn').forEach(function(b) {
      b.classList.toggle('active', b.dataset.mode === mode);
    });
    currentAnimator.setMode(mode);
    currentAnimator.start();

    // Result card
    document.getElementById('result-card').innerHTML = `
      <span class="rarity-badge rarity-${meta.rarity}">${meta.rarity.toUpperCase()}</span>
      <div class="result-character">${meta.character}</div>
      <div style="font-size:8px;color:var(--text-dim);margin-bottom:4px">${meta.anime_title || ''} ${meta.anime_rank ? '#' + meta.anime_rank : ''}</div>
      <div class="result-flavor">"${meta.flavor_text}"</div>
      <div class="mt-8" style="font-size:7px;color:var(--text-dim)">
        SOURCE: ${meta.source} | FRAMES: ${meta.frame_count}
      </div>
      <div id="ipfs-slot" class="mt-8"></div>
    `;
    renderIpfsSlot(meta.date, meta.ipfs_cid);


    // Scroll to result card so it's visible
    setTimeout(function() {
      document.getElementById('result-card').scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }, 500);

  } catch (err) {
    clearInterval(loadingInterval);
    if (status) status.textContent = 'ERROR: ' + err;
    if (countdown) countdown.textContent = 'PULL FAILED';
  }
}

// ============================================
// IPFS PIN BUTTON
// ============================================

async function renderIpfsSlot(date, existingCid) {
  var slot = document.getElementById('ipfs-slot');
  if (!slot) return;

  var cid = existingCid || (await invoke('get_pull_cid', { date: date }));
  if (cid) {
    slot.innerHTML =
      '<a href="https://gateway.pinata.cloud/ipfs/' + cid + '" target="_blank" class="ipfs-link">' +
      'IPFS: ' + cid.substring(0, 16) + '...' +
      '</a>';
    return;
  }

  var config = appConfig || (await invoke('get_config'));
  if (!config.pinata_jwt) {
    slot.innerHTML = '';
    return;
  }

  slot.innerHTML = '<button class="pin-btn" id="pin-ipfs-btn">PIN TO IPFS</button>';
  document.getElementById('pin-ipfs-btn').addEventListener('click', async function() {
    var btn = this;
    btn.disabled = true;
    btn.textContent = 'PINNING...';
    try {
      var cid = await invoke('pin_pull', { date: date });
      slot.innerHTML =
        '<a href="https://gateway.pinata.cloud/ipfs/' + cid + '" target="_blank" class="ipfs-link">' +
        'IPFS: ' + cid.substring(0, 16) + '...' +
        '</a>';
    } catch (err) {
      btn.disabled = false;
      btn.textContent = 'PIN FAILED — RETRY';
      console.error('[dagashi] IPFS pin failed:', err);
    }
  });
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
    var pipeline = await invoke('load_pull_frames', { date: date });
    var meta = await invoke('load_pull_meta', { date: date });
    var stats = await invoke('get_stats');
    var ramp = buildCharRamp(stats.chars);

    // Switch to pull page and show the art
    document.querySelectorAll('.nav-tab').forEach(function(t) { t.classList.remove('active'); });
    document.querySelectorAll('.page').forEach(function(p) { p.classList.remove('active'); });
    document.querySelector('[data-page="pull"]').classList.add('active');
    document.getElementById('page-pull').classList.add('active');

    var resultDiv = document.getElementById('pull-result');
    if (resultDiv) {
      resultDiv.classList.remove('hidden');
      var container = document.getElementById('ascii-container');
      if (currentAnimator) currentAnimator.stop();
      currentAnimator = new AsciiAnimator(container, pipeline.frames, ramp);

      var mode = 'color';
      document.querySelectorAll('.mode-btn').forEach(function(b) {
        b.classList.toggle('active', b.dataset.mode === mode);
      });
      currentAnimator.setMode(mode);
      currentAnimator.start();

      // Show result card
      document.getElementById('result-card').innerHTML =
        '<span class="rarity-badge rarity-' + meta.rarity + '">' + meta.rarity.toUpperCase() + '</span>' +
        '<div class="result-character">' + meta.character + '</div>' +
        '<div style="font-size:8px;color:var(--text-dim);margin-bottom:4px">' + (meta.anime_title || '') + ' ' + (meta.anime_rank ? '#' + meta.anime_rank : '') + '</div>' +
        '<div class="result-flavor">"' + meta.flavor_text + '"</div>' +
        '<div class="mt-8" style="font-size:7px;color:var(--text-dim)">' +
        'SOURCE: ' + meta.source + ' | FRAMES: ' + meta.frame_count + ' | DATE: ' + meta.date +
        '</div>' +
        '<div id="ipfs-slot" class="mt-8"></div>';
      renderIpfsSlot(date, meta.ipfs_cid);

      setTimeout(function() {
        document.getElementById('result-card').scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }, 500);
    }
  } catch (err) {
    console.error('Failed to load pull:', err);
  }
}

// ============================================
// COLLECTION / ANIME DATABASE PAGE
// ============================================

var animeDbPollInterval = null;

async function renderCollection() {
  var page = document.getElementById('page-collection');

  // Get anime DB status and collection
  var dbStatus, collection;
  try {
    dbStatus = await invoke('get_anime_db_status');
    collection = await invoke('get_collection');
  } catch (err) {
    page.innerHTML = '<div style="color:var(--text-dim);font-size:8px;padding:16px">LOADING...</div>';
    return;
  }

  var collected = collection.unique_characters || {};
  var collectedAnime = collection.pulls ? [...new Set(collection.pulls.map(function(p) { return p.anime_title; }))] : [];
  var totalAnime = dbStatus.count || 0;
  var loading = totalAnime === 0;

  var html = '';

  // Stats bar
  html += '<div class="roster-progress">';
  if (loading) {
    html += 'LOADING ANIME DATABASE... <span id="db-count">0</span> LOADED';
  } else {
    html += 'ANIME DATABASE: ' + totalAnime + ' SHOWS | PULLED FROM: ' + collectedAnime.length;
  }
  html += '<div class="progress-bar"><div class="progress-fill" style="width:' + Math.min(100, (totalAnime / 1000) * 100) + '%"></div></div>';
  html += '</div>';

  // Rarity filter tabs
  html += '<div class="mode-toggle mt-8">';
  html += '<button class="mode-btn active" data-filter="all">ALL</button>';
  html += '<button class="mode-btn" data-filter="legendary">LEGENDARY</button>';
  html += '<button class="mode-btn" data-filter="epic">EPIC</button>';
  html += '<button class="mode-btn" data-filter="rare">RARE</button>';
  html += '<button class="mode-btn" data-filter="uncommon">UNCOMMON</button>';
  html += '<button class="mode-btn" data-filter="common">COMMON</button>';
  html += '</div>';

  // Anime grid
  html += '<div class="roster-grid mt-8" id="anime-grid">';
  html += renderAnimeGrid(dbStatus, collectedAnime, 'all');
  html += '</div>';

  page.innerHTML = html;

  // Filter buttons
  page.querySelectorAll('[data-filter]').forEach(function(btn) {
    btn.addEventListener('click', function() {
      page.querySelectorAll('[data-filter]').forEach(function(b) { b.classList.remove('active'); });
      btn.classList.add('active');
      document.getElementById('anime-grid').innerHTML = renderAnimeGrid(dbStatus, collectedAnime, btn.dataset.filter);
    });
  });

  // Poll for DB updates if still loading
  if (loading && !animeDbPollInterval) {
    animeDbPollInterval = setInterval(async function() {
      try {
        var s = await invoke('get_anime_db_status');
        if (s.count > 0) {
          clearInterval(animeDbPollInterval);
          animeDbPollInterval = null;
          renderCollection(); // re-render with full data
        } else {
          var el = document.getElementById('db-count');
          if (el) el.textContent = s.count;
        }
      } catch (e) {}
    }, 3000);
  }
}

function renderAnimeGrid(dbStatus, collectedAnime, filter) {
  var html = '';
  var animeList = dbStatus.anime || [];

  if (!animeList.length) {
    html += '<div style="font-size:8px;color:var(--text-dim);grid-column:1/-1">FETCHING ANIME DATA...</div>';
    return html;
  }

  var filtered = filter === 'all' ? animeList : animeList.filter(function(a) { return a.rarity === filter; });

  for (var anime of filtered) {
    var isPulled = collectedAnime.indexOf(anime.title) >= 0;
    var cls = isPulled ? 'collected' : 'uncollected';
    html += '<div class="roster-card ' + cls + '">';
    html += '<span class="rarity-badge rarity-' + anime.rarity + '">#' + anime.rank + '</span>';
    html += '<div class="roster-name">' + anime.title + '</div>';
    if (anime.score) {
      html += '<div class="roster-count">MAL ' + anime.score + ' | ' + (anime.members / 1000).toFixed(0) + 'K</div>';
    }
    html += '</div>';
  }

  return html;
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
      <div class="settings-label">PINATA JWT</div>
      <input class="settings-input" id="set-pinata-jwt"
        value="${config.pinata_jwt || ''}"
        placeholder="OPTIONAL — ENABLES IPFS PULL RECEIPTS">
      <div class="settings-hint">FREE JWT FROM APP.PINATA.CLOUD — PINS VERIFIABLE PULL PROOFS TO IPFS</div>
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
    const modelBtn = page.querySelector('[data-model].active');

    config.giphy_api_key = document.getElementById('set-giphy-key').value || null;
    config.pinata_jwt = document.getElementById('set-pinata-jwt').value || null;
    config.llm.cli_model = modelBtn?.dataset.model || 'haiku';

    try {
      await invoke('save_config_cmd', { newConfig: config });
      document.getElementById('settings-status').textContent = 'SAVED!';
    } catch (err) {
      document.getElementById('settings-status').textContent = `ERROR: ${err}`;
    }
  });

  // Toggle buttons
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
    ascii: { columns: 100 },
    giphy_api_key: null,
    pinata_jwt: null,
  };
}

// ============================================
// INIT
// ============================================

initPullPage().catch(err => {
  console.error('[dagashi] init failed:', err);
  document.getElementById('page-pull').innerHTML = `
    <div class="text-center mt-16" style="font-size:8px;color:#f44">
      INIT ERROR: ${err}<br><br>
      IF RUNNING IN BROWSER, THIS IS NORMAL — TAURI API NOT AVAILABLE.
    </div>
  `;
});
