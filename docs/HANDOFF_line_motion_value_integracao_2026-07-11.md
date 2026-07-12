# HANDOFF de integração — linha `line/motion-value` (docs 16–31: valor + M3 + M4 + cauda M1)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (fork fresco, **sem drift
  pré-fork**).
- **DEZESSEIS fatias**, todas fan-out aditivo de nós + cena boot: **valor** (Math+Compare 16 · Switch+On Change
  17); **M3** (Fibonacci+Twist 18 · Scatter+Morph 19 · Bend+Look At 20 · Lattice+Voronoi 23 · Four-Point-Warp+
  Spherize 24 · Radial+Mirror 25 · Kaleidoscope+Collide 26 · Sort+Cull 27 · Distribute-Curve+Spline-Wrap 28);
  **SIMULAÇÃO** (Verlet-Rope+Boids 21 · Soft-Body+Wave 22); **cauda M1** (Color-Ramp+Color-Array 29 ·
  Combine+Mixer 30 · **Make-Point+Luminance 31**). **A cena boot atual demonstra Make-Point+Luminance** (um
  Lissajous plotado de LFOs + um grid recolorido pela própria luminância); os nós das outras fatias ficam
  registrados/drop-in. (Também: 2 fixes de perf no voronoi + o plano `docs/plans/2026-07-gpu-resident-node-
  pipeline.md`.)
- **Auto-contida:** sem dependência de outra linha nem de outro módulo-feature → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última = Lissajous make_point + recolor luminance, 2 cenas, 13 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + `build`→`Vec` de 2 sinks + contagem 13 + testes de integração (Lissajous/luminance) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 (beat→strobe) — NÃO depende da cena boot; **intocado** | shell, módulo Motion; baixo |
| `Cargo.lock` | só as crates PATH novas + a aresta `ph2d-color` (já era membro) | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado.

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` **GERADOS** (região `<ph2d-node-sync>`; **68 crates** agora). Qualquer outra linha
  que adicione um nó conflita aqui. **Resolução — NÃO fundir à mão:** `cargo run -p ph2d-node-sync` (o gate
  `staleness` prova sync).

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **32 crates-nó novas**, tipos únicos: …(16–30)… + **`motion.make_point`**/**`motion.luminance`** (31). Grep
  base: `grep -rnE '"motion\.(fibonacci|twist|scatter|morph|bend|look_at|verlet_rope|boids|soft_body|wave|lattice|voronoi|four_point_warp|spherize|distribute_radial|mirror|kaleidoscope|collide|sort|cull|distribute_curve|spline_wrap|color_ramp|color_array|combine|mixer|make_point|luminance)"' crates/`.
- Helpers copiados por-crate (leaf): `trig.rs` · `hash.rs` · `atan2_approx` · `shape.rs` · `curve.rs`.
- **Deps:** `rayon` (voronoi) + `ph2d-color` (color-ramp) — libs já no workspace (foundational; nada novo no
  lockfile/RUSTSEC). Os demais nós sem deps extra.
- **Multi-input:** `combine`/`mixer` (4 ports) + `make_point` (3 ports) — o contrato já suportava (`inputs:
  &[PortSpec]`), zero mudança. `luminance` emite **VALUE** (campo `v`, como `value.instance_field`).
- **ZERO** `NodeId` numérico / token / variant de enum congelado novos.
- **Nota replay-hash:** o output do voronoi mudou (res adaptativa) → re-lockar golden se algum cobrir Motion.

### 4. Contratos congelados encostados: **NENHUM**
Gate `architecture_contract_surface` verde (2/1/8). Fan-out aditivo (caminho A). Zero acoplamento a módulo-
feature. **NB:** a `motion.expression` (fórmula editável) NÃO está aqui — precisa de param string (M4.N1
ParamSpec tipado, ADR); foi conscientemente deixada de fora (ver §6 do doc 31).

### 5. O que só o `ship.sh` pega (o `foundational-integrate.sh` NÃO roda fmt/typos/machete/deny)
- **Drift pré-fork: BAIXO** — fork == tip de `1c7c9a22`. Rode **`ship.sh` completo** na árvore combinada.
  `nextest-impacted` funciona (só ADIÇÃO). fmt/machete/clippy `--all-targets -D warnings`/HR-5/LOC/typos verdes
  no fechamento (pin 1.95).
- **typos:** "Secord" parafraseado fora do voronoi. **`no_tofu_glyphs`:** setas `→` latentes em strings de teste
  trocadas nas fatias 26–29 (30/31 não tiveram); o gate só roda no ship e escaneia só string literals.

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → `ship.sh`.
- **Se main moveu:** rebase → UM conflito mecânico em `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}`
  (`cargo run -p ph2d-node-sync`) + `Cargo.lock` → `foundational-integrate.sh` + `ship.sh`.
- **Smoke APROVADAS (Enio):** 4/5 · M3.1–3.3 · M4.1/M4.2 · Lattice+Voronoi · Four-Point-Warp+Spherize ·
  Sort+Cull · Color-Ramp+Color-Array · Combine+Mixer. **Make-Point+Luminance** (cena atual) + Radial+Mirror,
  Kaleidoscope+Collide, Distribute-Curve+Spline-Wrap: nós testados headless, só a cena boot mais recente é vista
  por vez (integração já provada). A cena atual demonstra **Make-Point+Luminance**. Headless: `the_lissajous_is_
  plotted_and_animates` + `the_grid_is_recoloured_by_luminance`. Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA (âmbar) um **Lissajous de 64 pts** plotado por `motion.make_point` de dois
  `value.lfo` com `phase_stagger`, animando; à DIREITA (Heat) um grid **recolorido pela própria luminância**
  (`motion.luminance` lê o Rainbow → `v` → indexa um Heat ramp). Editor: dropar `motion.make_point` (in/x/y) e
  `motion.luminance` (in → v).

**Resumo:** *Linha `motion-value`, 16 fatias (fork `1c7c9a22`). Aditiva: 32 crates-nó (valor completo · M3
completo · SIMULAÇÃO · cauda M1 cor+streams+adapters) + cena boot pequena (Make-Point+Luminance) + 2 fixes perf
voronoi + plano GPU. Único conflito mecânico = codegen `registry-init` → `ph2d-node-sync` (68 crates). Zero
substrato, zero contrato congelado, zero acoplamento a módulo-feature; deps externas `rayon`+`ph2d-color`, já no
workspace. 12 fatias smoke-aprovadas; 4 pendentes de smoke visual individual. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (docs 16–31)

Fecha o **valor** (16–17), o **M3** completo, a **SIMULAÇÃO** (21–22) e a **cauda M1** (cor ramp/array + streams
combine/mixer + adapters make_point/luminance), sempre pesquisando o padrão-ouro ANTES de codar. Pesquisa por
fatia: docs 16–31.

**Fatia Combine+Mixer (doc 30) — os operadores de STREAM (branch-and-merge):**
- **`motion.combine`**: concatena ≤4 streams (Merge/Join). `Pure`, Utility.
- **`motion.mixer`**: mistura ≤4 streams (Avg/Add/Blend; `blend` → morph). `Pure`, Utility.

**Fatia Make-Point+Luminance (doc 31) — os adapters valor↔geometria↔cor (a cauda M1 quase fecha):**
- **`motion.make_point`** (`ph2d-node-motion-make-point`): campos de valor `x`/`y` → `P` (Lissajous/plotting
  data-driven). `Pure`, Utility.
- **`motion.luminance`** (`ph2d-node-motion-luminance`): `tint` → campo `v` (Rec.709 luma); output **VALUE**
  (como `instance_field`). `Pure`, Utility.
- **Cena boot:** 2 cenas (13 nós) — Lissajous animado + grid recolorido por luminância.

(Fatias anteriores: docs 16–30.)

## 1. Gates no fechamento (paridade §7) — última fatia (Make-Point+Luminance)
- **Unit:** `motion.make_point` 4 (empacota/broadcast/ausente-0/cook) + `motion.luminance` 4 (branco-preto-cinza/
  G>R>B/ausente-0/cook; falsificados).
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop motion`
  = **24 passed** (Lissajous 64 + anima · luminance 100 + >5 cores Heat · determinismo · loop-replay · bridge).
