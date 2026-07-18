# HANDOFF / Tracker — `line/physics` (o motor de física global)

> **Tracker VIVO do módulo** (o `docs/HANDOFF_*` da física). Toda jornada futura **atualiza este
> arquivo**: estado por-wave, decisões, gotchas, ids/consts alocados. LLM nova lê ISTO + a
> [ADR-0130](../architecture/decisions/0130-physics-global-runtime-truth-rapier-ecs-bridge.md) +
> [`00_plano_waves.md`](00_plano_waves.md) antes de tocar código.
>
> **Norte (não re-litigar):** runtime-truth + bake opcional; rígido primeiro; solver = `rapier2d 0.28`
> (M10, já determinístico) — esta linha escreve **integração e autoria**, não solver.

---

## Estado por-wave

| Wave | Estado | Commit | Nota |
|---|---|---|---|
| **W0 — Arquitetura** | ✅ **FECHADO** (2026-07-17) | `456e8b99` | ADR-0130 + plano de waves + tracker + visão. **Zero código.** |
| **W1 — Ponte ECS + tick + hash** | ✅ **FECHADO** (2026-07-17, pendente smoke) | `44e08cf5`→`9f5fee05` | o alicerce — ver §W1 abaixo |
| **W1.5 — Scrub (checkpoint ring)** | ⏳ pendente | — | kill-check de serialização ANTES do build |
| **W2 — Painel + Inspector body** | ⏳ pendente | — | categoria de painel NOVA (mundo) |
| **W3 — Joints** | ⏳ pendente | — | pêndulo/corrente/ragdoll |
| **W4 — Bake-to-timeline** | ⏳ pendente | — | acopla `ph2d-anim` (outra linha) |

**W0 entregou:** [ADR-0130](../architecture/decisions/0130-physics-global-runtime-truth-rapier-ecs-bridge.md) ·
[`00_plano_waves.md`](00_plano_waves.md) · [`01_visao.md`](01_visao.md) · este tracker. Nenhuma linha de
código, nenhum contrato tocado, nenhum foundational tocado.

---

## §W1 — O alicerce LANDOU (2026-07-17, pendente smoke)

Um sprite com `RigidBody{Dynamic}` + `Collider` **cai e assenta** sobre um `Collider{Static}` no ECS real,
ao dar play, e o mundo é determinístico. A ponte promoveu o wrapper M10 de dormente a **wired e global**.

**Crate-ponte nova `ph2d-physics-ecs`** (glob `crates/*` a pega — zero edit central): components
`RigidBody{kind}` + `Collider{shape,density}` (**config only** — nunca estado vivo de solver, senão o
`canonicalize` do undo diffaria um passo espúrio por tick); `PhysicsBridge` (owns `PhysicsWorld` +
`BTreeMap<Entity, handle>` + `last_stepped`); `register_physics_components`; `deterministic_hash` sobre os
`Transform` do readback; bin `physics_ecs_c9`.

**A ponte (`bridge.rs`), o coração:** `dispatch(sim, playing, target)` — **play** = `reconcile_structure`
(spawn/remove em ordem entity-sorted, HR-5) + `step()×(target−last_stepped)` sequencial + `readback`
(pose→`Transform`, só corpos Dynamic); **paused** = `settle` (corpos seguem o `Transform` autorado,
read-only no Transform ⇒ frame parado não gera passo de undo). `QueryState` cacheado (zero-alloc, idiom do
`propagate_transforms`). O `BTreeMap` (não `HashMap`) é a **espinha do determinismo**: itera por `Entity`,
ordem estável per-run e cross-OS; a lint disallowed-`HashMap` é o guarda estrutural.

**`ph2d-physics` estendido (append-only, meu módulo):** `spawn_body(BodyDesc)`/`set_body_pose`/`remove_body`
+ `BodyDesc`/`ShapeDesc` — cobre os 4 combos body×shape. **Os helpers existentes + `step` + o hash c9
ficaram byte-idênticos** ⇒ o gate M10 (`physics-c9`) segue verde (`2114f483…`).

**Escala (D4 CORRIGIDO medindo):** o `Transform` já é METROS (Y-up, radianos CCW); rapier é metros ⇒
**fronteira 1:1, sem conversão**. A única px→m já existe: `ProjectSettings.pixels_per_meter` (default 100)
no import, do PROJETO. **NÃO** criei um 2º `PIXELS_PER_METER` (seria a 2ª porta que diverge).

