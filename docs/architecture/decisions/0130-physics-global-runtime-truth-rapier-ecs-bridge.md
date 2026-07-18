# ADR-0130 — Física global: a simulação É o estado, o rígido primeiro, e a ponte tem UMA porta de escala

- **Status:** Accepted (direção — desenho de abertura W0; implementação faseada W1–W4)
- **Data:** 2026-07-17
- **Decisor(es):** Enio + Claude (`line/physics` W0)
- **Linha:** `line/physics` (Modo L, workstation)
- **Pré-requisitos / herança:** [ph2d-physics M10](../../../crates/ph2d-physics/src/world.rs) (wrapper rapier + gate cross-OS, dormente) ·
  [ADR-0021](0021-simulation-presentation-boundary.md) (`SimWorld` = o estado simulado canônico; a física vive nele, lado simulation da fronteira) ·
  [ADR-0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md) (desacoplar por ECS) ·
  `ph2d-core::FixedStep`/`Playhead` (o relógio) · o precedente Motion (`ticks_owed`, `CheckpointRing`)
- **Emenda:** SKILL §11.5 (escrita antes da Zona de Simulação — este ADR declara a **fronteira tríplice**)
- **Explicitamente NÃO herda:** [ADR-0063](0063-vector-runtime-physics-dormant-fractures.md) (collider-gen vetorial + fratura, amarrada ao vector-runtime que a [ADR-0108](0108-vector-reposition-rive-referenced-native-editor-first.md) aposentou)
- **Tags:** physics · rapier · ecs-bridge · determinism · runtime-truth · bake · foundational

---

## Contexto

O PH2D é uma **Power House Game Engine 2D** com subsistemas peer que têm painel próprio — Painter,
Vector, Audio, Timeline, Motion. **Falta o motor de física global:** o subsistema que faz o mundo
cair, empilhar, colidir e articular ao vivo, com painel de mundo dedicado.

**A parte assustadora já foi paga.** Existe [`ph2d-physics`](../../../crates/ph2d-physics/src/world.rs)
(M10, 320 LOC, `#![forbid(unsafe_code)]`): um wrapper sobre `rapier2d 0.28` com `enhanced-determinism`
ON e um **gate de hash cross-OS na CI** (o bin `ph2d_physics_c9`, rodado na matriz Linux/macOS/Windows
de [`.github/workflows/spike.yml`](../../../.github/workflows/spike.yml)). Determinismo bit-a-bit — o
que mataria física caseira numa matriz de CI com replay-hash — **está resolvido e gateado**. Mas o
crate diz de si mesmo:

> *"M10 ships the wrapper + the cross-OS determinism gate. ECS integration (`PhysicsWorld` ↔ `SimWorld`)
> lands when there is a real scene asking for it."*

**Agora há uma cena pedindo.** Esta linha promove o wrapper de **dormente** a **wired e global**. Ela
**não escreve solver** (rapier já é o solver, e já é determinístico); escreve **integração e autoria**:
a ponte ECS, o relógio único, o painel de mundo, o scrub bit-exato e o bake.

