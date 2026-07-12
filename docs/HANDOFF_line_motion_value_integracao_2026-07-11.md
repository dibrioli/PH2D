# HANDOFF de integração — linha `line/motion-value` (docs 16–32: valor + M3 + M4 + cauda M1 + expression)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

> **⚠ MUDANÇA DE CARÁTER (fatia 32, autorizada pelo Enio "abre a expression"):** a linha **agora TOCA o
> substrato** `ph2d-nodegraph` (Graph + EvalCtx + cook), de forma **ADITIVA**, pra dar o canal de *text param*
> à `motion.expression`. **O contrato congelado foi PROVADO intacto** (`architecture_contract_surface` = 8/2/1
> depois da mudança). A story "zero substrato" das fatias 16–31 **não vale mais** pra fatia 32 — ver §2/§4.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base = `1c7c9a22`** (fork fresco, sem drift pré-fork).
- **DEZESSETE fatias:** **valor** (16–17); **M3** completo (18–20, 23–28); **SIMULAÇÃO** (21–22); **cauda M1**
  (Color-Ramp+Color-Array 29 · Combine+Mixer 30 · Make-Point+Luminance 31 · **Expression 32**). **A cena boot
  atual demonstra Expression** (um espiral e uma onda de cor, ambos por fórmula). (Também: 2 fixes de perf no
  voronoi + o plano GPU.)
- **Auto-contida:** sem dependência de outra linha. Fatias 16–31 = fan-out puro; **fatia 32 = foundational
  aditivo** (substrato, contrato provado intacto).

### 2. Foundational/compartilhado tocado
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| **`crates/ph2d-nodegraph/src/graph.rs`** | **SUBSTRATO (fatia 32, ADITIVO):** `Graph.node_text_params` + `set_text_param` + `node_text_param_overrides` + `remove_node`. Undo coberto (Clone/PartialEq). | **NOVO — não era tocado antes.** Aditivo (append-only). `foundational-integrate.sh` roda o gate combinado. Conflito mesmo-símbolo só se outra linha editar graph.rs |
| **`crates/ph2d-nodegraph/src/cook.rs`** | **SUBSTRATO (fatia 32, ADITIVO):** `EvalCtx.text_overrides`+`text_param` + `Fingerprint.text_params` + `text_params_fingerprint` + threading (2 sites). | idem — aditivo; comportamento existente inalterado |
| **`crates/ph2d-nodegraph/src/format.rs`** | **SUBSTRATO (fatia 32 +serialização, ADITIVO):** record `x` (text param, campo livre como o `b` title) + header `v2` (só se houver text param; senão `v1` byte-idêntico) + `Graph.node_text_params()` getter. | aditivo; `from_text` aceita v1\|v2; `MotionDoc` delega transparente |
| `shells/desktop/src/motion_demo_strobe.rs` + `motion_state.rs` + `motion_state_tests.rs` | cena boot (espiral+onda, 12 nós) + testes | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 — intocado | baixo |
| `Cargo.lock` | crates PATH novas + `ph2d-color`/`ph2d-expr` (já membros) | regenera na árvore combinada |

**Substrato `ph2d-eval-motion` NÃO tocado.** O `ph2d-nodegraph` foi tocado **aditivamente** (fatia 32) — a
prova de que o contrato segue é o gate (§4). **`ph2d-expr` consumido, NÃO alterado.**

### 3. Ponto de conflito MECÂNICO: `ph2d-node-registry-init`
- `src/lib.rs`/`Cargo.toml` GERADOS (**69 crates**). Conflito → `cargo run -p ph2d-node-sync` (gate `staleness`
  prova sync).

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **33 crates-nó novas** (16–31) + **`motion.expression`** (32; dep `ph2d-expr`). Grep dos tipos: como antes +
  `|expression`.
- **SUBSTRATO (fatia 32):** símbolos novos em `ph2d-nodegraph` — `node_text_params`/`set_text_param`/
  `node_text_param_overrides`/`text_overrides`/`text_param`/`text_params`/`text_params_fingerprint`. Todos
  **aditivos** (nenhum renomeia/remove existente).
