# HANDOFF — Abertura de linha: `line/physics`
## O motor global de física da engine

> **Data:** 2026-07-17 · **Autor:** Coordenador (sessão de planejamento com o Enio)
> **Modelo:** [`MODELO_ABERTURA_LINHA.md`](../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md)
> **Modo:** L (workstation, DIRETRIZ §1.5) · **Branch:** `line/physics` · **Worktree:** `Worktrees/line-physics/`
> **Pasta de docs do módulo:** [`docs/Physics/`](.) — TODO doc deste módulo nasce aqui.
>
> **Direção CONFIRMADA pelo Enio (não re-litigar):** **runtime-truth + bake opcional.**
> A simulação é a verdade viva do mundo; assar em keys de timeline é um recurso opt-in por cima.
> **Stack CONFIRMADA (não escrever do zero):** rigidos = **`rapier2d 0.28`** (`enhanced-determinism`).
> O solver já existe e já é determinístico cross-OS. Esta linha escreve **integração e autoria**, não solver.

---

## 0 · O que é esta linha (o norte — leia antes de tudo)

O PH2D é uma **Power House Game Engine 2D**. Ela já tem subsistemas peer com painel próprio:
Painter, Vector, Audio, Timeline, Motion. **Falta o motor de física global** — o subsistema que faz
o mundo cair, empilhar, colidir e articular, ao vivo, com seu **painel de mundo dedicado**.

A parte assustadora **já foi paga**. Existe [`ph2d-physics`](../../crates/ph2d-physics/src/world.rs)
(M10): um wrapper sobre `rapier2d` com `enhanced-determinism` ON e um **gate de hash cross-OS na CI**
(o bin `ph2d_physics_c9`). Determinismo bit-a-bit em Linux/Mac/Windows — o que mataria física caseira
numa matriz de CI com replay-hash — está resolvido e gateado. Mas o crate diz de si mesmo:

> *"M10 ships the wrapper + the cross-OS determinism gate. ECS integration (`PhysicsWorld` ↔ `SimWorld`)
> lands when there is a real scene asking for it."*

**Agora há uma cena pedindo.** Esta linha promove o wrapper de **dormente** a **wired e global**.

**Framing (Enio):** *runtime-truth* — o mundo simula ao vivo, a sim É o estado — com **bake-to-timeline
opcional** por cima (o Newton do After Effects / o physics do Rive, mas o motor é de engine). O mesmo
wrapper determinístico serve os dois usos; escolher runtime-truth **não queima ponte nenhuma**.

**Escopo de ABERTURA = só o mundo RÍGIDO (rapier).** É a metade de-riscada. Soft-body (XPBD próprio,
`ph2d-physics-soft`, Müller 2020) e fluidos (FLIP/PIC) são **M13+, linhas próprias, fora daqui** (SKILL §11.5).

---

## 1 · FASE 1 — SETUP (execute já, sem pedir confirmação; reporte cada ✗)

> Bloco derivado do [`MODELO_ABERTURA_LINHA.md`](../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md).
> **O módulo é `physics`.** Nos comandos, `$MODULO` = `physics` (env não persiste entre shells — escreva literal).

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
Você é um agente-de-linha. Sua linha: line/physics
Sua branch:    line/physics
Sua worktree:  Worktrees/line-physics/   (você vai criá-la agora)

FASE 1 — SETUP:
1. bash scripts/hw-profile.sh
      → tem que dizer `workstation`. Disse `constrained`? PARE (Modo C proíbe linhas aqui).
2. git status -sb
      → você está na RAIZ do repo primário, branch main. M/?? alheios podem existir
        (outros agentes de linha ativos: FLIP, Painter, Vector, anim). NÃO toque neles.
3. git pull --ff-only origin main
      → falhou (rede/divergência)? Siga com o main local e reporte.
4. mkdir -p Worktrees
   git worktree add -b line/physics Worktrees/line-physics main
      → branch já existe (linha reaberta)? git worktree add Worktrees/line-physics line/physics
        e DENTRO dela: git rebase main
