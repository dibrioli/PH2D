# Handoff de INTEGRAÇÃO — `line/physics` (DIRETRIZ §1.5.9)

> Para o **agente integrador**. A linha está fechada e **não integra nem faz ship** por conta própria.
> Estado técnico completo do módulo: [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) ·
> decisão: [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md).

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/physics` |
| HEAD | `64e41e31` + o commit que carimba este handoff — **`git rev-parse HEAD` é a fonte**; um documento que registra o próprio hash está sempre um atrás |
| Base do fork (merge-base com `main`) | `389676f9` |
| Commits | **45** (`git rev-list --count main..HEAD`) |
| Waves cobertas | W0 · W1 · W1.5 · W2a · W2b · W2c · W3 · W4 (a última do plano) · **W4b** (o toggle Physics) · **W5** (corpos filhos) |
| Smoke | **as 8 cenas APROVADAS pelo Enio** (a última em 2026-07-19) |
| Fechada em | **2026-07-19** |
| ⚠️ `main` andou desde o fork? | **NÃO** — `merge-base == main HEAD == 389676f9` no momento deste handoff |

**Consequência:** enquanto a `main` não andar, isto é **fast-forward puro** (`--ff-only` passa sem
Mergiraf). Se outra linha integrar antes, veja o §3 — os pontos de colisão estão todos listados com valor
literal.

---

## 2. Foundational / compartilhado tocado, e por quê

A linha é **Modo L**, então foundational é permitido sob o protocolo testado (ADR-0107). Nada aqui é
gratuito — cada item existe porque a física precisava de uma porta que não havia.

### 2.1 Crates NOVAS (glob `crates/*` — **zero edit central**)
- **`ph2d-physics-ecs`** — a ponte ECS↔rapier (components + `PhysicsBridge`). É o coração da linha.

### 2.2 `ph2d-physics` (a crate M10, era dormente) — **aditivo**
`spawn_body`/`set_body_pose`/`remove_body` + `BodyDesc`/`ShapeDesc` · `checkpoint`/`restore` +
`PhysicsCheckpointRing` (módulo filho `world/checkpoint.rs`) · knobs de contato/substeps.
⚠️ **Os helpers antigos e `step` NÃO foram tocados**, mas os **defaults de integração mudaram** (§3.6).

### 2.3 `ph2d-editor-core` — **aditivo, exceto uma catraca de LOC**
| Arquivo | O quê |
|---|---|
| `action_bus.rs` | +2 variants (`Transport`, `InspectorPhysicsEdit`) — enum `#[non_exhaustive]`, **não serializado** |
| `ids/inspector.rs` | +14 ids §11 (slugs hasheados) |
| `screens/hero/inspector_model.rs` | +`InspectorPhysicsInfo` +`PhysicsFieldEdit` |
| `lib.rs`, `screens/mod.rs` | re-exports dos dois acima |
| `screens/hero/chrome/transport.rs` | **NOVO** handler (`z=300`) |
| `screens/hero/chrome/mod.rs` | **GERADO** por `ph2d-chrome-sync` (§3.3) |
| `screens/hero/topbar/mod.rs` | tooltips Play/Pause/Reset (eram mentira) |
| `tests/node_id_collisions.rs` | +14 linhas na tabela à mão |
| `tests/architecture_panel_loc_cap.rs` | ⚠️ **allowance `paint_inspector` 431 → 424** (§3.5) |

### 2.4 `ph2d-panel-inspector` — seção §11 + **um split forçado pelo teto de LOC**
Novos: `sections/physics.rs`, `event_physics.rs`, `paint_frame.rs`, `tests/seam_physics.rs`.
⚠️ **`paint.rs` foi REESTRUTURADO** (não só acrescido): o corpo do macro `live_section!` e a fase B
saíram para `paint_frame.rs` porque `paint_inspector` estava congelado em 431 LOC e uma seção custa ~18.
**É o arquivo com maior risco de conflito textual** se outra linha mexeu no Inspector.

### 2.4b W3 — o que a wave de joints acrescentou (tudo **aditivo**)
| Onde | O quê |
|---|---|
| `ph2d-ecs` | **`stable_name_id`** em `name.rs` + re-export. ⚠️ `shells/desktop/src/timeline_persist.rs::wire_id_for_name` passou a **DELEGAR** — mesma FNV-1a byte a byte (pinada contra valores externos), mas é **edição num arquivo da timeline**: num conflito, mantenha a delegação |
| `ph2d-physics` | módulo `world/joints.rs` + re-export de `ImpulseJointHandle` |
| `ph2d-physics-ecs` | `src/joint.rs` (component `PhysicsJoint` + enum `JointKind`), `src/bridge/joints.rs`, dev-dep `postcard`. ⚠️ **a contagem do registry foi 2 → 3** (o teste `registers_every_physics_component`) |
| `ph2d-editor-core` | `InspectorJointInfo`/`JointFieldEdit`; `InspectorPhysicsInfo.can_join`; `PhysicsFieldEdit::Join`; **23 ids §12** na tabela do `node_id_collisions`. ⚠️ `any_live_section` `[bool; 8] → [bool; 9]` e os slots de nota `10 → 11` — **os dois são rígidos DE PROPÓSITO** |
| `ph2d-panel-inspector` | `sections/joint.rs`, `sections/rows.rs` (helpers **movidos** de `physics.rs`), `event_joint.rs`, `tests/seam_joint.rs`. ⚠️ `paint.rs` perdeu a família de física para `paint_frame::paint_physics_sections` |
| `shells/desktop` | `render_loop/inspector_joint.rs` (+tests), `render_loop/physics_overlay_joints.rs`, `tests/join_is_one_gesture_not_a_fan_out.rs`, cena de smoke **6** |

### 2.4c W4 — o que a wave de bake acrescentou (tudo **aditivo**, mas com DOIS renames)
| Onde | O quê |
|---|---|
| `ph2d-physics` | módulo **`world/kinematic.rs`** (split forçado: `world.rs` bateu **779/700**) — `set_next_kinematic_pose`, `kinematic_slice`, `kinematic_aim_count`; campo `PhysicsWorld.kinematic_targets`. ⚠️ **`PhysicsWorld::step` mudou**: ganhou um laço de fatiamento ANTES do `drag::apply`. Empty quando nada é kinematic ⇒ toda cena que já existia roda byte-idêntica (gate `a_world_with_no_kinematic_body_is_untouched`) |
| `ph2d-physics-ecs` | módulo **`src/bake.rs`**; variant **`BodyKind::Kinematic`** (APENDADO, tag `2`) + `solver_owns_pose`/`tag`/`from_tag`; estágio `PhysicsBridge::drive_kinematic`. ⚠️ **`BodyKind` é `match`ado exaustivamente** em `bridge.rs::body_desc` e em `shells/desktop/src/render_loop/physics_overlay.rs` — um merge que traga outro leitor do enum **não compila** até tratar o 3º braço, que é o comportamento desejado |
| `ph2d-editor-core` | id `INSP_PHYS_BAKE`; ⚠️ **`INSP_PHYS_KIND` foi `[NodeId; 2]` → `[NodeId; 3]`** (a tabela do `node_id_collisions` é escrita **à MÃO por índice** e parava no `[1]` — se outra linha tocar essa tabela, o `[2]` tem de sobreviver); variant `PhysicsFieldEdit::Bake`; campo `InspectorPhysicsInfo.bake_seconds` |
| `ph2d-panel-inspector` | `sections/physics.rs::paint_body_actions` (split: `paint_physics_section` bateu **211/200**); `KIND_LABELS` `[&str; 2] → [&str; 3]` |
| `shells/desktop` | `render_loop/physics_bake.rs` (+tests) · **`render_loop/record_fit.rs`** · `physics_smoke_bake` (cena **7**) · `KINEMATIC_RGBA` no overlay |

⚠️ **DOIS renames, e o integrador tem de saber dos dois:**
1. `shells/desktop/tests/join_is_one_gesture_not_a_fan_out.rs` → **`selection_gestures_are_not_fanned_out.rs`**
   (feito com `git mv`, então o diff é um rename; agora cobre Join **e** Bake).
2. **`simplify_recorded`, `RecSpan`, `value_tol` e as 4 consts do record SAÍRAM de
   `render_loop/autokey_pass.rs` para `render_loop/record_fit.rs`** — e `simplify_recorded` **ganhou o
   parâmetro `smooth_passes`** (o record passa `REC_SMOOTH_PASSES`, o bake passa `0`; ver §W4 do tracker
   para os números que decidiram isso). ⚠️ **Este é o único ponto da wave que mexe em código de OUTRA
   linha (a timeline).** `autokey_pass.rs` encolheu de 466 para 330 LOC. Num conflito: a lógica é
   idêntica à que estava lá — o que não pode voltar é uma **segunda cópia** do ajuste, que é a coisa
   inteira que a extração comprou. As 26 gates do record rodam verdes depois dela.

⚠️ **Assinaturas que mudaram (chamadores atualizados nesta árvore, mas um merge pode trazer outro):**
- `render_loop::snapshots::publish(...)` ganhou um último parâmetro `bake_seconds: f32`.
- `render_loop::inspector_physics::build_physics_info(world, bits, can_join, bake_seconds)`.
- `render_loop::physics_bridge::dispatch(...)` ganhou `doc: &mut TimelineDoc` (a ponte precisa
  saber onde a timeline põe os corpos dirigidos — §2.4d).

### 2.4d W4 — o que a AUDITORIA mudou (dois commits depois do W4)

A auditoria de 2 lentes achou dez coisas; o detalhe está no §W4 do tracker. O que o **integrador**
precisa saber:

| Onde | O quê |
|---|---|
| `ph2d-physics-ecs` | **trait pública `SceneAtTick` + `FrozenScene`** e **`PhysicsBridge::dispatch_with_scene`** (`src/bridge/kinematic.rs`, split do teto de 700). ⚠️ `dispatch` **mantém a assinatura** e delega — os 99 chamadores existentes não mudaram. `bake_trajectories_with_scene` idem |
| `ph2d-physics` | **`PhysicsWorld::slice_pose`** (pública, dados simples) — a lei de interpolação que o `step` e a ponte **têm** de compartilhar |
| `ph2d-panel-inspector` | **`pub fn bake_label`** re-exportado da crate; `Kind`/`Shape` ganharam recusa por `has_body`; o botão Bake só é oferecido para `kind_tag == 0` |
| **`ph2d-editor-core`** | ⚠️ **`architecture_panel_wiring_parity` mudou de COBERTURA** (`read_paint_sources` agora lê `paint*` **+ `sections/`**). Não é aditivo: se outra linha trouxer um painel com widgets em `sections/` que não estejam no `populate.rs` dele, este gate fica **VERMELHO no merge** — e estará certo. **Buraco pré-existente**, não desta wave |

⚠️ **Quatro ids de OUTRAS waves ficam nomeados aqui em vez de allowlistados por mim.** Ler *todo* `src/`
(em vez de `paint*` + `sections/`) também acusa `TIMELINE_LANES`, `TIMELINE_SCROLLBAR`,
`TIMELINE_CLIP_RENAME_INPUT` e `PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP`. Parecem os widgets **dinâmicos**
que a allowlist do gate já documenta (um campo de rename, lanes, uma scrollbar), mas decidir isso é dos
**donos deles** — quem quiser fechar o resto do buraco começa por aqui.

### 2.4e W4b — o toggle **Physics** do transporte ⚠️ **encosta na linha da ANIMAÇÃO**

Enio reportou o conflito (*"o play ativa a simulação física … a simulação roda junto com a animação"*)
e pediu o checkbox. É a única parte desta linha que **edita crates da timeline**, então é a de maior
risco de merge. Tudo append-only, **zero bump de schema** (nada disto é serializado).

| Crate | O quê | Risco de merge |
|---|---|---|
| `ph2d-timeline` | campo `TimelineFlags::simulate_physics` (default **false**) · variant `TimelineIntent::SetSimulatePhysics(bool)` · campo `TimelineViewSnapshot::simulate_physics` + fill no `rebuild` | **baixo** — 3 campos/variants apendados. `DOC_VERSION` **intacto**: `TimelineFlags` não é serializado (o Record estabeleceu o precedente) |
| `ph2d-editor-core` | `TIMELINE_PHYSICS` no bloco *Transport bar* de `ids/chrome/timeline.rs` | baixo (append) |
| `ph2d-i18n` | `panel.timeline.physics` → `"Physics"` | baixo (append) |
| `ph2d-panel-timeline` | `ids.rs` (re-export) · `populate.rs` (+1) · `event.rs` (`is_toggle` +1) · **`transport.rs`** | ⚠️ **`ITEMS: [Item; 13] → [Item; 14]`** — é uma **CONTAGEM COMPARTILHADA**: se outra linha também acrescentou um item de transporte, o número **se CONTA, não se escolhe** (§3.7) |
| `shells/desktop` | `timeline_bridge.rs` (+1 braço) · `render_loop/mod.rs` (lê o flag) · `physics_smoke.rs` (arma o flag) | baixo |
| `ph2d-physics-ecs` | **`PhysicsBridge::hold`** público (`src/bridge/hold.rs`, módulo novo por LOC) + `prepare` privado | nenhum (aditivo) |
| `shells/desktop` | ⚠️ **`physics_bridge::dispatch` ganhou o parâmetro `simulate: bool`** | **assinatura MUDOU** — 1 chamador só (`render_loop/mod.rs`), mas um merge que traga outro chamador não compila |

⚠️ **`00_plano_waves.md` §W4 foi CORRIGIDO no lugar.** Ele afirmava que *"o desligamento manual seria o
desenho errado de qualquer jeito"* — frase minha, do W4, que generalizava demais: respondia *"o Bake
desliga a física no corpo assado?"* (não — entrega a pose via `Kinematic`) como se valesse para qualquer
interruptor. O toggle é do **transporte**, o `Kinematic` é do **corpo**; não se tocam. A correção está
datada no plano, porque nota velha que contradiz o código faz a próxima LLM propor desfazer o que existe.

### 2.4f W5 — corpos FILHOS ⚠️ **encosta em `ph2d-ecs`, a foundational mais compartilhada**

Correção de bug, não capacidade: um corpo físico parenteado **simulava num lugar e
desenhava noutro**, em silêncio (o collider não estava sob o sprite). Detalhe:
[`BUGS_physics.md`](BUGS_physics.md) #2.

| Crate | O quê | Risco de merge |
|---|---|---|
| **`ph2d-ecs`** | módulo NOVO `transform_inverse.rs`: `Transform::inverse_compose` + `is_finite` + **`world_transform{,_into}`**, e as duas `parent_world_transform{,_into}` **MOVIDAS** de `transform.rs` para lá | ⚠️ **médio** — é um MOVE dentro da foundational mais compartilhada do repo. Os símbolos seguem re-exportados do `lib.rs`, então **nenhum chamador muda**; um merge que edite `transform.rs` na região movida conflita textualmente |
| `ph2d-physics-ecs` | módulo NOVO `bridge/space.rs` + campo `chain`; 5 sítios religados. `space::world_transform` **delega** a `ph2d_ecs::world_transform_into` | nenhum (só desta linha) |
| `shells/desktop` | ⚠️ `render_loop/physics_overlay.rs` — o **SEXTO** leitor da pose, achado no smoke: lia o `Transform` cru e desenhava o contorno na pose LOCAL. Agora chama `ph2d_ecs::world_transform_into` | baixo (só desta linha), mas ver o aviso abaixo |
| `shells/desktop` | módulo NOVO `physics_smoke_rigs.rs` (cenas 6/7/8 movidas para lá); `spawn_floor` virou `pub(crate)` | baixo |
| **`ph2d-editor-core`** | ⚠️ a catraca de LOC `ph2d-ecs/src/transform.rs` **BAIXOU 784 → 768** | ⚠️ **se outra linha crescer aquele arquivo, o gate fica VERMELHO no merge** — e estará certo. O número se **CONTA** (mede-se o arquivo depois do `fmt`), não se escolhe |

⚠️ **`Transform` NÃO é contrato congelado** (o gate `architecture_vector_contract_surface`
escaneia `ph2d-vector-doc`/`-traits`, não `ph2d-ecs`) e nada foi removido — só acrescentado
e movido dentro da mesma crate, com re-export. Verificado, §4.

⚠️ **O AVISO QUE VALE PARA AS OUTRAS LINHAS, e é o achado mais transferível desta wave.**
`ph2d_ecs::world_transform(world, entity)` é agora **a** resposta a *"onde esta entidade está no
mundo?"* para quem trabalha sobre o `SimWorld`. Quem lê `Transform` cru e o trata como mundo está
certo **apenas enquanto a entidade for raiz** — e essa premissa é invisível, porque toda fixture
construída sobre raízes passa. Esta linha achou seis desses sítios (cinco na ponte, um no overlay,
o último só no smoke do Enio, com os colliders empilhados no centro da cena longe das artes).
**Se a sua linha computa em espaço de mundo e lê `Transform`, vale a pena `grep`.** O gate que
pega a classe é ter **um pai** na fixture: os 12 gates do overlay eram todos raiz e ficaram verdes
sobre o bug.

### 2.5 `ph2d-vector` — **1 linha, aditiva**
`PathEl` acrescentado à lista de re-export do kurbo (`src/lib.rs:58`). **Não é a superfície congelada** —
o gate `architecture_vector_contract_surface` escaneia só `-doc` e `-traits` (verificado, §4).

### 2.6 `shells/desktop`
Novos: `transport.rs`, `physics_smoke.rs`, `render_loop/{physics_bridge,physics_overlay,inspector_physics,inspector_physics_tests}.rs`.
Modificados: `main.rs`/`app_state.rs` (2 campos), `init.rs` (registro + gate do smoke), `input_handlers.rs`
(**tecla `B`**, §3.4), `render_loop/mod.rs` (mod decls + 1 chamada de dispatch + 1 braço de dreno),
`snapshots.rs` (publica o snapshot §11), `inspector_{commits,ordering}.rs`, `project.rs`/`project_tests.rs`
(**schema**, §3.1), `Cargo.toml` (dep na ponte).

### 2.7 `.github/workflows/spike.yml`
+1 job de matriz `physics-ecs-c9` + artifact + 3ª comparação no `determinism-compare` (espelho do
`ph2d_physics_c9` que já existia).

---

## 3. Símbolos que podem COLIDIR (grep isto — §1.5.5)

### 3.1 ⚠️ `PROJECT_SCHEMA = 21` — **o valor se CONTA, não se escolhe**
`shells/desktop/src/project.rs:86` e a tripla-pin `(21, 8, 8)` em `project_tests.rs`.

**Esta linha bumpou CINCO vezes**: 16 (componentes de física no `WorldSnapshot`) · 17
(`restitution`/`friction` apendados ao `Collider`) · 19 (as settings de mundo do W2b viajam no arquivo;
o 18 veio da re-contagem que somou o bump da `line/FLIP` na integração anterior) · 20 (`air_drag`
apendado pós-smoke) · 21 (camada + matriz do W2c).

⚠️ **O W3 NÃO bumpou, de propósito, e isso também é a contagem falando.** Um componente NOVO não move
layout nenhum: o blob é chaveado por `stable_type_id = blake3(nome_canônico)[..8]`, derivado do **NOME**,
então registrar `ph2d::physics::PhysicsJoint` cunha um id novo e todo blob já em disco fica onde estava.
É o oposto do W2c, que apendou um campo **DENTRO** do `Collider`, onde postcard é posicional. E bumpar à
toa não é neutro: schema divergente **recusa o arquivo inteiro**, jogando fora todo projeto já salvo. O
raciocínio está falsificável em `crates/ph2d-physics-ecs/tests/joint_persistence.rs` — se algo mover o
layout, o 1º gate fica vermelho e o bump passa a ser devido. **Se outra linha também
bumpou, o valor certo não está em nenhum dos dois lados: some os bumps.** Escolher um lado faz os saves
da outra passarem na checagem de versão e serem lidos com o layout errado — e postcard não tem nome de
campo para reclamar, ele devolve lixo bem-formado. (É o cenário que a doc do próprio gate descreve, e que
já aconteceu com QUATRO linhas em 2026-07-13.)

### 3.2 `EditorAction` — 3 variants apendados
`Transport(TransportCmd)` · `InspectorPhysicsEdit { entity_bits, edit }` ·
**`InspectorJointEdit { entity_bits, edit }`** (W3).
Enum `#[non_exhaustive]` e **não serializado** (é barramento de frame) ⇒ **a ordem é livre**; num conflito,
**mantenha os dois lados**. Mesma regra para `TransportCmd` (enum novo no mesmo arquivo).

### 3.3 ⚠️ Chrome `z=300` — o dispatch é **GERADO**
`chrome/transport.rs` traz o marcador `// ph2d-chrome-sync:z=300`. Os z ocupados hoje: …240, 270, 271,
280, 290, **300**. Se outra linha também tomou 300, **renumere uma delas** e **re-rode
`cargo run -p ph2d-chrome-sync`** — o `chrome/mod.rs` é saída de codegen, nunca resolva o conflito à mão.

### 3.4 ⚠️ **Tecla `B` global** (`input_handlers.rs:132`) — toggle do contorno de collider
Espaço de teclas é compartilhado e não tem gate de colisão. `B` estava livre desde que o W4.T5 da timeline
aposentou a demo de `SpriteAnimation`. **Se outra linha também tomou `B`, é conflito de verdade** (as duas
compilam e a última ganha em silêncio) — escale ao Enio para escolher a tecla.

### 3.5 ⚠️ Allowance de LOC `paint_inspector` **431 → 424** — também se **RE-MEDE**
`tests/architecture_panel_loc_cap.rs:114`. As allowances **só encolhem** (é catraca). Se outra linha também
mexeu em `paint_inspector`, **o número certo é o medido DEPOIS do merge**, não o de nenhum dos lados —
rode o gate e use o valor que ele reportar. Mesma classe do §3.1.

### 3.6 ⚠️ Os dois hashes C9 **MUDARAM** (esperado, não é regressão)
`physics-c9` → `2f7e2d586d395dd7…` · `physics-ecs-c9` → `54fea29671c866b5…`. Causa: os defaults de
integração (`DEFAULT_SUBSTEPS = 4`, `DEFAULT_CONTACT_HZ = 120`) entram no solver.
**Nenhum é pinado em literal** — o CI compara os 3 OSes entre si (`sort -u | wc -l`), então o gate segue
válido. Não "conserte" o hash.

### 3.7 Listas ordenadas / contagens compartilhadas
- `tests/node_id_collisions.rs` — +14 linhas, **append-only**, num conflito mantenha os dois lados.
- `paint.rs:217` — `notes_per_section: [_; 10]` (era 9). **Se outra linha também adicionou seção, re-conte.**
- `ph2d-vector/src/lib.rs:58` — `PathEl` na lista de re-export: mantenha os dois lados.
- ⚠️ `crates/ph2d-editor-core/tests/architecture_workspace_file_loc_cap.rs` — a entrada
  `ph2d-ecs/src/transform.rs` **baixou 784 → 768** (W5). Re-**MEÇA** o arquivo depois do
  `fmt` no merge; não escolha um dos dois números.
- ⚠️ `ph2d-panel-timeline/src/transport.rs` — **`const ITEMS: [Item; 14]`** (era 13). Contagem
  compartilhada com a linha da animação: se ela também acrescentou um item de transporte, **re-conte**
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

### 3.8 Ids novos (slugs hasheados — únicos por construção, mas a TABELA é compartilhada)
`insp_live_physics_{section,color}` · `insp_phys_{add,remove,radius,half_x,half_y,density,restitution,friction}` ·
`insp_phys_kind_{dynamic,static}` · `insp_phys_shape_{ball,box}`.
Mais, do W4b: `timeline.physics` (**`TIMELINE_PHYSICS`**). O W5 não acrescenta id nenhum.
Sem `NodeId(N)` numérico novo. **`PHYSICS_SCROLLBAR_ID = NodeId(836)` foi RESERVADO no plano mas NÃO
usado** (o painel global não foi construído) — está livre.

---

## 4. Contratos congelados encostados: **NENHUM**

Verificado por grep (nenhum arquivo de `ph2d-nodegraph`, `ph2d-vector-doc`, `ph2d-vector-traits`,
`tool-registry` ou dos 3 `architecture_*_contract_surface` foi tocado) **e** pelos gates rodando verdes
nesta árvore:

```
architecture_contract_surface .......... 3 passed
architecture_tool_contract_surface ..... 4 passed
architecture_vector_contract_surface ... 11 passed
```

Nenhum ADR adicional é exigido além do **0131** (escrito por esta linha como 0130 e renumerado na integração — a `line/gpu-nodes` reclamou o 0130 no mesmo dia).

---

## 5. O que só o `ship.sh` pega — **já rodei todos nesta árvore**

O gate de integração não roda estes; para poupar iterações do integrador
([[project_integration_prefork_lines_ship_drift]]), rodei o **`scripts/ship.sh` inteiro** nesta árvore,
no fechamento (2026-07-19), e o estado é:

⚠️ **Rodado com `env RUSTUP_TOOLCHAIN=1.95`.** O default do rustup se perdeu nesta máquina no meio da
sessão e só a 1.95 (o pin do repo) está instalada — é ambiente, não código
([[feedback_a_ship_x_can_be_the_environment_not_the_code]]). O `ship.sh` chama `cargo` nu, então sem a
env ele morre antes do primeiro gate.

⚠️ **E ele achou um `✗` real que a versão anterior deste handoff declarava limpo:** `typos` pegou
`entitys` num nome de teste do W4 (`a_bake_lands_on_the_entitys_clock_not_the_playheads`). Corrigido para
`a_bake_lands_on_the_clock_of_its_entity_not_the_playhead`. É o motivo de re-rodar em vez de confiar na
tabela: entre o W4 e o fechamento entraram ~1500 linhas de prosa e código novos.

| Gate | Resultado nesta árvore |
|---|---|
| `cargo fmt --all --check` | **limpo** |
| `typos` | **limpo** |
| `cargo machete` | **limpo** (nenhuma dep não-usada) |
| `cargo deny check` | **advisories ok · bans ok · licenses ok · sources ok** |
| `cargo audit` | 3 warnings **allowlistados e pré-existentes** (memmap2 etc.); **nada novo desta linha** |
| `cargo clippy --workspace --all-targets` | **limpo** |
| `cargo nextest run --workspace --cargo-profile ci-test` | **7805 passed, 157 skipped, 0 falhas** |

**Deps novas** (para o machete/deny do integrador conferirem pós-merge): `ph2d-physics-ecs` puxa
`bevy_ecs 0.18`, `ph2d-core`, `ph2d-ecs`, `ph2d-physics`, `serde` — todas já no workspace. **O W4 não
acrescentou dep nenhuma**: o bake chama `ph2d-anim`/`ph2d-timeline` de dentro do shell, que já dependia
das duas, e a crate de física continua sem saber que uma timeline existe. **A única dep
externa nova em toda a linha é `dhat 0.3` como `[dev-dependencies]` de `ph2d-physics`** (o gate de
memória do ring; mesma dep que a `ph2d-audio-edit` já usa).

⚠️ **`rapier2d` NÃO ganhou features novas.** `parallel`/`simd-*` seguem OFF — ligá-las quebra HR-5.

---

## 6. Ordem, dependências e o que smoke-testar

### 6.1 Ordem
Os commits são **estritamente sequenciais** e não há como reordená-los: W1 cria a ponte, W1.5 depende
dela, o overlay depende do W1.5 (a cena de smoke), o W2 depende dos componentes, o W3 depende dos
corpos, o W4 depende do W1.5 (bake = a mesma sim anotada), o W4b depende do W4 (o toggle é
demonstrado pelo bake) e o W5 corrige uma premissa que todos eles carregavam.
**Integre a linha inteira ou nenhuma parte** — não há corte parcial coerente.

### 6.2 Depois do merge, ANTES de declarar verde
1. `cargo run -p ph2d-chrome-sync` — o `chrome/mod.rs` é gerado (§3.3).
2. `cargo check --workspace --all-targets` — **merge textual limpo pode estar semanticamente quebrado**
   ([[feedback_clean_text_merge_can_be_semantically_broken]]); só a checagem cruzada pega.
3. Se `main` andou: re-medir §3.1 (schema), §3.5 (allowance de LOC) e §3.7 (contagens) — **os três se
   contam, não se escolhem.**

### 6.3 Smoke

Todo comando abaixo é para rodar **de dentro do worktree**, e com `env` porque o shell do Enio é
**fish** (`VAR=valor comando` não funciona lá — a env var seria ignorada em silêncio e a cena nunca
montaria):

```
env PH2D_PHYSICS_SMOKE=<n> cargo run -p ph2d-host-desktop
```

| Cena | O quê | Estado |
|---|---|---|
| `=1` | W1: um corpo cai e assenta | ✅ aprovado |
| `=2` | W1.5: pilha + scrub da régua pra trás | ✅ aprovado |
| (mesma cena) | contorno de collider + tecla `B` | ✅ aprovado |
| `=3` | W2a: Add Physics Body no Inspector | ✅ aprovado |
| (mesma cena) | interpenetração no pouso | ✅ aprovado |
| `=4` | W2b: o painel de mundo (tecla `W`) | ✅ aprovado |
| `=5` | W2c: a matriz de camadas | ✅ aprovado |
| `=6` | W3: pêndulo · corrente · ragdoll | ✅ aprovado |
| `=7` | W4: assar a sim em curvas da timeline | ✅ aprovado |
| (mesma cena) | W4b: o toggle Physics do transporte | ✅ aprovado |
| `=8` | W5: corpos FILHOS (o collider sob o sprite) | ✅ aprovado |

✅ **TODAS as 8 cenas foram aprovadas** — a 7 em 2026-07-18 (*"smoke OK. Funciona muito bem"*) e a 8 em
2026-07-19, depois de **três rodadas** (ver o aviso ao fim desta seção). A cena 7 nasce **PAUSADA** com a
timeline aberta. O gesto: Play uma vez para ver o movimento, rebobinar,
selecionar o `Roller` e as duas caixas, e **Inspector › Physics Body › `Bake 5.0s to Timeline`**.

O que ela tem de mostrar (detalhe no §W4 do tracker):
- a timeline enche com **poucas chaves por canal, em colunas alinhadas** — *não* uma por frame (isso
  seria inutilizável, e é exatamente o bug que o gate novo pega);
- o chip **Body vira KINEMATIC** e o toast diz isso. Tecla `B`: os contornos passam de ciano a
  **VIOLETA** — o solver não é mais o dono daquelas poses;
- **Play**: os objetos repetem o MESMO movimento, agora dirigidos pelas curvas;
- **UM** Ctrl+Z tira o bake inteiro (todas as chaves, uma pressionada).

**Cena 8 (W5).** Três rigs — um nível, dois níveis, e um **rotacionado** — cada um com uma bola física
parenteada, cada um sobre um pedestal **estreito**. Mostra: cada bola pousa no pedestal **do seu próprio
rig**; a tecla `B` põe cada contorno exatamente sobre o seu sprite, em toda profundidade; e **arrastar um
rig** (o quadradinho azul) leva a bola junto, com o collider acompanhando. A regressão é inconfundível por
construção — um corpo que volte a ler a pose local como mundo cai pela linha `x = 0`, erra o pedestal
sobre o qual foi desenhado, e some de quadro.

⚠️ **A cena 8 levou TRÊS rodadas de smoke, e as três falhas foram minhas — vale ler antes de julgar um
smoke desta linha.** (1ª) os rigs eram **invisíveis** e a fixture do rig rotacionado estava **invertida**
(premiava a implementação bugada); (2ª) o conserto **não chegou ao disco** — um script de edição com
`write` no fim morreu num `assert` tardio e eu tratei o `ok` do script seguinte como se o primeiro tivesse
aplicado, *e* "confirmei" com uma sonda que continha a mudança em vez de importá-la do produto; (3ª) o
**overlay era um sexto leitor** da pose e desenhava os contornos nas coordenadas locais. As três estão
registradas com as lições em [`BUGS_physics.md`](BUGS_physics.md) #2, e cada uma virou gate.

E as duas coisas para as quais o bake existe: arrastar uma chave na timeline (o movimento agora é
editável) e conferir que os corpos assados ainda **empurram** — são kinematic, não fantasmas.

**E então o W4b, na MESMA cena** (o conflito que o Enio reportou depois do smoke do W4): na barra de
transporte, ao lado de Loop/PingPong, há agora um toggle **Physics**.

- **Desmarque-o e dê Play:** o movimento assado **continua tocando** — virou animação, e é para isso que
  se assa. A caixa que você **não** assou fica onde está em vez de cair.
- **Marque de volta:** a simulação volta a rodar, **de onde a cena está** — não replaya em avalanche o
  trecho que passou desarmado (é o gate `arming_mid_take_resumes…`).
- ⚠️ **Nas cenas `=1`..`=6` o toggle nasce MARCADO** — o `physics_smoke.rs` o arma, porque são demos de
  física e o default do produto é **desmarcado**. Num projeto de verdade o app abre com o Play dirigindo
  **só a animação**; física é opt-in por sessão.

### 6.4 O que NÃO foi construído (para o integrador não procurar)
**O plano de waves está COMPLETO** — W0 · W1 · W1.5 · W2a · W2b · W2c · W3 · W4, nada em aberto na
lista. O que ficou de fora ficou **de propósito**, e cada item tem o motivo no tracker:
- **Assar um JOINT** — o bake lê a pose de **corpos**. Uma corrente assada vira N corpos kinematic com
  curvas próprias: reproduz o movimento, descarta a articulação. Assar *a restrição* (ou recusar assar
  corpos unidos) é decisão de design, não mecânica.
- **`Weld`/`FixedJoint`**, motor em mola/corda, gizmo de âncora no canvas, re-escolher os corpos de um
  joint — todos nomeados no §W3.
- **Fora de TODAS as waves (ADR-0131 D9):** soft-body XPBD, fluidos FLIP/PIC, collider-gen vetorial +
  fratura.

---

## 7. Resumo para o Enio

> **Linha `physics` FECHADA — 45 commits, HEAD `64e41e31`, base `389676f9` = `main` atual ⇒
> fast-forward puro.** As 8 waves do plano (W0 · W1 · W1.5 · W2a · W2b · W2c · W3 · W4) mais duas que
> você pediu depois do smoke: **W4b** (o toggle **Physics** no transporte) e **W5** (corpos FILHOS na
> hierarquia). **As 8 cenas de smoke estão aprovadas** — nada pendente de olhos.
>
> **`scripts/ship.sh` INTEIRO verde nesta árvore** (2026-07-19): fmt · clippy `--all-targets` + features
> de CI · machete · deny · audit · typos · **nextest `--cargo-profile ci-test`: 7805 passados, 0 falhas**.
> Ele achou um `✗` real que a versão anterior deste handoff declarava limpo (um `typos` num nome de
> teste) — está corrigido, e é a razão de rodar em vez de confiar na tabela (§5).
>
> **O que grepar antes de fundir (§3), tudo com valor literal:** **`PROJECT_SCHEMA = 21`** e a
> **allowance de LOC `paint_inspector` 424** e a **catraca `ph2d-ecs/src/transform.rs` 768** — os três se
> **CONTAM, não se escolhem** · **chrome `z=300`** (re-rodar o `chrome-sync`) · teclas **`B`** e **`W`** ·
> `any_live_section[;9]`, slots de nota `[;11]` e **`ITEMS: [Item; 14]`** do transporte · 3 variants de
> `EditorAction` · **`INSP_PHYS_KIND` com 3 entradas**. Contratos congelados: **nenhum** (3 gates verdes).
>
> **Onde esta linha encosta em outras** — é o que decide a ordem de integração:
> - **`ph2d-ecs`** (W5): módulo novo `transform_inverse.rs`, com `parent_world_transform{,_into}`
>   **movidas** de `transform.rs`. Re-exportadas do `lib.rs`, então **nenhum chamador muda**; o risco é
>   conflito **textual** se outra linha editou aquela região.
> - **`ph2d-timeline` + `ph2d-panel-timeline`** (W4b): 3 campos/variants apendados, `DOC_VERSION`
>   **intacto**, mais `ITEMS: [Item; 13] → [14]` (contagem compartilhada com a linha da animação).
> - **`ph2d-editor-core`** (W4 auditoria): `architecture_panel_wiring_parity` mudou de **COBERTURA**
>   (lê `paint*` **+** `sections/`). **Não é aditivo** — um painel de outra linha com widgets em
>   `sections/` fora do `populate.rs` fica **VERMELHO no merge**, e estará certo.
>
> ⚠️ **O achado mais transferível, e vale um aviso às outras linhas:**
> `ph2d_ecs::world_transform(world, entity)` é agora **a** resposta a *"onde esta entidade está no
> mundo?"* sobre o `SimWorld`. Quem lê `Transform` cru e o trata como mundo está certo **apenas enquanto
> a entidade for raiz**, e essa premissa é invisível porque toda fixture construída sobre raízes passa.
> Esta linha achou **seis** desses sítios; o último só apareceu no seu smoke, com os colliders empilhados
> no centro da cena. **Se a sua linha computa em mundo e lê `Transform`, vale `grep`** — e o gate que
> pega a classe é ter **um pai** na fixture.
>
> **Duas correções de nota, não de código, que ficam registradas:** o plano dizia que um interruptor de
> física *"seria o desenho errado de qualquer jeito"* (era resposta a outra pergunta — corrigido no lugar,
> com data) e o `readback` prometia corpos-filhos *"no W2"* desde o W1, quatro waves atrás. As duas em
> [`BUGS_physics.md`](BUGS_physics.md), com as 9 lições.
>
> **Aguardo a sua ordem explícita de integração** (CLAUDE.md §0.7). Eu não integro nem pusho.
