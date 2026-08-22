# ph2d-code-facts (confidence high)

## decisive_facts
- O PH2D nunca avança o change tick do bevy (0 chamadas a clear_trackers/increment_change_tick; sem schedules): hoje todo componente tem added==changed==Tick(1). Change detection existe na memoria sem custo, mas so informa depois de um `world.clear_trackers()` por frame (ex.: fim de `post_frame_undo`).
- Instancia vetorial hoje = UMA entidade (retangulo-suporte + VecInstance) e geometria derivada por frame em LiveGeometry; as pecas do MESTRE sao entidades reais (sync cria uma por path). Aninhamento nao renderiza: cook_one le `cooked()` da cena, nunca o `live` (instance_live.rs:149-152).
- Detach materializa so GEOMETRIA (push_path + sync cunha entidades com Transform default/Name/VecPathRef/RootOrder) e perde todos os componentes parametricos da peca; nao cria entidades ele proprio. HierDuplicate copia so Transform+Sprite+Name+ChildOf, sem filhos. Nao existe copia profunda de sub-arvore — o substrato (extract_component_snapshot + insert_from_bytes) existe e tem zero consumidores.
- Ponte de fisica e por QUERY a cada dispatch (Entity,&RigidBody,&Collider,&Transform), pose de MUNDO via cadeia ChildOf, mapa `bodies: BTreeMap<Entity,_>` de runtime, sem persistencia: pecas materializadas com RigidBody funcionam sem tocar a ponte. O que quebra e toda referencia por NOME (PhysicsJoint.body_a/b, PulleyWheel.rope/body, WireId da timeline): copias recebem nome sufixado e o joint copiado continua apontando para o mestre.
- stable_name_id tem DUAS familias de consumidores de producao (nao tres — VecLabel.host e VecPathId): timeline (binding.rs:25 WireId, timeline_persist.rs:38, frame_solve.rs:139/218/251, persist.rs:28/54/88) e fisica (joint.rs:117-121, rope.rs:99/146/405, bridge/joints.rs:152/315, rope.rs:130, joint_group.rs:139 + 12 sitios de inspector na shell); ~26 linhas de producao em 13 arquivos, 431 no total contando testes/smokes/c9.
- A ordem de linha do WorldSnapshot e assumida em exatamente 3 lugares: save.rs:134-188 (DFS por to_bits + parent como INDICE), save.rs:219-224 (restore por indice) e undo.rs:152-180 (canonicalize sort-by-bytes + remap do parent). state_hash (save.rs:69) e o PartialEq do baseline (undo.rs:404) dependem dessa ordem. So entidades With<Transform> entram no snapshot/restore.
- Nao existe bench nenhum de capture_project/world_to_snapshot/canonicalize no repo; os 6,89 ms vem do spike apagado. capture_project e inalcancavel headless e canonicalize e privado na shell — o bench tem de mirar world_to_snapshot+canonicalize dentro de crates/ph2d-ecs.
- Mudar `parent: Option<u32>` para StableId (ou qualquer forma nova de linha) e bump de WorldSnapshot::VERSION=1 e de PROJECT_SCHEMA=84, e o load recusa qualquer versao diferente (project_load.rs:46) — nao ha migracao escrita.

## findings

## Fatos de código para a hipótese "instância MATERIALIZADA + link por StableId + sync por change-tick + undo incremental"

Leitura 100% read-only em `/home/enio/Documentos/Projetos/PH2D` (HEAD `ee1432203`) e no `bevy_ecs 0.18.1` vendorizado em `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src` (MIT/Apache — leitura de código permitida). Tags: [CODE] lido da fonte · [INF] inferência minha · [DOC] doc oficial. Nenhuma fonte GPL/proprietária foi consultada.

---

### (1) Como uma `VecInstance` é renderizada HOJE — e o que o Detach faz