5. cd Worktrees/line-physics
   git branch --show-current        # DEVE imprimir line/physics
6. cargo check -p ph2d-physics
      → warm-up do target/ próprio desta worktree; 1º build é frio (minutos). NÃO investigue a demora.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107); idempotente
      → "mergiraf not found" NÃO bloqueia (git faz fallback). Reporte a linha do ✗ e siga.
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/DIRETRIZ.md               → §0, §1.5, §2, §6
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md → TUDO (e releia a cada passo)
9. NÃO PARE no "aguardo a tarefa". Esta abertura JÁ traz a tarefa: siga direto para a
   FASE 2 deste handoff (§2 abaixo). Reporte "Linha line/physics pronta — começando a FASE 2".

REGRAS PERMANENTES (valem até o fim):
A. TODO read/edit/git/cargo DENTRO de Worktrees/line-physics/. `pwd` antes de editar — a raiz
   compartilha o MESMO path relativo; editar crates/... na raiz é editar a árvore ERRADA.
B. Edite a pasta do módulo (crates/ph2d-physics*, docs/Physics/, e o novo crate-ponte) à vontade.
   Foundational (ph2d-core/editor-core/ecs/tokens/host): PODE e DEVE tocar com cuidado sob o
   protocolo testado (ADR-0107). PARE e reporte ao Enio SÓ se: (a) contrato congelado (§4/§7),
   ou (b) rebase conflitar em código FORA dos seus arquivos (mesmo-símbolo). Nunca negocie com outra linha.
B'. Arquivo foundational novo = projete para ISOLAMENTO: módulo/arquivo IRMÃO novo, ponto de
   extensão append-only (lista ordenada, marcador de codegen, mod por responsabilidade). Todo
   id/const/variant novo → pegue o próximo livre e ANOTE no handoff de integração (regra H).
C. Commits locais frequentes: git commit --no-verify. NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar. Conflito em Cargo.lock ou
   arquivo GERADO (registry-init): regenere, nunca resolva na mão (DIRETRIZ §1.5.5).
E. Fechamento = gate batched (nextest-impacted + clippy --all-targets + audit ≥2 lentes + DIRETIVA §3-§5).
   Então PARE. NÃO integre nem faça ship. Integração é de um AGENTE INTEGRADOR DEDICADO, só por
   ORDEM EXPLÍCITA do Enio. Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria. Ordem explícita do Enio, feita pelo integrador.
G. UI canônica: zero hex, zero f32 literal de UI, zero string hardcoded — tudo por tokens/i18n. UI em inglês.
H. HANDOFF DE INTEGRAÇÃO ao fechar (DIRETRIZ §1.5.9): branch/HEAD/base; foundational tocado + por quê;
   ids/consts/variants novos com valores; contratos congelados encostados (deve ser nenhum);
   o que só o ship.sh pega; o que smoke-testar. Reporte "linha pronta + handoff" e ESPERE.
