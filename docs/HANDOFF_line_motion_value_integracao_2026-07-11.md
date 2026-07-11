# HANDOFF de integração — linha `line/motion-value` (Math+Compare doc 16 · Switch+OnChange doc 17 · Fibonacci+Twist doc 18)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (= o tip ATUAL
  do main no momento da abertura → fork fresco, **sem drift pré-fork**).
- **TRÊS fatias** na linha: fatia 4 (**Math + Compare**, doc 16) + fatia 5 (**Switch + On Change**, doc 17)
  + fatia M3.1 (**Fibonacci + Twist**, doc 18 — abre o M3). Cada uma = 2 crates-nó + a cena boot reescrita.
  **A cena boot atual demonstra a fatia M3.1** (um girassol que torce); os nós das fatias 4–5 ficam
  registrados/drop-in.
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
- **6 crates-nó novas**, tipos (string, namespaced, únicos): fatia 4 **`value.math`**/**`pulse.compare`**;
  fatia 5 **`value.switch`**/**`pulse.on_change`**; fatia M3.1 **`motion.fibonacci`**/**`motion.twist`**.
  Grep de colisão: `grep -rn '"value.math"\|"pulse.compare"\|"value.switch"\|"pulse.on_change"\|"motion.fibonacci"\|"motion.twist"' crates/`.
- pub consts / colunas locais (mirrors do tipo / cols locais ao stream, sem registro global): os `VALUE`/
  `PULSE` por-crate, `cmp_armed`, `oc_prev`/`oc_primed`. Cada crate de M3.1 tem um `trig.rs` local
  (`cos_sin_cycles`, copiado de `motion.orbit` — convenção leaf, sem símbolo compartilhado).
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
- **Smoke: fatias 4 (math+compare) e 5 (switch+on_change) APROVADAS (Enio, 2026-07-11); fatia M3.1
  (fibonacci+twist) PENDENTE.** Desde o feedback do Enio ("com tantos nós fica difícil entender o
  conceito") a cena boot é sempre PEQUENA e isola a fatia mais recente. A cena atual demonstra a **M3.1**
  (headless provado por `the_fibonacci_lays_out_a_phyllotaxis_spiral` +
  `the_twist_coils_the_spiral_over_time`). Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → um **girassol de 180 sementes** (o `motion.fibonacci`, golden-angle phyllotaxis) que
  **enrola e desenrola** no tempo (o `motion.twist`, com o `amount` dirigido por uma `value.lfo`), as
  sementes graduadas pequenas→grandes do centro à borda. O pipeline M3 *gerar → deformar*. No editor:
  dropar `motion.fibonacci` (Source) e `motion.twist` (com um valor no `amount` pra animar).

**Resumo:** *Linha `motion-value` com TRÊS fatias (fork em `1c7c9a22`). Aditiva: 6 crates-nó
(`value.math`+`pulse.compare` doc 16; `value.switch`+`pulse.on_change` doc 17 → domínio de valor completo;
`motion.fibonacci`+`motion.twist` doc 18 → abre o M3) + a cena boot pequena (demonstra a M3.1). Único
conflito mecânico = codegen `registry-init` → `ph2d-node-sync` (42 crates). Zero substrato, zero contrato
congelado, zero dep externa. Fatias 4–5 smoke-aprovadas; M3.1 pendente. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (fatias 4–5 = domínio de valor completo · M3.1 = abre o M3 — docs 16, 17, 18)

Fecha TODOS os follow-ups nomeados dos docs 12–14 §5 (fatias 4–5 → vocabulário de valor COMPLETO) e abre
o **M3** (fatia M3.1: 1ª distribuição + 1º deformer), sempre pesquisando o padrão-ouro ANTES de codar
(DIRETIVA §1). Detalhe + pesquisa: [`16_…`](Motion%20Nodes/16_math_compare_nota_adr.md),
[`17_…`](Motion%20Nodes/17_switch_on_change_nota_adr.md),
[`18_fibonacci_twist_nota_adr.md`](Motion%20Nodes/18_fibonacci_twist_nota_adr.md).

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
  (amount ← `value.lfo`), sementes graduadas por `instance_field`. O pipeline M3 *gerar → deformar*, com
  o domínio de valor animando o deformer. Playhead-pura (sem estado `pre`).

## 1. Gates no fechamento (paridade §7) — última fatia (M3.1)
- **Unit:** `motion.fibonacci` 6 + `motion.twist` 7 (falsificados). Fatias 4–5: math 7 / compare 7 /
  switch 5 / on_change 6.
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bins motion` = **24 passed**
  (inclui `the_fibonacci_lays_out_a_phyllotaxis_spiral` + `the_twist_coils_the_spiral_over_time` +
  loop-replay REESCRITO com doc próprio + motion_bridge params). `ph2d-eval-motion` + membrane gate verdes.
- **Contrato:** `ph2d-nodegraph --test architecture_contract_surface` = 3 pass (NodeOp=2/OpResolver=1/
  NodeManifest=8, intacto).
- **Registry:** `ph2d-node-registry-init --test staleness` = 2 pass (em sync após `node-sync`, 42 crates).
- **Lint/estilo:** `clippy --all-targets` (crates novas + registry-init + shell `--bins`) = 0 warnings ·
  `cargo fmt` pin 1.95 (style_edition 2024) · `typos` 0 (bare, docs excluídos) · `cargo machete` 0 ·
  sweep HR-5 (`\.(sin|cos|tan|atan2|exp|ln|log|powf|powi)\b`) = 0 na produção (o trig é o `cos_sin_cycles`
  parabólico, não transcendental).
- **LOC:** M3.1 `motion.fibonacci` 235 / `motion.twist` 321 (cap 700); shell `motion_demo_strobe.rs` 141 /
  `motion_state_tests.rs` 164 / `motion_state.rs` 118 (cap 600). Todos folgados.

## 2. Follow-up restante (doc 18 §5) — M3 aberto
Vocabulário de valor COMPLETO (fatias 4–5); M3 aberto com Fibonacci+Twist (o padrão *gerar → deformar
dirigido por valor*). O que segue (doc 01 §3):
- **Distribuições:** `motion.distribute-poisson` (Bridson) · `-voronoi` (Lloyd) · `-path` (vector.*) ·
  `motion.lattice`.
- **Deformers:** `motion.bend` · `-four-point-warp` · `motion.morph` · `-look-at` · `-slit-scan`.
- **Sim/agentes:** `motion.boids` · `-verlet-rope` · `-soft-body` (XPBD) · `-pin-constraint`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

*"Linha `motion-value` com 3 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