**Todo path É uma entidade (ADR-0110), inclusive cada peça do mestre.** [CODE]
- `shells/desktop/src/vec_entities.rs:36-101` — `sync()` mantém UMA entidade por `VecPath`, nas duas direções; path novo ⇒ `spawn((Transform::default(), Name::new(unique), VecPathRef(id), RootOrder(order)))` (linhas 95-101). O mapa é `VecEntityMap = BTreeMap<VecPathId, u64 /*Entity bits*/>` (`:21`), runtime-only, reconstruído por `rebuild_map` (`:120-127`) varrendo `VecPathRef` após undo/load.
- O **mestre** é um path comum com o marcador `VecComponentMain` (`crates/ph2d-ecs/src/vec_component.rs:50`); as **peças** são a sub-árvore ECS dele: `instance_live::subtree_paths` (`shells/desktop/src/instance_live.rs:224-241`) percorre `scene.paths()` (ordem z) e mantém quem é `main_e` ou descendente via `ChildOf` (`is_descendant_of`, `:244-256`, teto `MAX_DEPTH = 64`).

**A instância tem UMA entidade, zero filhos; a geometria é derivada por frame.** [CODE]
- `place_instance` (`shells/desktop/src/vec_component_edit.rs:191-207`) só faz `scene.push_path(rectangle(lo, hi))` — o "suporte" (bbox do conteúdo do mestre, `main_content_box` `:224-265`). A entidade nasce no `sync` do mesmo frame; `arm_instance` (`:268-287`) insere `(Transform deslocado, VecInstance::new(main))`.
- `VecInstance { main: u64, overrides: Vec<InstanceOverride{ sub: u64 /*VecPathId da peça NO MESTRE*/, slot: OverrideSlot }> }` com `OverrideSlot::{Fill([u8;4]), Hidden}`; lista canônica por `(sub, slot.kind())` via `binary_search` em `VecInstance::set` (`vec_component.rs:94-161`). Registrados no `ComponentRegistry` (`crates/ph2d-ecs/src/scene/registry.rs:380-381`).
- **Produtor:** `InstanceLive::recook` (`instance_live.rs:82-110`) roda 1×/frame DEPOIS do `sync` (`shells/desktop/src/render_loop/mod.rs:7451-7452`); `cook_one` (`:122-167`) para cada peça visível clona `src.cooked()`, assa `xform_of(piece)` e depois `place_delta` (`:190-194` — remove só a TRANSLAÇÃO do mestre, preserva linear), aplica `Fill` override, e grava em `LiveGeometry = BTreeMap<VecPathId, Vec<VecPath>>` (`crates/ph2d-vec-render/src/lib.rs:128`) sob o id DA INSTÂNCIA.
- **Consumo:** `vec_render::dispatch` (`lib.rs:190-247`): para cada path, se `live.get(&path.id)` existe desenha os itens derivados (já em MUNDO, só câmera) **em vez** da geometria própria; senão desenha o path. Fusão dos produtores em `render_loop/mod.rs:7457-7488` (offset → pattern → contour → symmetry → profile → instance).
- Mestre apagado/sem marcador ⇒ `cook_one` devolve `None` ⇒ vai para `orphans`, e o suporte (retângulo) é desenhado (`:30-40`, `:107`).

**⚠️ Aninhamento NÃO funciona hoje no modelo derivado.** [CODE→INF] `cook_one` lê `scene.paths().find(|p| p.id == piece)` e usa `src.cooked()` (`instance_live.rs:149-152`) — nunca consulta `self.live`. Se uma peça do mestre for ela própria uma instância, o que entra na cópia externa é o **retângulo-suporte** dela, não a geometria derivada. A única recusa explícita é `main_id == at` (`:133-135`). Não há gate para o caso aninhado (não determinado se alguém já mediu).