═══════════════════════════════════════════════════════════════════
```

---

## 2 · FASE 2 — A TAREFA DESTA SESSÃO (o entregável)

**Esta é uma sessão de ARQUITETURA, não de implementação.** Você **não** constrói W1 agora. Você
produz o alicerce em que as waves vão assentar, e o faz com o mesmo rigor de um PR de código.

Três entregáveis, todos em `docs/Physics/` (ou `docs/architecture/decisions/` para o ADR):

### 2.1 · A ADR de abertura — `ADR-0131`
Arquivo: `docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md`
(ajuste o slug se preferir, mas **mantenha o número 0130** — é o próximo livre hoje).

Siga o **formato da casa** — abra qualquer ADR recente para o molde exato (ex.
[`0124`](../architecture/decisions/0124-audio-a-range-edit-must-be-told-its-range.md),
[`0118`](../architecture/decisions/0118-audio-streaming-voices-residency.md)): cabeçalho
(**Status: Accepted** · **Decisor(es): Enio + Claude (line/physics W0)** · pré-requisitos · tags),
Contexto, Decisão, Consequências, Alternativas rejeitadas. A ADR-0131 deve **decidir** os 11 pontos
do §4 deste handoff — cada um com o *porquê*, não só o *quê*.

> ⚠️ **Colisão de número de ADR:** há 4 linhas ativas (FLIP/Painter/Vector/anim) que podem reclamar
> 0130 em paralelo. O gate `architecture_adr_numbers_are_unique` pega isso na integração; se o
> integrador precisar renumerar, o renomeio é escopado **só aos arquivos que a linha mudou**
> (`git diff --name-only`), **nunca** um `git grep` de árvore inteira — um token de 4 dígitos não é
> único ao seu ADR (fontes, UUIDs, hashes carregam "0130"). Anote o número no handoff de integração.

### 2.2 · O plano de waves — `docs/Physics/00_plano_waves.md`
Escreva e **salve** o plano detalhado das 4 waves (conteúdo normativo no §5 deste handoff). Cada wave
com: objetivo · entregáveis (crates/arquivos) · **gates red-first, mutation-tested** · cena de smoke
(`PH2D_PHYSICS_SMOKE=…`) · o que fica FORA. O plano é vivo — waves seguintes o refinam.

### 2.3 · O tracker + a estrutura de docs do módulo
- `docs/Physics/HANDOFF_line_physics.md` — o tracker vivo (estado por-wave, decisões, gotchas,
  ids/consts alocados). É o `docs/HANDOFF_*` do módulo; toda jornada futura atualiza este arquivo.
- `docs/Physics/01_visao.md` (opcional mas recomendado) — a visão de 1 página: o motor global,
  runtime-truth+bake, a fronteira tríplice, o que o diferencia (Rive/Cavalry/AE/Godot 2D).

**Ao terminar os três, PARE e reporte** ("ADR-0131 + plano de waves + tracker prontos; aguardo ordem
para W1"). **Não comece W1** sem a próxima instrução do Enio. Feche → handoff → PARE (regra E).

---

## 3 · O terreno (contexto que você NÃO deve re-derivar)

### 3.1 · Os três mundos de física, e a fronteira tríplice — a decisão nº 1 da ADR
A SKILL §11.5 foi escrita **antes** da Zona de Simulação existir. Hoje há **três** coisas que fazem
dinâmica, e a ADR tem que posicionar as três explicitamente — senão vira **"dois motores, um estado"**
([[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]):

| Mundo | O que é | Dono | Estado |
|---|---|---|---|
| **Rígido (rapier)** | corpos de cena que caem/empilham/colidem/articulam | **ESTA linha** (painel global + Inspector) | wrapper pronto (M10), dormente |
| **Zona de nós** (`sim.zone`/`sim.step`/`sim.collide`) | dinâmica procedural autorada no grafo (partículas, molas, bounce) | Motion Nodes (já landada) | **completa e wired** — NÃO refazer |
| **XPBD soft** (`ph2d-physics-soft`) | deformável/cloth/rope, compute GPU + fallback CPU | linha futura M13+ | stub vazio — **fora daqui** |

Divisão limpa (é o que Houdini/Unity fazem — DOPs vs POPs; rigidbody vs particle system): rapier é
**corpo de cena**; a Zona é **grafo procedural**; XPBD é **deformável**. Coexistem, com fronteira declarada.

### 3.2 · O que você HERDA pronto — [`ph2d-physics`](../../crates/ph2d-physics/src/world.rs) (M10)
Não reescreva nada disto. A API do wrapper (399 LOC, `#![forbid(unsafe_code)]`):
- `PhysicsWorld::new()` · `set_gravity(x,y)` · `set_dt(dt)` · `dt()` · `step_count()`
- `add_dynamic_circle(x,y,r,density)` · `add_static_cuboid(x,y,hx,hy)` · `insert_body(RigidBody)`
- `bodies()/bodies_mut()` · `colliders()/colliders_mut()` (acesso rapier cru p/ formas/joints)
- `step()` — **sempre usa `PhysicsWorld::dt` interno; NUNCA aceita dt externo** (HR-5)
- `body_pose(handle)` · `body_snapshots()` (ordenado por `handle_index` — ordem estável cross-OS)
- `deterministic_hash() -> [u8;32]` — **blake3 sobre snapshots ordenados**; é o hash do gate cross-OS
- Constantes: `DEFAULT_DT = 1/60` · `DEFAULT_GRAVITY_Y = -9.81` · **mundo Y-up** (baixo = −y, SKILL §11.1)
- **Gate atual:** o bin [`ph2d_physics_c9`](../../crates/ph2d-physics/src/bin/c9.rs) roda fixture de
  50 corpos + chão, 120 steps, imprime `physics-c9 hash: <hex64>`; a CI coleta os 3 hashes
  (Linux/Mac/Windows) e exige byte-identidade. **W1 estende ESTE gate para o mundo ligado ao ECS.**

