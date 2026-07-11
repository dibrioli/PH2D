# HANDOFF de integração — linha `line/motion-value` (docs 16–25: valor + M3 + M4)

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`.

---

## §1.5.9 — BRIEFING DO INTEGRADOR (LER PRIMEIRO)

### 1. Identidade
- Branch **`line/motion-value`** · **fork base (merge-base com `main`) = `1c7c9a22`** (tip do main na
  abertura → fork fresco, **sem drift pré-fork**).
- **DEZ fatias**, todas fan-out aditivo de nós + cena boot: 4 (**Math+Compare**, doc 16) + 5
  (**Switch+On Change**, doc 17) → domínio de valor COMPLETO; M3.1 (**Fibonacci+Twist**, 18) + M3.2
  (**Scatter+Morph**, 19) + M3.3 (**Bend+Look At**, 20) + M3-dist (**Lattice+Voronoi**, 23) + M3-def
  (**Four-Point-Warp+Spherize**, 24) + M3-radial (**Radial+Mirror**, 25) → M3 (5 distribuições + 6 deformers + simetria); M4.1 (**Verlet-Rope+Boids**, 21) +
  M4.2 (**Soft-Body+Wave**, 22) → SIMULAÇÃO (2 sims discretas + 2 contínuas). **A cena boot atual demonstra Radial+Mirror** (array radial + um voronoi 180 espelhado, o teste de perf ao vivo); os nós das outras fatias ficam
  registrados/drop-in. (Também: 2 fixes de perf no voronoi + o plano `docs/plans/2026-07-gpu-resident-node-
  pipeline.md`.)
- **Auto-contida:** sem dependência de outra linha → integra como bloco único.

### 2. Foundational/compartilhado tocado (fora de `crates/ph2d-node-*`) — tudo ADITIVO
| Arquivo | O quê | Nota p/ o integrador |
|---|---|---|
| `shells/desktop/src/motion_demo_strobe.rs` | cena boot (última = radial + voronoi(180)+mirror, 2 cenas, 11 nós) | shell, módulo Motion; baixo |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | doc-comments + `build`→`Vec` de 2 sinks + contagem 11 + testes de integração (radial/mirror + voronoi-perf) + determinismo | shell, módulo Motion; baixo |
| `shells/desktop/src/render_loop/motion_bridge_tests.rs` | loop-replay do doc 11 com doc PRÓPRIO (beat→strobe) — NÃO depende da cena boot; **intocado** | shell, módulo Motion; baixo |
| `Cargo.lock` | só as crates PATH novas | regenera (`cargo build`) na árvore combinada |

**Nenhum** arquivo de substrato (`ph2d-nodegraph/*`, `ph2d-eval-motion/*`) tocado — as dez fatias são
100% fan-out de nós + cena.

### 3. Ponto de conflito MECÂNICO (o único esperado): `ph2d-node-registry-init`
- `src/lib.rs` e `Cargo.toml` são **GERADOS** (região `<ph2d-node-sync>`; **56 crates** agora). QUALQUER
  outra linha que adicione um nó conflita aqui.
- **Resolução — NÃO fundir à mão:** depois de juntar as árvores, rode **`cargo run -p ph2d-node-sync`**
  — regenera dos `crates/ph2d-node-*`; o gate `staleness` (em `ph2d-node-registry-init`) prova sync.

**Símbolos novos (grep de mesmo-símbolo, §1.5.5):**
- **20 crates-nó novas**, tipos únicos: `value.math`/`pulse.compare` (16); `value.switch`/`pulse.on_change`
  (17); `motion.fibonacci`/`motion.twist` (18); `motion.scatter`/`motion.morph` (19); `motion.bend`/
  `motion.look_at` (20); `motion.verlet_rope`/`motion.boids` (21); `motion.soft_body`/`motion.wave` (22);
  `motion.lattice`/`motion.voronoi` (23); `motion.four_point_warp`/`motion.spherize` (24); **`motion.distribute_radial`**/**`motion.mirror`** (25). Grep:
  `grep -rnE '"motion\.(fibonacci|twist|scatter|morph|bend|look_at|verlet_rope|boids|soft_body|wave|lattice|voronoi|four_point_warp|spherize|distribute_radial|mirror)"' crates/`.