**Detach (`vec_component_edit.rs:322-381`) materializa GEOMETRIA, não componentes.** [CODE]
- Recebe `drawn` (o que o produtor desenhou no frame anterior) + `pieces` (ids das peças do mestre na mesma ordem, `InstanceLive::pieces_of`, `render_loop/mod.rs:5400-5407`).
- Escolhe a raiz por IDENTIDADE (`src.iter().position(|p| *p == main_id)`), sobrescreve o path da instância com a peça-raiz assada por `I⁻¹` (`:342-356`), e para as demais faz `scene.push_path(g)` (`:358-367`) — **não cria entidades**: elas nascem no `sync` seguinte com `(Transform::default, Name único, VecPathRef, RootOrder)`. `arm_detached` (`:417-429`) insere `ChildOf` conforme `plan.parents`, derivado da árvore do MESTRE por `master_parent_path` (`:385-407`).
- Remove `VecInstance` (`:377-379`). **Consequência:** as peças destacadas perdem TODO componente paramétrico que a peça do mestre tinha (`VecShape`, `VecFilter`, `VecLayout`, `RigidBody`…) — só a curva cozida viaja. Isto é o oposto do que a hipótese materializada precisa (onde "detach" = remover os componentes de link, trivial).

**Verbos que existem e são reutilizáveis como UI:** Create/Place/Detach/Reset/UpdateMain/Swap/PieceVisible/Variant (`vec_component_edit.rs:33-49`); `vec_component_pieces.rs` (override por peça, `swap_main`), `vec_variants.rs` (variants derivados dos mestres irmãos), `vec_instance_follow.rs` (a cópia segue a âncora do mestre em resize — lei `ΔTi = ΔTm·I_lin`, aplicada na MUDANÇA, não derivável do estado). Cap de UI `MAX_INSTANCE_PIECES = 16` (`vec_component_pieces.rs:~48`).

**"Duplicate" hoje é RASO.** [CODE] `render_loop/hierarchy.rs:171-238`: para path vetorial, `duplicate_vec_paths` (`input_dispatch.rs:357-385`) copia/cola só geometria (`copy_paths/paste_clip`) e o `sync` cunha entidades novas com componentes default; para entidade não-vetorial copia **apenas `Transform`, `Sprite`, `Name`(único) e `ChildOf`** — sem filhos, sem os outros 87 componentes registrados. Uma cópia profunda de sub-árvore com todos os componentes não existe; o substrato (`extract_component_snapshot` + `insert_from_bytes`, `crates/ph2d-ecs/src/scene/snapshot.rs:245-274`, `registry.rs:156-163`) existe e tem zero consumidores de produção.

---

### (2) Ponte de física — como um corpo rapier nasce de uma entidade

**"Entidade tem `RigidBody+Collider+Transform` ⇒ corpo existe", por QUERY, a cada dispatch.** [CODE]
- `BodyQuery = QueryState<(Entity, &RigidBody, &Collider, &Transform)>` (`crates/ph2d-physics-ecs/src/bridge.rs:84-89`), cacheada; `prepare()` (`bridge/dispatch.rs:38-73`) roda em TODO dispatch: `reconcile_structure → reconcile_parts → reconcile_surfaces → reconcile_joints → restamp_damping → settle_static`.
- `reconcile_structure` (`bridge/bodies.rs:17-222`): itera a query, pose de **MUNDO** via `space::world_transform` → `ph2d_ecs::world_transform_into` (`crates/ph2d-ecs/src/transform_inverse.rs:85-95`, compõe a cadeia `ChildOf`); dobra os ~20 componentes opcionais num `BodyDesc`; `None` no mapa ⇒ `to_spawn`; em repouso `rest != desc` ⇒ respawn (`:169-172`); entidade que sumiu da query ⇒ `to_remove` (`:181-185`, O(N²) sobre `seen`). Spawn ordenado por `to_bits` (`:206`). **Qualquer mudança estrutural limpa o ring de checkpoints** (`:191-193`).
- Peças compostas: `PartQuery = (Entity, &Collider, &Transform) Without<RigidBody>` (`bridge.rs:115-118`) — collider sem corpo pendura no ancestral-corpo mais próximo (`owner_body`, atravessa grupos).