- Helpers leaf: `trig.rs`/`hash.rs`/`atan2_approx`/`shape.rs`/`curve.rs` + `parse.rs` (VEX-lite, na expression).
- **Deps:** `rayon`(voronoi) · `ph2d-color`(color-ramp) · **`ph2d-expr`(expression)** — todas libs já no
  workspace (foundational/congeladas; nada novo no lockfile/RUSTSEC).
- **ZERO campo em `NodeManifest`, ZERO método em `NodeOp`/`OpResolver`, ZERO alteração em `ph2d-expr`.**

### 4. Contratos congelados: **PROVADOS INTACTOS (8/2/1)**
`architecture_contract_surface` = **3 pass** DEPOIS da fatia 32 (`NodeManifest=8`/`NodeOp=2`/`OpResolver=1`). Os
70 testes do `ph2d-nodegraph` verdes. A fatia 32 estende o *armazenamento* de params (Graph/EvalCtx), **não a
superfície do contrato** — é a realização isolada do M4.N1 sem bump (doc 32 §2). **Sugestão ao Enio:**
ratificar o canal text-param como ADR real (superseding o plano M4.N1 de bumpar o `NodeManifest`).

### 5. O que só o `ship.sh` pega + CAVEATS DA FATIA 32 (LER)
- **Drift pré-fork BAIXO** (fork == `1c7c9a22`). Rode **`ship.sh` completo** na árvore combinada. `nextest-
  impacted` funciona (adição). fmt (pin 1.95, **edition 2024** — cook.rs tem let-chain)/machete/clippy/HR-5/
  LOC/typos verdes no fechamento.
- **✅ Serialização textual FECHADA:** o `format.rs` serializa text params via o record **`x`** (campo livre,
  como o `b` title — sem escaping) + header **`v2`** (só com text param; senão `v1` byte-idêntico). Round-trip
  testado (12 format tests) + prova end-to-end no shell (fórmula da boot doc sobrevive a `MotionDoc::to_text/
  from_text`). `from_text` aceita v1\|v2. Contrato de nó intocado. (Detalhe: doc 32 §5.)
- **⚠ Replay-hash cross-máquina:** `ph2d_expr::eval` usa transcendentais f32 (libm) que **variam entre máquinas**
  (presentation-side/HR-5-exempt por contrato). Determinístico dentro do processo (replay test passa). Se um
  golden replay-hash cross-máquina cobrir a boot doc, a expression pode divergir → re-lockar ou boot doc sem
  fórmula transcendental no hash. (Também: o voronoi mudou o output — res adaptativa.)

### 6. Procedimento sugerido + smoke
- **main não moveu:** `git merge --ff-only line/motion-value` → `ship.sh`.
- **main moveu:** rebase → conflito em `ph2d-node-registry-init` (`cargo run -p ph2d-node-sync`) + `Cargo.lock`
  (+ possível mesmo-símbolo em `ph2d-nodegraph/{graph,cook}.rs` se outra linha os editou → aditivo, funde fácil)
  → `foundational-integrate.sh` (roda o gate combinado, INCLUINDO o contract gate) + `ship.sh`.
- **Smoke APROVADAS (Enio):** 4/5 · M3.1–3.3 · M4.1/M4.2 · Lattice+Voronoi · Four-Point-Warp+Spherize ·
  Sort+Cull · Color-Ramp+Color-Array · Combine+Mixer · Make-Point+Luminance. **Expression** (cena atual) +
  Radial+Mirror, Kaleidoscope+Collide, Distribute-Curve+Spline-Wrap: testados headless. Headless da atual:
  `the_spiral_is_plotted_and_rotates` + `the_colour_wave_scrolls`. Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA (âmbar) um **espiral de 144 pts** girando (x/y = fórmulas cos/sin via
  `motion.expression` → `make_point`); à DIREITA (Ice) um grid com uma **onda de cor rolando** (fórmula
  `sin(t·2+f·a)` → `t` de um ramp). Editor: dropar `motion.expression` (params a..d; a fórmula é text param —
  **sem campo de texto no painel ainda**, ver caveat).