- Helpers copiados por-crate (convenção leaf, sem símbolo compartilhado): `trig.rs` (`cos_sin_cycles`, em
  fibonacci/twist/bend/**radial**), `hash.rs` (`hash3`, em scatter/boids/lattice/voronoi), `atan2_approx`
  inline em look_at, `shape.rs` (módulo irmão do soft_body). Colunas de estado de stream (`rope_prev`,
  `vel`, `sb_vel`, `wave_h`/`wave_prev`, `sim_t`) são **locais, não símbolos**.
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

### 6. Procedimento sugerido + smoke
- **Se main não moveu de `1c7c9a22`:** `git merge --ff-only line/motion-value` → ff limpo, depois `ship.sh`.
- **Se main moveu:** rebase → UM conflito mecânico em `ph2d-node-registry-init/{src/lib.rs,Cargo.toml}`
  (resolve com `cargo run -p ph2d-node-sync`, **não à mão**) + `Cargo.lock`. `motion_demo_strobe.rs`/
  `motion_state*` só conflitam se outra linha editar a cena Motion (improvável). Depois
  `scripts/foundational-integrate.sh` + `ship.sh`.
- **Smoke: fatias 4/5/M3.1–3.3/M4.1/M4.2/Lattice+Voronoi/Four-Point-Warp+Spherize APROVADAS (Enio,
  2026-07-11); fatia Radial+Mirror PENDENTE.** A cena boot é sempre PEQUENA e isola a fatia mais recente. A
  atual demonstra **Radial+Mirror** — e roda o voronoi a **count 180** de propósito (o número que caía pra
  20fps), pra o Enio **conferir a perf ao vivo**. Headless: `the_radial_array_swings_round` +
  `the_voronoi_is_mirrored_and_relaxes`. Smoke:
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → à ESQUERDA um **array radial âmbar** (3 anéis) que balança (o `motion.distribute_radial`,
  spin ← `value.lfo`); à DIREITA um **voronoi ciano de 180 pontos** que relaxa (Lloyd paralelizado — suave a
  esse count) e é **espelhado** numa colmeia simétrica de 360 (`motion.mirror`, relax ← `value.lfo`). No
  editor: dropar `motion.distribute_radial` (spin, sliders count/rings/radius) e `motion.mirror` (axis V/H).

**Resumo:** *Linha `motion-value`, 10 fatias (fork `1c7c9a22`). Aditiva: 20 crates-nó (valor completo · M3 com
5 distribuições + 6 deformers + simetria · SIMULAÇÃO com 2 discretas + 2 contínuas) + cena boot pequena
(Radial+Mirror, com voronoi 180 de perf-check) + 2 fixes de perf no voronoi + o plano GPU. Único conflito
mecânico = codegen `registry-init` → `ph2d-node-sync` (56 crates). Zero substrato, zero contrato congelado,
dep externa só `rayon` (já no workspace). 9 fatias smoke-aprovadas; Radial+Mirror pendente. Aguardo ordem de
integração.*

---

## 0. O que a linha entrega (docs 16–23)

Fecha o valor (4–5), o **M3** (M3.1–3.3 + Lattice/Voronoi — **4 distribuições** grid/fibonacci/scatter/
lattice/voronoi… na verdade 5 contando o grid pré-existente + **4 deformers** twist/morph/bend/look_at) e a
**SIMULAÇÃO** (M4.1 discretas, M4.2 contínuas), sempre pesquisando o padrão-ouro ANTES de codar. Pesquisa por
fatia: docs [16](Motion%20Nodes/16_math_compare_nota_adr.md)..[20](Motion%20Nodes/20_bend_look_at_nota_adr.md),
[21](Motion%20Nodes/21_verlet_rope_boids_nota_adr.md), [22](Motion%20Nodes/22_soft_body_wave_nota_adr.md),
[23_lattice_voronoi_nota_adr.md](Motion%20Nodes/23_lattice_voronoi_nota_adr.md).

**Fatia Lattice+Voronoi (doc 23) — as distribuições que faltavam:**
- **`motion.lattice`** (`ph2d-node-motion-lattice`): a **rede hexagonal/triangular** (empacotamento 2D mais
  denso, passo de linha `√3/2`, NN = spacing exato); `jitter` value input melta a colmeia. `Pure`, Source.
- **`motion.voronoi`** (`ph2d-node-motion-voronoi`): **relaxação de Lloyd** (CVT, grade `64²`) — a nuvem
  migra pros centroides de Voronoi; `relax` value input lerpa raw→CVT (organiza/dissolve). `Pure`, Source,
  `sqrt`-free.
- **Cena boot:** 2 cenas (10 nós) — colmeia (jitter ← lfo) à esquerda + nuvem-Lloyd (relax ← lfo) à direita.

**Fatia Four-Point-Warp+Spherize (doc 24) — os deformers que faltavam:**
- **`motion.four_point_warp`** (`ph2d-node-motion-four-point-warp`): **corner-pin projetivo** (homografia de
  Heckbert unit-square→quad, divisão de perspectiva → retas retas); cantos por offset-param, `warp` value
  input escala 0→1. `Pure`, Transform, sem trig/sqrt.
- **`motion.spherize`** (`ph2d-node-motion-spherize`): **lente radial** (bulge/pinch,
  `(p−c)·(1+amount·(1−(r/R)²))`); `amount` value input, `radius` param. `Pure`, Transform, um `sqrt`.
- **Cena boot:** 2 cenas (12 nós) — grade em perspectiva (warp ← lfo) à esquerda + grade lente (amount ← lfo)
  à direita.

**Fatia Radial+Mirror (doc 25) — array polar + simetria (+ perf-check do voronoi):**
- **`motion.distribute_radial`** (`ph2d-node-motion-distribute-radial`): **array radial** (`count` pontos em
  `rings` anéis concêntricos, `spin` value input gira). `Pure`, Source, `cos_sin_cycles`.
- **`motion.mirror`** (`ph2d-node-motion-mirror`): **reflete + duplica** o layout no eixo (V/H) pelo
  centroide → `2·count` simétrico. `Pure`, Transform, sem trig/sqrt.
- **Cena boot:** 2 cenas (11 nós) — array radial (spin ← lfo) à esquerda + **voronoi(180)+mirror** à direita
  (relax ← lfo; o voronoi a 180 pontos é o teste de perf ao vivo da paralelização).

(Fatias anteriores 4–5/M3.1–3.3/M4.1–4.2/Lattice+Voronoi/Four-Point-Warp+Spherize: ver docs 16–24.)

## 1. Gates no fechamento (paridade §7) — última fatia (Radial+Mirror)
- **Unit:** `motion.distribute_radial` 7 (5 lógica + 2 trig; inclui anel-único/count-exato/spin) +
  `motion.mirror` 4 (falsificados). Fatias anteriores: …/four_point_warp(6)/spherize(5)/lattice(7)/voronoi(7).
- **Integração (shell, registry real):** `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop motion`
  = **24 passed** (inclui as 2 novas — a do voronoi checa **360 pontos** [180 espelhado] + relaxação viva +
  determinismo + loop-replay + motion_bridge params). Suíte roda em ~0.25s (voronoi 180 paralelizado).
- **Contrato:** `architecture_contract_surface` = 3 pass (2/1/8). **Registry:** `staleness` = 2 pass
  (**56 crates**).
- **Lint/estilo:** `clippy --all-targets -D warnings` = 0 · `cargo fmt` pin 1.95 · `typos` 0 · `cargo machete`
  0 · HR-5 = 0 na produção (radial: `cos_sin_cycles`; mirror: aritmética pura).
- **LOC:** distribute-radial 283 / mirror 244 (cap 700); shell `motion_demo_strobe.rs` 191 /
  `motion_state_tests.rs` 152 / `motion_state.rs` 119 (cap 600).

## 2. Follow-up restante (doc 25 §5)
- **Distribuições/deformers auto-contidos do M3: COMPLETOS** (grid/fibonacci/scatter/lattice/voronoi/radial +
  twist/morph/bend/look_at/four_point_warp/spherize + mirror).
- **Distribuição:** `motion.distribute-path` (curva — **integra vector.***; DEFERIDO).
- **Deformer:** `motion.slit-scan` (amostragem temporal; DEFERIDO).
- **Simulação:** `motion.pin_constraint` (port de constraints; DEFERIDO) · spatial hash no boids · colisão ·
  **motor GPU** (`docs/plans/2026-07-gpu-resident-node-pipeline.md`).
- Straggler do M2: `motion.delay` (precisa do time-scope do editor, como `trail`/`time_remap`; DEFERIDO).

*"Linha `motion-value` com 10 fatias (fork em `1c7c9a22`). Handoff acima. Aguardo ordem de integração."*