**Mapas de identidade.** [CODE]
- `bodies: BTreeMap<Entity, BodyRef{handle, kind, rest: BodyDesc}>` (`bridge.rs:157`) — chave = `Entity` **de runtime**; `joints: BTreeMap<Entity, JointRef>` (`:167`), e `JointRef.entities: (Entity, Option<Entity>)` (usado em `joint_respawn.rs:39`, `fk.rs:262`, `ik.rs:360`, `ik_lead.rs:209/227`).
- `names: BTreeMap<u64 /*stable_name_id*/, Entity>` (`bridge.rs:200`) reconstruído a cada `reconcile_joints` a partir das chaves de `self.bodies` (`bridge/joints.rs:147-154`) — só corpos NOMEADOS são alcançáveis por joint.
- Nada é persistido (ADR-0131 D2): `PhysicsBridge::rebuild()` (`bridge.rs:565-580`) é chamado só no load (`shells/desktop/src/project_load.rs:123`); no undo o reconcile se auto-cura (bits velhos somem da query ⇒ removidos; novos ⇒ spawnados).
- Nenhum componente de física guarda bits de entidade (grep em `components/*.rs`+`joint.rs`: `Entity` só em argumentos de função). Referências duráveis são TODAS `stable_name_id`: `PhysicsJoint.body_a/body_b: u64` (`joint.rs:117-121`), `PulleyWheel.rope: u64` (`components/rope.rs:99`), `PulleyWheel.body: u64` (`:146`).

**O que quebra se as peças da instância forem entidades reais com `RigidBody`:** [INF sobre CODE]
- **Nada na metade CORPO** — a query as vê, a pose compõe pela cadeia `ChildOf`, peças/superfícies/zonas seguem a mesma regra. Custo: +N corpos por instância no reconcile O(N) + O(N²) de `seen`, e o ring limpa ao instanciar.
- **Quebra a metade por NOME:** um `PhysicsJoint` copiado para dentro da instância continua com `body_a = stable_name_id("Hook")` = a peça do **MESTRE**, porque a cópia recebe nome único sufixado (`name_unique.rs:29-40`; `vec_entities.rs:93`: "Hook (1)"). O joint da cópia ligaria os corpos do mestre (ou ficaria dormente). O mesmo vale para `PulleyWheel.rope/.body` e para a `WireId` da timeline. ⇒ referências **dentro** de uma sub-árvore têm de ser relativas à peça (`StableId` + remap na instanciação), exatamente o `{master_root, master_piece}` da hipótese.
- Os 32 componentes de física são registrados (`crates/ph2d-physics-ecs/src/lib.rs:146-179`) ⇒ cada peça materializada paga no snapshot do undo (fato §0.1 do doc 04).

---

### (3) Change detection no `bevy_ecs 0.18.1` vendorizado — assinaturas exatas

