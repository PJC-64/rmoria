<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  let socket: WebSocket | null = null;
  let connected = false;
  let hasSave = false;
  let stage = 'init'; // 'init', 'create_char', 'playing', 'disconnected'
  let screenLines: string[] = [];
  let statusMsg = '';
  let player: any = null;

  // Character Creation Form
  let name = 'Moria Hero';
  let selectedRace = 'Human';
  let selectedClass = 'Warrior';

  const races = ['Human', 'HalfElf', 'Elf', 'Halfling', 'Gnome', 'Dwarf', 'HalfOrc', 'HalfTroll'];
  const classes = ['Warrior', 'Mage', 'Priest', 'Rogue', 'Ranger', 'Paladin'];

  let isTauri = false;
  let tauriUnlisten: (() => void) | null = null;
  let rolledStats: any = null;
  let rolledHistory = '';

  onMount(async () => {
    isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined;
    if (isTauri) {
      const { listen } = await import('@tauri-apps/api/event');
      const { invoke } = await import('@tauri-apps/api/core');

      connected = true;
      hasSave = await invoke('gui_check_save');
      if (!hasSave) {
        stage = 'create_char';
      } else {
        stage = 'init';
      }

      tauriUnlisten = await listen('gui-update', (event: any) => {
        const msg = event.payload;
        if (msg.status_msg === 'rolled_character') {
          rolledStats = msg.player.stats;
          rolledHistory = msg.player.history;
          stage = 'character_rolled';
        } else {
          screenLines = msg.screen || [];
          statusMsg = msg.status_msg || '';
          player = msg.player;
          stage = 'playing';
        }
      });
    } else {
      connect();
    }
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    if (socket) socket.close();
    if (tauriUnlisten) tauriUnlisten();
    window.removeEventListener('keydown', handleKeydown);
  });

  function connect() {
    stage = 'init';
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.hostname || '127.0.0.1';
    socket = new WebSocket(`${protocol}//${host}:8080`);

    socket.onopen = () => {
      connected = true;
    };

    socket.onclose = () => {
      connected = false;
      stage = 'disconnected';
    };

    socket.onerror = (err) => {
      console.error('Socket error:', err);
      connected = false;
      stage = 'disconnected';
    };

    socket.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'init') {
          hasSave = msg.has_save;
          if (!hasSave) {
            stage = 'create_char';
          } else {
            stage = 'init';
          }
        } else if (msg.type === 'update') {
          if (msg.status_msg === 'rolled_character') {
            rolledStats = msg.player.stats;
            rolledHistory = msg.player.history;
            stage = 'character_rolled';
          } else {
            screenLines = msg.screen || [];
            statusMsg = msg.status_msg || '';
            player = msg.player;
            stage = 'playing';
          }
        }
      } catch (e) {
        console.error('Failed to parse message:', e);
      }
    };
  }

  async function loadGame() {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('gui_send_setup', { action: { type: 'load' } });
    } else if (socket && connected) {
      socket.send(JSON.stringify({ type: 'load' }));
    }
  }

  function startNewGame() {
    stage = 'create_char';
  }

  async function rollCharacter() {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('gui_send_setup', {
        action: {
          type: 'roll_character',
          race: selectedRace,
          class: selectedClass
        }
      });
    } else if (socket && connected) {
      socket.send(JSON.stringify({
        type: 'roll_character',
        race: selectedRace,
        class: selectedClass
      }));
    }
  }

  async function acceptCharacter() {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('gui_send_setup', {
        action: {
          type: 'accept_character',
          name,
          race: selectedRace,
          class: selectedClass,
          stats: rolledStats,
          history: rolledHistory
        }
      });
    } else if (socket && connected) {
      socket.send(JSON.stringify({
        type: 'accept_character',
        name,
        race: selectedRace,
        class: selectedClass,
        stats: rolledStats,
        history: rolledHistory
      }));
    }
  }

  async function sendKey(key: string) {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('gui_send_key', { key });
    } else if (socket && connected && stage === 'playing') {
      socket.send(JSON.stringify({ type: 'key', key }));
    }
  }

  async function handleRestart() {
    await sendKey('r');
    stage = 'create_char';
    player = null;
    rolledStats = null;
    rolledHistory = '';
    name = 'Moria Hero';
    selectedRace = 'Human';
    selectedClass = 'Warrior';
    hasSave = false;
  }

  async function handleExit() {
    await sendKey('q');
  }

  let showCheatModal = false;
  let cheatPasswordInput = '';
  let cheatErrorMsg = '';

  function openCheatModal() {
    showCheatModal = true;
    cheatPasswordInput = '';
    cheatErrorMsg = '';
  }

  async function submitCheatPassword() {
    const pwd = cheatPasswordInput.trim();
    if (pwd === 'valar' || pwd === 'moria' || pwd === 'cheat') {
      await activateCheat();
      showCheatModal = false;
      cheatPasswordInput = '';
      cheatErrorMsg = '';
    } else {
      cheatErrorMsg = 'Incorrect password!';
    }
  }

  async function activateCheat() {
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('gui_activate_cheat');
    } else if (socket && connected && stage === 'playing') {
      socket.send(JSON.stringify({ type: 'activate_cheat' }));
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (stage !== 'playing' || (player && player.hp <= 0)) return;

    if (showCheatModal) {
      if (event.key === 'Escape') {
        showCheatModal = false;
        cheatPasswordInput = '';
        cheatErrorMsg = '';
      }
      return;
    }

    // Check for Cheat/Test Mode Activation (Ctrl/Cmd/Alt + D or K or C)
    const isModifier = event.ctrlKey || event.metaKey || event.altKey;
    const isCheatKey = ['KeyD', 'KeyK', 'KeyC'].includes(event.code) || ['d', 'D', 'k', 'K', 'c', 'C', 'ç', 'Ç'].includes(event.key);
    if (isModifier && isCheatKey) {
      event.preventDefault();
      openCheatModal();
      return;
    }

    // Prevent default scrolling/navigation keys
    if (['Space', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'PageUp', 'PageDown'].includes(event.key)) {
      event.preventDefault();
    }

    let key = event.key;
    let code = event.code;
    let isNumpad = event.location === 3;

    // In text input / barter / number prompt modes, let numpad keys type their actual numbers
    const isTextInputMode = player && ['BarterBuyMenu', 'BarterSellMenu', 'SelectRestCount', 'Inscribing'].includes(player.screen_mode);

    // 1. Numpad movement controls (handles NumLock ON and OFF states)
    if (isNumpad) {
      if (isTextInputMode) {
        if (key >= '0' && key <= '9') {
          sendKey(key);
          return;
        }
        if (key === 'Enter') {
          sendKey('\n');
          return;
        }
      } else {
        if (key === '7' || code === 'Numpad7' || key === 'Home') {
          sendKey('q');
          return;
        }
        if (key === '8' || code === 'Numpad8' || key === 'ArrowUp') {
          sendKey('w');
          return;
        }
        if (key === '9' || code === 'Numpad9' || key === 'PageUp') {
          sendKey('e');
          return;
        }
        if (key === '4' || code === 'Numpad4' || key === 'ArrowLeft') {
          sendKey('a');
          return;
        }
        if (key === '5' || code === 'Numpad5' || key === 'Clear') {
          sendKey('s');
          return;
        }
        if (key === '6' || code === 'Numpad6' || key === 'ArrowRight') {
          sendKey('d');
          return;
        }
        if (key === '1' || code === 'Numpad1' || key === 'End') {
          sendKey('z');
          return;
        }
        if (key === '2' || code === 'Numpad2' || key === 'ArrowDown') {
          sendKey('x');
          return;
        }
        if (key === '3' || code === 'Numpad3' || key === 'PageDown') {
          sendKey('c');
          return;
        }
      }
    }

    // 2. Standalone diagonal navigation keys (laptop layouts)
    if (key === 'Home') {
      sendKey('q');
      return;
    }
    if (key === 'PageUp') {
      sendKey('e');
      return;
    }
    if (key === 'End') {
      sendKey('z');
      return;
    }
    if (key === 'PageDown') {
      sendKey('c');
      return;
    }

    // 3. Standard character typing and control keys
    if (key.length === 1) {
      sendKey(key);
    } else {
      switch (key) {
        case 'ArrowUp':
          sendKey('w');
          break;
        case 'ArrowDown':
          sendKey('x');
          break;
        case 'ArrowLeft':
          sendKey('a');
          break;
        case 'ArrowRight':
          sendKey('d');
          break;
        case 'Escape':
          sendKey('\x1b');
          break;
        case 'Enter':
          sendKey('\n');
          break;
        case 'Backspace':
          sendKey('\u0008');
          break;
      }
    }
  }

  // Parse lines to add styling per character
  function parseLine(line: string): { char: string, colorClass: string }[] {
    const result = [];
    for (let i = 0; i < line.length; i++) {
      const char = line[i];
      let colorClass = 'char-default';

      if (char === '@') {
        colorClass = 'char-player';
      } else if (char === '#') {
        colorClass = 'char-wall';
      } else if (char === '.') {
        colorClass = 'char-floor';
      } else if (char === '%') {
        colorClass = 'char-vein';
      } else if (char === '*') {
        colorClass = 'char-rubble';
      } else if (char === '+' || char === '\'') {
        colorClass = 'char-door';
      } else if (char === '<' || char === '>') {
        colorClass = 'char-stairs';
      } else if (char === '&') {
        colorClass = 'char-chest';
      } else if (char === '^') {
        colorClass = 'char-trap';
      } else if (/[A-Za-z]/.test(char) && !line.includes('---') && !line.includes('Inventory') && !line.includes('Equipment')) {
        // Simple heuristic: color-code single letters that represent monsters or items
        colorClass = 'char-monster';
      } else if (['!', '?', ')', '[', '_', ',', '/'].includes(char)) {
        colorClass = 'char-item';
      }

      result.push({ char, colorClass });
    }
    return result;
  }
