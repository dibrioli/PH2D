# HANDOFF de integração — linha `line/motion-value` (docs 16–20: valor completo + M3.1/M3.2/M3.3)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (= o tip ATUAL
  do main no momento da abertura → fork fresco, **sem drift pré-fork**).
- **CINCO fatias** na linha: fatia 4 (**Math + Compare**, doc 16) + fatia 5 (**Switch + On Change**, doc 17)
  → domínio de valor COMPLETO; fatia M3.1 (**Fibonacci + Twist**, doc 18) + M3.2 (**Scatter + Morph**,
  doc 19) + M3.3 (**Bend + Look At**, doc 20) → M3 (2 distribuições + 4 deformers). Cada uma = 2 crates-nó
  + a cena boot reescrita. **A cena boot atual demonstra a fatia M3.3** (uma grade que encurva seguindo um
  alvo); os nós das outras fatias ficam registrados/drop-in.
- **Auto-contida:** sem dependência de outra linha → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última reescrita = demo switch+on_change, ~11 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + contagem 11 + testes de integração (switch/on_change) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 ajustado à cena atual (LAP 45→90, sinal = max vermelho) | shell, módulo Motion; baixo |
| `Cargo.lock` | só as 4 crates PATH novas | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado — as duas fatias são
100% fan-out de nós + cena. (Contraste com a linha anterior, que mexeu em `cook.rs`.)

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` são **GERADOS** (região `<ph2d-node-sync>`; 40 crates agora). QUALQUER
  outra linha que adicione um nó conflita aqui.
- **Resolução — NÃO fundir à mão:** depois de juntar as árvores, rode **`cargo run -p ph2d-node-sync`**
  — regenera determinísticamente dos `crates/ph2d-node-*`; o gate `staleness` (em
  `ph2d-node-registry-init`) prova sync.

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **10 crates-nó novas**, tipos (string, namespaced, únicos): `value.math`/`pulse.compare` (doc 16);
  `value.switch`/`pulse.on_change` (doc 17); `motion.fibonacci`/`motion.twist` (doc 18);
  `motion.scatter`/`motion.morph` (doc 19); **`motion.bend`**/**`motion.look_at`** (doc 20). Grep de colisão:
  `grep -rnE '"(value\.(math|switch)|pulse\.(compare|on_change)|motion\.(fibonacci|twist|scatter|morph|bend|look_at))"' crates/`.
- pub consts / colunas locais (sem registro global): os `VALUE`/`PULSE` por-crate, `cmp_armed`,
  `oc_prev`/`oc_primed`. Helpers copiados por-crate (convenção leaf, sem símbolo compartilhado): `trig.rs`
  (`cos_sin_cycles`, em fibonacci/twist/bend, de `motion.orbit`), `hash.rs` (`hash3`, em scatter, de
  `value.instance_field`), e o `atan2_approx` inline (Rajan) em look_at.
- **ZERO** `NodeId` numérico / token / variant de enum congelado novos. **ZERO dep EXTERNA nova**
  (só path crates) → machete/deny/audit/RUSTSEC não mexem.

### 4. Contratos congelados encostados: **NENHUM**
Gate `architecture_contract_surface` verde (`NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8) — rodado no
fechamento. Fan-out aditivo (caminho A). Sem ADR necessário.

### 5. O que só o `ship.sh` pega (o gate `foundational-integrate.sh` NÃO roda fmt/typos/machete/deny)
- **Drift pré-fork: BAIXO** — fork == tip atual do main (`1c7c9a22`), então fmt (style_edition 2024,
  pin 1.95)/typos batem com o main de hoje. ([[project_integration_prefork_lines_ship_drift]] não morde
  aqui.) Ainda assim rode **`ship.sh` completo** na árvore combinada.
- **`nextest-impacted` funciona** — a linha só ADICIONA crates (sem rename/delete), então o filterset
  não quebra (contraste com [[feedback_ship_parity_gaps_ci_only]], que mordeu no cutover que deletava
  crates). `advisory-db` local pode envelhecer → `ship.sh` roda audit fresco.
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
- **Smoke: fatias 4, 5, M3.1 e M3.2 APROVADAS (Enio, 2026-07-11); fatia M3.3 (bend+look_at) PENDENTE.**
  Desde o feedback do Enio ("com tantos nós fica difícil entender o conceito") a cena boot é sempre PEQUENA
  e isola a fatia mais recente. A cena atual demonstra a **M3.3** (headless provado por
  `the_bend_curls_the_grid_over_time` + `the_look_at_aims_each_square_at_the_moving_target`). Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → uma **grade 4×5 que encurva pra cima e pra baixo** como uma onda (o `motion.bend`,
  amount ← `value.lfo`) enquanto **cada quadrado gira pra seguir um alvo** que desliza esquerda↔direita
  (o `motion.look_at`, target_x ← `value.lfo`). No editor: dropar `motion.bend` (amount) e `motion.look_at`
  (target_x/target_y).

