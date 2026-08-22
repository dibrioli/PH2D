# Inventário PH2D — o que JÁ EXISTE como "componente de objeto de cena"

> Levantamento somente-leitura, 2026-08-20. Fontes: grep `derive(Component)`, CLAUDE.md §5,
> leitura direta de `ph2d-physics-ecs`, `ph2d-platformer`, `ph2d-render/sprite`, `ph2d-runtime`,
> `ph2d-timeline`/`ph2d-anim`, `ph2d-audio(-edit)`, `ph2d-script`, `shells/desktop/{project,undo}.rs`,
> `ph2d-panel-inspector`.

## 0. Números brutos (`derive(Component)` por crate)

| Crate | # Components | Observação |
|---|---|---|
| `crates/ph2d-ecs` | **62** | núcleo: pose, nome, sorting, masking, visibilidade + toda a família `Vec*` (path vetorial como entidade) |
| `crates/ph2d-physics-ecs` | **32** | config de física (nunca estado vivo do solver) |
| `crates/ph2d-render` | 2 | `Sprite` (sim) + `RenderInstance` (interno do present-world) |
| `crates/ph2d-timeline` | 1 | `SpriteAnimation` (Clip → Transform) |
| `crates/ph2d-script` | 1 | `LuauScript` |
| `shells/desktop` | 1 | `Velocity` (demo local, não promovido) |

**Registrados no `ComponentRegistry`** (= persistem no save E entram no undo): 57 do `ph2d-ecs`
+ `Sprite` + `LuauScript` (asserts de 58 nos gates das crates render/script) + **32** da física
(`register_physics_components`, gate `registers_every_physics_component`). Total ≈ **91 tipos
persistíveis**. Component que NÃO registra é **descartado em silêncio** pelo snapshot (o
comentário no registry lista os que já se perderam assim: `Locked`, `GroupedChildren`, `VecPathRef`).

---

## (a) Tabela: componente/recurso → crate → o que faz → exposto na UI?

### Núcleo de cena (`ph2d-ecs`)

| Componente | O que já faz | UI? |
|---|---|---|
| `Transform` / `GlobalTransform` | pose local (metros, Y-up), propagação hierárquica | ✅ Inspector §transform + gizmo |
| `Name` (+ `stable_name_id`) | rótulo humano; hash do Name é a REFERÊNCIA DURÁVEL entre objetos (sobrevive ao undo/load) | ✅ Inspector §identity + Hierarchy |
| `RootOrder` | ordem explícita de raízes (anti-empate do undo) | ✅ implícito (Hierarchy) |
| `Visibility`, `VisibilityLayer`, `OnScreenEnabler` | visível/oculto, máscara de camadas (cull por câmera), ativação on-screen | ✅ Inspector §visibility |
| `SortingLayer`, `OrderInLayer`, `ZIndexOverride`, `ZAsRelative`, `YSort`, `SortingGroup`, `ShowBehindParent`, `TopLevel` | sorting completo estilo Unity/Godot (Sprite Inspector v2 W3) | ✅ Inspector §ordering |
| `ClipChildren`, `Mask2D`, `MaskInteraction` | clipping/máscara 2D hierárquica | ✅ Inspector |
| `TextureFilter`, `TextureRepeat`, `UvTransform` | amostragem por-objeto (nearest/linear, repeat, UV offset/scale — scroll de fundo) | ✅ Inspector §sampling |
| `BlendMode` | blend por-objeto | ✅ Inspector §material_blend |
| `Locked`, `GroupedChildren` | trava de edição, group-lock | ✅ Hierarchy |
| `VecPathRef`, `FlipObjectRef`, `PaintedDoc`, `BakedForm` | as 4 PONTES de identidade: path vetorial ↔ entidade, objeto Flip ↔ entidade, documento do Painter ↔ sprite, canais assados 3D ↔ sprite | ✅ via ferramentas dos módulos |
| Família `Vec*` (~30: `VecShape`, `VecConnector`, `VecBlend`, `VecOffset`, `VecSymmetry`, `VecCutPath`, `VecBoolGroup`, `VecStrokeProfile`, `VecMorph`, `VecFrame`, `VecLayout*` (taffy), `VecWidget`/`VecWidgetBind`/`VecWidgetValue`/`VecWidgetIcon`, `VecTextPath`, `VecPatternPath`, `VecFilter`, `VecEnvelope`, `VecAnchors`, `VecLabel`, `VecResizeBox`…) | todo o motor vetorial vivo COMO COMPONENTES: formas paramétricas, booleana viva, blend/morph, simetria, auto-layout, e a UI AUTORADA (widget skin + estados) | ✅ tool Vector + painéis próprios |
| `SimRef` | back-reference present→sim (ponte p/ renderer/inspector/áudio espacial) | interno |

