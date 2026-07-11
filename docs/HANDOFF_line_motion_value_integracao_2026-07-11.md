# HANDOFF de integração — linha `line/motion-value` (docs 16–29: valor + M3 + M4 + cor M1)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (fork fresco, **sem drift
  pré-fork**).
- **CATORZE fatias**, todas fan-out aditivo de nós + cena boot: 4 (**Math+Compare**, 16) + 5 (**Switch+On
  Change**, 17) → valor COMPLETO; M3.1 (**Fibonacci+Twist**, 18) + M3.2 (**Scatter+Morph**, 19) + M3.3
  (**Bend+Look At**, 20) + **Lattice+Voronoi** (23) + **Four-Point-Warp+Spherize** (24) + **Radial+Mirror** (25)
  + **Kaleidoscope+Collide** (26) + **Sort+Cull** (27) + **Distribute-Curve+Spline-Wrap** (28) → M3 completo;
  M4.1 (**Verlet-Rope+Boids**, 21) + M4.2 (**Soft-Body+Wave**, 22) → SIMULAÇÃO; **M1-cor (Color-Ramp+Color-
  Array**, 29) → abre a família de cor. **A cena boot atual demonstra Color-Ramp+Color-Array** (um sunburst
  arco-íris girando + um grid de paleta marchando); os nós das outras fatias ficam registrados/drop-in.
  (Também: 2 fixes de perf no voronoi + o plano `docs/plans/2026-07-gpu-resident-node-pipeline.md`.)
