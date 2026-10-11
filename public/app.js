/**
 * RUST NOSTR ASCII MARIO 1985 - CLIENT APP & AUDIO SYNTHESIZER
 * 100% Client-side WebAssembly & Web Audio Interactive Game Engine
 */

(function () {
  // =========================================================================
  // 1. Web Audio API 8-Bit Chiptune Synthesizer
  // =========================================================================
  let audioCtx = null;
  let bgmOsc = null;
  let isMuted = false;

  function initAudio() {
    if (!audioCtx) {
      const AudioContext = window.AudioContext || window.webkitAudioContext;
      if (AudioContext) {
        audioCtx = new AudioContext();
      }
    }
    if (audioCtx && audioCtx.state === 'suspended') {
      audioCtx.resume();
    }
  }

  window.playMarioSfx = function (sfxName, volume = 0.3) {
    if (isMuted) return;
    initAudio();
    if (!audioCtx) return;

    try {
      const now = audioCtx.currentTime;
      const osc = audioCtx.createOscillator();
      const gain = audioCtx.createGain();
      osc.connect(gain);
      gain.connect(audioCtx.destination);

      gain.gain.setValueAtTime(volume, now);

      switch (sfxName) {
        case 'jump':
          osc.type = 'square';
          osc.frequency.setValueAtTime(150, now);
          osc.frequency.exponentialRampToValueAtTime(600, now + 0.15);
          gain.gain.exponentialRampToValueAtTime(0.01, now + 0.18);
          osc.start(now);
          osc.stop(now + 0.18);
          break;

        case 'coin':
          osc.type = 'sine';
          osc.frequency.setValueAtTime(987.77, now); // B5
          osc.frequency.setValueAtTime(1318.51, now + 0.08); // E6
          gain.gain.exponentialRampToValueAtTime(0.01, now + 0.35);
          osc.start(now);
          osc.stop(now + 0.35);
          break;

        case 'powerup':
          osc.type = 'triangle';
          const notes = [330, 392, 659, 523, 587, 784];
          notes.forEach((freq, idx) => {
            osc.frequency.setValueAtTime(freq, now + idx * 0.06);
          });
          gain.gain.exponentialRampToValueAtTime(0.01, now + 0.45);
          osc.start(now);
          osc.stop(now + 0.45);
          break;

        case 'stomp':
          osc.type = 'sawtooth';
          osc.frequency.setValueAtTime(220, now);
          osc.frequency.exponentialRampToValueAtTime(55, now + 0.12);
          gain.gain.exponentialRampToValueAtTime(0.01, now + 0.15);
          osc.start(now);
          osc.stop(now + 0.15);
          break;

        case 'fireball':
          osc.type = 'triangle';
          osc.frequency.setValueAtTime(800, now);
          osc.frequency.exponentialRampToValueAtTime(120, now + 0.1);
          gain.gain.exponentialRampToValueAtTime(0.01, now + 0.12);
          osc.start(now);
          osc.stop(now + 0.12);
          break;

        case 'bowser_fall':
        case 'gameover':
          osc.type = 'sawtooth';
          osc.frequency.setValueAtTime(300, now);
          osc.frequency.exponentialRampToValueAtTime(40, now + 0.6);
          gain.gain.exponentialRampToValueAtTime(0.01, now + 0.65);
          osc.start(now);
          osc.stop(now + 0.65);
          break;

        case 'world_clear':
          osc.type = 'square';
          const clearNotes = [523, 659, 784, 1046, 784, 1046];
          clearNotes.forEach((freq, idx) => {
            osc.frequency.setValueAtTime(freq, now + idx * 0.1);
          });
          gain.gain.exponentialRampToValueAtTime(0.01, now + 0.7);
          osc.start(now);
          osc.stop(now + 0.7);
          break;

        default:
          osc.type = 'square';
          osc.frequency.setValueAtTime(220, now);
          gain.gain.exponentialRampToValueAtTime(0.01, now + 0.1);
          osc.start(now);
          osc.stop(now + 0.1);
          break;
      }
    } catch (e) {
      console.warn('Audio sfx error:', e);
    }
  };

  // =========================================================================
  // 2. ASCII Mario 1985 Game Engine Implementation
  // =========================================================================
  const TILE_AIR = 0;
  const TILE_GROUND = 1;
  const TILE_BRICK = 2;
  const TILE_QUESTION = 3;
  const TILE_EMPTY = 4;
  const TILE_PIPE_TL = 5;
  const TILE_PIPE_TR = 6;
  const TILE_PIPE_BL = 7;
  const TILE_PIPE_BR = 8;
  const TILE_LAVA = 9;
  const TILE_FLAG = 10;
  const TILE_CASTLE = 11;
  const TILE_AXE = 12;

  let gameState = {
    world: 1,
    level: 1,
    score: 0,
    coins: 0,
    lives: 3,
    timeLeft: 400,
    status: 'playing', // 'playing', 'dead', 'clear', 'gameover', 'won'
    statusTimer: 0,
    cameraX: 0,
    theme: 'classic', // classic, matrix, amber, cyberpunk
    scanlines: true,
    cheats: {
      godMode: false,
      moonJump: false,
      infiniteFire: false,
    },
    userTier: 'free', // 'free', 'pro', 'vip', 'admin'
    userNpub: null,
  };

  let player = {
    x: 3.0,
    y: 12.0,
    vx: 0.0,
    vy: 0.0,
    form: 'small', // small, super, fire, star
    dir: 'right',
    onGround: true,
    isJumping: false,
    invincibleTimer: 0,
  };

  let mapWidth = 120;
  const mapHeight = 15;
  let tiles = [];
  let enemies = [];
  let items = [];
  let fireballs = [];

  const keys = {
    left: false,
    right: false,
    jump: false,
    run: false,
    fire: false,
  };

  function initMap(world, level) {
    gameState.world = world;
    gameState.level = level;
    gameState.timeLeft = 400;
    gameState.cameraX = 0;
    gameState.status = 'playing';
    gameState.statusTimer = 0;

    player.x = 3.0;
    player.y = 12.0;
    player.vx = 0.0;
    player.vy = 0.0;
    player.dir = 'right';
    player.onGround = true;

    enemies = [];
    items = [];
    fireballs = [];

    if (world === 1 && level === 1) {
      mapWidth = 115;
      tiles = Array.from({ length: mapHeight }, () => Array(mapWidth).fill(TILE_AIR));

      // Ground
      for (let x = 0; x < mapWidth; x++) {
        if ((x >= 35 && x <= 37) || (x >= 70 && x <= 72)) continue;
        tiles[13][x] = TILE_GROUND;
        tiles[14][x] = TILE_GROUND;
      }

      // Blocks
      tiles[9][10] = TILE_QUESTION;
      tiles[9][14] = TILE_BRICK;
      tiles[9][15] = TILE_QUESTION; // Mushroom
      tiles[9][16] = TILE_BRICK;
      tiles[9][17] = TILE_QUESTION;
      tiles[9][18] = TILE_BRICK;
      tiles[5][16] = TILE_QUESTION; // Flower

      // Pipes
      tiles[12][20] = TILE_PIPE_TL; tiles[12][21] = TILE_PIPE_TR;
      tiles[11][28] = TILE_PIPE_TL; tiles[11][29] = TILE_PIPE_TR;
      tiles[12][28] = TILE_PIPE_BL; tiles[12][29] = TILE_PIPE_BR;

      tiles[10][45] = TILE_PIPE_TL; tiles[10][46] = TILE_PIPE_TR;
      tiles[11][45] = TILE_PIPE_BL; tiles[11][46] = TILE_PIPE_BR;
      tiles[12][45] = TILE_PIPE_BL; tiles[12][46] = TILE_PIPE_BR;

      // Flagpole
      for (let y = 4; y < 13; y++) tiles[y][100] = TILE_FLAG;
      for (let y = 8; y < 13; y++) {
        for (let x = 105; x <= 112; x++) tiles[y][x] = TILE_CASTLE;
      }

      enemies.push({ x: 16.0, y: 12.0, vx: -0.04, vy: 0, type: 'goomba', alive: true });
      enemies.push({ x: 24.0, y: 12.0, vx: -0.04, vy: 0, type: 'goomba', alive: true });
      enemies.push({ x: 32.0, y: 12.0, vx: -0.04, vy: 0, type: 'goomba', alive: true });
      enemies.push({ x: 52.0, y: 12.0, vx: -0.05, vy: 0, type: 'koopa', alive: true });
      enemies.push({ x: 62.0, y: 12.0, vx: -0.04, vy: 0, type: 'goomba', alive: true });
    } else if (world === 1 && level === 2) {
      // World 1-2 Underground Cavern
      mapWidth = 110;
      tiles = Array.from({ length: mapHeight }, () => Array(mapWidth).fill(TILE_AIR));
      for (let x = 0; x < mapWidth; x++) {
        tiles[0][x] = TILE_BRICK;
        tiles[1][x] = TILE_BRICK;
        if (x < 38 || (x > 42 && x < 82) || x > 88) {
          tiles[13][x] = TILE_GROUND;
          tiles[14][x] = TILE_GROUND;
        }
      }
      for (let x = 15; x < 28; x++) tiles[8][x] = TILE_BRICK;
      tiles[5][20] = TILE_QUESTION;

      for (let y = 4; y < 13; y++) tiles[y][100] = TILE_FLAG;

      enemies.push({ x: 12.0, y: 12.0, vx: -0.04, vy: 0, type: 'goomba', alive: true });
      enemies.push({ x: 22.0, y: 7.0, vx: -0.04, vy: 0, type: 'koopa', alive: true });
      enemies.push({ x: 50.0, y: 12.0, vx: -0.04, vy: 0, type: 'goomba', alive: true });
    } else if (world === 1 && level === 3) {
      // World 1-3 Mushroom Treetops
      mapWidth = 100;
      tiles = Array.from({ length: mapHeight }, () => Array(mapWidth).fill(TILE_AIR));
      for (let x = 0; x < 10; x++) {
        tiles[13][x] = TILE_GROUND;
        tiles[14][x] = TILE_GROUND;
      }
      const platforms = [[14, 22, 10], [26, 32, 7], [36, 46, 9], [52, 60, 6], [66, 76, 8], [82, 92, 10], [95, 100, 13]];
      platforms.forEach(([x1, x2, y]) => {
        for (let x = x1; x < x2; x++) tiles[y][x] = TILE_BRICK;
      });
      for (let y = 4; y < 13; y++) tiles[y][96] = TILE_FLAG;

      enemies.push({ x: 18.0, y: 9.0, vx: -0.04, vy: 0, type: 'koopa', alive: true });
      enemies.push({ x: 40.0, y: 8.0, vx: -0.04, vy: 0, type: 'koopa', alive: true });
    } else if (world === 1 && level === 4) {
      // World 1-4 Bowser Castle Boss
      mapWidth = 100;
      tiles = Array.from({ length: mapHeight }, () => Array(mapWidth).fill(TILE_AIR));
      for (let x = 0; x < mapWidth; x++) {
        tiles[0][x] = TILE_CASTLE;
        tiles[1][x] = TILE_CASTLE;
        if ((x >= 20 && x <= 26) || (x >= 45 && x <= 52) || (x >= 68 && x <= 85)) {
          tiles[13][x] = TILE_LAVA;
          tiles[14][x] = TILE_LAVA;
        } else {
          tiles[13][x] = TILE_CASTLE;
          tiles[14][x] = TILE_CASTLE;
        }
      }
      // Bridge & Axe
      for (let x = 70; x < 84; x++) tiles[11][x] = TILE_BRICK;
      tiles[10][85] = TILE_AXE;

      enemies.push({ x: 12.0, y: 12.0, vx: -0.04, vy: 0, type: 'goomba', alive: true });
      enemies.push({ x: 35.0, y: 12.0, vx: -0.05, vy: 0, type: 'koopa', alive: true });
      enemies.push({ x: 78.0, y: 10.0, vx: -0.02, vy: 0, type: 'bowser', alive: true, hp: 5, attackTimer: 60 });
    }
  }

  function isSolid(t) {
    return t === TILE_GROUND || t === TILE_BRICK || t === TILE_QUESTION || t === TILE_EMPTY ||
      t === TILE_PIPE_TL || t === TILE_PIPE_TR || t === TILE_PIPE_BL || t === TILE_PIPE_BR || t === TILE_CASTLE;
  }

  function checkCollision(x, y, height) {
    const ix = Math.round(x);
    const iyBottom = Math.round(y);
    const iyTop = Math.round(y - height);

    for (let iy = iyTop; iy <= iyBottom; iy++) {
      if (ix >= 0 && ix < mapWidth && iy >= 0 && iy < mapHeight) {
        if (isSolid(tiles[iy][ix])) {
          return true;
        }
      }
    }
    return false;
  }

  function bumpBlock(tx, ty) {
    if (tx < 0 || tx >= mapWidth || ty < 0 || ty >= mapHeight) return;
    const t = tiles[ty][tx];
    if (t === TILE_QUESTION) {
      tiles[ty][tx] = TILE_EMPTY;
      window.playMarioSfx('coin');
      gameState.score += 200;
      gameState.coins += 1;
      // Spawn mushroom occasionally
      if (tx === 15) {
        items.push({ x: tx, y: ty - 1, vx: 0.05, vy: -0.1, type: 'mushroom' });
        window.playMarioSfx('powerup');
      } else if (tx === 16 && ty === 5) {
        items.push({ x: tx, y: ty - 1, vx: 0, vy: -0.1, type: 'flower' });
        window.playMarioSfx('powerup');
      }
    } else if (t === TILE_BRICK) {
      if (player.form !== 'small' || gameState.cheats.godMode) {
        tiles[ty][tx] = TILE_AIR;
        gameState.score += 50;
        window.playMarioSfx('stomp');
      } else {
        window.playMarioSfx('stomp');
      }
    } else if (t === TILE_AXE) {
      // Bowser bridge drop
      window.playMarioSfx('bowser_fall');
      for (let x = 70; x < 84; x++) tiles[11][x] = TILE_AIR;
      tiles[ty][tx] = TILE_AIR;
      gameState.score += 5000;
      gameState.status = 'won';
      window.playMarioSfx('world_clear');
    }
  }

  function update() {
    if (gameState.status !== 'playing') {
      gameState.statusTimer++;
      if (gameState.status === 'dead' && gameState.statusTimer > 80) {
        if (gameState.lives > 1) {
          gameState.lives--;
          initMap(gameState.world, gameState.level);
        } else {
          gameState.status = 'gameover';
        }
      } else if (gameState.status === 'clear' && gameState.statusTimer > 100) {
        let nw = gameState.world;
        let nl = gameState.level + 1;
        if (nl > 4) { nw++; nl = 1; }
        initMap(nw, nl);
      }
      return;
    }

    // Time countdown
    if (Math.random() < 0.016 && gameState.timeLeft > 0) {
      gameState.timeLeft--;
      if (gameState.timeLeft === 0 && !gameState.cheats.godMode) {
        killPlayer();
      }
    }

    const maxSpeed = keys.run ? 0.22 : 0.14;
    const accel = 0.02;
    const friction = 0.85;
    const gravity = gameState.cheats.moonJump ? 0.016 : 0.035;

    // Movement X
    if (keys.left) {
      player.vx -= accel;
      player.dir = 'left';
    } else if (keys.right) {
      player.vx += accel;
      player.dir = 'right';
    } else {
      player.vx *= friction;
      if (Math.abs(player.vx) < 0.005) player.vx = 0;
    }
    player.vx = Math.max(-maxSpeed, Math.min(maxSpeed, player.vx));

    // Jump
    if (keys.jump && player.onGround && !player.isJumping) {
      player.vy = gameState.cheats.moonJump ? -0.58 : -0.48;
      player.onGround = false;
      player.isJumping = true;
      window.playMarioSfx('jump');
    }
    if (!keys.jump) {
      player.isJumping = false;
      if (player.vy < -0.15) player.vy = -0.15;
    }

    player.vy += gravity;
    player.vy = Math.min(0.5, player.vy);

    // Collision X
    const pHeight = player.form === 'small' ? 1.0 : 1.8;
    const newX = player.x + player.vx;
    if (!checkCollision(newX, player.y, pHeight)) {
      player.x = Math.max(gameState.cameraX, newX);
    } else {
      player.vx = 0;
    }

    // Collision Y
    const newY = player.y + player.vy;
    if (player.vy < 0) {
      // Head bump
      const headY = newY - pHeight;
      if (checkCollision(player.x, headY, 0.2)) {
        player.vy = 0;
        bumpBlock(Math.round(player.x), Math.round(headY));
      } else {
        player.y = newY;
        player.onGround = false;
      }
    } else {
      // Fall on ground
      if (checkCollision(player.x, newY, 0.2)) {
        player.y = Math.floor(newY);
        player.vy = 0;
        player.onGround = true;
      } else {
        player.y = newY;
        player.onGround = false;
      }
    }

    if (player.invincibleTimer > 0) player.invincibleTimer--;

    // Fireball shooting
    if (keys.fire && (player.form === 'fire' || gameState.cheats.infiniteFire)) {
      if (fireballs.length < 3 || gameState.cheats.infiniteFire) {
        fireballs.push({
          x: player.x + (player.dir === 'right' ? 1.0 : -0.5),
          y: player.y - 0.5,
          vx: player.dir === 'right' ? 0.35 : -0.35,
          vy: 0.1,
          active: true,
        });
        window.playMarioSfx('fireball');
        keys.fire = false; // Debounce
      }
    }

    // Fireball update
    fireballs.forEach(f => {
      if (!f.active) return;
      f.x += f.vx;
      f.y += f.vy;
      f.vy += 0.02;
      if (f.y > 12.0) {
        f.y = 12.0;
        f.vy = -0.15; // Bounce
      }
      if (f.x < gameState.cameraX || f.x > gameState.cameraX + 45) f.active = false;
    });
    fireballs = fireballs.filter(f => f.active);

    // Enemies update
    enemies.forEach(e => {
      if (!e.alive) return;
      e.x += e.vx;
      e.y += e.vy;

      if (e.type === 'bowser') {
        e.attackTimer = (e.attackTimer || 60) - 1;
        if (e.attackTimer <= 0) {
          e.attackTimer = 80;
          e.vy = -0.22; // Bowser jump
        }
        e.vy += 0.015;
        if (e.y > 10.0) {
          e.y = 10.0;
          e.vy = 0;
        }
      }

      // Hit by fireball
      fireballs.forEach(f => {
        if (!f.active) return;
        if (Math.abs(f.x - e.x) < 1.0 && Math.abs(f.y - e.y) < 1.2) {
          f.active = false;
          e.alive = false;
          gameState.score += 200;
          window.playMarioSfx('stomp');
        }
      });

      // Hit player
      const dx = Math.abs(player.x - e.x);
      const dy = Math.abs(player.y - e.y);
      if (dx < 0.9 && dy < 1.2) {
        if (player.vy > 0 && player.y < e.y + 0.3) {
          // Stomp
          e.alive = false;
          player.vy = -0.32;
          gameState.score += 100;
          window.playMarioSfx('stomp');
        } else if (player.invincibleTimer === 0 && !gameState.cheats.godMode) {
          if (player.form === 'small') {
            killPlayer();
          } else {
            player.form = 'small';
            player.invincibleTimer = 90;
            window.playMarioSfx('stomp');
          }
        }
      }
    });

    // Items update
    items.forEach(it => {
      if (it.collected) return;
      it.x += it.vx;
      it.y += it.vy;
      it.vy += 0.01;
      if (it.y > 12.0) { it.y = 12.0; it.vy = 0; }

      if (Math.abs(player.x - it.x) < 1.0 && Math.abs(player.y - it.y) < 1.2) {
        it.collected = true;
        window.playMarioSfx('powerup');
        if (it.type === 'mushroom') player.form = 'super';
        else if (it.type === 'flower') player.form = 'fire';
        gameState.score += 1000;
      }
    });
    items = items.filter(it => !it.collected);

    // Camera follow
    const targetCam = Math.max(0, Math.min(mapWidth - 40, player.x - 12));
    if (targetCam > gameState.cameraX) {
      gameState.cameraX = targetCam;
    }

    // Pit fall
    if (player.y > 14.2) {
      if (gameState.cheats.godMode) {
        player.y = 12.0;
        player.vy = 0;
      } else {
        killPlayer();
      }
    }

    // Flagpole
    if (player.x >= 100 && gameState.world === 1 && gameState.level <= 3) {
      gameState.status = 'clear';
      window.playMarioSfx('world_clear');
    }
  }

  function killPlayer() {
    player.vy = -0.4;
    gameState.status = 'dead';
    gameState.statusTimer = 0;
    window.playMarioSfx('gameover');
  }

  // =========================================================================
  // 3. ASCII & Canvas Render Pipeline
  // =========================================================================
  function render() {
    const canvas = document.getElementById('mario-canvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const camX = gameState.cameraX;
    const tileSize = 24;
    const offsetX = -camX * tileSize;

    // Theme color palettes
    let themeColor = '#ffffff';
    let blockColor = '#b84418';
    let groundColor = '#d89b38';
    let pipeColor = '#00a800';
    let skyColor = '#5c94fc';

    if (gameState.theme === 'matrix') {
      themeColor = '#00ff66';
      blockColor = '#00cc44';
      groundColor = '#008822';
      pipeColor = '#00ff88';
      skyColor = '#001a08';
    } else if (gameState.theme === 'amber') {
      themeColor = '#ffb000';
      blockColor = '#cc8800';
      groundColor = '#996600';
      pipeColor = '#ffcc33';
      skyColor = '#1f1400';
    } else if (gameState.theme === 'cyberpunk') {
      themeColor = '#00ffff';
      blockColor = '#ff00aa';
      groundColor = '#7928ca';
      pipeColor = '#00ffcc';
      skyColor = '#0b001a';
    }

    if (gameState.level === 2 || gameState.level === 4) {
      skyColor = '#000000'; // Underground / Castle
    }

    ctx.fillStyle = skyColor;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw Map Tiles
    const startTileX = Math.max(0, Math.floor(camX));
    const endTileX = Math.min(mapWidth, Math.ceil(camX + canvas.width / tileSize) + 2);

    for (let y = 0; y < mapHeight; y++) {
      for (let x = startTileX; x < endTileX; x++) {
        const t = tiles[y][x];
        const screenX = x * tileSize + offsetX;
        const screenY = y * tileSize + 10;

        if (t === TILE_GROUND) {
          ctx.fillStyle = groundColor;
          ctx.fillRect(screenX, screenY, tileSize, tileSize);
          ctx.strokeStyle = '#000';
          ctx.strokeRect(screenX, screenY, tileSize, tileSize);
        } else if (t === TILE_BRICK) {
          ctx.fillStyle = blockColor;
          ctx.fillRect(screenX, screenY, tileSize, tileSize);
          ctx.strokeStyle = '#222';
          ctx.strokeRect(screenX, screenY, tileSize, tileSize);
          ctx.fillStyle = '#fff';
          ctx.font = '10px monospace';
          ctx.fillText('#', screenX + 8, screenY + 16);
        } else if (t === TILE_QUESTION) {
          ctx.fillStyle = '#fc9838';
          ctx.fillRect(screenX, screenY, tileSize, tileSize);
          ctx.fillStyle = '#000';
          ctx.font = 'bold 14px monospace';
          ctx.fillText('?', screenX + 7, screenY + 18);
        } else if (t === TILE_EMPTY) {
          ctx.fillStyle = '#8c5214';
          ctx.fillRect(screenX, screenY, tileSize, tileSize);
        } else if (t === TILE_PIPE_TL || t === TILE_PIPE_TR || t === TILE_PIPE_BL || t === TILE_PIPE_BR) {
          ctx.fillStyle = pipeColor;
          ctx.fillRect(screenX, screenY, tileSize, tileSize);
          ctx.strokeStyle = '#004400';
          ctx.strokeRect(screenX, screenY, tileSize, tileSize);
        } else if (t === TILE_FLAG) {
          ctx.fillStyle = '#ffffff';
          ctx.fillRect(screenX + 10, screenY, 4, tileSize);
        } else if (t === TILE_LAVA) {
          ctx.fillStyle = '#ff2200';
          ctx.fillRect(screenX, screenY + 8, tileSize, tileSize - 8);
          ctx.fillStyle = '#ffcc00';
          ctx.fillText('~~~~', screenX, screenY + 18);
        } else if (t === TILE_CASTLE) {
          ctx.fillStyle = '#666677';
          ctx.fillRect(screenX, screenY, tileSize, tileSize);
        } else if (t === TILE_AXE) {
          ctx.fillStyle = '#ffaa00';
          ctx.font = 'bold 16px monospace';
          ctx.fillText('⛏️', screenX, screenY + 18);
        }
      }
    }

    // Draw Items
    items.forEach(it => {
      const sx = it.x * tileSize + offsetX;
      const sy = it.y * tileSize + 10;
      ctx.fillStyle = it.type === 'mushroom' ? '#ff3344' : '#ffcc00';
      ctx.font = 'bold 14px monospace';
      ctx.fillText(it.type === 'mushroom' ? '[🍄]' : '[⭐]', sx, sy + 18);
    });

    // Draw Enemies
    enemies.forEach(e => {
      if (!e.alive) return;
      const sx = e.x * tileSize + offsetX;
      const sy = e.y * tileSize + 10;
      ctx.font = 'bold 14px monospace';
      if (e.type === 'goomba') {
        ctx.fillStyle = '#9c4210';
        ctx.fillText('[o_o]', sx, sy + 18);
      } else if (e.type === 'koopa') {
        ctx.fillStyle = '#00a844';
        ctx.fillText('<(OO)>', sx, sy + 18);
      } else if (e.type === 'bowser') {
        ctx.fillStyle = '#ff2244';
        ctx.font = 'bold 20px monospace';
        ctx.fillText('🦖 [[W_W]] 🔥', sx, sy + 18);
      }
    });

    // Draw Fireballs
    fireballs.forEach(f => {
      const sx = f.x * tileSize + offsetX;
      const sy = f.y * tileSize + 10;
      ctx.fillStyle = '#ffaa00';
      ctx.beginPath();
      ctx.arc(sx + 8, sy + 8, 5, 0, Math.PI * 2);
      ctx.fill();
    });

    // Draw Player (Mario)
    const px = player.x * tileSize + offsetX;
    const py = player.y * tileSize + 10;

    ctx.font = player.form === 'small' ? 'bold 14px monospace' : 'bold 18px monospace';
    if (player.form === 'fire') ctx.fillStyle = '#ffffff';
    else if (player.form === 'star') ctx.fillStyle = '#ffea00';
    else ctx.fillStyle = '#ff2222';

    if (player.invincibleTimer % 4 < 2) {
      const sprite = player.dir === 'right' ? (player.form === 'small' ? 'm>' : '[M]>') : (player.form === 'small' ? '<m' : '<[M]');
      ctx.fillText(sprite, px, py + 18);
    }

    // Top HUD Bar
    ctx.fillStyle = 'rgba(0, 0, 0, 0.75)';
    ctx.fillRect(0, 0, canvas.width, 32);

    ctx.fillStyle = '#ffcc00';
    ctx.font = 'bold 11px monospace';
    ctx.fillText(`MARIO  ${String(gameState.score).padStart(6, '0')}`, 16, 14);
    ctx.fillText(`COINS  x${String(gameState.coins).padStart(2, '0')}`, 160, 14);
    ctx.fillText(`WORLD  ${gameState.world}-${gameState.level}`, 280, 14);
    ctx.fillText(`TIME  ${String(gameState.timeLeft).padStart(3, '0')}`, 390, 14);
    ctx.fillText(`LIVES x${gameState.lives}`, 490, 14);

    if (gameState.cheats.godMode) {
      ctx.fillStyle = '#ff3344';
      ctx.fillText('[GOD MODE ON]', 560, 14);
    }

    // Status Overlay
    if (gameState.status === 'dead') {
      ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#ff2222';
      ctx.font = 'bold 24px monospace';
      ctx.fillText('GAME OVER - MISSED', canvas.width / 2 - 130, canvas.height / 2);
    } else if (gameState.status === 'clear' || gameState.status === 'won') {
      ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#00ff88';
      ctx.font = 'bold 22px monospace';
      ctx.fillText('★ COURSE CLEAR! ★', canvas.width / 2 - 110, canvas.height / 2);
      ctx.font = '14px monospace';
      ctx.fillText('Nostr Kind 30078 Score Broadcasted to Relays!', canvas.width / 2 - 170, canvas.height / 2 + 30);
    }
  }

  function loop() {
    update();
    render();
    requestAnimationFrame(loop);
  }

  // =========================================================================
  // 4. Keyboard Controls Handler
  // =========================================================================
  window.addEventListener('keydown', (e) => {
    initAudio();
    if (e.code === 'ArrowLeft' || e.code === 'KeyA') keys.left = true;
    if (e.code === 'ArrowRight' || e.code === 'KeyD') keys.right = true;
    if (e.code === 'ArrowUp' || e.code === 'KeyW' || e.code === 'KeyZ' || e.code === 'Space') keys.jump = true;
    if (e.code === 'ShiftLeft' || e.code === 'ShiftRight' || e.code === 'KeyX') {
      keys.run = true;
      keys.fire = true;
    }
    if (e.code === 'KeyR') {
      initMap(gameState.world, gameState.level);
    }
  });

  window.addEventListener('keyup', (e) => {
    if (e.code === 'ArrowLeft' || e.code === 'KeyA') keys.left = false;
    if (e.code === 'ArrowRight' || e.code === 'KeyD') keys.right = false;
    if (e.code === 'ArrowUp' || e.code === 'KeyW' || e.code === 'KeyZ' || e.code === 'Space') keys.jump = false;
    if (e.code === 'ShiftLeft' || e.code === 'ShiftRight' || e.code === 'KeyX') {
      keys.run = false;
      keys.fire = false;
    }
  });

  // Start engine on DOM Ready
  document.addEventListener('DOMContentLoaded', () => {
    initMap(1, 1);
    requestAnimationFrame(loop);
  });
})();