- **Contrato:** `architecture_contract_surface` = 3 pass (2/1/8). **Registry:** `staleness` = 2 pass
  (**68 crates**).
- **Lint/estilo:** `clippy --all-targets -D warnings` = 0 · `cargo fmt` pin 1.95 · `typos` 0 · `cargo machete`
  0 · HR-5 = 0 (empacotar coord + soma ponderada) · `no_tofu_glyphs` 0.
- **LOC:** make-point 242 / luminance 211 (cap 700); shell `motion_demo_strobe.rs` 167 / `motion_state_tests.rs`
  169 / `motion_state.rs` 113 (cap 600).

## 2. Follow-up restante — a cauda M1 quase fecha (correção importante)
> **Correção:** a `motion.expression`, que eu vinha chamando de "próxima self-contained", **NÃO é** — precisa
> de param string, e `ParamSpec` é f32-only. Param tipado = **M4.N1 (contrato congelado, EXIGE ADR)**. A
> expression é **ADR-gated**, não fan-out. O resto da cauda M1 é subsumido/marginal.
- **Cauda M1 (aberta, self-contained mas marginal):** variações de make_point (make-line/make-vec2) · `value-
  to-color` (subsumido pelo `color_ramp` t) · `threshold`/`gate` (subsumidos por `pulse.threshold`/`value.
  switch`).
- **`motion.expression` — ADR-GATED (M4.N1 ParamSpec tipado):** o maior valor que resta do M1; precisa de
  ordem/ADR do Enio (param string) → vira linha foundational, não fan-out.
- **M2:** wiring do scrub-back (`Cook::checkpoint/restore` já existem) · `motion.delay` · `force.buoyancy`.
- **Cross-module (DEFERIDO):** `motion.distribute-path` (curva do doc vetorial) · `slit-scan`/`delay`.
- **Fronteiras grandes:** **M4** (Rig+FX — necks `ParamSpec` tipado + `Func::Pow`) · **M5** (motor GPU — ADR +
  foundational → linha dedicada).

*"Linha `motion-value` com 16 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