**Resumo:** *Linha `motion-value` com CINCO fatias (fork em `1c7c9a22`). Aditiva: 10 crates-nó (docs 16–17
→ domínio de valor completo; docs 18–20 → M3.1/M3.2/M3.3, 2 distribuições + 4 deformers) + a cena boot
pequena (demonstra a M3.3). Único conflito mecânico = codegen `registry-init` → `ph2d-node-sync`
(46 crates). Zero substrato, zero contrato congelado, zero dep externa. Fatias 4/5/M3.1/M3.2
smoke-aprovadas; M3.3 pendente. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (fatias 4–5 = valor completo · M3.1–M3.3 = M3 — docs 16–20)

Fecha TODOS os follow-ups nomeados dos docs 12–14 §5 (fatias 4–5 → vocabulário de valor COMPLETO) e abre
o **M3** com três fatias (M3.1 fibonacci+twist, M3.2 scatter+morph, M3.3 bend+look_at — 2 distribuições +
4 deformers), sempre pesquisando o padrão-ouro ANTES de codar (DIRETIVA §1). Detalhe + pesquisa:
[`16`](Motion%20Nodes/16_math_compare_nota_adr.md), [`17`](Motion%20Nodes/17_switch_on_change_nota_adr.md),
[`18`](Motion%20Nodes/18_fibonacci_twist_nota_adr.md), [`19`](Motion%20Nodes/19_scatter_morph_nota_adr.md),
[`20_bend_look_at_nota_adr.md`](Motion%20Nodes/20_bend_look_at_nota_adr.md).

**Fatia 4 (doc 16):**
- **`value.math`** (`ph2d-node-value-math`): o **1º combinador de DOIS campos** — 6 ops
  (Add/Subtract/Multiply/Divide/Min/Max) via `op` enum, **exercendo a regra de broadcast 1→N entre dois
  campos**. UM nó multi-op (TD Math CHOP / Cavalry Math), divide guardado. HR-5, `Pure`.
- **`pulse.compare`** (`ph2d-node-pulse-compare`): a **ponte valor→pulse** (dual do `sample_hold`) —
  Schmitt (`rise`/`fall`/`edge`), núcleo portado **verbatim** do `pulse.threshold` (só muda o domínio de
  entrada: campo `v` vs canal). Sequencial (`cmp_armed` no `pre`), `Pure`.

**Fatia 5 (doc 17):**
- **`value.switch`** (`ph2d-node-value-switch`): o **roteador / multiplexador** — `select` (valor,
  animável) + `in0..in3`, roteia por `clamp(round(select_i),0,N-1)` sob broadcast (TD Switch CHOP /
  Houdini Switch VOP / Nuke Switch). Per-element por construção. `Pure`.
- **`pulse.on_change`** (`ph2d-node-pulse-on-change`): o **detector de mudança** (Max/Pd `change`) — dispara
  quando `|v−prev| > epsilon` (dual do compare: derivada, não nível). Sequencial (`oc_prev`/`oc_primed` no
  `pre`), prime na 1ª tick, `Pure`.
**Fatia M3.1 (doc 18) — abre o M3:**
- **`motion.fibonacci`** (`ph2d-node-motion-fibonacci`): a distribuição **phyllotaxis** de Vogel —
  `count` sementes em `r = spacing·√i`, ângulo `i·golden`. Source node (como `motion.grid`), count capado,
  trig transcendental-free (`cos_sin_cycles`, copiado de `motion.orbit`). `Pure`.
- **`motion.twist`** (`ph2d-node-motion-twist`): o **deformer** Twist — rotaciona cada elemento por
  `angle·amount_i·(r/r_max)` (borda gira, centro fica; preserva raio). O `amount` é um **input de valor**
  (animável por `value.lfo`); desconectado → 1.0. Falloff-masked. `Pure`.