**Shell wired:** `AppGfx.physics: PhysicsBridge` (ao lado de `sim`/`motion`); `render_loop/physics_bridge::
dispatch` chamado **antes de `sim_extract`** (mod.rs, corpo renderiza same-frame; `target =
round(playhead.time()/dt)`, `playing = is_playing`); `register_physics_components` no boot (`init.rs`);
smoke `physics_smoke.rs`. **Persistência:** `PROJECT_SCHEMA` **15→16** + a **tripla-pin** de
`project_tests` para `(16,7,8)` (o gate disparou no bump — o valor se CONTA); `physics.rebuild()` no reset
de load (mundo é derivado — D2; reconcile self-heal é o backstop).

**Gates (6, todos mutation-verified RED-first):** e2e falls-and-settles (kill readback→RED) · determinismo
repeatability (guarda estrutural = BTreeMap + lint + CI cross-OS) · zero-alloc capacity (kill `seen.clear()`
→RED) · registry count=2 (kill um register→RED) · round-trip de snapshot (kill um register→RED) · self-heal
no respawn (kill remoção de stale→RED). **CI:** `physics-ecs-c9` na matriz do `spike.yml` + compare cross-OS
(`sort -u | wc -l`) — mirror do `ph2d_physics_c9`.

**Batched gate verde:** fmt · clippy `--all-targets` · `cargo check --workspace` · `nextest-impacted` (723
passed, 5 skipped).

**⚠️ SMOKE (Enio):** `cd Worktrees/line-physics && PH2D_PHYSICS_SMOKE=1 cargo run -p ph2d-host-desktop` —
uma bola laranja (dynamic) deve **cair e assentar** sobre a barra cinza (static floor). Ponte morta = bola
pendurada no ar.

**Deferido (por design, não esquecido):** scrub-back re-sim = **W1.5** (o `settle` seta `last_stepped=target`
no paused; scrub não rebobina o corpo ainda — o ring é a próxima wave, com o **kill-check de serialização
do rapier ANTES do build**). restituição/atrito/damping/Kinematic/camadas = **W2** (append + wire no painel).
`readback` só trata corpo root (Transform local = mundo); corpo filho = W2 (via `parent_world_transform`).
`reconcile` stale é O(N²) (trivial nos counts de W1).

---

## Decisões (ADR-0130, condensadas — o *porquê* está lá)

- **D1** runtime-truth + bake opcional (Enio). **D2** `PhysicsWorld` transiente shell-side (precedente
  `MotionCookPump`), dirigido por components; NÃO persistido (é rebuild). **D3** contrato
  `RigidBody`/`Collider` append-only, registrado pela crate-ponte, destinado a congelar. **D4** escala
  **D4 corrigido no W1: sem porta de escala** — `Transform` já é metros = rapier metros (1:1); a única px→m
  é `ProjectSettings.pixels_per_meter` no import (do projeto). **D5** relógio no `Playhead`
  (`ticks_owed`); scrub por **checkpoint ring esparso** (modelo `CheckpointRing`/`Cook`). **D6** fronteira
  tríplice (rapier / Zona-de-nós / XPBD). **D7** hash do mundo-ECS estende o gate c9 cross-OS. **D8**
  painel global (categoria nova) + seção "Physics Body" no Inspector. **D9** rígido apenas; 0063 fora.
  **D10** budgets 1,5 ms / 20 MB / zero-alloc. **D11** bake via `fit_fcurve`/Schneider.

---

## Terreno verificado on-disk (2026-07-17 — NÃO re-derive; cite daqui)

### O que herda pronto — `ph2d-physics` (M10)
- [`crates/ph2d-physics/src/world.rs`](../../crates/ph2d-physics/src/world.rs) (320 LOC,
  `#![forbid(unsafe_code)]`): `PhysicsWorld::new/set_gravity/set_dt/dt/step_count/add_dynamic_circle/
  add_static_cuboid/insert_body/bodies[_mut]/colliders[_mut]/step/body_pose/body_snapshots/
  deterministic_hash`. `step()` **sempre** usa `dt` interno (HR-5). `DEFAULT_DT=1/60`,
  `DEFAULT_GRAVITY_Y=-9.81`, mundo **Y-up**. `BodySnapshot{handle_index,x,y,rotation,linvel_x,linvel_y,angvel}`
  ordenado por `handle_index`; `deterministic_hash` = blake3 sobre snapshots ordenados (`to_bits` LE).
