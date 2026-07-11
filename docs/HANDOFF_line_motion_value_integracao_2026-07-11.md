# HANDOFF de integração — linha `line/motion-value` (docs 16–28: valor + M3 + M4)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (tip do main na
  abertura → fork fresco, **sem drift pré-fork**).
- **TREZE fatias**, todas fan-out aditivo de nós + cena boot: 4 (**Math+Compare**, 16) + 5 (**Switch+On
  Change**, 17) → valor COMPLETO; M3.1 (**Fibonacci+Twist**, 18) + M3.2 (**Scatter+Morph**, 19) + M3.3
  (**Bend+Look At**, 20) + M3-dist (**Lattice+Voronoi**, 23) + M3-def (**Four-Point-Warp+Spherize**, 24) +
  M3-radial (**Radial+Mirror**, 25) + M3-sym (**Kaleidoscope+Collide**, 26) + M3-struct (**Sort+Cull**, 27) +
  M3-curve (**Distribute-Curve+Spline-Wrap**, 28) → M3 (7 distribuições + 7 deformers + simetria + empacotamento
  + estruturais); M4.1 (**Verlet-Rope+Boids**, 21) + M4.2 (**Soft-Body+Wave**, 22) → SIMULAÇÃO. **A cena boot
  atual demonstra Distribute-Curve+Spline-Wrap** (um marquee fluindo numa Bézier + um grid envergado numa
  S-curve); os nós das outras fatias ficam registrados/drop-in. (Também: 2 fixes de perf no voronoi + o plano
  `docs/plans/2026-07-gpu-resident-node-pipeline.md`.)