Deps pinadas ([`Cargo.toml`](../../crates/ph2d-physics/Cargo.toml)): `rapier2d 0.28`
(`default-features=false`, features `dim2`/`f32`/`enhanced-determinism`) + `blake3`.
⚠️ **NUNCA ligue `parallel`/`simd-stable`/`simd-nightly`** — os três quebram determinismo cross-OS
(reordenam somatório float por-plataforma/por-thread). Está documentado no Cargo.toml; respeite.

### 3.3 · O relógio — um só, no Playhead (precedente Motion, replicá-lo)
- [`ph2d-core::time::FixedStep`](../../crates/ph2d-core/src/time.rs): `advance(wall_dt) -> FixedStepReport`,
  `tick_count()`, `DEFAULT_HZ = 60`, cap de substeps (anti "spiral of death"). `Playhead::advance` é
  chamado **uma vez por tick fixo**.
- **Precedente a copiar:** o Motion cozinha no tick do Playhead — `motion_bridge::ticks_owed` (play =
  **todo** tick pra frente, a sim é sequencial; scrub/jump = **uma** chamada, sem replay). O
  `MotionTransport` **morreu** (W4.T7) — **um relógio**. A física faz igual: play = N steps sequenciais;
  scrub = decisão do §4.5.
- **Scrub pra trás = o problema de engenharia sério.** O estado interno do rapier (contact manifolds,
  islands, sleep) é grande; re-simular do t=0 a cada scrub é O(t). O determinismo *permite* re-sim
  bit-exato, mas você vai querer um **checkpoint ring** como o `Cook::checkpoint`/`restore` +
  `CheckpointRing` dos Motion Nodes (save/load/advance, GGPO-style, scrub bit-exato). A ADR **escolhe**
  re-sim vs ring e precifica cada checkpoint (rapier tem `PhysicsWorld` serializável).

### 3.4 · O painel — categoria NOVA (mundo), + Inspector (corpo)
- **Painel docado, NUNCA FloatingPanel.** `Tool::build_panel()` (FloatingPanel) **não é pintado** — UI
  de subsistema é `ph2d-panel-*` + `panel_visibility` (igual painter/vector/audio). Registro em 5 sites
  (ver [[reference_topic_panel_registration]]).
- **O painel de física global é uma categoria nova neste app:** os painéis de hoje são *tool-gated* ou
  *selection-docked*. Física global é **world/scene-settings** — sempre disponível, edita resources da
  cena. Dois donos, e misturá-los é o erro:
  - **Painel global (mundo):** gravidade, substeps/iterações do solver, damping, sleep thresholds,
    **matriz de camadas de colisão**, e — crítico — **escala do mundo (pixel→metro, §3.5)**.
  - **Inspector (corpo, por-seleção):** tipo (dynamic/static/kinematic), massa, restituição, atrito,
    colisor. Seção **"Physics Body"** no Sprite/Vector Inspector — NÃO no painel global.
- **Widget pronto = um teste CLICA nele** ([[feedback_widget_is_done_when_a_test_clicks_it]]): sem
  `WidgetStore` populado não há Click; "pintado ≠ populado". Botão dimmed ainda despacha — a recusa
  mora no `event.rs`, não no laço de pintura.

