# HANDOFF de integração — linha `line/motion-value` (docs 16–22: valor completo + M3.1–3.3 + M4.1–4.2)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (= o tip ATUAL
  do main no momento da abertura → fork fresco, **sem drift pré-fork**).
- **SETE fatias** na linha: fatia 4 (**Math + Compare**, doc 16) + fatia 5 (**Switch + On Change**, doc 17)
  → domínio de valor COMPLETO; M3.1 (**Fibonacci + Twist**, doc 18) + M3.2 (**Scatter + Morph**, doc 19) +
  M3.3 (**Bend + Look At**, doc 20) → M3 (2 distribuições + 4 deformers); **M4.1** (**Verlet-Rope + Boids**,
  doc 21) + **M4.2** (**Soft-Body + Wave**, doc 22) → família de SIMULAÇÃO (2 sims de partículas discretas +
  2 de mídia contínua). Cada uma = 2 crates-nó + a cena boot reescrita. **A cena boot atual demonstra a M4.2**
  (duas cenas: um jelly + um campo de ondas); os nós das outras fatias ficam registrados/drop-in.
- **Auto-contida:** sem dependência de outra linha → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última reescrita = demo soft_body+wave, 2 cenas, 8 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + `build`→`Vec` de 2 sinks + contagem 8 + testes de integração (soft_body/wave) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 com doc PRÓPRIO (beat→strobe) — NÃO depende da cena boot; **intocado** | shell, módulo Motion; baixo |
| `Cargo.lock` | só as crates PATH novas | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado — as sete fatias são
100% fan-out de nós + cena.

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` são **GERADOS** (região `<ph2d-node-sync>`; **50 crates** agora). QUALQUER
  outra linha que adicione um nó conflita aqui.
- **Resolução — NÃO fundir à mão:** depois de juntar as árvores, rode **`cargo run -p ph2d-node-sync`**
  — regenera determinísticamente dos `crates/ph2d-node-*`; o gate `staleness` (em
  `ph2d-node-registry-init`) prova sync.

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **14 crates-nó novas**, tipos (string, namespaced, únicos): `value.math`/`pulse.compare` (16);
  `value.switch`/`pulse.on_change` (17); `motion.fibonacci`/`motion.twist` (18); `motion.scatter`/
  `motion.morph` (19); `motion.bend`/`motion.look_at` (20); `motion.verlet_rope`/`motion.boids` (21);
  **`motion.soft_body`**/**`motion.wave`** (22). Grep de colisão:
  `grep -rnE '"(value\.(math|switch)|pulse\.(compare|on_change)|motion\.(fibonacci|twist|scatter|morph|bend|look_at|verlet_rope|boids|soft_body|wave))"' crates/`.
- Helpers copiados por-crate (convenção leaf, sem símbolo compartilhado): `trig.rs` (`cos_sin_cycles`, em
  fibonacci/twist/bend), `hash.rs` (`hash3`, em scatter/boids), `atan2_approx` inline (Rajan) em look_at.
  **M4.2 é 100% self-contained** (nem hash/trig — só aritmética + `sqrt`). Colunas de estado de stream novas
  (`rope_prev`, `vel`, `sb_vel`, `wave_h`/`wave_prev`, `sim_t`) são **locais, não símbolos**.
- **ZERO** `NodeId` numérico / token / variant de enum congelado novos. **ZERO dep EXTERNA nova**
  (só path crates) → machete/deny/audit/RUSTSEC não mexem.

### 4. Contratos congelados encostados: **NENHUM**
Gate `architecture_contract_surface` verde (`NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8) — rodado no
fechamento. Fan-out aditivo (caminho A). Sem ADR necessário. As sims são **sequenciais** mas usam o
substrato `pre`/`state` que JÁ existe (como `motion.spring`/`motion.integrate`) — nada novo no substrato.

### 5. O que só o `ship.sh` pega (o gate `foundational-integrate.sh` NÃO roda fmt/typos/machete/deny)
- **Drift pré-fork: BAIXO** — fork == tip atual do main (`1c7c9a22`), então fmt (style_edition 2024,
  pin 1.95)/typos batem com o main de hoje. ([[project_integration_prefork_lines_ship_drift]] não morde
  aqui.) Ainda assim rode **`ship.sh` completo** na árvore combinada.