Raiz: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/`. [CODE]

| API | Assinatura | Arquivo:linha |
|---|---|---|
| `EntityRef::get_change_ticks` | `pub fn get_change_ticks<T: Component>(&self) -> Option<ComponentTicks>` | `world/entity_access/entity_ref.rs:129` |
| `EntityRef::get_change_ticks_by_id` | `pub fn get_change_ticks_by_id(&self, component_id: ComponentId) -> Option<ComponentTicks>` | `entity_ref.rs:141` (idem em `entity_mut.rs:393/406`, `world_mut.rs:726/741`) |
| `EntityRef::spawn_tick` | `pub fn spawn_tick(&self) -> Tick` | `entity_ref.rs:286` |
| `ComponentTicks` | `pub struct ComponentTicks { pub added: Tick, pub changed: Tick }` | `change_detection/tick.rs:137-143` |
| `ComponentTicks::is_changed` | `pub fn is_changed(&self, last_run: Tick, this_run: Tick) -> bool` | `tick.rs:156` |
| `ComponentTicks::is_added` | `pub fn is_added(&self, last_run: Tick, this_run: Tick) -> bool` | `tick.rs:149` |
| `Tick` | `pub struct Tick { tick: u32 }`; `Tick::new(u32)`, `.get() -> u32`, `is_newer_than(self, last_run, this_run) -> bool` (clamp `MAX_CHANGE_AGE`), `Tick::MAX`, `check_tick` | `tick.rs:18-80` |
| `World::change_tick` | `pub fn change_tick(&mut self) -> Tick` | `world/mod.rs:3011` |
| `World::read_change_tick` | `pub fn read_change_tick(&self) -> Tick` (atômico) | `world/mod.rs:3001` |
| `World::last_change_tick` | `pub fn last_change_tick(&self) -> Tick` | `world/mod.rs:3023` |
| `World::increment_change_tick` | `pub fn increment_change_tick(&mut self) -> Tick` (devolve o anterior) | `world/mod.rs:2989` |
| `World::clear_trackers` | `pub fn clear_trackers(&mut self)` = `removed_components.update(); last_change_tick = increment_change_tick()` | `world/mod.rs:1599-1602` |
| `World::check_change_ticks` | `pub fn check_change_ticks(&mut self) -> Option<CheckChangeTicks>` (no-op abaixo de `CHECK_TICK_THRESHOLD`) | `world/mod.rs:3153` |
| `World::removed` | `pub fn removed<T: Component>(&self) -> impl Iterator<Item = Entity> + '_` — desde o último `clear_trackers` | `world/mod.rs:1775` |
| `World::removed_with_id` | `pub fn removed_with_id(&self, component_id: ComponentId) -> impl Iterator<Item = Entity> + '_` | `world/mod.rs:1785` |
| `World::removed_components` | `pub fn removed_components(&self) -> &RemovedComponentMessages`; `.iter() -> (&ComponentId, &Messages<RemovedComponentEntity>)` | `world/mod.rs:281`, `lifecycle.rs:436-462` |
| `Entities::entity_get_spawn_or_despawn_tick` | `pub fn entity_get_spawn_or_despawn_tick(&self, entity: Entity) -> Option<Tick>` — válido para entidade morta enquanto o índice não for reutilizado (checa `generation`, `:1077-1085`) | `entity/mod.rs:1068` |

- **Despawn "desde o tick" sem sistema:** não há lista de despawnados; o que existe é `removed_components` (double-buffer, swap em `clear_trackers`; leitura por `iter_current_update_messages`, `message/messages.rs:265`). Um `despawn` emite remoção para cada componente que a entidade tinha ⇒ `world.removed::<Transform>()` enumera despawnadas (+ as que perderam só `Transform`) — **sem `SystemParam`** (`RemovedComponents<T>` em `lifecycle.rs:508` é só o wrapper de sistema). Inserção de bundle carimba `added/changed` com `world.change_tick()` (`bundle/insert.rs:31-125,370`; `world/mod.rs:1082,1146`); `Mut<T>` carimba `changed = this_run` no `DerefMut` **mesmo sem alterar o valor** [DOC/CODE].
- `Query<Entity>` sem filtro casa TODO arquétipo, inclusive o vazio (`query/fetch.rs:42-46`, `matches_component_set → true`) ⇒ dá para enumerar todas as entidades vivas sem sistema.