- [`crates/ph2d-physics/Cargo.toml`](../../crates/ph2d-physics/Cargo.toml): `rapier2d = "0.28"`,
  `default-features=false`, features `dim2`/`f32`/`enhanced-determinism` + `blake3`. **NUNCA** ligar
  `parallel`/`simd-stable`/`simd-nightly`.
- Bin [`c9.rs`](../../crates/ph2d-physics/src/bin/c9.rs): 50 corpos + chão, 120 steps, imprime
  `physics-c9 hash: <hex64>`.

### O gate cross-OS REAL (o path da SKILL não existe)
- **`.github/workflows/spike.yml`**: job `determinism` (matriz `[ubuntu-latest, macos-latest,
  windows-latest]`, `fail-fast:false`) roda `cargo run --release --locked --bin ph2d_physics_c9
  -p ph2d-physics`, parseia `grep -E '^physics-c9 hash: ' | awk '{print $3}'`, sobe artifact
  `physics-c9-hash-${os}`. Job `determinism-compare` (needs `determinism`) baixa os 3 e exige
  `sort -u | wc -l == 1`.
- ⚠️ **`tests/determinism/replay_cross_platform.rs` NÃO existe on-disk** (a SKILL mente). A verdade é o
  `spike.yml` + os bins `c9.rs` (physics) e `tests/spike/src/bin/c9_replay.rs` (ECS). **W1 adiciona
  `physics-ecs-c9`** (novo bin/harness + etapa de matriz + artifact + comparação).

### O relógio
- [`crates/ph2d-core/src/time.rs`](../../crates/ph2d-core/src/time.rs): `FixedStep` — `DEFAULT_HZ=60.0`
  (f64), `DEFAULT_MAX_SUBSTEPS=8`, `advance(wall_dt)->FixedStepReport{ticks:u32,alpha:f32,dropped_secs:f64}`,
  `tick_count()->u64`, `fixed_dt()->f64`.
- [`crates/ph2d-core/src/playhead.rs`](../../crates/ph2d-core/src/playhead.rs): `Playhead` — `time:f64` seg,
  `advance()` move só se `playing`, `advance_ticks(n)`, `seek/seek_frame` (scrub, não muda play state),
  `rewind()` (time=0, mantém rate+play), `is_playing`, loop Wrap/PingPong. Sequência bit-idêntica cross-OS
  (HR-5).
- **Precedente Motion** [`shells/desktop/src/render_loop/motion_bridge.rs`](../../shells/desktop/src/render_loop/motion_bridge.rs):
  `ticks_owed(last_cooked, target) -> RangeInclusive<u64>` (`Some(last) if target>last => last+1..=target`;
  senão `target..=target`); caller `for tick in ticks_owed(...) { pump.advance_or_scrub_scoped(...) }`;
  `target = round(playhead.time()/fixed_dt)`. **`MotionTransport` MORREU** — um relógio.

### O checkpoint (modelo do scrub — W1.5)
- [`crates/ph2d-nodegraph/src/cook.rs`](../../crates/ph2d-nodegraph/src/cook.rs): `CookCheckpoint`,
  `checkpoint()->CookCheckpoint`, `restore(&cp)` (reinstala estado + limpa memo/live-scope, mantém revision
  clock). GGPO save/load/advance.
- [`crates/ph2d-eval-motion/src/checkpoint.rs`](../../crates/ph2d-eval-motion/src/checkpoint.rs):
  `RECENT_CAPACITY=300` (~5 s @60Hz), `CheckpointRing{recent:VecDeque<(u64,CookCheckpoint)>}` denso,
  `record`/`anchor_at_or_before(target)->(u64,cp)`/`should_record`/`clear` (no `mark_dirty`). Física usa
  cadência **esparsa** (estado maior).