- **Auto-contida:** sem dependência de outra linha nem de outro módulo → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última = marquee curva + ribbon spline-wrap, 2 cenas, 11 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + `build`→`Vec` de 2 sinks + contagem 11 + testes de integração (marquee/ribbon) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 com doc PRÓPRIO (beat→strobe) — NÃO depende da cena boot; **intocado** | shell, módulo Motion; baixo |
| `Cargo.lock` | só as crates PATH novas | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado — as treze fatias são
100% fan-out de nós + cena.

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` são **GERADOS** (região `<ph2d-node-sync>`; **62 crates** agora). QUALQUER
  outra linha que adicione um nó conflita aqui.
- **Resolução — NÃO fundir à mão:** depois de juntar as árvores, rode **`cargo run -p ph2d-node-sync`**
  — regenera dos `crates/ph2d-node-*`; o gate `staleness` (em `ph2d-node-registry-init`) prova sync.

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **26 crates-nó novas**, tipos únicos: `value.math`/`pulse.compare` (16); `value.switch`/`pulse.on_change`
  (17); `motion.fibonacci`/`motion.twist` (18); `motion.scatter`/`motion.morph` (19); `motion.bend`/
  `motion.look_at` (20); `motion.verlet_rope`/`motion.boids` (21); `motion.soft_body`/`motion.wave` (22);
  `motion.lattice`/`motion.voronoi` (23); `motion.four_point_warp`/`motion.spherize` (24); `motion.distribute_
  radial`/`motion.mirror` (25); `motion.kaleidoscope`/`motion.collide` (26); `motion.sort`/`motion.cull` (27);
  **`motion.distribute_curve`**/**`motion.spline_wrap`** (28). Grep: `grep -rnE '"motion\.(fibonacci|twist|scatter|morph|bend|look_at|verlet_rope|boids|soft_body|wave|lattice|voronoi|four_point_warp|spherize|distribute_radial|mirror|kaleidoscope|collide|sort|cull|distribute_curve|spline_wrap)"' crates/`.
- Helpers copiados por-crate (convenção leaf, sem símbolo compartilhado): `trig.rs` (`cos_sin_cycles`, em
  fibonacci/twist/bend/radial/kaleidoscope), `hash.rs` (`hash3`/`rand01`, em scatter/boids/lattice/voronoi/
  sort), `atan2_approx` inline em look_at, `shape.rs` (soft_body), **`curve.rs`** (Bézier + arc-length LUT, em
  distribute_curve/spline_wrap). Colunas de estado de stream (`rope_prev`, `vel`, `sb_vel`, `wave_h`/`wave_
  prev`, `sim_t`) são **locais, não símbolos**.
- **ZERO** `NodeId` numérico / token / variant de enum congelado novos. **UMA dep externa nova, mas já no
  workspace:** `ph2d-node-motion-voronoi` usa `rayon = "1"` (já era dep do `ph2d-tool-painter`, nada novo no
  lockfile/RUSTSEC). **Nota replay-hash:** o output do voronoi mudou (res adaptativa) → se algum golden de
  replay-hash cobre a cena Motion, re-lockar no ship.

### 4. Contratos congelados encostados: **NENHUM**
Gate `architecture_contract_surface` verde (2/1/8). Fan-out aditivo (caminho A). Sem ADR necessário. As sims
usam o substrato `pre`/`state` que JÁ existe (como `motion.spring`) — nada novo no substrato. **Nenhum
acoplamento ao módulo vetor:** o `distribute_curve`/`spline_wrap` authoram a curva nos params (a versão que lê
o documento vetorial, `distribute-path`, é cross-module e fica deferida).

### 5. O que só o `ship.sh` pega (o `foundational-integrate.sh` NÃO roda fmt/typos/machete/deny)
- **Drift pré-fork: BAIXO** — fork == tip de `1c7c9a22`; fmt (style_edition 2024, pin 1.95)/typos batem.
  Ainda assim rode **`ship.sh` completo** na árvore combinada. `nextest-impacted` funciona (só ADIÇÃO).
- fmt/machete/clippy `--all-targets -D warnings`/HR-5/LOC/typos rodados verdes no fechamento (pin 1.95).
  **Nota typos:** o nome próprio "Secord" (stippling 2002) foi PARAFRASEADO fora do voronoi (sem tocar o
  `.typos.toml`).
- **Nota `no_tofu_glyphs`:** setas `→` latentes em string literals de teste foram trocadas por `->` no
  fechamento das fatias 26/27/28 — o gate de tofu só roda no ship/gate completo; escaneia só string literals,
  não comentários (as setas nos doc-comments das cenas passam).

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → ff limpo, depois `ship.sh`.
- **Se main moveu:** rebase → UM conflito mecânico em `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}`
  (resolve com `cargo run -p ph2d-node-sync`, **não à mão**) + `Cargo.lock`. Depois
  `scripts/foundational-integrate.sh` + `ship.sh`.
- **Smoke: 4/5/M3.1–3.3/M4.1/M4.2/Lattice+Voronoi/Four-Point-Warp+Spherize e Sort+Cull APROVADAS (Enio).**
  Radial+Mirror, Kaleidoscope+Collide e Distribute-Curve+Spline-Wrap: nós testados headless; só a cena boot
  MAIS RECENTE é vista por vez, então essas 3 não tiveram smoke visual individual (a integração já foi provada
  com o Sort+Cull ok sobre o registry completo). A cena atual demonstra **Distribute-Curve+Spline-Wrap**.
  Headless: `the_curve_marquee_flows` + `the_grid_wraps_onto_the_spline`. Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA (âmbar) um **marquee de 24 dots fluindo numa Bézier** (`motion.distribute_curve`,
  offset ← um saw `value.lfo`); à DIREITA (ciano) um **grid 3×12 envergado numa S-curve** (`motion.spline_wrap`,
  amount ← um sine `value.lfo`, flat↔wrapped). No editor: dropar `motion.distribute_curve` (count + 4 control
  points + offset) e `motion.spline_wrap` (height/offset + 4 control points + amount).

**Resumo:** *Linha `motion-value`, 13 fatias (fork `1c7c9a22`). Aditiva: 26 crates-nó (valor completo · M3 com
7 distribuições + 7 deformers + simetria + empacotamento + estruturais + curva · SIMULAÇÃO 2 discretas + 2
contínuas) + cena boot pequena (Distribute-Curve+Spline-Wrap) + 2 fixes de perf no voronoi + o plano GPU. Único
conflito mecânico = codegen `registry-init` → `ph2d-node-sync` (62 crates). Zero substrato, zero contrato
congelado, zero acoplamento vetor, dep externa só `rayon` (já no workspace). 10 fatias smoke-aprovadas; 3
pendentes de smoke visual individual. Aguardo ordem de integração.*

---

## 0. O que a linha entrega (docs 16–28)

Fecha o valor (4–5), o **M3** (distribuições grid/fibonacci/scatter/lattice/voronoi/radial/**curve** +
deformers twist/morph/bend/look_at/four_point_warp/spherize/**spline_wrap** + simetria mirror/kaleidoscope +
empacotamento collide + estruturais sort/cull) e a **SIMULAÇÃO** (M4.1 discretas, M4.2 contínuas), sempre
pesquisando o padrão-ouro ANTES de codar. Pesquisa por fatia: docs
[16](Motion%20Nodes/16_math_compare_nota_adr.md)..[20](Motion%20Nodes/20_bend_look_at_nota_adr.md),
[21](Motion%20Nodes/21_verlet_rope_boids_nota_adr.md), [22](Motion%20Nodes/22_soft_body_wave_nota_adr.md),
[23](Motion%20Nodes/23_lattice_voronoi_nota_adr.md), [24](Motion%20Nodes/24_four_point_warp_spherize_nota_adr.md),
[25](Motion%20Nodes/25_radial_mirror_nota_adr.md), [26](Motion%20Nodes/26_kaleidoscope_collide_nota_adr.md),
[27](Motion%20Nodes/27_sort_cull_nota_adr.md), [28](Motion%20Nodes/28_distribute_curve_spline_wrap_nota_adr.md).

**Fatia Sort+Cull (doc 27) — os operadores ESTRUTURAIS do stream:**
- **`motion.sort`** (`ph2d-node-motion-sort`): **reordena** por chave (Sort SOP — Radial/X/Y/Random/Index,
  estável, `descending`). `Pure`, Utility, `hash.rs`.
- **`motion.cull`** (`ph2d-node-motion-cull`): **poda** por predicado (Blast/Delete SOP — Fraction/Falloff,
  `invert`). O primeiro nó que ENCOLHE a contagem. `amount` value input anima. `Pure`, Utility.

**Fatia Distribute-Curve+Spline-Wrap (doc 28) — a família CURVA, self-contained (o poço M3 fecha):**
- **`motion.distribute_curve`** (`ph2d-node-motion-distribute-curve`): **coloca N pontos por arc-length** numa
  Bézier cúbica authored nos params (Blender "Curve to Points"); `offset` value input desliza (wrap). `Pure`,
  Source, `curve.rs`.
- **`motion.spline_wrap`** (`ph2d-node-motion-spline-wrap`): **enverga um layout** numa Bézier (C4D "Spline
  Wrap" — bbox-X → arco, Y → normal); `amount` value input mistura flat↔wrapped, falloff-masked. Mais geral que
  o `bend` (arco circular). `Pure`, Transform, `curve.rs`.
- **Cena boot:** 2 cenas (11 nós) — marquee fluindo (saw offset) à esquerda + ribbon envergado (sine amount) à
  direita.

(Fatias anteriores 4–5/M3.1–3.3/M4.1–4.2/Lattice+Voronoi/Four-Point-Warp+Spherize/Radial+Mirror/Kaleidoscope+
Collide: docs 16–26.)

## 1. Gates no fechamento (paridade §7) — última fatia (Distribute-Curve+Spline-Wrap)
- **Unit:** `motion.distribute_curve` 7 (4 lógica + 3 curve; uniforme-na-reta/on-curve/offset-desliza) +
  `motion.spline_wrap` 8 (5 lógica + 3 curve; amount-0/reta-fica-reta/curva-enverga/falloff; falsificados).
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop motion`
  = **24 passed** (inclui as 2 novas — marquee 24 dots flui + ribbon 36 dots envergam [y-extent cresce] +
  determinismo + loop-replay + motion_bridge params).