**⚠️ FATO DECISIVO: o PH2D nunca avança o change tick.** [CODE] `grep -rn "clear_trackers|increment_change_tick|last_change_tick|change_tick()|read_change_tick|get_change_ticks|Changed<|Added<|RemovedComponents|.removed::<"` em `crates/` + `shells/` ⇒ **0 linhas**; o PH2D não roda `Schedule`/sistemas do bevy (só API direta de `World`). Dentro do bevy o tick só avança em `clear_trackers`, execução de sistemas (`system/function_system.rs:436,661`, `exclusive_function_system.rs:138`) e `commands/mod.rs:2879`. `World::new()` parte de `change_tick = 1`, `last_change_tick = 0` (`world/mod.rs:122-125`). **Logo hoje TODO componente do `SimWorld` tem `added == changed == Tick(1)`**: os ticks existem na memória sem custo, mas carregam zero informação até o shell chamar `sim.world_mut().clear_trackers()` (ou `increment_change_tick()`) uma vez por frame — o lugar natural é o fim de `post_frame_undo` (`shells/desktop/src/undo.rs:363-435`). Custo: um incremento + swap dos buffers de remoção. `check_change_ticks()` deve ser chamado no mesmo ponto (só trabalha a cada `CHECK_TICK_THRESHOLD` ticks).

---

### (4) Onde o código assume "ordem de linha = DFS" ou "ordenado por conteúdo"

`crates/ph2d-ecs/src/scene/save.rs` [CODE]:
- `:46` doc-comment "Entities in stable DFS order (roots first, then children)".
- `:134-157` DFS: raízes = `TransformPropagationState.roots` = `QueryState<Entity, (With<Transform>, Without<ChildOf>)>` (`crates/ph2d-ecs/src/transform.rs:475,484`) ordenadas por `to_bits` (`:137`); filhos na ordem de inserção de `Children` (`:149-156`). ⇒ **entidade sem `Transform` nunca entra no snapshot** (nem no undo, nem no save).
- `:161-164` `index_of: BTreeMap<Entity, u32>` = posição na `visit_order`.
- `:171-181` componentes em ordem de `type_id` (BTreeMap do registro, `registry.rs:205-209`).
- `:184-188` `row.parent = index_of[ChildOf.0]` — **índice de linha**.
- `:201-226` `snapshot_to_world`: passo 1 spawna linha a linha (bits ascendentes na ordem das linhas), passo 2 `entities[p as usize]` — depende do índice, **não** de DFS (pais são resolvidos após todos spawnarem). Por isso `state_hash_is_deterministic_across_round_trip` (`:313-340`) passa sem `canonicalize`: spawn em ordem de linha reproduz a mesma DFS por `to_bits`.
- `:69-72` `state_hash` = blake3 do postcard do snapshot inteiro ⇒ dependente da ordem; é o hash de replay/3-OS.

`shells/desktop/src/undo.rs` [CODE]:
- `:139-180` `canonicalize`: chave = concatenação `(type_id LE, bytes)` de cada componente; `sort_by` sobre a chave; `row.parent` remapeado por `new_index`. Empate entre linhas byte-idênticas fica arbitrário — e como o `parent` dos filhos aponta para UMA das gêmeas, duas entidades idênticas com filhos diferentes não são de facto intercambiáveis [INF]; hoje raro porque `Name` é único no editor (`name_unique.rs`), mas entidades sem `Name` podem existir.
- `:74-78` captura = `world_to_snapshot` + `canonicalize` a cada frame com input; `:404` `undo_baseline == current` é `PartialEq` do `ProjectState` inteiro (ordem do `Vec` importa); `:109-117` restore despawna `With<Transform>` e respawna tudo (bits novos ⇒ `rebuild_map` `:120-121`).

Outros consumidores do índice/ordem: `crates/ph2d-ecs/src/scene/spawn.rs:78-102` (`spawn_prefab` recursivo, filhos após pai) e `:110-135` (`spawn_scene`: `relations` por índice em `instances`) — código morto em produção. `crates/ph2d-ecs/src/scene/snapshot.rs:149-203` (`HierarchySnapshot`, DFS por `(RootOrder, to_bits)`) é estrutura **separada**, só para o painel. Grep por `.entities`/`row.parent` fora desses: **zero** outros sítios. Testes que dependem do round-trip: `save.rs:265-352`, `crates/ph2d-ecs/tests/end_to_end_m14.rs:92-106`, `crates/ph2d-physics-ecs/tests/persistence.rs:104-107`, `joint_persistence.rs:57,107,173`, `shells/desktop/src/project_tests.rs`, `shells/desktop/tests/the_ui_state_machines_run_and_undo_waits.rs:89` (gate por grep do texto `capture_project`).

