# HANDOFF de integração — linha `line/motion-value` (docs 16–26: valor + M3 + M4)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (tip do main na
  abertura → fork fresco, **sem drift pré-fork**).
- **ONZE fatias**, todas fan-out aditivo de nós + cena boot: 4 (**Math+Compare**, doc 16) + 5
  (**Switch+On Change**, doc 17) → domínio de valor COMPLETO; M3.1 (**Fibonacci+Twist**, 18) + M3.2
  (**Scatter+Morph**, 19) + M3.3 (**Bend+Look At**, 20) + M3-dist (**Lattice+Voronoi**, 23) + M3-def
  (**Four-Point-Warp+Spherize**, 24) + M3-radial (**Radial+Mirror**, 25) + M3-sim (**Kaleidoscope+Collide**,
  26) → M3 (6 distribuições + 6 deformers + simetria N-fold + empacotamento); M4.1 (**Verlet-Rope+Boids**, 21) +
  M4.2 (**Soft-Body+Wave**, 22) → SIMULAÇÃO (2 sims discretas + 2 contínuas). **A cena boot atual demonstra
  Kaleidoscope+Collide** (um mandala girando + um grid que se empacota respirando); os nós das outras fatias
  ficam registrados/drop-in. (Também: 2 fixes de perf no voronoi + o plano `docs/plans/2026-07-gpu-resident-
  node-pipeline.md`.)
