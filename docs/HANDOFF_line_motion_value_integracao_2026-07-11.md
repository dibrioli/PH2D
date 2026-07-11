# HANDOFF de integração — linha `line/motion-value` (docs 16–30: valor + M3 + M4 + M1 cor/streams)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (fork fresco, **sem drift
  pré-fork**).
- **QUINZE fatias**, todas fan-out aditivo de nós + cena boot: **valor** (Math+Compare 16 · Switch+On Change 17);
  **M3** (Fibonacci+Twist 18 · Scatter+Morph 19 · Bend+Look At 20 · Lattice+Voronoi 23 · Four-Point-Warp+
  Spherize 24 · Radial+Mirror 25 · Kaleidoscope+Collide 26 · Sort+Cull 27 · Distribute-Curve+Spline-Wrap 28);
  **SIMULAÇÃO** (Verlet-Rope+Boids 21 · Soft-Body+Wave 22); **M1-cauda** (Color-Ramp+Color-Array 29 ·
  **Combine+Mixer 30**). **A cena boot atual demonstra Combine+Mixer** (grid+anel concatenados + grid↔círculo em
  morph — os primeiros grafos branch-and-merge); os nós das outras fatias ficam registrados/drop-in. (Também:
  2 fixes de perf no voronoi + o plano `docs/plans/2026-07-gpu-resident-node-pipeline.md`.)
- **Auto-contida:** sem dependência de outra linha nem de outro módulo-feature → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última = combine grid+anel + mixer grid↔círculo, 2 cenas Y, 14 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + `build`→`Vec` de 2 sinks + contagem 14 + testes de integração (combine/mixer) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 (beat→strobe) — NÃO depende da cena boot; **intocado** | shell, módulo Motion; baixo |
| `Cargo.lock` | só as crates PATH novas + a aresta `ph2d-color` (já era membro) | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado.

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` **GERADOS** (região `<ph2d-node-sync>`; **66 crates** agora). Qualquer outra linha
  que adicione um nó conflita aqui. **Resolução — NÃO fundir à mão:** `cargo run -p ph2d-node-sync` (o gate
  `staleness` prova sync).

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **30 crates-nó novas**, tipos únicos: …(16–29)… + **`motion.combine`**/**`motion.mixer`** (30). Grep base:
  `grep -rnE '"motion\.(fibonacci|twist|scatter|morph|bend|look_at|verlet_rope|boids|soft_body|wave|lattice|voronoi|four_point_warp|spherize|distribute_radial|mirror|kaleidoscope|collide|sort|cull|distribute_curve|spline_wrap|color_ramp|color_array|combine|mixer)"' crates/` (+ `value.(math|switch)`/`pulse.(compare|on_change)`).
- Helpers copiados por-crate (leaf): `trig.rs` · `hash.rs` · `atan2_approx` · `shape.rs` · `curve.rs`.
- **Deps:** `rayon` (voronoi) + `ph2d-color` (color-ramp) — ambas libs já no workspace (foundational; nada novo
  no lockfile/RUSTSEC). `combine`/`mixer` sem deps extra.
- **Multi-input:** `combine`/`mixer` são os 1ºs nós com 4 input ports — o contrato já suportava (`inputs:
  &[PortSpec]`), zero mudança.
- **ZERO** `NodeId` numérico / token / variant de enum congelado novos.
- **Nota replay-hash:** o output do voronoi mudou (res adaptativa) → re-lockar golden se algum cobrir Motion.

### 4. Contratos congelados encostados: **NENHUM**
Gate `architecture_contract_surface` verde (2/1/8). Fan-out aditivo (caminho A). Zero acoplamento a módulo-
feature (curva nos params; cor via lib foundational `ph2d-color`; streams são plumbing de grafo puro).

### 5. O que só o `ship.sh` pega (o `foundational-integrate.sh` NÃO roda fmt/typos/machete/deny)
- **Drift pré-fork: BAIXO** — fork == tip de `1c7c9a22`. Rode **`ship.sh` completo** na árvore combinada.
  `nextest-impacted` funciona (só ADIÇÃO). fmt/machete/clippy `--all-targets -D warnings`/HR-5/LOC/typos verdes
  no fechamento (pin 1.95).
- **typos:** "Secord" parafraseado fora do voronoi. **`no_tofu_glyphs`:** setas `→` latentes em strings de teste
  trocadas nas fatias 26–29 (a 30 não teve); o gate só roda no ship e escaneia só string literals.

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → `ship.sh`.
- **Se main moveu:** rebase → UM conflito mecânico em `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}`
  (`cargo run -p ph2d-node-sync`) + `Cargo.lock` → `foundational-integrate.sh` + `ship.sh`.
- **Smoke APROVADAS (Enio):** 4/5 · M3.1–3.3 · M4.1/M4.2 · Lattice+Voronoi · Four-Point-Warp+Spherize ·
  Sort+Cull · Color-Ramp+Color-Array. **Combine+Mixer** (a cena atual) + Radial+Mirror, Kaleidoscope+Collide,
  Distribute-Curve+Spline-Wrap: nós testados headless, só a cena boot mais recente é vista por vez (integração
  já provada). A cena atual demonstra **Combine+Mixer**. Headless: `the_grid_and_ring_combine` + `the_grid_morphs_into_the_
  circle`. Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA (âmbar) um grid + um **anel girando concatenados** em 140 dots (`motion.combine`,
  spin ← sine lfo); à DIREITA (ciano) um grid **morphando num círculo** (`motion.mixer` Blend, blend ← sine
  lfo). Editor: dropar `motion.combine` (4 inputs) e `motion.mixer` (4 inputs + blend, modos Avg/Add/Blend).

**Resumo:** *Linha `motion-value`, 15 fatias (fork `1c7c9a22`). Aditiva: 30 crates-nó (valor completo · M3
completo · SIMULAÇÃO · cauda M1 cor+streams) + cena boot pequena (Combine+Mixer) + 2 fixes perf voronoi + plano
GPU. Único conflito mecânico = codegen `registry-init` → `ph2d-node-sync` (66 crates). Zero substrato, zero
contrato congelado, zero acoplamento a módulo-feature; deps externas `rayon`+`ph2d-color`, já no workspace. 11
fatias smoke-aprovadas; 4 pendentes de smoke visual individual. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (docs 16–30)

Fecha o **valor** (16–17), o **M3** completo (distribuições grid/fibonacci/scatter/lattice/voronoi/radial/curve
+ deformers twist/morph/bend/look_at/four_point_warp/spherize/spline_wrap + simetria + empacotamento +
estruturais), a **SIMULAÇÃO** (21–22) e a **cauda M1** (cor ramp/array + streams combine/mixer), sempre
pesquisando o padrão-ouro ANTES de codar. Pesquisa por fatia: docs 16–30.

**Fatia Color-Ramp+Color-Array (doc 29) — a família COR:**
- **`motion.color_ramp`**: escalar (índice/valor) → cor num gradiente multi-stop via `ph2d-color::ColorRamp`
  (Blender "Color Ramp"); presets + `t` input. Escreve `tint`. `Pure`, Fx, dep `ph2d-color`.
- **`motion.color_array`**: cicla paleta 2–4 cores por `i mod colors` (MoGraph "Color"); `offset` marcha.
  Escreve `tint`. `Pure`, Fx.

**Fatia Combine+Mixer (doc 30) — os operadores de STREAM (branch-and-merge; os 1ºs multi-input):**
- **`motion.combine`** (`ph2d-node-motion-combine`): **concatena** ≤4 streams (Merge/Join) — união de colunas com
  zero-fill, contagem somada. `Pure`, Utility.
- **`motion.mixer`** (`ph2d-node-motion-mixer`): **mistura** ≤4 streams element-wise (Attribute Interpolate) —
  Avg/Add/Blend, contagem = mínimo; `blend` value input → morph entre dois layouts. `Pure`, Utility.
- **Cena boot:** 2 cenas Y (14 nós) — grid+anel concatenados + grid↔círculo morphando.

(Fatias anteriores: docs 16–29.)

## 1. Gates no fechamento (paridade §7) — última fatia (Combine+Mixer)
- **Unit:** `motion.combine` 3 (soma-concatena/zero-fill/cook) + `motion.mixer` 5 (avg-midpoint/add/blend-lerp/
  count-mínimo/cook; falsificados).
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop motion`
  = **24 passed** (combine 140=100+40 · mixer 64=min + morph viaja · determinismo · loop-replay · bridge).