**O risco arquitetural não é a física — é a costura.** Sem cuidado, um segundo motor de dinâmica com
seu próprio estado vira *"dois motores, um estado"*
([[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]). Este ADR posiciona as três coisas
que fazem dinâmica no PH2D e declara quem possui o quê.

---

## Decisão

As 11 decisões que a abertura pediu, cada uma com o *porquê*.

### D1 · runtime-truth + bake opcional (Enio confirmou — não re-litigar)

**A simulação é a verdade viva do mundo — a sim É o estado.** O corpo cai porque o solver o faz cair,
neste tick do `Playhead`. Não é um pré-cálculo que vira dado morto.

Por cima, **bake-to-timeline é opt-in** (D11): o botão "Bake" amostra a pose simulada sobre um range e
escreve keys editáveis nas tracks da entidade.

- **Por que não "só bake"** (a física roda no editor, produz curvas, e o runtime só toca curvas): um
  motor de *engine* exige simulação viva — empilhamento reativo, ragdoll que responde a colisão em
  tempo real, gameplay. Bake-only mata isso.
- **Por que não "só sim"** (nunca vira curva): motion-graphics quer a **curva editável** — pegar o
  quique que a física deu e ajustar 2 keys à mão. Sim-only nega o idioma do After Effects/Rive.
- **O mesmo wrapper determinístico serve os dois** (D7). Escolher runtime-truth **não queima ponte
  nenhuma** — o bake é uma leitura da mesma sim.

### D2 · Um `PhysicsWorld` transiente, dirigido por components; a ponte reconcilia por frame

**Padrão bevy_rapier, ECS-nativo (ADR-0021, ADR-0075):** entidades carregam components
(`RigidBody`/`Collider`), **um system espelha components → `PhysicsWorld`** (spawn/update), **ticka o
mundo** (D5), e **lê os transforms de volta** para o ECS.

**Onde mora o `PhysicsWorld`:** é **estado de sim do editor/shell**, à imagem do pump do Motion
(`MotionCookPump` em `shells/desktop/src/motion_state.rs`, dirigido no `render_loop`), **NÃO** dentro do
`WorldSnapshot` serializado. Consequência decisiva, e é o coração do runtime-truth:

> **O `PhysicsWorld` NÃO é persistido. Os components SÃO.** O mundo vivo é DERIVADO das components a
> cada carga (rebuild), exatamente como o Motion re-cozinha do grafo. As components (`RigidBody`,
> `Collider`) são a **verdade-em-repouso**; o `PhysicsWorld` é a **verdade-em-movimento**; a ponte
> reconcilia. Isso evita **dois donos do mesmo fato** — não guardamos pose simulada e pose autorada em
> dois lugares que podem divergir.

A ponte mantém um mapa `Entity ↔ RigidBodyHandle` (transiente, reconstruído no rebuild — a lição de
`vec_entities::rebuild_map`, que o undo/load precisa refazer senão o sync duplica). O mapa vive junto do
`PhysicsWorld` (shell-owned), não no snapshot.

### D3 · Contrato de components — desenhado para ISOLAMENTO, destinado a CONGELAR

Dois components ECS novos, definidos e **registrados** pela crate-ponte (D2), não por `ph2d-ecs`:

```
RigidBody  { kind: BodyKind, linear_damping: f32, angular_damping: f32,
             gravity_scale: f32, can_sleep: bool, ccd: bool, ... }
Collider   { shape: ColliderShape, density: f32, restitution: f32,
             friction: f32, is_sensor: bool, membership: u32, filter: u32, ... }
enum BodyKind      { Dynamic, Static, Kinematic }          // append-only
enum ColliderShape { Ball{r}, Cuboid{hx,hy}, Capsule{..}, ... }  // append-only
```

- **Defaults byte-neutros:** os defaults reproduzem o comportamento de hoje do wrapper (`density=1.0`,
  `restitution=0.0`, `friction` = default rapier). Um sprite sem `RigidBody` não é tocado pela física —
  a ausência do component é o "off" (a lição [[feedback_a_gap_is_not_silence_two_answers_across_one_pixel]]:
  ausência e presença-neutra têm de coincidir; aqui a ausência = fora da sim, e é intencional).
- **Isolamento (regra B', §1.5.2.1):** os variants dos dois enums são **append-only** (novos no fim →
  índices postcard estáveis, saves antigos legíveis — o padrão `Interp::BezierW`). Todo id/const/variant
  novo pega o **próximo livre** e é anotado no tracker.
- **Registro (a armadilha do snapshot, §3.8):** todo component novo **tem que passar por
  `ComponentRegistry`** senão o `WorldSnapshot` o **descarta em silêncio** (o bug de
  `Locked`/`GroupedChildren`/`VecPathRef`; tripwire `register_ecs_components_populates_registry`,
  `reg.len() == 32`, *"este número existe para doer"*). Seguimos o precedente `ph2d-render`: a
  **crate-ponte possui `register_physics_components(reg)`** e o boot agrega
  (`shells/desktop/src/init.rs`, ao lado de `register_render_components`). Isso mantém a lista central de
  `ph2d-ecs` **intocada** (a contagem-32 não muda) e é append-only — a próxima linha registra a dela sem
  editar o site central. Registro no **MESMO commit** que cria o component.
- **Destinado a congelar:** o freeze é follow-up (como Nodes/Tools/Vector-doc, §6), com um gate
  `architecture_physics_contract_surface` à la os outros. Desenhar para isolamento agora é o que deixa
  esse freeze ser barato.

### D4 · Escala pixel→metro — a porta ÚNICA de conversão na ponte (a decisão do dia 1)

Rapier trabalha bem perto de **unidade-1** (1 unidade ≈ 1 metro); constantes internas (sleep, slop,
margens de contato) são calibradas nessa escala. Nossos sprites são medidos em **pixels** (centenas).
Alimentar o solver com velocidades de centenas de unidades **destuneia**: enrijece joints, estoura o
sleep, degrada a estabilidade de empilhamento. É a mesma classe de bug do `DEPTH_UNIT_PX` do impasto
([[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]) — dois eixos que não são a mesma
unidade, e uma grandeza que cruza a fronteira **crua** fica errada por um fator.

**Decisão:** existe **uma** constante de escala, `PIXELS_PER_METER`, com **uma porta de conversão na
ponte** — todo valor que cruza ECS↔física passa por ela, na entrada (px→m) e na leitura de volta (m→px):

- **Default `PIXELS_PER_METER = 100.0`** (um sprite de 100 px = 1 m; um de 256 px ≈ 2,56 m — dentro da
  faixa 0,1–10 u que o rapier prefere). É um **setting de MUNDO**, editável no painel global (D8).
- **Posição:** `world_px / PPM → metros` na entrada; `metros × PPM → px` na leitura. **Rotação é
  adimensional** (radianos no rapier) — **não** escala por PPM; a ponte a escreve no `Transform` na
  convenção do app (o app usa graus → `rad × 180/π`, multiplicação por constante, determinística).
- **A conversão é só multiplicação/divisão** — zero transcendental no caminho determinístico (D7); a
  disciplina transcendental é trivial de manter aqui, mas o gate a prova mesmo assim.
- **Uma porta, não N:** spawn, update, readback e bake leem a MESMA `to_meters`/`to_pixels`. Duas portas
  para a mesma pergunta divergem ([[feedback_two_doors_to_the_same_question_diverge]]).

Isto entra no **dia 1** (W1), não depois — é o footgun clássico de rapier, e retrofitá-lo depois
reescreve todo fixture.

### D5 · O relógio e o SCRUB — tick no `Playhead`; scrub por **checkpoint ring**

**UM relógio.** A física cozinha no tick do `Playhead` (`ph2d-core`), replicando o Motion ao pé da
letra — o `MotionTransport` **morreu** (W4.T7), e a física **não** ressuscita um segundo transporte.

- **Play = N steps sequenciais.** Copiar `motion_bridge::ticks_owed(last_stepped, target)`:
  `Some(last) if target > last => last+1..=target` (todo tick pra frente, a sim é sequencial — um frame
  lento que deve 2 ticks simula os 2, `rate > 1` idem); caso contrário `target..=target`. `target =
  round(playhead.time() / fixed_dt)`. O `step()` do wrapper **sempre** usa o `dt` interno (HR-5) — a
  ponte controla *quantos* steps, nunca o *tamanho* do step.
- **Scrub pra trás = o problema de engenharia sério.** O estado interno do rapier (contact manifolds,
  islands, sleep, warm-starting) é grande e **essencial para bit-exatidão** — re-simular só de poses+vels
  NÃO reproduz (o warm-start dos manifolds importa). Re-simular do t=0 a cada scrub é **O(t)** (60 s de
  timeline = 3600 steps; a ~1,5 ms/step = 5,4 s por scrub — inaceitável para scrub interativo).

  **Decisão: checkpoint ring**, à imagem do `CheckpointRing` do Motion
  (`ph2d-eval-motion/src/checkpoint.rs`) + `Cook::checkpoint`/`restore`
  (`ph2d-nodegraph/src/cook.rs`), modelo GGPO save/load/advance:
  - **Um `PhysicsCheckpoint`** = o estado cross-frame COMPLETO do `PhysicsWorld` (os campos que o `step()`
    muta: `bodies`/`colliders`/`impulse_joints`/`multibody_joints`/`ccd_solver`/`islands`/`broad_phase`/
    `narrow_phase` + `step_count`). Restaurar+re-simular o resíduo é **bit-exato** porque
    `enhanced-determinism` garante re-sim idêntico de condições idênticas.
  - **`anchor_at_or_before(target) → (tick, checkpoint)`** (newest ≤ target, senão o seed do tick-0),
    `record`, `clear` no `mark_dirty` — a MESMA API do `CheckpointRing`.
  - **Cadência = ESPARSA** (a diferença deliberada do Motion, cujo ring é denso, 1/tick, `RECENT_CAPACITY
    = 300`): o estado de um mundo rígido é maior que os outputs de um grafo de nós, então gravamos a cada
    **K ticks** e re-simulamos ≤ K steps a partir da âncora. K é tunado contra o budget de 20 MB (D10)
    quando o scrub landa — mais checkpoints = scrub mais rápido, mais memória.
  - **⚠️ kill-check de W-scrub (antes do build):** o `PhysicsCheckpoint` precisa **clonar ou serializar**
    esse estado. Os sets do rapier são `Clone`? Senão, ligar a feature `serde-serialize` do rapier
    (serialização **não é** matemática de sim → **não** afeta determinismo, HR-5) e snapshotar via bincode.
    **Confirmar no repo, não na doc** (a abertura afirmou "rapier serializável" — o nosso `Cargo.toml` NÃO
    liga serde hoje). Se nem clone nem serde forem viáveis a custo aceitável, o fallback é **keyframe
    esparso de full-snapshot + re-sim** — mesma arquitetura, mecanismo diferente. A arquitetura (ring +
    restore-nearest + re-sim-resíduo) está fixada; **o mecanismo de captura é decisão de W-scrub**.
- **W1 entrega só play** (forward-only, `ticks_owed`). O scrub-back completo (ring) é uma wave à parte
  (W-scrub, entre W1 e W2 no plano); W1 já deixa o gancho `should_record`/`record` no laço do tick.

### D6 · A fronteira tríplice — rapier-rígido vs Zona-de-nós vs XPBD-soft

A SKILL §11.5 foi escrita **antes** da Zona de Simulação existir. Hoje há **três** coisas que fazem
dinâmica, e este ADR as posiciona explicitamente (é o que Houdini/Unity fazem — DOPs vs POPs; rigidbody
vs particle system):

| Mundo | O que é | Dono | Estado |
|---|---|---|---|
| **Rígido (rapier)** | corpos de cena que caem/empilham/colidem/articulam | **ESTA linha** (painel global + Inspector) | wrapper M10, agora wired |
| **Zona de nós** (`sim.zone`/`sim.step`/`sim.collide`) | dinâmica procedural autorada no grafo (partículas, molas, bounce) | Motion Nodes (landada) | **completa e wired** — NÃO refazer |
| **XPBD soft** (`ph2d-physics-soft`) | deformável/cloth/rope, compute GPU + fallback CPU | linha futura M13+ | stub vazio — **fora daqui** |

- **rapier é corpo de cena** (uma entidade da Hierarchy que cai). **A Zona é grafo procedural** (partículas
  que vivem no cozimento do nó, não são entidades da cena). **XPBD é deformável** (a malha muda de forma).
  Não se sobrepõem: um sprite com `RigidBody` NÃO vira nó, e um sistema de partículas do Motion NÃO vira
  corpo rígido. Coexistem, com fronteira **declarada** — é isto que evita "dois motores, um estado".

### D7 · Determinismo — o hash do mundo-ECS estende o gate c9 na CI cross-OS

`enhanced-determinism` + `dt` fixo + hash blake3 ordenado. **O mundo ligado ao ECS TEM que alimentar o
mesmo tipo de hash e entrar na CI cross-OS.**

- O gate atual (verificado on-disk): `spike.yml` job `determinism` (matriz ubuntu/macos/windows,
  `fail-fast: false`) roda `cargo run --release --bin ph2d_physics_c9 -p ph2d-physics`, parseia
  `grep -E '^physics-c9 hash: ' | awk '{print $3}'`, faz upload do artifact `physics-c9-hash-${os}`; o job
  `determinism-compare` baixa os 3 e exige `sort -u | wc -l == 1`. (⚠️ o path `tests/determinism/
  replay_cross_platform.rs` que a SKILL cita **NÃO existe on-disk** — a fonte de verdade é `spike.yml` + os
  bins `c9.rs`/`c9_replay.rs`. Confie no repo.)
- **W1 adiciona um bin/harness gêmeo** (ex.: `physics-ecs-c9`) que exercita a **ponte + o caminho do tick
  do `Playhead`** (não o wrapper cru): monta uma `SimWorld` com N entidades carregando
  `RigidBody`/`Collider`, roda o sync + o `ticks_owed` por 120 ticks, imprime `physics-ecs-c9 hash: <hex>`.
  Ganha uma etapa de matriz + artifact `physics-ecs-c9-hash-${os}` + comparação em `determinism-compare`.
  Isto prova que **o código NOSSO no caminho determinístico** (a conversão PPM, a ordem de iteração da
  ponte, o readback) não quebra a bit-exatidão cross-OS.
- **Proibições (herdadas do M10, respeitar):** NUNCA ligar `parallel`/`simd-stable`/`simd-nightly` no
  rapier — reordenam somatório float por-plataforma/por-thread e matam o determinismo.
- **Disciplina transcendental** ([[feedback_determinism_sweep_grep_all_transcendentals]]): o código NOSSO
  na ponte/escala/bake é multiply/divide (D4) — grepe todo transcendental e garanta convenção única. **1
  ulp já é bug** ([[feedback_same_math_different_bookkeeping_diverges]]): a ponte e o bake NÃO podem
  computar a "mesma" pose por caminhos diferentes — uma porta, uma conta.

### D8 · O painel global (categoria NOVA) + a seção "Physics Body" no Inspector

Os painéis de hoje são *tool-gated* (painter/vector) ou *selection-docked* (inspector). **Física global é
uma categoria nova: world/scene-settings** — sempre disponível, edita resources da cena, não uma tool.
Dois donos, e misturá-los é o erro:

- **Painel global (mundo) — `ph2d-panel-physics`, docado:** gravidade (vetor), substeps/iterações do
  solver, damping global, sleep thresholds, **matriz de camadas de colisão**, e — crítico — **`PIXELS_PER_METER`
  (D4)**. Registrado nos **5 sites** (precedente canônico `ph2d-panel-vector`): (1) `impl Panel` com
  `ID="physics"`/`NODE_ID=ids::PHYSICS_PANEL`/`populate`/`paint`/`apply_event`; (2) push no
  `ph2d-panel-registry-init` (GERADO por `ph2d-panel-sync` + a const `EXPECTED_TYPED` à mão); (3) feature
  Cargo `panel-physics`; (4) **a lista de fallback de z-order em `hero/paint.rs`** — sem a entrada, o painel
  registrado+visível **nunca é pintado** (a armadilha "never painted"); (5) visibilidade dirigida pela
  ponte (`hero.panel_visibility.insert("physics", ...)` no `render_loop`).
- **Inspector (corpo, por-seleção):** seção **"Physics Body"** no Sprite/Vector Inspector — tipo
  (dynamic/static/kinematic), massa/densidade, restituição, atrito, colisor. **NÃO** no painel global.
- **Widget pronto = um teste CLICA nele** ([[feedback_widget_is_done_when_a_test_clicks_it]]): sem
  `WidgetStore` populado não há Click; **pintado ≠ populado** ([[feedback_painted_is_not_populated_paint_gate]]).
  Botão dimmed ainda despacha — a recusa mora no `event.rs`, não no laço de pintura
  ([[feedback_disabled_button_still_dispatches]]). UI canônica: zero hex, zero `f32` literal, zero string
  hardcoded — tokens/i18n, em inglês (G).

### D9 · Escopo de abertura = RÍGIDO apenas

XPBD soft (`ph2d-physics-soft`, M13+) e fluidos FLIP/PIC (`ph2d-fluids`, M13+) são **linhas próprias,
fora daqui** (SKILL §11.5). E **explicitamente fora:** a bagagem da
[ADR-0063](0063-vector-runtime-physics-dormant-fractures.md) — collider-gen a partir de forma vetorial +
fratura dinâmica (Dormant Fracture Edges, 3-tier boolean-cut). Ela foi construída **sobre o
`ph2d-vector-runtime`** que a [ADR-0108](0108-vector-reposition-rive-referenced-native-editor-first.md)
**aposentou**. Um motor de física **app-level** NÃO reabre a 0108, e **não herda os mecanismos da 0063**
(nem os gates de momentum que ela previu e que nunca foram construídos). É Cerca de Chesterton ao contrário:
a dormência da 0063 é uma decisão documentada de *não* fazer aquilo agora
([[feedback_documented_decision_chesterton_fence]]).

### D10 · Budgets (herdados, não-negociáveis)

- **HR-4 (frame):** física rígida = **1,5 ms/frame**, fixed step 60 Hz. Hot path (`physics_step` da ponte)
  = **zero alocação** (HR-3) — pools pré-alocados, `SmallVec`, sem `Vec::push` que realoque. Gate zero-alloc
  por **capacidade**, não contador global ([[feedback_zero_alloc_gate_capacity_not_global_counter]]).
- **HR-13 (memória):** "Physics state" = **20 / 20 / 80 / 10 MB** (iPad / Android / Desktop / Web,
  confirmado na tabela HR-13). E **quem declara budget MEDE** (dhat, `tests/measure_*.rs`) — HR-13 soma
  declarações e nunca observa um byte sozinho ([[feedback_a_rule_that_never_observes_cannot_fire]]). O
  checkpoint ring (D5) é o principal consumidor variável; a cadência K é tunada contra este teto e a medição
  é o gate.

### D11 · Bake-to-timeline — a pose simulada vira keys editáveis

O ponto de acoplamento (é W4, mas a ADR desenha a costura): o botão "Bake" **amostra a pose simulada
por frame** sobre um range → **`fit_fcurve`/Schneider do `ph2d-anim`** (least-squares cubic + Newton
reparam, colunas alinhadas, pré-filtro passa-baixa se preciso — a MESMA máquina que o record da timeline já
usa), escrevendo keys nas tracks da entidade em **1 passo de undo**, **determinístico**.

- **Reusa o `ph2d-anim`, não reinventa** — a curva assada é a mesma classe da curva do record (W5 da
  timeline). Colunas alinhadas por entidade.
- **Oráculo de APARÊNCIA, não de regra** ([[reference_topic_oracle_discipline]]): o gate mede que a curva
  assada **reproduz a trajetória simulada** dentro da tolerância (posição no tempo certo), não que ela
  bateu uma fórmula.
- **1 passo de undo** (não 1 por frame), e o bake é **determinístico** (mesma sim → mesma curva; D7).
- O acoplamento é com a timeline/anim (outra linha). W4 declara a superfície exata; a costura é: sim →
  amostra por tick → `fit_fcurve_at` → `Track::simplify_range` → 1 undo step.

---

## Consequências

- **Persistência:** as components de física entram no save/undo via `WorldSnapshot` **assim que
  registradas** (D3); o `PhysicsWorld` vivo **não** é serializado (é rebuild das components — D2). Bump de
  `PROJECT_SCHEMA` (13 → 14 hoje) quando W1 persistir os components. Save format versionado e migrável
  (HR-14). **Snapshot = ponto fixo** ([[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]]): a
  captura tem que ser tirada **depois** de a ponte convergir no frame, senão o diff registra passo espúrio
  (a lição exata do z-order do vetor).
- **O relógio é compartilhado** com Motion e Timeline (o mesmo `Playhead`). Play, scrub e bake da física
  usam o mesmo relógio fixo de 60 Hz. Nenhum estado de física fora do tick do `Playhead`.
- **O wrapper M10 fica intocado** na sua superfície pública (a ponte é uma crate NOVA que o consome). O
  gate c9 existente continua verde; o novo gate `physics-ecs-c9` é aditivo.
- **Uma crate-ponte nova** (`ph2d-physics-ecs` — sugestão) mantém a física fora do shell e do `ph2d-ecs`
  central, isolada para a próxima linha estender (soft/fluidos) sem colidir.
- **Um contrato novo, ainda NÃO congelado** — re-congelar (gate `architecture_physics_contract_surface`) é
  follow-up, mas o desenho append-only de D3 já o prepara.

## Gates (a intenção; o plano de waves os detalha red-first + mutation-tested)

Cada gate nasce **VERMELHO** sobre o bug real, com os números do PRODUTO, e morre por uma razão nomeável
(a DIRETIVA §3–§5; verde-de-compilação vale ZERO no audit). Por wave, resumido:

- **W1:** e2e no app REAL (sprite cai e assenta no chão via ECS — não unit do wrapper,
  [[feedback_tool_unit_green_integration_dead]]) · hash cross-OS estável do mundo ECS-bridged (mutar a ordem
  de iteração da ponte sangra) · zero-alloc no `physics_step` (dhat por capacidade) · tick único (play anda
  N, scrub anda 1 — gate de emenda com **advance fracionário**, taxa 1:1 nunca lê o 2º frame,
  [[feedback_seam_gates_need_fractional_advance]]) · snapshot é ponto fixo (nenhum passo de undo espúrio).
- **W2:** painel **pintado E populado E clicado** (WidgetStore) · toda row de setting muda o mundo (seam que
  CLICA) · sem string hardcoded (i18n) · botão dimmed recusa no `event.rs`.
- **W3 (joints):** pêndulo de 2 corpos determinístico (hash estável) · joint sobrevive save/load (schema
  bump) · mutação de um parâmetro de joint sangra o gate de repro.
- **W4 (bake):** curva assada reproduz a sim dentro da tolerância (oráculo de aparência) · bake
  determinístico · 1 undo step (não 1 por frame).

---

## Alternativas rejeitadas

1. **Escrever solver próprio.** Rejeitada: o wrapper M10 já é determinístico cross-OS e gateado — reescrever
   queima meses e reabre a matriz de CI. Esta linha é integração e autoria, não solver (D-preamble).
2. **Física como plugin de runtime / WASM.** Rejeitada por [ADR-0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)
   (sem ABI estável; nem resolve o coupling de schema). Desacoplar por ECS (D2).
3. **Segundo transporte para a física** (à la o `MotionTransport` que morreu). Rejeitada: um relógio (D5) —
   scrub e bake compartilham o `Playhead`. Dois relógios divergem ([[feedback_one_ruler_measures_one_clock]]).
4. **Persistir o `PhysicsWorld` vivo** (pose simulada no save). Rejeitada: dois donos do mesmo fato (pose
   autorada nas components + pose simulada no snapshot) divergem. O mundo é DERIVADO das components (D2) —
   runtime-truth (D1).
5. **Re-sim from t=0 para scrub-back.** Rejeitada: O(t), 5,4 s por scrub num timeline de 60 s (D5). Checkpoint
   ring esparso (modelo Cook/GGPO) dá scrub bit-exato com custo bounded.
6. **Conversão pixel→metro espalhada** (cada call-site converte). Rejeitada: N portas divergem — uma porta,
   uma conta (D4, [[feedback_two_doors_to_the_same_question_diverge]]).
7. **Herdar a 0063** (collider-gen vetorial + fratura). Rejeitada: amarrada ao vector-runtime que a 0108
   aposentou; motor app-level não reabre a 0108 (D9).