### Física (`ph2d-physics-ecs`) — config, nunca estado do solver; determinismo com hash 3-OS

| Componente | O que já faz | UI? |
|---|---|---|
| `RigidBody`, `Collider` | corpo + colisor (restitution/friction; `is_sensor` aguarda consumidor) | ✅ Inspector §physics_body |
| `GravityScale`, `InitialVelocity`, `Ccd`, `LockRotation`, `LockPositionX/Y`, `MassOverride`, `Dominance`, `DampingOverride`, `MaterialCombine` | o pacote completo de overrides estilo Unity/Godot | ✅ Inspector rows |
| `PhysicsJoint` (**joint = ENTIDADE**, kinds: Pin, Spring, Rope, Weld, Slider, Rod, Wheel, Pulley, Custom) + `PulleyWheel`, `WestonAxle`, `RopeStops`, `JointWorldAnchor` | 7 tipos + polia/talha/tambor + pino de mundo; eixo do slider = rotação do Transform da entidade-joint (autorável no Inspector sem widget novo); motores, limites, break | ✅ Inspector §joint (cards, custom, pair rows) + tool |
| Zonas: `AreaEffector`, `AreaDrag`, `AreaBuoyancy`, `AreaFormDrag`, `AreaTorque`, `AreaFalloff`, `AreaForceWorldAxes` | família de campos de força/arrasto/empuxo/torque com falloff — "effector areas" completas | ✅ Inspector |
| `OneWayPlatform`, `WalkSurface`, `NoWallCling` | plataforma one-way, material de superfície p/ o player, anti-cling | ✅ Inspector |
| `SignalOnHit(String)`, `SignalOnLeave(String)` | contato → publica `Signal` nomeado no outbox (event-sourced) | ✅ Inspector |
| `PlatformPlayer` + `PlayerMode` (Dynamic/Kinematic/Pure) + `PlayerSignals` | **player de plataforma completo por COMPONENTE**: cápsula flutuante (mola na perna), 3 modos | ✅ Inspector §14 (19 números + 3 botões, `PLAYER_ROW_COUNT` gateado) |
| Recursos: `PhysicsSettings` (painel de mundo, tecla `W`), `TapeWire` (corrida gravada → bake em curvas), ring GGPO de checkpoints, camadas de colisão | mundo, replay, rewind | ✅ painel Physics |

### Player de plataforma (`ph2d-platformer`) — a LEI pura (sem rapier, sem ECS)

`Motor {accel, boost}` + módulos: walk, jump (coyote/buffer/plataforma que levanta), dash, crouch,
ledge, wall (slide/grab/launch), swim, glide, slope/footing, corner-escape, ride (mola), react
(empurrão), kinematic, sense, event (`PlayerEvent` — eventos entre estados). Consumido pela ponte
`bridge/player*` da física. É o exemplo canônico de "gameplay pronto sem programar".

### Render (`ph2d-render`)

| Recurso | O que já faz | UI? |
|---|---|---|
| `Sprite` (v4) | source (Atlas/Individual/CookedTexture KTX2), size, **tint que CASCATEIA** + `self_tint` + per-corner gradient + `tint_fill` (flash de dano Phaser) + `opacity`, flip_x/y, pivot (`anchor`/`centered`/`offset`), **sprite-sheet inline** (`hframes×vframes×frame`), region rect | ✅ Inspector (color_tint, render_source, sprite_sheet) |
| `Camera2d` | câmera ortográfica 2D com zoom + `cull_mask` por `VisibilityLayer` — mas é RESOURCE do renderer/editor, **não componente de cena** | viewport do editor |
| `GameRt` | render-target offscreen do MUNDO (padrão SubViewport/RenderTexture), HDR `Rgba16Float` + tonemap AgX | interno |
| FX stack (`fx_stack*`, `motion_fx`) | efeitos pós por camada/motion (glow, drop-shadow, rgb-split como NODES) | ✅ via Motion Nodes |

### Animação (`ph2d-timeline` + `ph2d-anim`)