### 3.5 · A armadilha pixel→metro — é a TUA própria lição
Rapier trabalha bem perto de **unidade-1** (1 unidade ≈ 1 metro). Teus sprites são medidos em
**pixels** (centenas). Alimentar o solver com velocidades de centenas de unidades tuneia, enrijece
joints, estoura o sleep. **Tem que existir uma escala pixel→metro convertida num único ponto na
ponte** — exatamente [[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]] (o
`DEPTH_UNIT_PX` do impasto, outro eixo): toda grandeza que cruza a fronteira ECS↔física passa pela
conversão do consumidor, uma porta só. Isto entra no **dia 1** da ADR (decisão §4.4), não depois.

### 3.6 · Determinismo (HR-5) — a espinha, e o gate que a prova
- `enhanced-determinism` + `dt` fixo + hash blake3 ordenado. O mundo ligado ao ECS **tem que alimentar
  o mesmo hash** e entrar na CI cross-OS. Verifique o wiring REAL do gate (o bin c9 + o shell da CI que
  compara os 3 hashes) — a SKILL cita um caminho `tests/determinism/replay_cross_platform.rs` que **pode
  não existir ainda** on-disk; confie no que o repo mostra, não no que a doc promete.
- **Disciplina transcendental** ([[feedback_determinism_sweep_grep_all_transcendentals]]): qualquer
  código NOSSO no caminho determinístico (a ponte, a escala, o bake) grepa todos os transcendentais e
  garante convenção única. `round`/`to_bits` divergem CPU↔GPU e por-ramo ([[feedback_cpu_gpu_rounding_conventions_diverge]]).
- **1 ulp já é bug** ([[feedback_same_math_different_bookkeeping_diverges]]): se a ponte e o bake
  computam a "mesma" pose por caminhos diferentes, eles divergem. Uma porta, uma conta.

### 3.7 · Budgets herdados (não-negociáveis)
- **HR-4 (frame):** física rígida = **1.5 ms/frame**, fixed step 60 Hz. Hot path (`physics_step`) =
  **zero alocação** (HR-3) — pools pré-alocados, `SmallVec`, sem `Vec::push` que realoque. Gate zero-alloc
  por **capacidade**, não contador global ([[feedback_zero_alloc_gate_capacity_not_global_counter]]).
- **HR-13 (memória):** "Physics state" = 20/20/80/10 MB (tiers). E **quem declara budget MEDE** (dhat,
  `tests/measure_*.rs`) — HR-13 não observa bytes sozinho ([[feedback_a_rule_that_never_observes_cannot_fire]]).

### 3.8 · Registro de components, persistência, undo
- Todo component novo (`RigidBody`, `Collider`, joints) **tem que ser registrado no `ComponentRegistry`**
  — senão o `WorldSnapshot` os **descarta em silêncio** (o undo global e o save viajam nesse snapshot;
  foi o bug de `Locked`/`GroupedChildren`/`VecPathRef`). Registre no MESMO commit que cria o component.
- **Snapshot = PONTO FIXO dos sistemas** ([[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]]):
  a captura tem que ser tirada DEPOIS de a ponte convergir no frame, senão o diff registra passo espúrio.
- **Persistência:** o `ProjectState` (save + undo) é `{WorldSnapshot + VecScene}`. Física entra via
  `WorldSnapshot` se os components estiverem registrados; bump de `PROJECT_SCHEMA` quando W1 persistir.
  Save format é versionado e migrável (HR-14).

---

## 4 · O que a ADR-0131 tem que DECIDIR (checklist normativo)

Cada item é uma decisão com *porquê*. A ADR não está completa sem os 11.

1. **Runtime-truth + bake opcional** (Enio confirmou) — declare o modelo: a sim é a verdade viva;
   bake-to-timeline é opt-in. Por que não "só bake" (o motor de engine exige sim viva) nem "só sim"
   (motion-graphics quer a curva editável).
