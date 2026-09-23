// oraculo_driver.js — injectado no index.html do jogo EXPORTADO, antes do arranque.
//
// Ele NÃO toca na lógica da extensão. Faz três coisas:
//   1. troca o laço de jogo (requestAnimationFrame) por passos MANUAIS de dt fixo:
//      `game.getSceneStack().step(dt_ms)` — o mesmo `renderAndStep` que o laço real chama,
//      só que com o tempo escolhido por nós (o comentário do próprio runtimegame.ts
//      descreve este modo: «a game that is never started and only driven manually (as in tests)»);
//   2. escreve, antes de cada passo, os comandos do quadro nas variáveis de cena Cmd1..N/A1..N,
//      que os EVENTOS gerados pelo GDevelop lêem para chamar as acções públicas da extensão;
//   3. lê o estado pelas EXPRESSÕES/CONDIÇÕES públicas geradas (os mesmos métodos que um
//      evento chama: `getBehavior("Health").IsJustDamaged(null)`), em três momentos do quadro.
(function () {
  "use strict";

  const LEITURAS_EXPRESSOES = [
    "Health", "MaxHealth", "ShieldPoints", "MaxShield",
    "HealthRegenRate", "HealthRegenDelay", "DamageCooldownDuration", "DamageCooldownRemaining",
    "ChanceToDodge", "FlatDamageReduction", "PercentDamageReduction", "TimeSinceLastHit",
    "PreviousDamageTaken", "PreviousDamageToShield", "PreviousHealAmount",
    "ShieldRegenRate", "ShieldRegenDelay", "ShieldDuration", "ShieldTimeRemaining",
  ];
  const LEITURAS_CONDICOES = [
    "IsDead", "IsJustDamaged", "IsJustHealed", "IsJustDodged", "IsShieldJustDamaged",
    "IsShieldActive", "IsDamageCooldownActive",
    "HitAtLeastOnce", // privada na extensão, mas é uma condição gerada como as outras
  ];

  let gameRef = null;
  let registoAtual = null; // o registo do passo em curso (preenchido pelos ganchos)
  let lerAntes = true;

  function comportamento(objects) {
    if (!objects || objects.length === 0) return null;
    return objects[0].getBehavior("Health");
  }

  function numero(v) {
    // JSON não tem NaN/Infinity: codifica-os como texto para não os perder em silêncio.
    if (typeof v === "number" && !Number.isFinite(v)) return String(v);
    return v;
  }

  function ler(b) {
    const publico = {};
    for (const e of LEITURAS_EXPRESSOES) publico[e] = numero(b[e](null));
    for (const c of LEITURAS_CONDICOES) publico[c] = !!b[c](null);
    // Estado bruto: todos os `_get<Propriedade>()` gerados (as propriedades do comportamento).
    const bruto = {};
    let proto = Object.getPrototypeOf(b);
    const nomes = Object.getOwnPropertyNames(proto).filter(
      (n) => n.startsWith("_get") && typeof proto[n] === "function" && proto[n].length === 0
    );
    nomes.sort();
    for (const n of nomes) bruto[n.slice(4)] = numero(b[n]());
    const o = b.owner;
    const timers = {};
    for (const t of ["__Health.TimeSinceLastHit", "__Health.ShieldDuration"]) {
      timers[t] = numero(o.getTimerElapsedTimeInSecondsOrNaN(t));
    }
    return { publico, bruto, timers };
  }

  window.__oraculo = {
    antes(runtimeScene, objects) {
      if (!registoAtual || !lerAntes) return;
      const b = comportamento(objects);
      if (b) registoAtual.antes = ler(b);
    },
    depois(runtimeScene, objects) {
      if (!registoAtual) return;
      const b = comportamento(objects);
      if (b) registoAtual.depois = ler(b);
    },
  };

  // Gerador semeado (mulberry32) para Math.random — a extensão chama
  // gdjs.randomFloatInRange(0, 1), que chama Math.random(). Semear é do HARNESS.
  function mulberry32(a) {
    return function () {
      a |= 0; a = (a + 0x6d2b79f5) | 0;
      let t = Math.imul(a ^ (a >>> 15), 1 | a);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }

  function instalarGancho() {
    gdjs.RuntimeGame.prototype.startGameLoop = function () {
      // O que o startGameLoop real faz ANTES de ligar o requestAnimationFrame, e mais nada.
      gameRef = this;
      this._forceGameResolutionUpdate();
      const nome = this._getFirstSceneName();
      this.getSceneStack().replace({ sceneName: nome, clear: true });
      window.__pronto = true;
    };
  }

  window.__correr = function (cenario, ids, ranhuras, opcoes) {
    opcoes = opcoes || {};
    lerAntes = opcoes.lerAntes !== false;
    // Math.random é SEMPRE embrulhado para CONTAR os sorteios de cada passo (quem sorteia é a
    // extensão, via gdjs.randomFloatInRange). Com semente, guardam-se também os valores; sem
    // semente só a contagem (os valores mudariam entre corridas e partiriam o controlo).
    const semeado = cenario.semente !== null && cenario.semente !== undefined;
    const fonte = semeado ? mulberry32(cenario.semente >>> 0) : Math.random.bind(Math);
    Math.random = function () {
      const v = fonte();
      if (registoAtual) {
        registoAtual.sorteios.n += 1;
        if (semeado) registoAtual.sorteios.valores.push(v);
      }
      return v;
    };
    const stack = gameRef.getSceneStack();
    const cena = stack.getCurrentScene();
    const vars = cena.getVariables();
    const hero = cena.getObjects("Hero")[0];
    const registos = [];
    const inicial = ler(hero.getBehavior("Health"));
    let i = 0;
    for (const passo of cenario.passos) {
      const acoes = passo.acoes || [];
      if (acoes.length > ranhuras) throw new Error("mais acções que ranhuras num quadro");
      for (let s = 1; s <= ranhuras; s++) {
        const a = acoes[s - 1];
        if (a) {
          const id = ids[a[0]];
          if (!id) throw new Error("comando desconhecido: " + a[0]);
          vars.get("Cmd" + s).setNumber(id);
          vars.get("A" + s).setNumber(a.length > 1 ? a[1] : 0);
        } else {
          vars.get("Cmd" + s).setNumber(0);
          vars.get("A" + s).setNumber(0);
        }
      }
      vars.get("ChkIsDead").setNumber(0);
      registoAtual = { passo: i, acoes: acoes, antes: null, depois: null, sorteios: { n: 0, valores: [] } };
      stack.step(cenario.dt_ms);
      const tm = cena.getTimeManager();
      registoAtual.tempo_desde_inicio_ms = tm.getTimeFromStart();
      registoAtual.dt_efectivo_ms = tm.getElapsedTime();
      // «fim»: entre quadros (depois do doStepPostEvents e do render).
      registoAtual.fim = ler(hero.getBehavior("Health"));
      registoAtual.chk_por_eventos = {
        Health: vars.get("ChkHealth").getAsNumber(),
        ShieldPoints: vars.get("ChkShield").getAsNumber(),
        IsDead: vars.get("ChkIsDead").getAsNumber() === 1,
      };
      if (passo.nota) registoAtual.nota = passo.nota;
      registos.push(registoAtual);
      registoAtual = null;
      i++;
    }
    return { inicial, passos: registos };
  };

  if (typeof gdjs !== "undefined" && gdjs.RuntimeGame) instalarGancho();
  else throw new Error("oraculo_driver.js carregado antes do gdjs");
})();