Versões: `WorldSnapshot::VERSION = 1` (`save.rs:53`, nunca bumpado); `PROJECT_SCHEMA = 84`, load **recusa** `!=` (`shells/desktop/src/project_load.rs:46`). Trocar `parent: Option<u32>` por `parent: StableId` é bump dos dois.

---

### (5) Consumidores de `stable_name_id` — escopo exato da migração para `StableId`

Definição: `crates/ph2d-ecs/src/name.rs:80-89` (FNV-1a, `0` reservado ⇒ `1`); re-export `lib.rs:72`. O doc-comment `:73-79` já prescreve "`StableId` real atribuído no spawn, migrado para os dois consumidores de uma vez". Total no repo: **431 linhas em ~75 arquivos**, das quais **produção ≈ 26 linhas em 13 arquivos**; o resto é teste/smoke/bin (constroem joints por nome — fixtures a migrar ou a manter por helper).

**Timeline (WireId):** [CODE]
- `crates/ph2d-timeline/src/binding.rs:25` `pub struct WireId(pub u64)`; `:41-57` `TargetBinding { target: AnimTarget, prop: PropKind, wire_id: WireId /*serializado*/, #[serde(skip)] entity: u64, #[serde(skip)] missing: bool, … }`.
- `shells/desktop/src/timeline_persist.rs:38` `wire_id_for_name → WireId(stable_name_id(name))` (único escritor do hash); `crates/ph2d-timeline/src/persist.rs:28` `stamp_wire_ids`, `:54` `resolve_entities`, `:88` `refresh_and_heal_bindings` (heal→purge por frame).
- `crates/ph2d-timeline/src/frame_solve.rs:139` (`resolve_link`, prop-links `Name.prop` das expressões), `:218` (`build_names`, TODAS as entidades, desempate por menor bits), `:251` (`build_names_bound`).

**Física (joints/roldanas):** [CODE]
- `crates/ph2d-physics-ecs/src/joint.rs:117-121` `PhysicsJoint.body_a/body_b: u64`; `components/rope.rs:99` `PulleyWheel.rope`, `:146` `.body`, `:405` `rope_joint_of`; `bridge/joints.rs:152` (mapa `names`), `:315` (corda pelo nome da entidade-joint); `bridge/rope.rs:130`; `joint_group.rs:139`.
- Shell (produção): `render_loop/inspector_joint.rs:136,266`; `inspector_joint_create.rs:87,169-170,193`; `inspector_joint_world.rs:83`; `inspector_joint_wheel.rs:24,57,127,140,316,362`; `joint_rig.rs:200`.
- Bin do hash 3-OS: `crates/ph2d-physics-ecs/src/bin/physics_ecs_c9/{rigs.rs:66-204, joints.rs:54-105, main.rs:466-598}`.

**Correção ao doc 01 §1.3:** `VecLabel.host` é um `VecPathId` cru (`crates/ph2d-ecs/src/vec_label.rs:35`), **não** `stable_name_id` — há DUAS famílias consumidoras (timeline, física), não três.

---

### (6) Benches de `capture_project` / `world_to_snapshot`