2. **Um `PhysicsWorld` global como resource ECS** + a ponte `PhysicsWorld ↔ SimWorld` (ADR-0021):
   entidades carregam components, um system espelha components→mundo e lê transforms de volta. Padrão
   bevy_rapier, ECS-nativo. Onde mora o resource, quem o ticka.
3. **Contrato de components** `RigidBody`/`Collider` (+ shape enum, body type, mass/restitution/friction):
   campos, defaults byte-neutros, **projetado para ISOLAMENTO** (append-only, próximo variant/id livre
   anotado) e **destinado a congelar** (freeze é follow-up, como Nodes/Tools/Vector-doc — §7).
4. **Escala pixel→metro** — a porta única de conversão na ponte (§3.5). Onde vive, qual o default,
   como o painel a expõe. Esta é a decisão que evita o footgun clássico.
5. **O relógio e o SCRUB** — tick no Playhead (play=N steps sequenciais; scrub=?). **Escolha
   explicitamente** checkpoint ring (à la `Cook`) vs re-sim from t=0, com o custo de cada. Cite o
   precedente Motion (`ticks_owed`, MotionTransport morto).
6. **A fronteira tríplice** (§3.1) — rapier-rígido vs Zona-de-nós vs XPBD-soft, declarada. O que cada
   um possui, e por que não se sobrepõem.
7. **Determinismo** — `enhanced-determinism`; o hash do mundo ligado ao ECS estende o gate c9 e entra
   na CI cross-OS; proibição de `parallel`/`simd`; disciplina transcendental no código NOSSO (HR-5).
8. **O painel global (categoria nova)** + a seção "Physics Body" do Inspector (§3.4) — a divisão
   mundo/corpo, e por que o painel de física global não é tool-gated.
9. **Escopo de abertura = rígido apenas.** XPBD soft (M13+) e fluidos FLIP/PIC fora. E **explicitamente
   fora:** a bagagem da [ADR-0063](../architecture/decisions/0063-vector-runtime-physics-dormant-fractures.md)
   (collider-gen a partir de forma vetorial + fratura dinâmica) — estava amarrada ao vector-runtime que a
   [ADR-0108](../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)
   aposentou. Um motor de física **app-level** NÃO reabre a 0108; mas não herde os mecanismos da 0063.
10. **Budgets** — 1.5 ms/frame (HR-4), 20 MB estado medido (HR-13), zero-alloc no step (HR-3).
11. **Bake-to-timeline** — como a pose simulada vira keys editáveis: amostra sobre um range → curva
    (reusar o `fit_fcurve`/Schneider do `ph2d-anim`, colunas alinhadas), 1 passo de undo, determinístico.
    Depende da timeline/anim — declare o ponto de acoplamento (é W4, mas a ADR desenha a costura).

---

## 5 · O plano de waves (o que escrever em `00_plano_waves.md`)

Cada wave: objetivo · entregáveis · **gates red-first + mutation-tested** · smoke · fora-de-escopo.
As waves são sequenciais; cada uma fecha com o gate batched e um handoff de tracker.

### W1 — Ponte ECS + tick no Playhead + hash no replay gate  *(o alicerce)*
**Objetivo:** um sprite com `RigidBody{dynamic}` cai e assenta sobre um `Collider{static}` no ECS REAL,
ao dar play — e o mundo é determinístico cross-OS.
**Entregáveis:**
- Crate-ponte novo (sugestão: `ph2d-physics-ecs`, ou módulo no editor-core) — components `RigidBody`/
  `Collider`, registrados no `ComponentRegistry`; a escala pixel→metro (porta única).
- System de sync: components → `PhysicsWorld` (spawn/update) → `step()` no tick do Playhead
  (`ticks_owed`: play=sequencial, scrub=1 chamada) → readback de transforms para o ECS.