### Registro de components (a armadilha do snapshot)
- [`crates/ph2d-ecs/src/scene/registry.rs`](../../crates/ph2d-ecs/src/scene/registry.rs):
  `register::<T>("ph2d::ecs::Nome")`; ids = blake3(name) 8 bytes LE. `register_ecs_components(reg)` +
  tripwire `register_ecs_components_populates_registry` (`reg.len()==32`, *"este número existe para doer"*).
  **Padrão:** a crate-home possui `register_*` e o boot agrega
  ([`shells/desktop/src/init.rs`](../../shells/desktop/src/init.rs), ao lado de `register_render_components`).
  Physics segue isso → `register_physics_components` na crate-ponte, contagem-32 de `ph2d-ecs` **intocada**.

### Painel docado — 5 sites (canônico: `ph2d-panel-vector`)
1. `impl Panel` (`ID`/`NODE_ID`/`DEFAULT_VISIBLE`/`populate`/`paint`/`apply_event`).
2. push no `ph2d-panel-registry-init` (GERADO por `ph2d-panel-sync`) + const `EXPECTED_TYPED` à mão.
3. feature Cargo `panel-<x>`.
4. **lista de fallback de z-order em `hero/paint.rs`** (sem ela = registrado+visível mas NUNCA pintado).
5. visibilidade dirigida pela ponte (`hero.panel_visibility.insert("<x>", ...)` no `render_loop`).

### Fora de escopo (Chesterton)
- **ADR-0063** (collider-gen vetorial + fratura dinâmica): amarrada ao `ph2d-vector-runtime` que a
  **ADR-0108** aposentou. Motor app-level **não reabre a 0108 nem herda os mecanismos da 0063**.
- **XPBD soft** (`ph2d-physics-soft`, M13+) e **FLIP/PIC** (`ph2d-fluids`, M13+): linhas próprias.

---

## Ids / consts / variants — ALOCADOS e A ALOCAR (regra §1.5.9.3)

**Alocados e CRIADOS no W1:**
- Crate-ponte **`ph2d-physics-ecs`** (glob `crates/*` — zero edit central). Components `RigidBody`/`Collider`;
  enums `BodyKind{Dynamic,Static}` / `ColliderShape{Ball,Cuboid}` (append-only, variants novos no FIM).
  Nomes canônicos de registro: **`ph2d::physics::RigidBody`** / **`ph2d::physics::Collider`**.
  `register_physics_components`; `PhysicsBridge`; bin **`physics_ecs_c9`**.
- `ph2d-physics` (aditivo): `BodyDesc`/`ShapeDesc`/`spawn_body`/`set_body_pose`/`remove_body`.
- Shell: campo **`AppGfx.physics`**; módulo **`render_loop/physics_bridge`**; **`mod physics_smoke`**;
  **`App.physics_smoke_done`**; feature de Cargo `ph2d-physics-ecs` (dep de path no shell).
- Env de smoke: **`PH2D_PHYSICS_SMOKE`** (=1 usado; 2=painel/3=joint/4=bake **reservados**).
- CI: **`physics-ecs-c9`** + artifact **`physics-ecs-c9-hash-${os}`** (spike.yml).
- **`PROJECT_SCHEMA` = 16** (era 15) + tripla-pin `(16,7,8)` em `project_tests`.
- ADR **0130**.
- ~~`PIXELS_PER_METER`~~ **NÃO existe** — D4 corrigido; reusa `ProjectSettings.pixels_per_meter`.

**A alocar na wave que os cria (próximo LIVRE):**
- W2: crate `ph2d-panel-physics`, `Panel::ID="physics"`, feature `panel-physics`, `ids::PHYSICS_PANEL`,
  `EXPECTED_TYPED` bump em `ph2d-panel-registry-init`; campos novos append em `RigidBody`/`Collider`
  (restituição/atrito/damping/…) + variants `Kinematic`/formas.
- W3: `PROJECT_SCHEMA` **16 → 17** (joints) + a tripla-pin; components de joint.

---

## Handoff de INTEGRAÇÃO — W0 + W1 (§1.5.9)

> Reportar ao Enio e **PARAR** (regra E/H). NÃO integrar, NÃO pushar.

1. **Identidade:** branch `line/physics`; base (merge-base com main) = `cdc3acc1`; HEAD = **`9f5fee05`**;
   **5 commits** (W0: `456e8b99`,`bc7a5990` docs · W1: `44e08cf5` core, `018b00e9` wiring, `9f5fee05` gate).
