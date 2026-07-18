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
| **W0 — Arquitetura** | ✅ **FECHADO** (2026-07-17) | *(este commit)* | ADR-0130 + plano de waves + tracker + visão. **Zero código.** |
| **W1 — Ponte ECS + tick + hash** | ⏳ pendente (aguarda ordem do Enio) | — | o alicerce |
| **W1.5 — Scrub (checkpoint ring)** | ⏳ pendente | — | kill-check de serialização ANTES do build |
| **W2 — Painel + Inspector body** | ⏳ pendente | — | categoria de painel NOVA (mundo) |
| **W3 — Joints** | ⏳ pendente | — | pêndulo/corrente/ragdoll |
| **W4 — Bake-to-timeline** | ⏳ pendente | — | acopla `ph2d-anim` (outra linha) |

**W0 entregou:** [ADR-0130](../architecture/decisions/0130-physics-global-runtime-truth-rapier-ecs-bridge.md) ·
[`00_plano_waves.md`](00_plano_waves.md) · [`01_visao.md`](01_visao.md) · este tracker. Nenhuma linha de
código, nenhum contrato tocado, nenhum foundational tocado.

---

## Decisões (ADR-0130, condensadas — o *porquê* está lá)

- **D1** runtime-truth + bake opcional (Enio). **D2** `PhysicsWorld` transiente shell-side (precedente
  `MotionCookPump`), dirigido por components; NÃO persistido (é rebuild). **D3** contrato
  `RigidBody`/`Collider` append-only, registrado pela crate-ponte, destinado a congelar. **D4** escala
  pixel→metro numa PORTA ÚNICA (`PIXELS_PER_METER=100.0`). **D5** relógio no `Playhead`
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

**Alocados no W0 (nomes de design; reservados):**
- Crate-ponte: **`ph2d-physics-ecs`** (nome reservado; não criada ainda).
- Painel: crate **`ph2d-panel-physics`**, `Panel::ID = "physics"`, feature Cargo **`panel-physics`**.
- Const de escala: **`PIXELS_PER_METER = 100.0`** (na porta da ponte).
- Env de smoke: **`PH2D_PHYSICS_SMOKE`** (1=drop, 2=painel, 3=joint, 4=bake).
- Gate/artifact CI: **`physics-ecs-c9`** / `physics-ecs-c9-hash-${os}`.
- ADR: **0130** (confirmado próximo livre; maior on-disk = 0129).

**A alocar na wave que os cria (próximo LIVRE, anotar aqui quando criados):**
- `ids::PHYSICS_PANEL` (IconId/panel-node) — **W2**.
- `EXPECTED_TYPED` bump em `ph2d-panel-registry-init` — **W2**.
- `PROJECT_SCHEMA` 13 → 14 (components de física) — **W1**; 14 → 15 (joints) — **W3**.
- Variants de `BodyKind`/`ColliderShape`/joint-kind — nas waves W1/W3, append-only.

---

## Handoff de INTEGRAÇÃO — sessão W0 (§1.5.9)

> Preencher HEAD após o commit. Reportar ao Enio e **PARAR** (regra E/H). NÃO integrar, NÃO pushar.

1. **Identidade:** branch `line/physics`; base (merge-base com main) = `cdc3acc1`; HEAD = *(o commit
   destes docs)*; **1 commit**, docs-only.
2. **Foundational/compartilhado tocado:** **NENHUM.** Arquivos: só `docs/Physics/*` (novos) +
   `docs/architecture/decisions/0130-*.md` (novo). Zero `.rs`, zero `Cargo.*`, zero código de app.
3. **Símbolos que podem COLIDIR:** **o número de ADR `0130`** — 4 linhas ativas (FLIP/Painter/Vector/anim)
   podem reclamar 0130 em paralelo; o gate `architecture_adr_numbers_are_unique` pega na integração. Se o
   integrador renumerar, o renomeio é escopado **só aos arquivos que a linha mudou** (`git diff
   --name-only`), NUNCA `git grep` de árvore inteira ([[feedback_a_token_rewrite_scopes_to_changed_files_not_the_whole_tree]]).
   Nomes de design reservados (não são símbolos de código ainda): `ph2d-physics-ecs`, `ph2d-panel-physics`,
   `PIXELS_PER_METER`, `PH2D_PHYSICS_SMOKE`, `physics-ecs-c9`.
4. **Contratos congelados encostados:** **NENHUM** (docs-only). O contrato de física é novo e não-congelado
   (freeze é follow-up).
5. **O que só o `ship.sh` pega:** gate `typos` (docs em pt-BR — checar allowlist se algum termo novo
   dispara); markdownlint/link-check se existir. Nenhuma dep nova, nenhum clippy/RUSTSEC (zero código).
6. **Ordem/dependências:** 1 commit, sem ordem. **O que smoke-testar:** **NADA** — sessão de arquitetura,
   sem superfície de runtime. O smoke real começa em W1 (`PH2D_PHYSICS_SMOKE=1`).

**Resumo:** *Linha `physics` (W0) pronta — HEAD `<sha>`, 1 commit docs-only. Foundational tocado: nenhum.
Contratos congelados: nenhum. Colisão possível: só o nº de ADR 0130. Smoke: nada (arquitetura). Aguardo
ordem para W1.*