**Resumo:** *Linha `motion-value`, 17 fatias (fork `1c7c9a22`). 33 crates-nó (valor · M3 · SIMULAÇÃO · cauda M1
completa incl. **expression**) + cena boot (Expression) + o **canal text-param aditivo no substrato**
`ph2d-nodegraph` (fatia 32 — contrato PROVADO 8/2/1, realização isolada do M4.N1) + 2 fixes perf voronoi + plano
GPU. Conflito mecânico = codegen `registry-init` → `ph2d-node-sync`. Contrato congelado intacto; `ph2d-expr`
consumido; serialização textual de text params FECHADA (record `x`/v2). Caveat: replay-hash cross-máquina da
expression. 13 fatias smoke-aprovadas; 4 pendentes de smoke visual. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (docs 16–32)

Fecha o **valor**, o **M3** completo, a **SIMULAÇÃO** e a **cauda M1** (cor + streams + adapters + **expression**),
sempre pesquisando o padrão-ouro ANTES de codar. Pesquisa por fatia: docs 16–32.

**Fatia Make-Point+Luminance (doc 31) — adapters valor↔geometria↔cor.**

**Fatia Expression (doc 32) — o escape-hatch de fórmula + o canal TEXT PARAM (foundational):**
- **`motion.expression`** (`ph2d-node-motion-expression`, dep `ph2d-expr`): fórmula VEX-lite por-instância →
  campo `v`. Vars `i`/`n`/`t`/`f` + colunas + params `a`..`d`; parser recursive-descent (`parse.rs`) → o
  `ph2d_expr::Expr` congelado; `eval` HR-5-exempt; erro→zero. `Effect::Temporal`, Utility.
- **Canal text-param (substrato, aditivo):** `Graph.node_text_params` + `EvalCtx::text_param` + Fingerprint — a
  fórmula vive fora do `NodeManifest` congelado (doc 32). **Contrato provado 8/2/1.**
- **Cena boot:** 2 cenas (12 nós) — espiral cos/sin girando + onda de cor rolando.

(Fatias anteriores: docs 16–31.)

## 1. Gates no fechamento (paridade §7) — última fatia (Expression)
- **Unit:** `parse.rs` 3 (precedência/funções-select-erros/unário) + node 5 (fórmula-por-elemento [prova o
  plumbing text-param end-to-end]/colunas+params/funções-select/erro→zero/registra).
- **Integração (shell):** `motion` = **24 passed** (espiral 144 gira · onda 100 rola · determinismo intra-
  processo · loop-replay · bridge).
- **Contrato:** `architecture_contract_surface` = **3 pass (8/2/1) DEPOIS da mudança de substrato** ·
  `ph2d-nodegraph` 70 testes · **Registry:** `staleness` = 2 pass (**69 crates**).
- **Lint/estilo:** clippy `--all-targets -D warnings` (incl. `ph2d-nodegraph`) = 0 · fmt pin 1.95 edition 2024
  · typos 0 · machete 0 (`ph2d-expr` usado) · HR-5 = 0 no meu código (`ph2d_expr::eval` é HR-5-exempt) · tofu 0.
- **LOC:** expression lib 296 / parse 344 (cap 700); shell demo 155 / tests 171 / state 113 (cap 600).

## 2. Follow-up restante
- **Cauda M1: COMPLETA** (cor, streams, adapters, expression). A `motion.expression` era o item de maior valor
  e o único ADR-gated — feito via o caminho aditivo.
- **Follow-ups da fatia 32 (foundational):** ~~serialização textual~~ **FEITA** (record `x`/v2, §2/§5) · UI de
  texto no painel de params (editor; precisa `ParamWidget::Text` em `ph2d-node-registry`) · (opcional) lowering
  WGSL da expression (`ph2d-expr` já tem `to_wgsl` — combina com o motor GPU).
- **M2:** wiring do scrub-back (`Cook::checkpoint/restore` já existem) · `motion.delay` · `force.buoyancy`.
- **Cross-module (DEFERIDO):** `distribute-path` (vetor) · `slit-scan`/`delay`.
- **Fronteiras:** **M4** (Rig+FX, necks) · **M5** (motor GPU — ADR + foundational → linha dedicada).

*"Linha `motion-value` com 17 fatias (fork em `1c7c9a22`). Toca o substrato (fatia 32, aditivo, contrato provado
8/2/1). Handoff acima. Aguardo ordem de integração."*