2. **Foundational/compartilhado tocado:**
   - `crates/ph2d-physics/` — **meu módulo** (regra B), **aditivo**: `spawn_body`/`set_body_pose`/
     `remove_body` + `BodyDesc`/`ShapeDesc`. Helpers existentes + `step` + c9 **byte-idênticos** (hash
     `physics-c9` intacto = `2114f483…`).
   - `shells/desktop/` (o consumidor É parte do work item): `Cargo.toml` (+dep), `app_state.rs` (+campo
     `physics` + `physics_smoke_done`), `init.rs` (+construtor + registro), `main.rs` (+`mod physics_smoke`
     + init do latch), `project.rs` (schema 15→16 + `rebuild()` no load), `project_tests.rs` (tripla-pin),
     `render_loop/mod.rs` (+`mod physics_bridge` + `dispatch` antes do `sim_extract`), **novos**
     `physics_smoke.rs` + `render_loop/physics_bridge.rs`.
   - `.github/workflows/spike.yml` (+step/artifact/compare `physics-ecs-c9`). `Cargo.lock`.
   - **`ph2d-ecs` NÃO foi tocado** (só lido; o registro mora na minha crate).
3. **Símbolos que podem COLIDIR (grep na integração):**
   - **ADR `0130`** — 4 linhas ativas podem reclamá-lo; gate `architecture_adr_numbers_are_unique`. Renomeio
     escopado a `git diff --name-only`, **nunca** `git grep` de árvore ([[feedback_a_token_rewrite_scopes_to_changed_files_not_the_whole_tree]]).
   - **`PROJECT_SCHEMA` = 16 + a tripla-pin `(16,7,8)`** — ⚠️ **se OUTRA linha também bumpar o schema, o
     valor se CONTA, não se escolhe** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]): some os
     dois deltas (ex.: se outra linha subiu p/ 16 por outro motivo, o combinado é 17) e atualize a tripla.
     O gate `a_schema_bump_anywhere_must_bump_the_project_schema` fica **vermelho** até baterem.
   - Listas append-only que o Mergiraf funde mas o integrador confere: `mod physics_smoke;`(main.rs),
     `mod physics_bridge;`(render_loop/mod.rs), o campo `AppGfx.physics` + seu destructure, o bloco
     `component_registry` de `init.rs`, os `mod`/prólogo do frame.
   - Nomes de código (únicos, improváveis de colidir): `ph2d::physics::{RigidBody,Collider}`,
     `physics-ecs-c9-hash-*`, `PH2D_PHYSICS_SMOKE`.
4. **Contratos congelados encostados:** **NENHUM**. O contrato de física é novo e não-congelado.
5. **O que só o `ship.sh`/CI pega:** `typos` (pt-BR + comentários) · `machete` (deps novas: `bevy_ecs`+`blake3`
   na ponte, `ph2d-physics-ecs` no shell — todas USADAS) · `deny`/`audit` (sem crate externa nova além de
   `bevy_ecs`, já na árvore) · a **matriz cross-OS do `physics-ecs-c9`** (o verdadeiro gate HR-5 — só roda no
   push; localmente só provei repeatability + os guardas estruturais). O `spike.yml` **não** é validável por
   yamllint local (indisponível) — os blocos são mirror exato dos existentes.
6. **O que smoke-testar (Enio):** `cd Worktrees/line-physics && PH2D_PHYSICS_SMOKE=1 cargo run -p
   ph2d-host-desktop` → a bola cai e assenta. **E confirme que o app normal (sem a env) segue igual** — o
   `physics_bridge::dispatch` roda todo frame, mas é no-op sem entidades de física (query vazia).

**Resumo:** *Linha `physics` (W0+W1) pronta — HEAD `9f5fee05`, 5 commits. Foundational tocado: `ph2d-physics`
(meu módulo, aditivo, c9 intacto) + shell (consumidor). Contratos congelados: nenhum. Colisões a grepar: ADR
0130 · `PROJECT_SCHEMA=16`+tripla-pin (CONTAR se outra linha bumpar). 6 gates mutation-verified; batched gate
verde. Smoke pendente: `PH2D_PHYSICS_SMOKE=1`. Aguardo ordem de integração / W1.5 / W2.*