- **Cena boot** (a última reescrita): PEQUENA (~8 nós), um **girassol que torce** — `fibonacci` → `twist`
  (amount ← `value.lfo`), sementes graduadas por `instance_field`. Playhead-pura (sem estado `pre`).

**Fatia M3.2 (doc 19) — blue-noise + crossfade:**
- **`motion.scatter`** (`ph2d-node-motion-scatter`): a distribuição **blue-noise** — `count` pontos por
  **Mitchell best-candidate** (K=12 dardos/ponto, o mais distante vence), count EXATO (não Bridson, que
  dá count implícito). Source node, stateless (hash). `Pure`.
- **`motion.morph`** (`ph2d-node-motion-morph`): o deformer **crossfade** — `lerp(a_i, b_i, blend_i)`; o
  `blend` é um **input de valor** (animável), per-element, saída `min(len)`. `Pure`.
- **Cena boot** (reescrita): PEQUENA (~9 nós), um **girassol que dissolve numa nuvem** —
  `fibonacci`(a) + `scatter`(b) → `morph` (blend ← `value.lfo`). Ordem ⇄ aleatoriedade. Playhead-pura.

**Fatia M3.3 (doc 20) — dois deformers:**
- **`motion.bend`** (`ph2d-node-motion-bend`): o **arc-wrap** — dobra a extensão X num arco de
  `angle·amount` (centro fica, borda encurva; preserva comprimento de arco). `amount` value input
  (desconectado → 1.0), falloff-masked. Trig `cos_sin_cycles` + constante π. `Pure`.
- **`motion.look_at`** (`ph2d-node-motion-look-at`): o **orient-toward** — escreve `rot = atan2(target −
  pos) + offset`; alvo value inputs (desconectado → origem). `atan2` pela aproximação de **Rajan** (~0.09°,
  transcendental-free). `Pure`.
- **Cena boot** (a última reescrita): PEQUENA (~7 nós), uma **grade que encurva seguindo um alvo** —
  `bend`(amount ← lfo) + `look_at`(target_x ← lfo). Playhead-pura.

## 1. Gates no fechamento (paridade §7) — última fatia (M3.3)
- **Unit:** `motion.bend` 7 + `motion.look_at` 6 (falsificados). Fatias anteriores: math 7 / compare 7 /
  switch 5 / on_change 6 / fibonacci 6 / twist 7 / scatter 6 / morph 6.
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bins motion` = **24 passed**
  (inclui `the_bend_curls_the_grid_over_time` + `the_look_at_aims_each_square_at_the_moving_target` +
  loop-replay com doc próprio + motion_bridge params). `ph2d-eval-motion` + membrane gate verdes.
- **Contrato:** `ph2d-nodegraph --test architecture_contract_surface` = 3 pass (NodeOp=2/OpResolver=1/
  NodeManifest=8, intacto).
- **Registry:** `ph2d-node-registry-init --test staleness` = 2 pass (em sync após `node-sync`, 46 crates).
- **Lint/estilo:** `clippy --all-targets` (crates novas + registry-init + shell `--bins`) = 0 warnings ·
  `cargo fmt` pin 1.95 · `typos` 0 (bare, docs excluídos) · `cargo machete` 0 · sweep HR-5 = 0 na produção
  (bend usa `cos_sin_cycles` + π; look_at usa a Rajan `atan2_approx`, nenhuma chamada transcendental).
- **LOC:** M3.3 `motion.look_at` 332 / `motion.bend` 311 (cap 700); shell `motion_demo_strobe.rs` 125 /
  `motion_state_tests.rs` 158 / `motion_state.rs` 118 (cap 600). Todos folgados.

## 2. Follow-up restante (doc 20 §5) — M3 em andamento
Valor COMPLETO (4–5); M3 com M3.1–M3.3 — o vocabulário geométrico (distribuições spiral/blue-noise +
deformers rotação/interpolação/arc/orient), tudo dirigido por valor. O que segue (doc 01 §3):
- **Distribuições:** `motion.distribute-voronoi` (Lloyd) · `-path` (vector.*) · `motion.lattice`.
- **Deformers:** `motion.four-point-warp` · `-slit-scan`.
- **Sim/agentes (próxima CLASSE de capacidade — dinâmica sequencial):** `motion.boids` (spatial hash) ·
  `-verlet-rope` · `-soft-body` (XPBD) · `-pin-constraint`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

*"Linha `motion-value` com 5 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