- **Contrato:** `architecture_contract_surface` = 3 pass (2/1/8). **Registry:** `staleness` = 2 pass
  (**66 crates**).
- **Lint/estilo:** `clippy --all-targets -D warnings` = 0 · `cargo fmt` pin 1.95 · `typos` 0 · `cargo machete`
  0 · HR-5 = 0 (cópia + aritmética de componente) · `no_tofu_glyphs` 0.
- **LOC:** combine 283 / mixer 402 (cap 700); shell `motion_demo_strobe.rs` 187 / `motion_state_tests.rs` 136 /
  `motion_state.rs` 112 (cap 600).

## 2. Follow-up restante — a cauda M1 (correção mantida)
> O poço self-contained NÃO secou: distribuições/deformers M3 completos, mas a cauda M1 segue self-contained
> (deps = libs foundational já no workspace).
- **Expressão (aberta, self-contained):** `motion.expression` (VEX-lite → `ph2d-expr`; `i,n,t` virtuais; erro →
  badge) — o mais poderoso do lote M1, merece fatia própria. **É o próximo natural.**
- **Adapters (self-contained):** `value-to-color` · `luminance` · `threshold` · `gate` · `make-point`.
- **Cor:** `color-range-to-color` subsumido pelo `color_ramp` Custom (opcional).
- **M2 pendências:** wiring do scrub-back (`Cook::checkpoint/restore` já existem) · `motion.delay` ·
  `force.buoyancy`.
- **Cross-module (DEFERIDO):** `motion.distribute-path` (curva do doc vetorial) · `slit-scan`/`delay` (time-scope).
- **Fronteiras grandes:** **M4** (Rig+FX — necks `ParamSpec` tipado + `Func::Pow`) · **M5** (motor GPU — ADR +
  foundational → linha dedicada).

*"Linha `motion-value` com 15 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
