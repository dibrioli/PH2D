# HANDOFF de integração — linha `line/motion-value` (Math + Compare, doc 16 · Switch + On Change, doc 17)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (= o tip ATUAL
  do main no momento da abertura → fork fresco, **sem drift pré-fork**).
- **DUAS fatias** na linha: fatia 4 (**Math + Compare**, doc 16, commit `9ba2f6ad`) + fatia 5
  (**Switch + On Change**, doc 17). Cada uma = 2 crates-nó + a cena boot reescrita. **A cena boot atual
  demonstra a fatia 5** (switch+on_change); os nós da fatia 4 (math/compare) ficam registrados/drop-in.
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
- **4 crates-nó novas**, tipos (string, namespaced, únicos): fatia 4 **`value.math`**, **`pulse.compare`**;
  fatia 5 **`value.switch`**, **`pulse.on_change`**. Grep de colisão:
  `grep -rn '"value.math"\|"pulse.compare"\|"value.switch"\|"pulse.on_change"' crates/`.
- pub consts locais: `value_math::VALUE`, `pulse_compare::{VALUE,PULSE}`, `value_switch::VALUE`,
  `pulse_on_change::{VALUE,PULSE}` (mirrors do tipo, não símbolos compartilhados). Colunas de stream novas
  `cmp_armed` (compare), `oc_prev`/`oc_primed` (on_change) — locais ao stream, sem registro global.
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
- **Smoke: fatia 4 (math+compare) APROVADA (Enio, 2026-07-11); fatia 5 (switch+on_change) PENDENTE.** O
  Enio aprovou o smoke da fatia 4 e pediu p/ **simplificar** ("com tantos nós fica difícil entender o
  conceito"); desde então a cena boot é sempre PEQUENA e isola a fatia mais recente. A cena atual demonstra
  a **fatia 5** (headless provado por `the_switch_routes_the_size_between_two_patterns` +
  `the_on_change_flashes_the_grid_on_each_pattern_flip`). Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → uma grade 3×4 cujo padrão de TAMANHO **alterna entre um gradiente ordenado e um
  scatter aleatório** (o `value.switch` roteando `instance_field` Ramp ↔ Random, `select` = uma
  `value.lfo` lenta) E **pisca branco a cada troca** (o `pulse.on_change` → strobe). Roteamento + detecção
  de mudança. No editor: dropar `value.switch` (select + in0..in3) e `pulse.on_change` depois de qualquer
  valor em degraus (counter/sample_hold/switch).

**Resumo:** *Linha `motion-value` com DUAS fatias (fork em `1c7c9a22`). Aditiva: 4 crates-nó
(`value.math`+`pulse.compare` doc 16; `value.switch`+`pulse.on_change` doc 17) + a cena boot pequena
(demonstra a fatia 5). Único conflito mecânico = codegen `registry-init` → `ph2d-node-sync`. Zero
substrato, zero contrato congelado, zero dep externa. Fatia 4 smoke-aprovada; fatia 5 pendente. Aguardo
ordem de integração.*

---

## 0. O que a linha entrega (fatias 4 e 5 do domínio de valor — docs 16 e 17)

Fecha TODOS os follow-ups nomeados dos docs 12–14 §5, pesquisando o padrão-ouro ANTES de codar
(DIRETIVA §1). Com estas 4 crates o **vocabulário-núcleo do domínio de valor está completo**. Detalhe +
pesquisa: [`docs/Motion Nodes/16_math_compare_nota_adr.md`](Motion%20Nodes/16_math_compare_nota_adr.md)
e [`17_switch_on_change_nota_adr.md`](Motion%20Nodes/17_switch_on_change_nota_adr.md).

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
- **Cena boot** (a última reescrita, pedida pelo Enio p/ simplificar): PEQUENA (~11 nós), isola a fatia 5 —
  `value.switch` roteia o Size entre Ramp e Random (padrão alterna), `pulse.on_change` pisca o strobe no
  flip. Os nós da fatia 4 ficam registrados/drop-in.

## 1. Gates no fechamento (paridade §7) — última fatia (5)
- **Unit:** `value.switch` 5 + `pulse.on_change` 6 (falsificados). Fatia 4: `value.math` 7 +
  `pulse.compare` 7.
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bins motion` = **24 passed**
  (inclui `the_switch_routes_the_size_between_two_patterns` +
  `the_on_change_flashes_the_grid_on_each_pattern_flip` + loop-replay ajustado + motion_bridge params).
  `ph2d-eval-motion` + membrane gate verdes.
- **Contrato:** `ph2d-nodegraph --test architecture_contract_surface` = 3 pass (NodeOp=2/OpResolver=1/
  NodeManifest=8, intacto).
- **Registry:** `ph2d-node-registry-init --test staleness` = 2 pass (em sync após `node-sync`, 40 crates).
- **Lint/estilo:** `clippy --all-targets` (crates novas + registry-init + shell `--bins`) = 0 warnings ·
  `cargo fmt` pin 1.95 (style_edition 2024) · `typos` 0 (bare, docs excluídos) · `cargo machete` 0 ·
  sweep HR-5 (`\.(sin|cos|tan|atan2|exp|ln|log|powf|powi)\b`) = 0 na produção das crates novas.
- **LOC:** fatia 5 `value.switch` 278 / `pulse.on_change` 238 (cap 700); fatia 4 `value.math` 380 /
  `pulse.compare` 351; shell `motion_demo_strobe.rs` 182 / `motion_state_tests.rs` 185 /
  `motion_state.rs` 118 (cap 600). Todos folgados.

## 2. Follow-up restante (doc 17 §5)
Com Switch + On Change o **vocabulário-núcleo do domínio de valor está COMPLETO** (produzir → combinar →
amostrar → comparar → detectar-mudança → remapear → rotear → dirigir).
- **`motion.delay`** — atrasa um canal N ticks (eco/time-shift puro, distinto do `motion.trail`) — o
  último utilitário do M2 (doc 01 §3).
- **M3** — distribuições avançadas (Fibonacci/Poisson/Voronoi/Path) + deformers (lattice/bend/twist/
  morph/look-at/boids/verlet-rope) — o próximo grande passo (geometria/distribuições), doc 01 §3.

*"Linha `motion-value` com 2 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