| Recurso | O que já faz | UI? |
|---|---|---|
| `SpriteAnimation` (Component) | um `Clip` amostrado no Playhead escrevendo `Transform` (X/Y/rot/scale) | via Timeline |
| `TimelineDoc` + `TargetBinding` (**`WireId` = hash do Name**, religado no upkeep do frame) | dope-sheet, curvas bézier weighted, clips + composição + **nesting**, motion path (posição = UM canal 2D), onion, retiming, extrapolação, autokey, **markers que emitem `Signal`** (ADR-0143), expressões (motor vivo; autoria retirada) | ✅ painel Timeline completo |
| `ph2d-anim` | `Clip`/`Track`/curvas/easing/extrapolação/rove — target-agnóstico (`AnimTarget` opaco) | — |
| `ph2d-ui-state` (StateSets no undo!) | **Smart Animate**: estados de cena (idle/hover/press), tween automático via `ph2d-vec-blend` + OKLab; `Machine` sem relógio próprio | ✅ estados de UI no editor |

### Sinais / runtime (`ph2d-runtime`)

`Signal`/`SignalOrigin`/`SignalOutbox`/`SignalReader` — timeline (marker) e física (`SignalOnHit`)
PUBLICAM; cada consumidor lê com cursor próprio (produtor não chama ninguém, ADR-0075). Ordem no
quadro gateada. Medido: 8 consumidores custam 1,00× o de 2. **Falta o R3: a tabela nome→ação
(conteúdo autorado + UI)** — hoje o consumidor visível é toast/log no shell.

### Script (`ph2d-script`)

`LuauScript { bytecode: AssetId, lateral_key }` — Component registrado/persistível; bytecode
compartilhado entre instâncias; estado por-instância na `StateTable` (chave determinística —
sobrevive a hot-reload). `ScriptHost` (mlua/Luau): reads/writes por entidade (`ph2d.set/get`),
`ph2d.input` (snapshot do `ph2d-input`), `spawn_named` + `SpawnQueue`, nomes→entidades, gc_step
por frame. `MessageBus` com interning de nomes + handlers. **Wired no boot com script
placeholder; NÃO há UI nenhuma** (sem editor de script, sem seção no Inspector, sem attach visual).

### Áudio (`ph2d-audio*`)

Rack com 42 efeitos + 23 presets, espectral, export Ogg/Opus, streaming de vozes, AI denoise —
**tudo DOCUMENTO do editor (EditClip), nada é componente de cena**. Não existe `AudioSource`
espacial nem gatilho som↔sinal (é exatamente o R3 pendente). `SimRef` já prevê "audio spatial"
no doc-comment.

### Input (`ph2d-input`)

Gamepad press/held/released/axis, snapshot por tick, projetado no Luau. **Sem action mapping**
(nada de "Jump = A ou Espaço" configurável), pencil stub.

### Grid (`ph2d-grid`)

11 tipos de grid + snap math + **A\* determinístico pronto** (custo uniforme; BTreeMap, HR-5) —
"any gameplay code can share". Hoje só o editor (grid-snap) consome.

### Outros

- `ph2d-net`: **vazio** (lockstep+rollback planejado, M13 por demanda).
- `ph2d-light`: rig de lâmpadas ÚNICO (Painter relevo + 3D relight). Iluminação 2D de jogo: **ADIADA (decisão do dono, 2026-08-20)** — listar, não priorizar.
- `PrefabDoc`/`SceneDoc` (`ph2d-asset` + `ph2d-ecs::scene::spawn`): prefab cozido (bundle de components + filhos, postcard versionado, `ComponentTypeId` = blake3 do nome). **Sem UI de prefab** (não há "salvar seleção como prefab" nem instanciar do browser).
- `ph2d-mesh`/`ph2d-sculpt3d`: escultura doa normal → tinta 2D acesa pela forma (persistida em `sculpt` + `baked_forms` do ProjectFile).
- Partículas: existem como **Motion Nodes** (`motion.emitter`, forças, boids, clone, GPU-resident 4,19M @ 3,6ms) ligados a objetos via bake/bridge (`motion_object_bake`, `PH2D_MOTION_OBJ_SMOKE`) — não como componente "ParticleSystem" no objeto.

---

## (b) Lacunas óbvias contra uma engine de jogo completa