- Estender o hash c9 ao mundo ligado ao ECS e plugar na CI cross-OS.
**Gates (red-first, mutation-tested):**
- e2e no app REAL: sprite cai e assenta no chão (não unit do wrapper — [[feedback_tool_unit_green_integration_dead]]).
- hash cross-OS estável do mundo ECS-bridged (byte-idêntico; mutar a ordem de iteração sangra).
- zero-alloc no `physics_step` (dhat por capacidade, HR-3).
- tick único: play anda N steps, scrub anda 1 (gate de emenda com **advance fracionário** —
  taxa 1:1 nunca lê o 2º frame, [[feedback_seam_gates_need_fractional_advance]]).
- snapshot é ponto fixo (nenhum passo de undo espúrio por frame parado).
**Smoke:** `PH2D_PHYSICS_SMOKE=1` — cena auto-play que dropa 1 sprite sobre um chão. **Exemplo pronto
pra smoke, auto-play** ([[feedback_ready_to_smoke_example]]); comando com o `cd <worktree> &&` junto.
**Fora:** painel, joints, bake.

### W2 — Painel global + Inspector body  *(a autoria)*
**Objetivo:** o artista liga/desliga a física, seta gravidade/escala no painel de mundo, e edita
massa/restituição/atrito/tipo num sprite selecionado.
**Entregáveis:**
- `ph2d-panel-physics` docado (categoria mundo): gravidade (vetor), substeps/iterações, escala
  pixel→metro, damping, sleep, matriz de camadas. Tokens + i18n (zero hex/f32/string hardcoded).
- Seção "Physics Body" no Inspector (por-seleção): type/mass/restitution/friction/collider-shape.
- Registro do painel nos 5 sites; visibilidade; NumberInput com range/clamp const.
**Gates:** painel **pintado E populado E clicado** (WidgetStore; [[feedback_painted_is_not_populated_paint_gate]]
+ [[feedback_widget_is_done_when_a_test_clicks_it]]); toda row de setting muda o mundo (seam que CLICA);
sem string hardcoded (gate i18n); botão dimmed recusa no event.rs.
**Smoke:** `PH2D_PHYSICS_SMOKE=2` — abre o painel, ajusta gravidade, dropa corpos.
**Fora:** joints, bake.

### W3 — Joints  *(as articulações)*
**Objetivo:** pino/mola/motor/distância entre corpos; pêndulo, corrente, rag-doll simples.
**Entregáveis:** components de joint (registrados), autoria no Inspector/canvas (gizmo de ancoragem),
mapeamento para `ImpulseJointSet`/`MultibodyJointSet` do rapier. Determinismo preservado.
**Gates:** pêndulo de 2 corpos determinístico (hash estável); joint sobrevive save/load (schema bump);
mutação de um parâmetro de joint sangra o gate de repro.
**Smoke:** `PH2D_PHYSICS_SMOKE=3` — pêndulo/corrente auto-play.
**Fora:** bake.

### W4 — Bake-to-timeline  *(runtime-truth vira animação)*
**Objetivo:** o botão "Bake" amostra a sim sobre um range e escreve keys editáveis nas tracks da
entidade — a metade motion-graphics do framing.
**Entregáveis:** amostragem determinística da pose por frame → `fit_fcurve`/Schneider do `ph2d-anim`
(colunas alinhadas, pré-filtro passa-baixa se preciso), 1 passo de undo, via a ponte da timeline/anim.
**Gates:** curva assada reproduz a sim dentro da tolerância (oráculo de APARÊNCIA, não de regra —
[[reference_topic_oracle_discipline]]); bake é determinístico; 1 undo step (não 1 por frame).
**Smoke:** `PH2D_PHYSICS_SMOKE=4` — dropa corpos, assa, dá play na timeline sem a física ligada.
**Fora:** soft-body, fluidos, collider-gen vetorial, fratura (M13+ / linhas próprias).

---

## 6 · Regras de engenharia (a DIRETIVA, condensada)