- **`nextest-impacted` funciona** — a linha só ADICIONA crates (sem rename/delete). `advisory-db` local pode
  envelhecer → `ship.sh` roda audit fresco.
- fmt/machete/clippy `--all-targets -D warnings`/HR-5/LOC/typos rodados verdes no fechamento com o toolchain
  pinado (§ "Gates" abaixo).

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → fast-forward limpo,
  depois `ship.sh`.
- **Se main moveu (outra linha integrou antes):** rebase → esperar UM conflito mecânico em
  `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}` (resolve com `cargo run -p ph2d-node-sync`, **não à
  mão**) + `Cargo.lock` (regenera). O `motion_demo_strobe.rs`/`motion_state*` só conflitam se outra linha
  também editar a cena Motion (improvável). Depois `scripts/foundational-integrate.sh` + `ship.sh`.
- **Smoke: fatias 4, 5, M3.1, M3.2, M3.3, M4.1 APROVADAS (Enio, 2026-07-11); fatia M4.2 (soft_body+wave)
  PENDENTE.** A cena boot é sempre PEQUENA e isola a fatia mais recente (feedback do Enio). A cena atual
  demonstra a **M4.2** (headless provada por `the_soft_body_hangs_and_wobbles_from_the_moving_anchor` +
  `the_wave_ripples_outward_from_the_driven_center`). Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA um **jelly magenta** que pendura e balança de uma âncora que desliza (o
  `motion.soft_body`, anchor_x ← `value.lfo`); à DIREITA um **campo ciano** de pontos que incham em **anéis
  concêntricos** radiando do centro (o `motion.wave`, drive ← `value.lfo`). No editor: dropar
  `motion.soft_body` (anchor_x/anchor_y, sliders stiffness/gravity/pin) e `motion.wave` (drive, sliders
  speed/damping/center_x/center_y).

**Resumo:** *Linha `motion-value` com SETE fatias (fork em `1c7c9a22`). Aditiva: 14 crates-nó (16–17 → valor
completo; 18–20 → M3, 2 distribuições + 4 deformers; 21–22 → simulação, 2 sims discretas + 2 contínuas) + a
cena boot pequena (demonstra a M4.2). Único conflito mecânico = codegen `registry-init` → `ph2d-node-sync`
(50 crates). Zero substrato, zero contrato congelado, zero dep externa. Fatias 4/5/M3.1–3.3/M4.1
smoke-aprovadas; M4.2 pendente. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (docs 16–22)

Fecha os follow-ups de valor (4–5), abre e avança o **M3** (M3.1–3.3, 2 distribuições + 4 deformers) e abre a
família de **SIMULAÇÃO** (M4.1 partículas discretas, M4.2 mídia contínua), sempre pesquisando o padrão-ouro
ANTES de codar (DIRETIVA §1). Pesquisa por fatia:
[`16`](Motion%20Nodes/16_math_compare_nota_adr.md), [`17`](Motion%20Nodes/17_switch_on_change_nota_adr.md),
[`18`](Motion%20Nodes/18_fibonacci_twist_nota_adr.md), [`19`](Motion%20Nodes/19_scatter_morph_nota_adr.md),
[`20`](Motion%20Nodes/20_bend_look_at_nota_adr.md), [`21`](Motion%20Nodes/21_verlet_rope_boids_nota_adr.md),
[`22_soft_body_wave_nota_adr.md`](Motion%20Nodes/22_soft_body_wave_nota_adr.md).

**Fatia 4 (16):** `value.math` (combinador de 2 campos, broadcast) + `pulse.compare` (valor→pulse Schmitt).
**Fatia 5 (17):** `value.switch` (mux) + `pulse.on_change` (detector de mudança).
**M3.1 (18):** `motion.fibonacci` (phyllotaxis Vogel) + `motion.twist` (deformer rotacional).
**M3.2 (19):** `motion.scatter` (blue-noise Mitchell) + `motion.morph` (crossfade).
**M3.3 (20):** `motion.bend` (arc-wrap) + `motion.look_at` (orient-toward, Rajan atan2).
**M4.1 (21):** `motion.verlet_rope` (Verlet/Jakobsen) + `motion.boids` (Reynolds) — partículas discretas.