1. **Câmera de GAMEPLAY como componente** — `Camera2d` é resource do editor; não há camera-follow, limites, deadzone, shake, zoom por área, múltiplas câmeras autoráveis na cena.
2. **Play mode / shell de jogo (R1)** — adiado por decisão do Enio; sem ele "o jogo" só roda dentro do editor. É o teto de quase todas as outras lacunas.
3. **Tabela sinal→ação (R3)** — `Signal` já viaja (contato, marker de timeline), mas NADA autorável reage: tocar som, trocar cena, spawnar, incrementar contador. É a peça que transforma sinais em gameplay sem código.
4. **Áudio na cena** — sem `AudioSource`/listener espacial, sem "toca este clip quando este sinal chega". O rack de 42 efeitos é todo editor-side.
5. **Tilemap** — nada no runtime (docs/Tilling é MVP gitignored fora do PH2D). `ph2d-grid` já dá a matemática (11 grids + A*), falta o componente tilemap + pintura + colisão por tile.
6. **Input action mapping** — só gamepad cru; sem mapa de ações nomeadas/rebinding/multi-device (teclado de gameplay nem entra no snapshot do script hoje).
7. **UI de script / attach visual do `LuauScript`** — o componente e o host existem e persistem, mas não há como um artista ANEXAR um script, ver campos expostos, ou editar código no editor. (E os componentes que "reduzem a necessidade de programação" — FSM visual, spawner, timer, contador, health — não existem.)
8. **Save de JOGO (player-facing)** — `ProjectFile` é save de AUTORIA; não há slot/save-state de runtime (o ring GGPO + TapeWire são a semente técnica).
9. **Spawner/factory + UI de prefab** — `PrefabDoc`+`spawn_prefab`+`SpawnQueue` do Luau existem; falta o componente Spawner (onda, taxa, pool) e o fluxo de prefab no editor (salvar/instanciar/override).
10. **Timer/tween/state-machine de gameplay** — easing e Smart Animate existem para UI/cena; não há componente Timer, Tween-to, contador, nem FSM de gameplay (o `ph2d-ui-state::Machine` é o embrião).
11. **Navegação de gameplay** — A* pronto em `ph2d-grid` mas nenhum componente NavAgent/obstáculo o consome.
12. **Rede** — `ph2d-net` vazio (planejado, por demanda).
13. **Texto dinâmico em runtime** — texto existe como vetor autorado (`VecTextPath`, fonte vetorial); sem componente "Label" que um script mude (placar, diálogo).
14. **Iluminação 2D de jogo** — **ADIADO por decisão do dono (2026-08-20)**; `ph2d-light` já centraliza o rig p/ relevo/3D.

## (c) O caminho que um componente NOVO percorre hoje

1. **Definir**: `#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]` + `impl SimComponent` (marca sim-world) na crate do módulo (drop-crate, ADR-0075). Config plain-data, determinística (BTreeMap, nunca HashMap na física); referência a outro objeto = `stable_name_id` (hash do `Name`), NUNCA `Entity::to_bits()` (bits morrem no undo-respawn e envenenam o diff).
2. **Registrar**: adicionar em `register_<crate>_components(reg)` com nome canônico `"ph2d::<crate>::<Tipo>"` → `ComponentTypeId = blake3(nome)[0..8]` (estável entre toolchains; manual de propósito — sem `inventory`, por wasm32 + grep-ability). O shell chama todos os `register_*` no boot. ⚠️ Gates de contagem: `registers_every_physics_component` (32) e os asserts 58 nas suítes de ecs/render/script — somar em TODOS.
3. **Persistir**: de graça a partir do registro — `world_to_snapshot` inclui todo componente registrado no `WorldSnapshot`, que é o `ProjectState` (undo) E o corpo do save (`ProjectFile`, postcard posicional). Mudou a FORMA de um componente já shipado ⇒ bump do `PROJECT_SCHEMA` (hoje **84**, `project_schema.rs`) com o degrau escrito na escada + a tripla em `project_schema_tests.rs` (3 sítios, nunca 1). Dado que vive FORA do undo global (cf. `motion`/`timeline`/`physics`/`sculpt`) = campo próprio do `ProjectFile`, undo próprio.
4. **Undo**: de graça também — `App::post_frame_undo` roda 1×/frame, diff do `ProjectState` contra baseline; snapshot passa por `canonicalize()` (ordena por CONTEÚDO). Requisito: o componente serializa determinístico (senão todo frame vira passo espúrio).
5. **UI**: NÃO é automático — escrever uma seção no `ph2d-panel-inspector` (`sections/*.rs` + populate/sync/event/paint, snapshots via thread-local setters; listas de rows exportadas p/ os gates de seam, ex. `PLAYER_ROW_COUNT`). Zero hex/f32/string hardcoded (tokens + i18n, HR-15).
6. **Smoke**: cena `PH2D_*_SMOKE` numerada lendo o roteador de cenas (número se CONTA), auto-play (feedback_ready_to_smoke_example).

**Resumo do custo**: passos 1-4 são baratos e mecânicos (o registro dá persistência+undo de graça); o custo real de "AddComponent estilo Unity" está no passo 5 — cada componente hoje exige seção artesanal no Inspector, e não existe UI genérica de "Add Component" nem derive de painel a partir do tipo.
