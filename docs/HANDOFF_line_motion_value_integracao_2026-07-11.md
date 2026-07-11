# HANDOFF de integração — linha `line/motion-value` (docs 16–21: valor completo + M3.1/M3.2/M3.3 + M4.1)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (= o tip ATUAL
  do main no momento da abertura → fork fresco, **sem drift pré-fork**).
- **SEIS fatias** na linha: fatia 4 (**Math + Compare**, doc 16) + fatia 5 (**Switch + On Change**, doc 17)
  → domínio de valor COMPLETO; fatia M3.1 (**Fibonacci + Twist**, doc 18) + M3.2 (**Scatter + Morph**,
  doc 19) + M3.3 (**Bend + Look At**, doc 20) → M3 (2 distribuições + 4 deformers); fatia **M4.1**
  (**Verlet-Rope + Boids**, doc 21) → abre a **família de SIMULAÇÃO** (dinâmica sequencial). Cada uma =
  2 crates-nó + a cena boot reescrita. **A cena boot atual demonstra a fatia M4.1** (duas cenas: um chicote
  de corda + um enxame); os nós das outras fatias ficam registrados/drop-in.
- **Auto-contida:** sem dependência de outra linha → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última reescrita = demo rope+boids, 2 cenas, 8 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + `build`→`Vec` de 2 sinks + contagem 8 + testes de integração (rope/boids) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 com doc PRÓPRIO (beat→strobe) — NÃO depende da cena boot; **intocado nesta fatia** | shell, módulo Motion; baixo |
| `Cargo.lock` | só as crates PATH novas | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado — as seis fatias são
100% fan-out de nós + cena. (Contraste com a linha anterior, que mexeu em `cook.rs`.)

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` são **GERADOS** (região `<ph2d-node-sync>`; **48 crates** agora). QUALQUER
  outra linha que adicione um nó conflita aqui.
- **Resolução — NÃO fundir à mão:** depois de juntar as árvores, rode **`cargo run -p ph2d-node-sync`**
  — regenera determinísticamente dos `crates/ph2d-node-*`; o gate `staleness` (em
  `ph2d-node-registry-init`) prova sync.

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **12 crates-nó novas**, tipos (string, namespaced, únicos): `value.math`/`pulse.compare` (doc 16);
  `value.switch`/`pulse.on_change` (doc 17); `motion.fibonacci`/`motion.twist` (doc 18);
  `motion.scatter`/`motion.morph` (doc 19); `motion.bend`/`motion.look_at` (doc 20);
  **`motion.verlet_rope`**/**`motion.boids`** (doc 21). Grep de colisão:
  `grep -rnE '"(value\.(math|switch)|pulse\.(compare|on_change)|motion\.(fibonacci|twist|scatter|morph|bend|look_at|verlet_rope|boids))"' crates/`.
- pub consts / colunas locais (sem registro global): os `VALUE`/`PULSE` por-crate, `cmp_armed`,
  `oc_prev`/`oc_primed`; colunas de estado das sims `rope_prev`/`sim_t` (rope) e `vel`/`sim_t` (boids) —
  **columns de stream locais, não símbolos**. Helpers copiados por-crate (convenção leaf, sem símbolo
  compartilhado): `trig.rs` (`cos_sin_cycles`, em fibonacci/twist/bend), `hash.rs` (`hash3`, em scatter +
  **boids**), e o `atan2_approx` inline (Rajan) em look_at.
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
- **`nextest-impacted` funciona** — a linha só ADICIONA crates (sem rename/delete), então o filterset
  não quebra. `advisory-db` local pode envelhecer → `ship.sh` roda audit fresco.
- fmt/machete/clippy `--all-targets`/HR-5/LOC/typos rodados verdes no fechamento com o toolchain pinado
  (§ "Gates" abaixo).

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → fast-forward limpo,
  depois `ship.sh`.
- **Se main moveu (outra linha integrou antes):** rebase em cima do main novo → esperar UM conflito
  mecânico em `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}` (resolve com `cargo run -p
  ph2d-node-sync`, **não à mão**) + `Cargo.lock` (regenera). O `motion_demo_strobe.rs`/`motion_state*`
  só conflitam se outra linha também editar a cena Motion (improvável — cada módulo tem a sua). Depois
  `scripts/foundational-integrate.sh` (gate da árvore combinada) + `ship.sh`.
- **Smoke: fatias 4, 5, M3.1, M3.2 e M3.3 APROVADAS (Enio, 2026-07-11); fatia M4.1 (rope+boids) PENDENTE.**
  A cena boot é sempre PEQUENA e isola a fatia mais recente (feedback do Enio: "com tantos nós fica difícil
  entender o conceito"). A cena atual demonstra a **M4.1** (headless provada por
  `the_verlet_rope_whips_from_the_moving_anchor` + `the_boids_flock_seeks_the_moving_target`). Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA um **chicote âmbar** que balança e cai de uma âncora que desliza (o
  `motion.verlet_rope`, anchor_x ← `value.lfo`); à DIREITA um **enxame ciano** que roda pra perseguir um
  alvo deslizante (o `motion.boids`, target_x ← `value.lfo`). No editor: dropar `motion.verlet_rope`
  (anchor_x/anchor_y) e `motion.boids` (target_x/target_y, sliders sep/align/cohesion/seek).

**Resumo:** *Linha `motion-value` com SEIS fatias (fork em `1c7c9a22`). Aditiva: 12 crates-nó (docs 16–17
→ domínio de valor completo; docs 18–20 → M3.1/M3.2/M3.3, 2 distribuições + 4 deformers; doc 21 → M4.1,
2 simulações sequenciais) + a cena boot pequena (demonstra a M4.1). Único conflito mecânico = codegen
`registry-init` → `ph2d-node-sync` (48 crates). Zero substrato, zero contrato congelado, zero dep externa.
Fatias 4/5/M3.1/M3.2/M3.3 smoke-aprovadas; M4.1 pendente. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (fatias 4–5 = valor completo · M3.1–M3.3 = M3 · M4.1 = simulação — docs 16–21)

Fecha TODOS os follow-ups nomeados dos docs 12–14 §5 (fatias 4–5 → vocabulário de valor COMPLETO), abre o
**M3** (M3.1 fibonacci+twist, M3.2 scatter+morph, M3.3 bend+look_at — 2 distribuições + 4 deformers) e abre a
**família de SIMULAÇÃO** (M4.1 verlet-rope+boids — dinâmica sequencial), sempre pesquisando o padrão-ouro
ANTES de codar (DIRETIVA §1). Detalhe + pesquisa:
[`16`](Motion%20Nodes/16_math_compare_nota_adr.md), [`17`](Motion%20Nodes/17_switch_on_change_nota_adr.md),
[`18`](Motion%20Nodes/18_fibonacci_twist_nota_adr.md), [`19`](Motion%20Nodes/19_scatter_morph_nota_adr.md),
[`20`](Motion%20Nodes/20_bend_look_at_nota_adr.md),
[`21_verlet_rope_boids_nota_adr.md`](Motion%20Nodes/21_verlet_rope_boids_nota_adr.md).

**Fatia 4 (doc 16):** `value.math` (combinador de 2 campos, 6 ops, broadcast; `Pure`) + `pulse.compare`
(ponte valor→pulse, Schmitt; sequencial, `Pure`).

**Fatia 5 (doc 17):** `value.switch` (mux `select`+`in0..3`, roteia por broadcast; `Pure`) + `pulse.on_change`
(detector de mudança `|v−prev|>ε`; sequencial, prime na 1ª tick, `Pure`).

**Fatia M3.1 (doc 18):** `motion.fibonacci` (phyllotaxis de Vogel, Source, `cos_sin_cycles`; `Pure`) +
`motion.twist` (deformer rotacional, `amount` value input, falloff-masked; `Pure`).

**Fatia M3.2 (doc 19):** `motion.scatter` (blue-noise por Mitchell best-candidate, count exato, Source,
stateless hash; `Pure`) + `motion.morph` (crossfade `lerp(a,b,blend)`, `blend` value input; `Pure`).

**Fatia M3.3 (doc 20):** `motion.bend` (arc-wrap, preserva comprimento de arco, `amount` value input,
`cos_sin_cycles`+π; `Pure`) + `motion.look_at` (orient-toward, `rot=atan2(target−pos)`, Rajan approx,
alvo value inputs; `Pure`).

**Fatia M4.1 (doc 21) — abre a família de SIMULAÇÃO (dinâmica sequencial no `pre` self-loop):**
- **`motion.verlet_rope`** (`ph2d-node-motion-verlet-rope`): a **corda/chicote** por **Verlet posicional de
  Jakobsen (2001)** — `count` pontos, integração `x'=x+(x−x_prev)(1−damp)+a·dt²` + passes de relaxação de
  distância; cabeça fixada na **âncora animável** (`anchor_x`/`anchor_y` value inputs), `pin_tail` opcional.
  Incondicionalmente estável, `sqrt` só no comprimento do segmento. Sequencial (`pre`), `Temporal`, Source.