**Fatia M4.2 (22) — mídia contínua na simulação (sequencial, `pre` self-loop):**
- **`motion.soft_body`** (`ph2d-node-motion-soft-body` + módulo irmão `shape.rs`): **shape-matching (Müller
  2005)** — malha `rows×cols` que amassa e volta à forma; frame por **decomposição polar 2D em forma fechada**
  (só `sqrt`, sem trig) + **modo linear β `stretch`** (squash & stretch, área-preservado); integração
  **PBD/Ten-Minute-Physics** (predict→project→derivar-velocidade). Fileira de cima fixada na âncora animável.
  `Temporal`, Source. (Verificado contra as equações canônicas dos autores; atribuição corrigida.)
- **`motion.wave`** (`ph2d-node-motion-wave`): **equação de onda por diferenças finitas, leapfrog** — grade
  `rows×cols`, Laplaciano de 5 pontos (Neumann), `C=(c·dt)²` clampado sob CFL 0.5, centro dirigido por
  `drive`; altura → `size` (anéis concêntricos). Aritmética pura. `Temporal`, Source.
- **Cena boot** (a última reescrita): PEQUENA (8 nós, 2 cenas), um **jelly** (soft_body, anchor_x ← lfo) à
  esquerda + um **campo de ondas** (wave, drive ← lfo) à direita — um corpo deformável, um campo que propaga.

## 1. Gates no fechamento (paridade §7) — última fatia (M4.2)
- **Unit:** `motion.soft_body` 11 (7 sim em `lib.rs` + 4 geometria em `shape.rs`, inclui modo-linear β) +
  `motion.wave` 7 (falsificados). Fatias anteriores: math 7 / compare 7 /
  switch 5 / on_change 6 / fibonacci 6 / twist 7 / scatter 6 / morph 6 / bend 7 / look_at 6 / verlet_rope 9 /
  boids 9.
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop motion`
  = **24 passed** (inclui `the_soft_body_hangs_and_wobbles_from_the_moving_anchor` +
  `the_wave_ripples_outward_from_the_driven_center` + `the_default_document_replays_deterministically`
  [drena o pump sequencial] + loop-replay [doc próprio] + motion_bridge params).
- **Contrato:** `ph2d-nodegraph --test architecture_contract_surface` = 3 pass (2/1/8, intacto).
- **Registry:** `ph2d-node-registry-init --test staleness` = 2 pass (em sync após `node-sync`, **50 crates**).
- **Lint/estilo:** `clippy --all-targets -D warnings` (crates novas + shell) = 0 warnings · `cargo fmt` pin
  1.95 · `typos` 0 · `cargo machete` 0 · sweep HR-5 = 0 na produção (soft_body: `sqrt` só na polar 2D; wave:
  aritmética pura, nem `sqrt`; testes sem `hypot`/`powi`).
- **LOC:** M4.2 `motion.soft-body` 659 / `motion.wave` 516 (cap 700); shell `motion_demo_strobe.rs` 183 /
  `motion_state_tests.rs` 148 / `motion_state.rs` 119 (cap 600).

## 2. Follow-up restante (doc 22 §5) — simulação + M3 em andamento
- **Simulação:** `motion.pin_constraint` (fixa instâncias de um stream simulado — **precisa de um port de
  cadeia de constraints no sim**, à la `motion.integrate.forces`; DEFERIDO até esse design) · **spatial hash**
  no boids (só-perf) · colisão entre corpos.
- **Distribuições (M3):** `motion.distribute-voronoi` (Lloyd) · `-path` (vector.*) · `motion.lattice`.
- **Deformers (M3):** `motion.four-point-warp` · `-slit-scan`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

*"Linha `motion-value` com 7 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