- **Auto-contida:** sem dependência de outra linha → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última = mandala kaleidoscope + grid collide, 2 cenas, 12 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + `build`→`Vec` de 2 sinks + contagem 12 + testes de integração (mandala/packing) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 com doc PRÓPRIO (beat→strobe) — NÃO depende da cena boot; **intocado** | shell, módulo Motion; baixo |
| `Cargo.lock` | só as crates PATH novas | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado — as onze fatias são
100% fan-out de nós + cena.

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` são **GERADOS** (região `<ph2d-node-sync>`; **58 crates** agora). QUALQUER
  outra linha que adicione um nó conflita aqui.
- **Resolução — NÃO fundir à mão:** depois de juntar as árvores, rode **`cargo run -p ph2d-node-sync`**
  — regenera dos `crates/ph2d-node-*`; o gate `staleness` (em `ph2d-node-registry-init`) prova sync.

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **22 crates-nó novas**, tipos únicos: `value.math`/`pulse.compare` (16); `value.switch`/`pulse.on_change`
  (17); `motion.fibonacci`/`motion.twist` (18); `motion.scatter`/`motion.morph` (19); `motion.bend`/
  `motion.look_at` (20); `motion.verlet_rope`/`motion.boids` (21); `motion.soft_body`/`motion.wave` (22);
  `motion.lattice`/`motion.voronoi` (23); `motion.four_point_warp`/`motion.spherize` (24); `motion.distribute_
  radial`/`motion.mirror` (25); **`motion.kaleidoscope`**/**`motion.collide`** (26). Grep:
  `grep -rnE '"motion\.(fibonacci|twist|scatter|morph|bend|look_at|verlet_rope|boids|soft_body|wave|lattice|voronoi|four_point_warp|spherize|distribute_radial|mirror|kaleidoscope|collide)"' crates/`.
- Helpers copiados por-crate (convenção leaf, sem símbolo compartilhado): `trig.rs` (`cos_sin_cycles`, em
  fibonacci/twist/bend/radial/**kaleidoscope**), `hash.rs` (`hash3`, em scatter/boids/lattice/voronoi),
  `atan2_approx` inline em look_at, `shape.rs` (módulo irmão do soft_body). Colunas de estado de stream
  (`rope_prev`, `vel`, `sb_vel`, `wave_h`/`wave_prev`, `sim_t`) são **locais, não símbolos**.
- **ZERO** `NodeId` numérico / token / variant de enum congelado novos. **UMA dep externa nova, mas já no
  workspace:** `ph2d-node-motion-voronoi` usa `rayon = "1"` (paralelizar a busca do Lloyd — já era dep do
  `ph2d-tool-painter`, então nada novo no lockfile/RUSTSEC). **Nota replay-hash:** o output do voronoi mudou
  (res adaptativa; a paralelização é bit-idêntica ao serial, mas a res-adaptativa não) → se algum golden de
  replay-hash cobre a cena Motion, re-lockar no ship.

### 4. Contratos congelados encostados: **NENHUM**
Gate `architecture_contract_surface` verde (2/1/8). Fan-out aditivo (caminho A). Sem ADR necessário. As sims
usam o substrato `pre`/`state` que JÁ existe (como `motion.spring`) — nada novo no substrato.

### 5. O que só o `ship.sh` pega (o `foundational-integrate.sh` NÃO roda fmt/typos/machete/deny)
- **Drift pré-fork: BAIXO** — fork == tip de `1c7c9a22`; fmt (style_edition 2024, pin 1.95)/typos batem.
  Ainda assim rode **`ship.sh` completo** na árvore combinada. `nextest-impacted` funciona (só ADIÇÃO).
- fmt/machete/clippy `--all-targets -D warnings`/HR-5/LOC/typos rodados verdes no fechamento (pin 1.95).
  **Nota typos:** o nome próprio "Secord" (autor do stippling 2002) foi PARAFRASEADO fora do voronoi pra
  não tropeçar no `typos` bare (sem tocar o `.typos.toml` compartilhado).
- **Nota `no_tofu_glyphs`:** duas setas `→` latentes em string literals de teste (uma da fatia 25, uma nova em
  collide) foram trocadas por `->` no fechamento da fatia 26 — o gate de tofu só roda no ship/gate completo.

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → ff limpo, depois `ship.sh`.
- **Se main moveu:** rebase → UM conflito mecânico em `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}`
  (resolve com `cargo run -p ph2d-node-sync`, **não à mão**) + `Cargo.lock`. `motion_demo_strobe.rs`/
  `motion_state*` só conflitam se outra linha editar a cena Motion (improvável). Depois
  `scripts/foundational-integrate.sh` + `ship.sh`.
- **Smoke: fatias 4/5/M3.1–3.3/M4.1/M4.2/Lattice+Voronoi/Four-Point-Warp+Spherize APROVADAS (Enio,
  2026-07-11); Radial+Mirror e Kaleidoscope+Collide PENDENTES.** A cena boot é sempre PEQUENA e isola a fatia
  mais recente. A atual demonstra **Kaleidoscope+Collide**. Headless: `the_mandala_spins` +
  `the_grid_packs_apart_and_breathes`. Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA um **mandala âmbar** (uma espiral Fibonacci dobrada em 8 fatias espelhadas, 48
  dots) que gira (o `motion.kaleidoscope`, spin ← `value.lfo`); à DIREITA um **grid ciano 8×8** cujas células
  começam sobrepostas e são **empurradas** para um empacotamento (64 dots) que **respira** (o `motion.collide`,
  radius ← `value.lfo`). No editor: dropar `motion.kaleidoscope` (segments, reflect Rotational/Mirrored, pivot)
  e `motion.collide` (radius/iterations/strength, spread input).

**Resumo:** *Linha `motion-value`, 11 fatias (fork `1c7c9a22`). Aditiva: 22 crates-nó (valor completo · M3 com
6 distribuições + 6 deformers + simetria N-fold + empacotamento · SIMULAÇÃO com 2 discretas + 2 contínuas) +
cena boot pequena (Kaleidoscope+Collide) + 2 fixes de perf no voronoi + o plano GPU. Único conflito mecânico =
codegen `registry-init` → `ph2d-node-sync` (58 crates). Zero substrato, zero contrato congelado, dep externa só
`rayon` (já no workspace). 9 fatias smoke-aprovadas; Radial+Mirror e Kaleidoscope+Collide pendentes. Aguardo
ordem de integração.*

---

## 0. O que a linha entrega (docs 16–26)

Fecha o valor (4–5), o **M3** (distribuições grid/fibonacci/scatter/lattice/voronoi/radial + deformers
twist/morph/bend/look_at/four_point_warp/spherize + simetria mirror/kaleidoscope + empacotamento collide) e a
**SIMULAÇÃO** (M4.1 discretas, M4.2 contínuas), sempre pesquisando o padrão-ouro ANTES de codar. Pesquisa por
fatia: docs [16](Motion%20Nodes/16_math_compare_nota_adr.md)..[20](Motion%20Nodes/20_bend_look_at_nota_adr.md),
[21](Motion%20Nodes/21_verlet_rope_boids_nota_adr.md), [22](Motion%20Nodes/22_soft_body_wave_nota_adr.md),
[23](Motion%20Nodes/23_lattice_voronoi_nota_adr.md), [24](Motion%20Nodes/24_four_point_warp_spherize_nota_adr.md),
[25](Motion%20Nodes/25_radial_mirror_nota_adr.md), [26](Motion%20Nodes/26_kaleidoscope_collide_nota_adr.md).

**Fatia Radial+Mirror (doc 25) — array polar + simetria D₁:**
- **`motion.distribute_radial`** (`ph2d-node-motion-distribute-radial`): **array radial** (`count` pontos em
  `rings` anéis concêntricos, `spin` value input gira). `Pure`, Source, `cos_sin_cycles`.
- **`motion.mirror`** (`ph2d-node-motion-mirror`): **reflete + duplica** o layout no eixo (V/H) pelo
  centroide → `2·count` simétrico. `Pure`, Transform, sem trig/sqrt.

**Fatia Kaleidoscope+Collide (doc 26) — simetria N-fold + empacotamento (o poço auto-contido do M3 fecha):**
- **`motion.kaleidoscope`** (`ph2d-node-motion-kaleidoscope`): **simetria N-fold** (órbita da fonte sob o grupo
  diedral Dₙ — `segments` fatias giradas por `pivot`; `reflect` espelha as ímpares → mandala; a generalização
  do `motion.mirror` D₁). `spin` value input gira. `Pure`, Transform, `cos_sin_cycles`.
- **`motion.collide`** (`ph2d-node-motion-collide`): **empacotamento/push-apart** (restrição de não-penetração
  PBD — Müller 2007/Jakobsen 2001, o "Push Apart Effector" do C4D; pares mais perto que `2·radius` são
  empurrados até encostar, `iterations` varreduras Gauss–Seidel). Relaxação PURA da entrada (como o Lloyd do
  voronoi), `spread` value input respira o raio. `Pure`, Transform, aritmética + `sqrt`. Distinto do voronoi
  (CVT = densidade uniforme; collide = hard-radius).
- **Cena boot:** 2 cenas (12 nós) — mandala Fibonacci-dobrado (spin ← lfo) à esquerda + grid 8×8 empacotando
  (radius ← lfo) à direita.

(Fatias anteriores 4–5/M3.1–3.3/M4.1–4.2/Lattice+Voronoi/Four-Point-Warp+Spherize: ver docs 16–24.)

## 1. Gates no fechamento (paridade §7) — última fatia (Kaleidoscope+Collide)
- **Unit:** `motion.kaleidoscope` 8 (6 lógica + 2 trig; inclui conta×segments/reflect-espelha/pivô-fixo) +
  `motion.collide` 7 (separa-até-encostar/cluster-empacota/coincidentes-determinístico; falsificados).
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop motion`
  = **24 passed** (inclui as 2 novas — mandala 48 dots + packing 64 dots com collide quebrando o grid 0.45 +
  determinismo + loop-replay + motion_bridge params). Suíte roda em ~0.03s.