- **Auto-contida:** sem dependência de outra linha nem de outro módulo-feature → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última = sunburst rainbow + grid paleta, 2 cenas, 10 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + `build`→`Vec` de 2 sinks + contagem 10 + testes de integração (rainbow/palette, lêem `tint`) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 (beat→strobe) — NÃO depende da cena boot; **intocado** | shell, módulo Motion; baixo |
| `Cargo.lock` | só as crates PATH novas + a aresta `ph2d-color` (já era membro) | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado.

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` **GERADOS** (região `<ph2d-node-sync>`; **64 crates** agora). Qualquer outra
  linha que adicione um nó conflita aqui. **Resolução — NÃO fundir à mão:** `cargo run -p ph2d-node-sync`
  (regenera dos `crates/ph2d-node-*`; o gate `staleness` prova sync).

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **28 crates-nó novas**, tipos únicos: …(16–28 como antes)… + **`motion.color_ramp`**/**`motion.color_array`**
  (29). Grep: `grep -rnE '"motion\.(fibonacci|twist|scatter|morph|bend|look_at|verlet_rope|boids|soft_body|wave|lattice|voronoi|four_point_warp|spherize|distribute_radial|mirror|kaleidoscope|collide|sort|cull|distribute_curve|spline_wrap|color_ramp|color_array)"' crates/`.
- Helpers copiados por-crate (leaf): `trig.rs` · `hash.rs` · `atan2_approx` · `shape.rs` · `curve.rs`.
- **Deps:** `rayon` no voronoi (já no workspace) · **`ph2d-color` no color-ramp** — 1ª node-crate a depender de
  uma lib **foundational** além de `ph2d-nodegraph`/`-node-registry`. É foundational (não módulo-feature) e já
  membro do workspace → **nada novo no lockfile/RUSTSEC**. `color_array` não tem deps extra.
- **ZERO** `NodeId` numérico / token / variant de enum congelado novos.
- **Nota replay-hash:** o output do voronoi mudou (res adaptativa) → re-lockar golden se algum cobrir Motion.

### 4. Contratos congelados encostados: **NENHUM**
Gate `architecture_contract_surface` verde (2/1/8). Fan-out aditivo (caminho A). As sims usam o substrato
`pre`/`state` existente. **Zero acoplamento a módulo-feature** (vetor/timeline): o `distribute_curve`/`spline_
wrap` authoram a curva nos params; a cor usa a lib foundational `ph2d-color`.

### 5. O que só o `ship.sh` pega (o `foundational-integrate.sh` NÃO roda fmt/typos/machete/deny)
- **Drift pré-fork: BAIXO** — fork == tip de `1c7c9a22`. Rode **`ship.sh` completo** na árvore combinada.
  `nextest-impacted` funciona (só ADIÇÃO).
- fmt/machete/clippy `--all-targets -D warnings`/HR-5/LOC/typos verdes no fechamento (pin 1.95).
- **typos:** "Secord" parafraseado fora do voronoi. **`no_tofu_glyphs`:** setas `→` latentes em strings de teste
  trocadas por `->`/texto nas fatias 26/27/28/29 (o gate só roda no ship; escaneia só string literals).

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → depois `ship.sh`.
- **Se main moveu:** rebase → UM conflito mecânico em `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}`
  (`cargo run -p ph2d-node-sync`) + `Cargo.lock` → `foundational-integrate.sh` + `ship.sh`.
- **Smoke: 4/5/M3.1–3.3/M4.1/M4.2/Lattice+Voronoi/Four-Point-Warp+Spherize/Sort+Cull/Color-Ramp+Color-Array
  APROVADAS (Enio).** Radial+Mirror, Kaleidoscope+Collide, Distribute-Curve+Spline-Wrap: nós testados headless;
  só a cena boot MAIS RECENTE é vista por vez, então não tiveram smoke visual individual (a integração já foi
  provada). A cena atual demonstra **Color-Ramp+Color-Array**. Headless: `the_rainbow_sunburst_spins_and_is_
  colourful` + `the_palette_grid_marches`. Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA um **sunburst arco-íris de 60 pts** (anéis espectrais) que gira (`motion.color_
  ramp` Rainbow, spin ← sine lfo); à DIREITA um **grid 10×10 com paleta de 4 cores** marchando (`motion.color_
  array`, offset ← saw lfo). Editor: dropar `motion.color_ramp` (preset Rainbow/Heat/Ice/Gray/Custom + t input)
  e `motion.color_array` (2–4 cores + offset).

**Resumo:** *Linha `motion-value`, 14 fatias (fork `1c7c9a22`). Aditiva: 28 crates-nó (valor completo · M3
completo · SIMULAÇÃO · **cor M1**) + cena boot pequena (Color-Ramp+Color-Array) + 2 fixes perf voronoi + plano
GPU. Único conflito mecânico = codegen `registry-init` → `ph2d-node-sync` (64 crates). Zero substrato, zero
contrato congelado, zero acoplamento a módulo-feature; deps externas: `rayon` + `ph2d-color`, ambas já no
workspace. 11 fatias smoke-aprovadas; 3 pendentes de smoke visual individual. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (docs 16–29)

Fecha o **valor** (4–5), o **M3** completo (distribuições grid/fibonacci/scatter/lattice/voronoi/radial/curve +
deformers twist/morph/bend/look_at/four_point_warp/spherize/spline_wrap + simetria mirror/kaleidoscope +
empacotamento collide + estruturais sort/cull), a **SIMULAÇÃO** (M4.1/M4.2) e abre a **cor do M1** (ramp/array),
sempre pesquisando o padrão-ouro ANTES de codar. Pesquisa por fatia: docs 16–29.

**Fatia Distribute-Curve+Spline-Wrap (doc 28) — a família CURVA, self-contained:**
- **`motion.distribute_curve`**: N pontos por arc-length numa Bézier authored nos params (Blender "Curve to
  Points"); `offset` desliza. `Pure`, Source, `curve.rs`.
- **`motion.spline_wrap`**: enverga um layout numa Bézier (C4D "Spline Wrap"); `amount` flat↔wrapped, falloff-
  masked. `Pure`, Transform, `curve.rs`.

**Fatia Color-Ramp+Color-Array (doc 29) — a família COR, cauda M1 self-contained:**
- **`motion.color_ramp`** (`ph2d-node-motion-color-ramp`): colore por um escalar (índice/valor) num **gradiente
  multi-stop** via `ph2d-color::ColorRamp` (Blender "Color Ramp"); presets Rainbow/Heat/Ice/Gray/Custom, `t`
  value input. Escreve `tint`. `Pure`, Fx, dep foundational `ph2d-color`.
- **`motion.color_array`** (`ph2d-node-motion-color-array`): cicla uma **paleta** de 2–4 cores por `i mod
  colors` (MoGraph "Color"); `offset` value input marcha. Escreve `tint`. `Pure`, Fx.
- **Cena boot:** 2 cenas (10 nós) — sunburst rainbow girando + grid de paleta marchando.

(Fatias anteriores: docs 16–28.)

## 1. Gates no fechamento (paridade §7) — última fatia (Color-Ramp+Color-Array)
- **Unit:** `motion.color_ramp` 4 (grayscale-por-índice/t-field-sobrepõe/rainbow-abrange/cook) +
  `motion.color_array` 4 (cicla/offset-marcha/colors-limita/cook; falsificados).
- **Integração (shell, registry real, lê `tint`):** `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop
  motion` = **24 passed** (rainbow >20 cores + gira · paleta exatamente 4 cores + marcha · determinismo · loop-
  replay · motion_bridge).
- **Contrato:** `architecture_contract_surface` = 3 pass (2/1/8). **Registry:** `staleness` = 2 pass
  (**64 crates**).
- **Lint/estilo:** `clippy --all-targets -D warnings` = 0 · `cargo fmt` pin 1.95 · `typos` 0 · `cargo machete`
  0 (`ph2d-color` usado) · HR-5 = 0 no MEU código (a matemática de cor é da `ph2d-color`) · `no_tofu_glyphs` 0.
- **LOC:** color-ramp 377 / color-array 324 (cap 700); shell `motion_demo_strobe.rs` 137 / `motion_state_
  tests.rs` 168 / `motion_state.rs` 112 (cap 600).

## 2. Follow-up restante — a cauda M1 CONTINUA self-contained (correção)
> Correção ao handoff anterior: o poço self-contained **não** secou — secou só nas distribuições/deformers M3.
> A **cauda M1 (cor/expressão/adapters/streams)** segue self-contained (deps = libs foundational `ph2d-color`/
> `ph2d-expr`, já no workspace).
- **Cor (aberta):** `color-range-to-color` subsumido pelo `color_ramp` Custom. **Próximos self-contained:**
  `motion.expression` (VEX-lite → `ph2d-expr`; o mais poderoso do lote M1) · adapters (`value-to-color`,
  `luminance`, `threshold`, `gate`, `make-point`) · streams (`mixer` avg/add, `combine` concat).
- **M2 pendências:** wiring do scrub-back no transporte (`Cook::checkpoint/restore` já existem em cook.rs) ·
  `motion.delay` · `force.buoyancy`.
- **Cross-module (DEFERIDO):** `motion.distribute-path` (curva do documento vetorial) · `slit-scan`/`delay`
  (time-scope).
- **Fronteiras grandes:** **M4** (Rig+FX — necks `ParamSpec` tipado + `Func::Pow`) · **M5** (motor GPU
  `docs/plans/2026-07-gpu-resident-node-pipeline.md` — exige ADR + foundational → linha dedicada).

*"Linha `motion-value` com 14 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