- **Contrato:** `architecture_contract_surface` = 3 pass (2/1/8). **Registry:** `staleness` = 2 pass
  (**62 crates**).
- **Lint/estilo:** `clippy --all-targets -D warnings` = 0 · `cargo fmt` pin 1.95 · `typos` 0 · `cargo machete`
  0 · HR-5 = 0 na produção (curva/derivada polinomiais + `sqrt` só nas cordas/normal) · `no_tofu_glyphs` 0.
- **LOC:** distribute-curve 261+curve 136 / spline-wrap 382+curve 136 (cap 700); shell `motion_demo_strobe.rs`
  149 / `motion_state_tests.rs` 155 / `motion_state.rs` 112 (cap 600).

## 2. Follow-up restante (doc 28 §5)
- **Todo o M3 auto-contido: COMPLETO** — distribuições (grid/fibonacci/scatter/lattice/voronoi/radial/curve),
  deformers (twist/morph/bend/look_at/four_point_warp/spherize/spline_wrap), simetria (mirror/kaleidoscope),
  empacotamento (collide), estruturais (sort/cull). O poço de nós que dependem só de `ph2d-nodegraph` está
  **esgotado** — todo próximo nó é cross-module ou é a fronteira GPU.
- **Distribuição:** `motion.distribute-path` — a versão que lê a curva do **documento vetorial** (`vector.*`);
  cross-module (crate satélite que só LÊ o contrato vetor). DEFERIDA.
- **Deformer:** `motion.slit-scan` (amostragem temporal; DEFERIDO).
- **Simulação:** `motion.pin_constraint` (port de constraints; DEFERIDO) · spatial hash (acelera collide/boids)
  · colisão contra bordas · **motor GPU** (`docs/plans/2026-07-gpu-resident-node-pipeline.md` — a próxima
  grande alavanca; exige ADR + toca foundational → linha dedicada).
- Straggler do M2: `motion.delay` (precisa do time-scope do editor; DEFERIDO).

*"Linha `motion-value` com 13 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