- **Contrato:** `architecture_contract_surface` = 3 pass (2/1/8). **Registry:** `staleness` = 2 pass
  (**58 crates**).
- **Lint/estilo:** `clippy --all-targets -D warnings` = 0 · `cargo fmt` pin 1.95 · `typos` 0 · `cargo machete`
  0 · HR-5 = 0 na produção (kaleidoscope: `cos_sin_cycles`; collide: aritmética + `sqrt`) · `no_tofu_glyphs` 0.
- **LOC:** kaleidoscope 383 / collide 371 (cap 700); shell `motion_demo_strobe.rs` 179 /
  `motion_state_tests.rs` 175 / `motion_state.rs` 114 (cap 600).

## 2. Follow-up restante (doc 26 §5)
- **Distribuições/deformers/simetria/empacotamento auto-contidos do M3: COMPLETOS** (grid/fibonacci/scatter/
  lattice/voronoi/radial + twist/morph/bend/look_at/four_point_warp/spherize + mirror/kaleidoscope + collide).
  O poço de nós que dependem só de `ph2d-nodegraph` está **essencialmente esgotado**.
- **Distribuição:** `motion.distribute-path` (curva — **integra vector.***; DEFERIDO).
- **Deformer:** `motion.slit-scan` (amostragem temporal; DEFERIDO).
- **Simulação:** `motion.pin_constraint` (port de constraints; DEFERIDO) · spatial hash (acelera collide/boids)
  · colisão contra bordas · **motor GPU** (`docs/plans/2026-07-gpu-resident-node-pipeline.md` — a próxima
  grande alavanca; exige ADR + toca foundational → linha dedicada).
- Straggler do M2: `motion.delay` (precisa do time-scope do editor, como `trail`/`time_remap`; DEFERIDO).

*"Linha `motion-value` com 11 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