</script>

<main class="app-container">
  {#if stage === 'init'}
    <div class="menu-card glass-panel text-center">
      <h1 class="title">RMORIA</h1>
      <p class="subtitle font-gold">A Rust Rewrite of Classic Umoria</p>

      <div class="actions-container">
        {#if hasSave}
          <button class="btn btn-primary animate-pulse" on:click={loadGame}>
            Load Saved Adventure
          </button>
          <button class="btn btn-secondary" on:click={startNewGame}>
            Embark on New Journey
          </button>
        {:else}
          <button class="btn btn-primary animate-pulse" on:click={startNewGame}>
            Embark on Journey
          </button>
        {/if}
      </div>
    </div>

  {:else if stage === 'create_char'}
    <div class="menu-card glass-panel">
      <h2 class="title text-center">CHARACTER CREATION</h2>
      
      <div class="form-group">
        <label for="char-name">Character Name</label>
        <input type="text" id="char-name" class="form-input" bind:value={name} placeholder="Enter name..." />
      </div>

      <div class="form-grid">
        <div class="form-group">
          <label for="char-race">Select Race</label>
          <select id="char-race" class="form-select" bind:value={selectedRace}>
            {#each races as r}
              <option value={r}>{r}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label for="char-class">Select Class</label>
          <select id="char-class" class="form-select" bind:value={selectedClass}>
            {#each classes as c}
              <option value={c}>{c}</option>
            {/each}
          </select>
        </div>
      </div>

      <button class="btn btn-primary btn-block" on:click={rollCharacter}>
        Roll Attributes & Background
      </button>
    </div>

  {:else if stage === 'character_rolled'}
    <div class="menu-card glass-panel rolled-card" style="max-width: 900px; width: 95%;">
      <h2 class="title text-center">CHARACTER PREVIEW</h2>
      
      <div class="rolled-layout">
        <!-- Stats panel -->
        <div class="rolled-stats glass-card">
          <h3 class="rolled-section-title">ATTRIBUTES</h3>
          <div class="stat-roll-row">
            <span class="stat-name">Race / Class</span>
            <span class="stat-val font-gold">{selectedRace} / {selectedClass}</span>
          </div>
          {#if rolledStats}
            <div class="stat-roll-row">
              <span class="stat-name">Strength (STR)</span>
              <span class="stat-val font-green">{rolledStats.str}</span>
            </div>
            <div class="stat-roll-row">
              <span class="stat-name">Intelligence (INT)</span>
              <span class="stat-val font-green">{rolledStats.int}</span>
            </div>
            <div class="stat-roll-row">
              <span class="stat-name">Wisdom (WIS)</span>
              <span class="stat-val font-green">{rolledStats.wis}</span>
            </div>
            <div class="stat-roll-row">
              <span class="stat-name">Dexterity (DEX)</span>
              <span class="stat-val font-green">{rolledStats.dex}</span>
            </div>
            <div class="stat-roll-row">
              <span class="stat-name">Constitution (CON)</span>
              <span class="stat-val font-green">{rolledStats.con}</span>
            </div>
            <div class="stat-roll-row">
              <span class="stat-name">Charisma (CHR)</span>
              <span class="stat-val font-green">{rolledStats.chr}</span>
            </div>
            <div class="stat-roll-row" style="margin-top: 1rem;">
              <span class="stat-name">Hit Points (HP)</span>
              <span class="stat-val font-gold">{rolledStats.max_hp}</span>
            </div>
            <div class="stat-roll-row">
              <span class="stat-name">Spell Mana</span>
              <span class="stat-val font-blue">{rolledStats.max_mana}</span>
            </div>
          {/if}
        </div>

        <!-- History panel -->
        <div class="rolled-history glass-card">
          <h3 class="rolled-section-title">BACKGROUND HISTORY</h3>
          <div class="history-content font-mono" style="white-space: pre-wrap; text-align: left;">
            {rolledHistory}
          </div>
        </div>
      </div>

      <div class="btn-group-row">
        <button class="btn btn-secondary" on:click={rollCharacter}>
          Re-roll Character
        </button>
        <button class="btn btn-primary" on:click={acceptCharacter}>
          Accept & Descend into Moria
        </button>
      </div>
    </div>

  {:else if stage === 'playing'}
    <div class="game-layout">
      <!-- MAIN DUNGEON DISPLAY -->
      <div class="dungeon-container glass-panel">
        <div class="dungeon-header">
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <span class="depth-text font-gold" style="cursor: pointer;" title="Double-click to activate Test Mode" on:dblclick={openCheatModal}>
            {player ? (player.depth === 0 ? "Town Level" : `${player.depth * 50} feet`) : 'Unknown depth'}
          </span>
          <div class="header-badges">
            {#if player && player.food_status}
              <span class="badge badge-food" class:badge-starving={player.food_status === 'STARVING' || player.food_status === 'FAINT'}>
                {player.food_status}
              </span>
            {/if}
            {#if player && player.searching}
              <span class="badge badge-searching">SEARCHING</span>
            {/if}
          </div>
        </div>

        <!-- ASCII DISPLAY PANE -->
        <div class="dungeon-screen map-font">
          <div class="dungeon-grid">
            {#each screenLines as line}
              <div class="screen-row">
                {#each parseLine(line) as segment}
                  <span class={segment.colorClass}>{segment.char}</span>
                {/each}
              </div>
            {/each}
          </div>
        </div>

        <!-- MESSAGE LOG -->
        <div class="message-pane">
          <span class="font-gold font-bold">Message:</span> {statusMsg || 'Your environment is quiet.'}
        </div>
      </div>

      <!-- PLAYER CHARACTERISTICS & STATS PANE -->
      <div class="sidebar">
        <!-- CHAR HEADER CARD -->
        <div class="sidebar-card glass-panel">
          <h2 class="sidebar-char-name">{player?.name || 'Adventurer'}</h2>
          <p class="sidebar-char-info font-gold">{player?.race} {player?.class}</p>

          <!-- HP & MANA BARS -->
          {#if player}
            <div class="progress-container">
              <div class="progress-labels">
                <span>HP</span>
                <span>{player.hp} / {player.max_hp}</span>
              </div>
              <div class="progress-bar-bg">
                <div class="progress-bar-fill fill-health" style="width: {Math.max(0, Math.min(100, (player.hp / player.max_hp) * 100))}%"></div>
              </div>
            </div>

            {#if player.max_mana > 0}
              <div class="progress-container">
                <div class="progress-labels">
                  <span>MANA</span>
                  <span>{player.mana} / {player.max_mana}</span>
                </div>
                <div class="progress-bar-bg">
                  <div class="progress-bar-fill fill-mana" style="width: {Math.max(0, Math.min(100, (player.mana / player.max_mana) * 100))}%"></div>
                </div>
              </div>
            {/if}
          {/if}

          <!-- SECONDARY CHARACTERISTICS -->
          <div class="stats-grid mt-4">
            <div class="stat-item">
              <span class="stat-label">Level</span>
              <span class="stat-value">{player?.level}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">EXP</span>
              <span class="stat-value">{player?.exp}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Gold</span>
              <span class="stat-value font-gold">{player?.gold} gp</span>
            </div>
          </div>
        </div>

        <!-- ATTRIBUTES CARD -->
        <div class="sidebar-card glass-panel">
          <h3 class="card-title">ATTRIBUTES</h3>
          <div class="attributes-grid" style="grid-template-columns: repeat(2, 1fr); gap: 0.5rem;">
            {#if player && player.stats}
              <div class="attribute-box">
                <span class="attr-lbl">STR</span>
                <span class="attr-val font-gold">{player.stats.strength}</span>
              </div>
              <div class="attribute-box">
                <span class="attr-lbl">INT</span>
                <span class="attr-val font-gold">{player.stats.intelligence}</span>
              </div>
              <div class="attribute-box">
                <span class="attr-lbl">WIS</span>
                <span class="attr-val font-gold">{player.stats.wisdom}</span>
              </div>
              <div class="attribute-box">
                <span class="attr-lbl">DEX</span>
                <span class="attr-val font-gold">{player.stats.dexterity}</span>
              </div>
              <div class="attribute-box">
                <span class="attr-lbl">CON</span>
                <span class="attr-val font-gold">{player.stats.constitution}</span>
              </div>
              <div class="attribute-box">
                <span class="attr-lbl">CHR</span>
                <span class="attr-val font-gold">{player.stats.charisma}</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- CONTROLS PAD -->
        <div class="sidebar-card glass-panel text-center">
          <h3 class="card-title">ACTIONS</h3>
          <div class="control-pad">
            <div class="control-pad-grid">
              <button class="ctrl-btn" on:click={() => sendKey('q')}>NW</button>
              <button class="ctrl-btn" on:click={() => sendKey('w')}>N</button>
              <button class="ctrl-btn" on:click={() => sendKey('e')}>NE</button>
              <button class="ctrl-btn" on:click={() => sendKey('a')}>W</button>
              <button class="ctrl-btn ctrl-rest" on:click={() => sendKey('R')}>Rest</button>
              <button class="ctrl-btn" on:click={() => sendKey('d')}>E</button>
              <button class="ctrl-btn" on:click={() => sendKey('z')}>SW</button>
              <button class="ctrl-btn" on:click={() => sendKey('x')}>S</button>
              <button class="ctrl-btn" on:click={() => sendKey('c')}>SE</button>
            </div>
          </div>

          <div class="action-buttons-flex mt-4">
            <button class="act-btn" on:click={() => sendKey('E')}>Eat</button>
            <button class="act-btn" on:click={() => sendKey('B')}>Bash</button>
            <button class="act-btn" on:click={() => sendKey('T')}>Tunnel</button>
            <button class="act-btn" on:click={() => sendKey('F')}>Refill</button>
            <button class="act-btn font-green" on:click={() => sendKey('l')}>Look</button>
            <button class="act-btn font-gold" on:click={() => sendKey('#')}>Search</button>
            <button class="act-btn" on:click={() => sendKey('{')}>Inscribe</button>
            <button class="act-btn font-red" on:click={() => sendKey('Q')}>Quit</button>
          </div>
        </div>
      </div>
    </div>

  {:else if stage === 'disconnected'}
    <div class="menu-card glass-panel text-center">
      <h2 class="title text-red">DISCONNECTED</h2>
      <p class="subtitle text-muted">The connection to Moria has faded.</p>
      <button class="btn btn-primary" on:click={connect}>
        Re-establish Connection
      </button>
    </div>
  {/if}

  {#if showCheatModal}
    <!-- TEST MODE ACTIVATION MODAL -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" on:click|self={() => showCheatModal = false}>
      <div class="modal-card glass-panel">
        <h3 class="modal-title font-gold">TEST MODE ACTIVATION</h3>
        <p class="modal-desc">Enter the test mode password to activate immortality and maximum stats:</p>
        
        <input
          type="password"
          class="modal-input font-mono"
          bind:value={cheatPasswordInput}
          placeholder="Enter password..."
          on:keydown={(e) => { if (e.key === 'Enter') submitCheatPassword(); }}
          autofocus
        />
        
        {#if cheatErrorMsg}
          <div class="modal-error font-red">{cheatErrorMsg}</div>
        {/if}

        <div class="modal-actions">
          <button class="btn btn-gold" on:click={submitCheatPassword}>Submit</button>
          <button class="btn btn-secondary" on:click={() => showCheatModal = false}>Cancel</button>
        </div>
      </div>
    </div>
  {/if}

  {#if player && player.hp <= 0}
    <!-- DEATH MODAL -->
    <div class="modal-backdrop death-backdrop">
      <div class="modal-card death-card glass-panel animate-fade-in">
        <h1 class="death-title font-red">YOU HAVE DIED</h1>
        <div class="death-divider"></div>
        <p class="death-desc">Your character, <span class="font-gold">{player.name}</span>, has perished in the deep dungeons of Moria.</p>
        
        <div class="death-stats font-mono">
          <div class="stat-row">
            <span class="stat-label">Race / Class:</span>
            <span class="stat-value">{player.race} / {player.class}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Level Reached:</span>
            <span class="stat-value font-gold">{player.level}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Depth Reached:</span>
            <span class="stat-value font-gold">{player.depth === 0 ? "Town" : `${player.depth * 50} feet`}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Gold Accumulated:</span>
            <span class="stat-value font-gold">{player.gold} AU</span>
          </div>
        </div>

        <div class="modal-actions death-actions">
          <button class="btn btn-gold btn-restart" on:click={handleRestart}>
            Restart Adventure
          </button>
          <button class="btn btn-secondary btn-quit" on:click={handleExit}>
            Quit Game
          </button>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  .app-container {
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100%;
    min-height: calc(100vh - 3rem);
  }

  .text-center {
    text-align: center;
  }

  .title {
    font-size: 2.8rem;
    font-weight: 700;
    letter-spacing: 0.15em;
    margin: 0 0 0.5rem 0;
    color: var(--text-primary);
    text-shadow: 0 0 16px rgba(255, 255, 255, 0.2);
  }

  .subtitle {
    font-size: 1.1rem;
    margin-bottom: 2rem;
    letter-spacing: 0.05em;
  }

  .menu-card {
    width: 100%;
    max-width: 480px;
    padding: 2.5rem;
    box-sizing: border-box;
  }

  .form-group {
    margin-bottom: 1.25rem;
    text-align: left;
  }

  .form-group label {
    display: block;
    font-size: 0.85rem;
    color: var(--text-muted);
    margin-bottom: 0.5rem;
    font-weight: 600;
    letter-spacing: 0.05em;
  }

  .form-input, .form-select {
    width: 100%;
    padding: 0.75rem;
    background: rgba(31, 41, 55, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: var(--text-primary);
    box-sizing: border-box;
    font-size: 1rem;
    transition: all 0.2s ease;
  }

  .form-input:focus, .form-select:focus {
    outline: none;
    border-color: var(--color-mana);
    box-shadow: 0 0 12px var(--color-mana-glow);
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .actions-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .btn {
    padding: 0.85rem 1.5rem;
    font-size: 1rem;
    font-weight: 600;
    border-radius: 10px;
    border: none;
    cursor: pointer;
    transition: all 0.25s ease;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .btn-primary {
    background: var(--color-gold);
    color: #0b0f19;
  }

  .btn-primary:hover {
    background: #fbbf24;
    transform: translateY(-2px);
    box-shadow: 0 6px 18px var(--color-gold-glow);
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    border: 1px solid rgba(255, 255, 255, 0.15);
  }

  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.15);
    transform: translateY(-2px);
  }

  .btn-block {
    width: 100%;
    margin-top: 1rem;
  }

  .animate-pulse {
    animation: btn-pulse 2s infinite alternate;
  }

  @keyframes btn-pulse {
    0% {
      box-shadow: 0 4px 12px var(--color-gold-glow);
    }
    100% {
      box-shadow: 0 4px 20px rgba(245, 158, 11, 0.6);
    }
  }

  /* Game Playing Layout */
  .game-layout {
    display: grid;
    grid-template-columns: 1fr 340px;
    gap: 1.5rem;
    width: 100%;
    align-items: start;
  }

  .dungeon-container {
    display: flex;
    flex-direction: column;
    padding: 1.5rem;
    height: 700px;
  }

  .dungeon-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 0.75rem;
    margin-bottom: 1rem;
  }

  .depth-text {
    font-size: 1.25rem;
    font-weight: 600;
  }

  .header-badges {
    display: flex;
    gap: 0.5rem;
  }

  .badge {
    padding: 0.25rem 0.6rem;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.05em;
  }

  .badge-food {
    background: var(--color-green-glow);
    color: var(--color-green);
    border: 1px solid rgba(16, 185, 129, 0.2);
  }

  .badge-starving {
    background: var(--color-health-glow);
    color: var(--color-health);
    border: 1px solid rgba(239, 68, 68, 0.2);
    animation: alert-blink 1s infinite alternate;
  }

  .badge-searching {
    background: var(--color-gold-glow);
    color: var(--color-gold);
    border: 1px solid rgba(245, 158, 11, 0.2);
  }

  @keyframes alert-blink {
    0% { opacity: 0.4; }
    100% { opacity: 1; }
  }

  .dungeon-screen {
    flex-grow: 1;
    background: #05070c;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 1rem;
    font-size: 20px;
    line-height: 1.2;
    white-space: pre;
    overflow: hidden;
    color: var(--text-primary);
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
  }

  .dungeon-grid {
    display: inline-block;
    text-align: left;
  }

  .screen-row {
    height: 1.2em;
  }

  .message-pane {
    background: rgba(31, 41, 55, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 0.75rem 1rem;
    margin-top: 1rem;
    font-size: 0.95rem;
    line-height: 1.4;
  }

  /* Sidebar styling */
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .sidebar-card {
    padding: 1.25rem;
  }

  .card-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-muted);
    letter-spacing: 0.1em;
    margin: 0 0 1rem 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    padding-bottom: 0.5rem;
  }

  .sidebar-char-name {
    font-size: 1.5rem;
    margin: 0 0 0.25rem 0;
    font-weight: 700;
  }

  .sidebar-char-info {
    font-size: 0.9rem;
    margin: 0 0 1.25rem 0;
    font-weight: 600;
  }

  /* Progress bars */
  .progress-container {
    margin-bottom: 0.85rem;
  }

  .progress-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    font-weight: 600;
    margin-bottom: 0.25rem;
  }

  .progress-bar-bg {
    width: 100%;
    height: 8px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .fill-health {
    background: var(--color-health);
    box-shadow: 0 0 8px var(--color-health-glow);
  }

  .fill-mana {
    background: var(--color-mana);
    box-shadow: 0 0 8px var(--color-mana-glow);
  }

  /* Stats grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
  }

  .stat-item {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 0.5rem;
    text-align: center;
  }

  .stat-label {
    display: block;
    font-size: 0.75rem;
    color: var(--text-muted);
    font-weight: 600;
    margin-bottom: 0.15rem;
  }

  .stat-value {
    font-size: 0.95rem;
    font-weight: 700;
  }

  /* Attribute boxes */
  .attribute-box {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 8px;
    padding: 0.5rem 0.75rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .attr-lbl {
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--text-muted);
  }

  .attr-val {
    font-size: 1rem;
    font-weight: 700;
  }

  /* Controls Pad */
  .control-pad {
    display: flex;
    justify-content: center;
    margin-bottom: 1rem;
  }

  .control-pad-grid {
    display: grid;
    grid-template-columns: repeat(3, 70px);
    gap: 0.5rem;
  }

  .ctrl-btn {
    height: 48px;
    padding: 0;
    font-size: 0.95rem;
    font-weight: 700;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    color: var(--text-primary);
    display: flex;
    justify-content: center;
    align-items: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .ctrl-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: var(--border-glow);
  }

  .ctrl-rest {
    font-size: 0.8rem;
    color: var(--color-gold);
    border-color: var(--color-gold-glow);
  }

  .action-buttons-flex {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
  }

  .act-btn {
    padding: 0.5rem 0.25rem;
    font-size: 0.85rem;
    font-weight: 600;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 6px;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .act-btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  /* Utility classes */
  .act-btn-full { grid-column: span 3; }
  .font-green { color: var(--color-green, #4caf50); }
  .font-gold { color: var(--color-gold); }
  .font-red { color: var(--color-health); }
  .font-bold { font-weight: 700; }
  .mt-4 { margin-top: 1rem; }

  /* Test Mode Password Modal Styles */
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(5px);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 9999;
  }

  .modal-card {
    background: #0d1117;
    border: 1px solid rgba(255, 215, 0, 0.3);
    border-radius: 12px;
    padding: 2rem;
    max-width: 420px;
    width: 90%;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.85);
    display: flex;
    flex-direction: column;
    gap: 1rem;
    text-align: center;
  }

  .modal-title {
    font-size: 1.2rem;
    font-weight: bold;
    letter-spacing: 1px;
    margin: 0;
  }

  .modal-desc {
    font-size: 0.9rem;
    color: var(--text-secondary, #9ca3af);
    line-height: 1.4;
    margin: 0;
  }

  .modal-input {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    padding: 0.75rem 1rem;
    color: #fff;
    font-size: 1rem;
    outline: none;
    text-align: center;
  }

  .modal-input:focus {
    border-color: var(--color-gold, #ffd700);
  }

  .modal-error {
    font-size: 0.85rem;
    font-weight: bold;
  }

  .modal-actions {
    display: flex;
    justify-content: center;
    gap: 1rem;
    margin-top: 0.5rem;
  }

  /* Death Modal Styles */
  .death-backdrop {
    background: rgba(0, 0, 0, 0.88);
    backdrop-filter: blur(12px);
  }

  .death-card {
    background: #080a0f;
    border: 1px solid rgba(220, 38, 38, 0.45);
    max-width: 480px;
    padding: 2.5rem;
    box-shadow: 0 0 50px rgba(220, 38, 38, 0.25);
  }

  .death-title {
    font-size: 2.4rem;
    font-weight: 900;
    letter-spacing: 0.15em;
    margin: 0;
    text-shadow: 0 0 24px rgba(220, 38, 38, 0.85);
  }

  .death-divider {
    height: 2px;
    background: linear-gradient(90deg, transparent, #dc2626, transparent);
    margin: 0.75rem 0 1.5rem 0;
  }

  .death-desc {
    font-size: 0.95rem;
    line-height: 1.5;
    color: #cbd5e1;
    margin-bottom: 1.25rem;
  }

  .death-stats {
    background: rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .stat-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.9rem;
    padding: 0.35rem 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .stat-row:last-child {
    border-bottom: none;
  }

  .stat-label {
    color: #94a3b8;
  }

  .stat-value {
    font-weight: bold;
  }

  .death-actions {
    display: flex;
    gap: 1.25rem;
    width: 100%;
  }

  .btn-restart {
    flex: 1;
    box-shadow: 0 0 15px rgba(255, 215, 0, 0.25);
    transition: all 0.25s ease;
  }

  .btn-restart:hover {
    box-shadow: 0 0 25px rgba(255, 215, 0, 0.45);
    transform: translateY(-1px);
  }

  .btn-quit {
    flex: 1;
    background: #1e1b1b !important;
    border-color: rgba(220, 38, 38, 0.3) !important;
    color: #fca5a5 !important;
    transition: all 0.25s ease;
  }

  .btn-quit:hover {
    background: #271c1c !important;
    border-color: rgba(220, 38, 38, 0.6) !important;
    box-shadow: 0 0 15px rgba(220, 38, 38, 0.2);
    transform: translateY(-1px);
  }

  .animate-fade-in {
    animation: fadeIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(12px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }
</style>