**Não existe nenhum.** [CODE] `[[bench]]` só em `crates/ph2d-input/Cargo.toml:21`, `crates/ph2d-render/Cargo.toml:93` e `tests/spike/Cargo.toml:90` (`c2_query.rs` = Luau vs Rust, criterion 0.7). `scripts/` não menciona captura/snapshot. A única sonda de relógio vizinha é `crates/ph2d-ecs/tests/nesting_sorts_as_a_block.rs:246` (`#[ignore]`, custo do `SortingGroup`, não da captura). Os 6,89 ms do doc 01 §7 vêm do spike apagado. ⚠️ `capture_project` é inalcançável headless (`gfx == None ⇒ None`, `shells/desktop/src/project_tests.rs:216`) e `canonicalize` é privado na shell ⇒ o bench tem de mirar `world_to_snapshot + canonicalize` em `crates/ph2d-ecs` (mover `canonicalize` para lá).

---

### O que isto diz sobre a hipótese [INF]

1. **Materializar é compatível com a física e com a hierarquia sem tocar a ponte**; o que tem de mudar é o esquema de referência (nome → `StableId` relativo à peça) para joints/roldanas/timeline, e isso é exatamente o escopo da §5.
2. **O sync por change-tick é viável, mas o tick tem de passar a andar** (uma linha por frame) e o passe tem de tolerar `changed` espúrio (`Mut` marca no `DerefMut`) — comparar bytes ou aceitar recozimento; e as escritas do próprio sync nas instâncias também marcam `changed`, o que o undo incremental deve incluir (é estado real) mas o sync não deve reconsumir (laço mestre↔override).
3. **Undo incremental:** com `StableId` como componente registrado, a chave de ordenação vira `StableId` e o `parent` vira `StableId` ⇒ `canonicalize` (2,08 ms) desaparece; a serialização por linha (4,81 ms) só cai se for feita só para entidades com `changed > last_capture` + despawnadas via `removed::<Transform>()` — e exige bump de `WorldSnapshot::VERSION`/`PROJECT_SCHEMA`.
4. **Aninhamento hoje não renderiza** no modelo derivado; no materializado é recursão de spawn + ordem topológica do sync (mestre antes de variant antes de instância).
5. **"Duplicate" real não existe** — precisa nascer sobre `extract_component_snapshot`/`insert_from_bytes` com remap de `StableId` e de referências internas; o `Detach` atual é geometria-only e não serve de molde.

## sources
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/vec_component.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/instance_live.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/vec_component_edit.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/vec_component_pieces.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/vec_instance_follow.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/vec_entities.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/render_loop/mod.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/render_loop/hierarchy.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/input_dispatch.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-vec-render/src/lib.rs
- /home/enio/Documentos/Projetos/PH2D/docs/architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/bridge.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/bridge/bodies.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/bridge/dispatch.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/bridge/joints.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/bridge/rope.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/bridge/space.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/joint.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/joint_group.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/components/rope.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-physics-ecs/src/lib.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/transform_inverse.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/transform.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/scene/save.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/scene/registry.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/scene/snapshot.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/scene/spawn.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/name.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/src/vec_label.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/undo.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/name_unique.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/timeline_persist.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/project_load.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/project_save.rs
- /home/enio/Documentos/Projetos/PH2D/shells/desktop/src/project_tests.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-timeline/src/binding.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-timeline/src/persist.rs
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-timeline/src/frame_solve.rs
- /home/enio/Documentos/Projetos/PH2D/tests/spike/benches/c2_query.rs
- /home/enio/Documentos/Projetos/PH2D/tests/spike/Cargo.toml
- /home/enio/Documentos/Projetos/PH2D/crates/ph2d-ecs/tests/nesting_sorts_as_a_block.rs
- /home/enio/Documentos/Projetos/PH2D/docs/Components/01_auditoria_modelo_de_objeto.md
- /home/enio/Documentos/Projetos/PH2D/docs/Components/04_decisao_arquitetura.md
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/world/mod.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/world/entity_access/entity_ref.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/world/entity_access/entity_mut.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/world/entity_access/world_mut.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/change_detection/tick.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/change_detection/mod.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/lifecycle.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/entity/mod.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/bundle/insert.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/query/fetch.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/message/messages.rs
- /home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/system/function_system.rs