- **`motion.boids`** (`ph2d-node-motion-boids`): o **enxame** por **boids de Reynolds (1987)** —
  separação/alinhamento/coesão sobre vizinhos dentro de `radius` (O(N²)) + **seek** p/ um alvo animável
  (mola linear → também a coleira que limita). Sequencial (`pre`, estado `P`+`vel`), `Temporal`, Source,
  seed hasheado (`hash.rs`). `sqrt` só na normalização.
- **Cena boot** (a última reescrita): PEQUENA (8 nós, 2 cenas), um **chicote** (rope, anchor_x ← lfo) à
  esquerda + um **enxame** (boids, target_x ← lfo) à direita. Duas sims sequenciais — uma restrita, uma
  emergente. Vários `motion.output` compõem num só draw.

## 1. Gates no fechamento (paridade §7) — última fatia (M4.1)
- **Unit:** `motion.verlet_rope` 9 + `motion.boids` 9 (7 sim + 2 hash) — falsificados. Fatias anteriores:
  math 7 / compare 7 / switch 5 / on_change 6 / fibonacci 6 / twist 7 / scatter 6 / morph 6 / bend 7 /
  look_at 6.
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop motion`
  = **24 passed** (inclui `the_verlet_rope_whips_from_the_moving_anchor` +
  `the_boids_flock_seeks_the_moving_target` + `the_default_document_replays_deterministically` [drena o pump
  sequencial completo] + loop-replay [doc próprio beat→strobe] + motion_bridge params).
- **Contrato:** `ph2d-nodegraph --test architecture_contract_surface` = 3 pass (NodeOp=2/OpResolver=1/
  NodeManifest=8, intacto).
- **Registry:** `ph2d-node-registry-init --test staleness` = 2 pass (em sync após `node-sync`, **48 crates**).
- **Lint/estilo:** `clippy --all-targets -D warnings` (crates novas + shell) = 0 warnings · `cargo fmt` pin
  1.95 (style_edition 2024) · `typos` 0 (bare, docs excluídos) · `cargo machete` 0 · sweep HR-5 = 0 na
  produção (rope: Verlet + relaxação + `sqrt`; boids: steering + `sqrt` na normalização; nenhuma chamada
  transcendental — `hypot` de teste substituído por distância²).
- **LOC:** M4.1 `motion.verlet-rope` 616 / `motion.boids` 666 (cap 700; folga menor — campo novo → orçar
  split); shell `motion_demo_strobe.rs` 180 / `motion_state_tests.rs` 154 / `motion_state.rs` 119 (cap 600).

## 2. Follow-up restante (doc 21 §5) — simulação + M3 em andamento
Valor COMPLETO (4–5); M3 com M3.1–M3.3; simulação aberta com M4.1. O que segue (doc 01 §3):
- **Simulação (continuação):** `motion.soft_body` (XPBD — malha 2D) · `motion.pin_constraint` (fixa
  instâncias de um stream simulado) · **spatial hash** no boids (só-perf, O(N²)→O(N)).
- **Distribuições (M3):** `motion.distribute-voronoi` (Lloyd) · `-path` (vector.*) · `motion.lattice`.
- **Deformers (M3):** `motion.four-point-warp` · `-slit-scan`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

*"Linha `motion-value` com 6 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