A regra-mãe da [`DIRETIVA_IMPLEMENTACAO.md`](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
(releia a cada passo): **verde-de-compilação é velocidade; no audit vale ZERO.** As 4 causas da semana
perdida no Painter — evite-as por construção:
1. **Costura não-testada** → todo seam tem um gate que EXERCITA a costura (que CLICA, que dá o tick,
   que roda o `paint`), não que só compila.
2. **"Audit" = compilar** → auditar é rodar o gate e OLHAR o resultado, com ≥2 lentes.
3. **Isolamento órfão** → a feature nasce ligada ao app real e provada e2e, não numa unit que passa sozinha.
4. **Alvo irrefutável** → o gate nasce **VERMELHO** sobre o bug real (com os números do PRODUTO,
   [[feedback_test_with_product_numbers_not_convenient_ones]]), depois fica verde. Verde de 1ª pode ser
   verde por acidente ([[feedback_a_green_gate_may_be_green_by_accident]]).

**Provas de mutação** ([[reference_topic_mutation_proofs]]): mute o código de produção; o gate tem que
ficar RED; sobrevivente = gate faltando. **Defesa em camadas = gate POR camada**
([[feedback_layered_defenses_need_per_layer_gates]]).

**Velocidade (§2):** inner loop = `cargo check -p ph2d-physics` (ou `ph2d-physics-ecs`). Teste/clippy/
auditoria **1× no fechamento** do módulo, sobre o diff acumulado — nunca por task. Workstation voa;
use rust-analyzer full como oráculo, não leia saída crua do cargo.

**LOC caps (HR-18):** shells/foundational têm cap de 600 LOC/arquivo; campo/mod novo que estoura →
**split em módulo irmão**, não allowlist ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).
`cargo fmt` re-expande → fmt ANTES de medir.

**Git (Modo L):** worktree próprio, sem colisão de commit; valem só os conflitos de merge + as
proibições da DIRETRIZ §1.5.5–1.5.6. Rebase main no início/antes de integrar. Cargo.lock/gerados:
regenere, não resolva na mão.

---

## 7 · Contratos congelados e isolamento

- **NÃO toque** nos contratos congelados (CLAUDE.md §6): `NodeOp`/`OpResolver`/`NodeManifest` (Nodes);
  `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` (Tools); `VectorOp`/`Vertex`/`Segment`/… (Vector-doc).
  Física não precisa deles.
- **O novo contrato de física é SEU para desenhar** — mas projete-o para ISOLAMENTO desde o dia 1
  (append-only, variants no fim, ids do próximo livre anotados) porque ele **vai congelar** (follow-up,
  com gate `architecture_physics_contract_surface` à la os outros). Um contrato desenhado para
  isolamento é o que deixa a próxima linha estendê-lo sem colidir ([[feedback_foundational_editable_design_for_isolation]]).
- **Cerca de Chesterton** ([[feedback_documented_decision_chesterton_fence]]): a dormência do M10 e o
  "XPBD é M13+" são decisões DOCUMENTADAS. Não as sobrescreva; herde-as.

---

## 8 · O padrão da casa (a inspiração)

Este módulo tem o **teto mais alto** do projeto agora: um motor de física de engine, determinístico
cross-plataforma, com autoria de artista e bake para animação — a peça que coloca o PH2D no páreo com
Rive/Cavalry e à frente de Godot/Unity em 2D autoral. E você começa com a parte assustadora **já
resolvida**: o solver existe, o determinismo está gateado. O que resta é a engenharia bonita — a
ponte, o relógio único, o painel de mundo, o scrub bit-exato, o bake.

Faça no **padrão-ouro** (§0.6): a melhor opção técnica vence custo de build e cronograma; gaps in-scope
fecham na sessão. Cada gate nasce vermelho sobre o bug real e morre por uma razão que você pode nomear.
Cada costura é exercitada, não só compilada. Cada número vem do produto, não da conveniência. Quando
fechar, o motor não vai *parecer* funcionar — vai funcionar, e um teste que clica, ticka e olha vai
provar isso.

Comece pela FASE 1. Depois escreva a ADR-0131, o plano de waves e o tracker. Então pare e reporte.

**Bom trabalho. Faça o seu melhor.**
