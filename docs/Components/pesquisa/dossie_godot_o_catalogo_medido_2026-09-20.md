# O catálogo da Godot, MEDIDO — o que ela tem, e o que disso vira objecto ou componente do PH2D

> **Encomenda do dono (2026-09-20):** *«Vá até o manual da godot. Descubra tudo que ela tem e que
> pode se transformar em um objeto ou em um componente de nossa engine. Faça essa pesquisa
> minuciosa. e documente para analisarmos depois»* — com duas capturas do diálogo *Create New Node*.
>
> ⛔ **Isto é um LEVANTAMENTO, não um plano.** Nenhuma linha aqui é uma decisão, uma prioridade ou
> uma ordem de trabalho. A coluna «veredito» diz *o que existe hoje do nosso lado*, medido; o que se
> constrói e em que ordem é do dono.

---

## §0 — Porque este documento existe ao lado do [`dossie_godot.md`](dossie_godot.md)

O dossiê de 2026-08-20 foi **LIDO** em `docs.godotengine.org`. Ele acerta nos nós famosos, e há uma
pergunta que uma leitura não pode responder: ***o que é que ela TEM?*** — porque uma leitura não
enumera, ela **recorda**. A diferença está medida:

| | dossiê de 20/08 | este |
|---|---|---|
| origem | páginas da documentação | `ClassDB` do **binário instalado** |
| classes citadas | **~60** | **1 054** |
| propriedades | as que a página destacava | **todas**, com tipo e dica |
| envelhece? | sim, em silêncio | não — **re-corre-se** |

É a lei §0.9 do `CLAUDE.md`: *«quando outro app JÁ FAZ o que estamos a construir, ele é um ORÁCULO
que se CORRE — nunca um fonte que se lê»*. O diálogo das capturas do dono é desenhado pelo editor a
partir desta mesma base de dados; perguntar-lhe directamente é ler a fonte da captura, não a
lembrança dela.

---

## §1 — Triagem de licença, instrumento e reprodução

**Passo 1 é sempre a licença, e ela pára na primeira porta ABERTA.** Medido no artefacto instalado,
nunca no nome do projecto:

```
$ pacman -Qi godot | grep -E 'Versão|Licenças'
Versão   : 4.7.2-1.1
Licenças : MIT
```

⇒ **porta aberta**: lê-se, corre-se e porta-se com atribuição. Não há clean-room aqui (o arsenal
`docs/_ComoInvestigarApps/01` já a lista como uma das três permissivas desta máquina).

**O instrumento é versionado** — [`godot_classdb_probe.gd`](../ferramentas/godot_classdb_probe.gd),
irmão das oito sondas que esta linha já correu:

```
godot --headless --script docs/Components/ferramentas/godot_classdb_probe.gd --quit -- <saída>
```

Ela escreve dois ficheiros: **`censo.tsv`** (uma linha por classe: pai · instanciável · família ·
nº de propriedades/sinais/métodos **declarados por ela**) e **`superficie.txt`** (cada propriedade
com tipo e dica, os grupos do inspector e os sinais).

⚠️ **`no_inheritance = true` é load-bearing**: sem isso todo nó 2D herdaria as ~30 propriedades do
`CanvasItem` e a contagem mediria a herança em vez da classe.

⛔ **O que a sonda NÃO colhe, declarado:** a **prosa**. Uma segunda corrida (`godot --headless
--doctool <dir>`) produz 1 076 XML e as descrições vêm **vazias** — o binário regenera a estrutura e
funde-a com a documentação do *source tree*, que não existe aqui. ⇒ as frases «o que é» deste
documento são **minhas**; as propriedades, contagens e sinais são **medidos**. A separação está
marcada em cada tabela.

---

## §2 — O funil, com os números

| passo | classes |
|---|---|
| `ClassDB` do binário instalado | **1 054** |
| − `Editor*` e afins (plugins, docas, inspector do editor) | −61 |
| − `OpenXR*` / `XR*` / `WebXR` | −83 |
| − `GLTF*` (importação 3D) | −22 |
| − `RD*` (`RenderingDevice`, o wrapper de Vulkan) | −20 |
| − `VisualShaderNode*` | −111 |
| − família `Node3D` | −105 |
| − outros `*3D` (formas, servidores, consultas) | −45 |
| − rede / multiplayer / sockets | −38 |
| − Java / JavaScript / plataforma | −6 |
| − `ResourceImporter*` | −17 |
| **no alcance de um editor 2D** | **546** (destas, **471 instanciáveis**) |

Dessas 546: **51 Node2D · 63 Control · 24 Node · 242 Resource · 96 RefCounted · 69 Object**.

⚠️ **O corte dos 111 `VisualShaderNode` é um AGRUPAMENTO, não um descarte**: eles são *um* assunto
— um grafo de shader — e a resposta do PH2D a esse assunto é o **nodegraph** que já temos. Contá-los
um a um inflaria o levantamento em 11 % sem acrescentar uma decisão.

⚠️ **O corte do 3D também é do CONTEXTO e não do valor**: o 3D do PH2D é *escultura e modelação*,
não uma cena de jogo 3D. `Node3D`, `Shape3D`, `VoxelGI` e companhia não têm consumidor deste lado.

---

## §3 — `Node2D`: as 51, uma a uma

É esta a lista das capturas do dono. Legenda dos vereditos:

> ✅ **TEMOS** (o nome do nosso componente/crate está na coluna) · ◑ **METADE** (existe noutra forma,
> e a metade que falta está nomeada) · ⭐ **CANDIDATO** (não existe e cabe no modelo) · ⛔ **FORA**
> (com o motivo)

### §3.1 — Desenho

| classe (props/sinais medidos) | o que é | PH2D hoje | |
|---|---|---|---|
| **`Sprite2D`** (12p/2s) | textura + `centered`/`offset`/`flip_h`/`flip_v` + grade `hframes`·`vframes`·`frame` + recorte de atlas `region_*` | `Sprite` · `SpriteRegion` · `SpriteGrid` · `SpriteSheetRef` · `SpritePixels` · `SpriteEmissive` · `SpriteCornerTint` | ✅ |
| **`AnimatedSprite2D`** (11p/5s) | animações nomeadas num recurso `SpriteFrames`; sinais `animation_finished`/`frame_changed`/`animation_looped` | `SpriteAnimations` + `SpriteAnimator` (§11 do Sprite Inspector), com `signal_on_finish`/`signal_on_loop` | ✅ |
| **`Line2D`** (14p) | polilinha espessa: `width_curve` · `gradient` · `texture_mode` · `joint_mode` · caps · `sharp_limit` · `closed` | o **motor** é nosso e é maior (`VecStrokeProfile`, `ph2d-stroke-width`, `ph2d-arclen`, caps/juntas do vector) | ◑ falta o **componente de RASTO**: um que apende a posição do objecto por quadro (hoje isso é o nó `motion.trail`, do grafo, não um componente) |
| **`Polygon2D`** (15p) | polígono com `uv`, `vertex_colors`, `polygons` (buracos), `internal_vertex_count` e **`skeleton`** (pesos por osso) | `ph2d-poly2d` + `Skin` + a 2.ª mídia do esqueleto (a imagem obedece aos ossos) | ✅ |
| **`MeshInstance2D`** (2p) | desenha um `Mesh` arbitrário em 2D | temos malha 2D (`ph2d-poly2d`) e desenho vectorial; **não** como componente autorável | ◑ |
| **`MultiMeshInstance2D`** (2p) | N cópias num draw call | a instanciação em massa é o **cozimento do Motion** (4,19 M objectos medidos no dispositivo) | ◑ por outra arquitectura |
| **`CanvasGroup`** (3p) | desenha os filhos como **UM** objecto e só então aplica alfa/blend — cura a interseção escura de sprites translúcidos | `ClipChildren`/`GroupedChildren`/`SortingGroup` são vizinhos e respondem a **outra** pergunta | ⭐ o *fade de personagem multi-parte* |
| **`CanvasModulate`** (1p) | tinge **todo** o canvas com uma cor (dia/noite) | nenhum tint global | ⭐ barato; e é a base do esquema de luz 2D, que está **adiado** |
| **`BackBufferCopy`** (2p) | copia uma região do ecrã para um buffer que um shader lê (`copy_mode`: Disabled/Rect/Viewport) | os nossos FX (`fx.glow`, `fx.rgb_split`, `fx.drop_shadow`) são nós do cozimento, não leitura de ecrã | ⭐ infra de distorção/refracção |
| **`Node2D`** (4p) | `position` · `rotation` · `scale` · **`skew`** | `Transform` — e com **`skew_x` E `skew_y`** (ele tem **um** só) | ✅ (à frente) |

### §3.2 — Câmera, paralaxe e visibilidade

| classe | o que é | PH2D hoje | |
|---|---|---|---|
| **`Camera2D`** (27p) | `zoom`/`offset`/`anchor_mode`, suavização de posição e rotação, **zona morta** por 4 margens, **limites** de mundo com `limit_smoothed`, gizmos de editor | `GameCamera` + `CameraFollow` + `CameraLimits` — **portados deste alvo**, paridade medida a `0,000061 px` em 45 fixturas | ✅ |
| **`Parallax2D`** (10p) | `scroll_scale` · `repeat_size`/`repeat_times` · `autoscroll` · `limit_begin/end` · `follow_viewport` | o **multiplano 2.5D** existe no módulo Flip; como componente de cena, não | ◑ |
| **`ParallaxLayer`** (3p) | `motion_scale` · `motion_offset` · `motion_mirroring` | idem | ◑ |
| **`VisibleOnScreenNotifier2D`** (2p/2s) | `rect` + sinais `screen_entered`/`screen_exited` | `OnScreenEnabler` | ✅ |
| **`VisibleOnScreenEnabler2D`** (2p) | liga/desliga um nó conforme ele está no ecrã (`enable_mode`) | `OnScreenEnabler` + `DestroyOutside` | ✅ |
| **`Marker2D`** (1p) | um vazio com gizmo (ponto de referência) | objecto vazio + `Name` + `Tags` (e um ponto marcado por tag é o que a `Factory` já consome) | ✅ |
| **`RemoteTransform2D`** (5p) | **empurra** a própria pose para outro nó (`update_position/rotation/scale`, `use_global_coordinates`) | `AnchorMount` faz o inverso (o filho monta-se numa âncora do pai) | ◑ falta o sentido «eu escrevo em ti» |

### §3.3 — Física

| classe | o que é | PH2D hoje | |
|---|---|---|---|
| **`CollisionObject2D`** (ABST, 5p/5s) | camadas/máscara, `collision_priority`, **`input_pickable`** + `mouse_entered`/`mouse_exited`/`input_event` | camadas de colisão ✅; **o rato sobre um corpo como sinal** não existe | ◑ ⭐ |
| **`StaticBody2D`** (3p) | corpo imóvel + **`constant_linear_velocity`** (esteira rolante sem se mover) | `RigidBody{Static}` ✅; a **esteira** é um campo que não temos | ◑ |
| **`AnimatableBody2D`** (1p) | corpo movido por animação que **estima a própria velocidade** e empurra os outros; `sync_to_physics` | `RigidBody{Kinematic}` + `WalkSurface` + `OneWayPlatform` + `PlatformPlayer` | ◑ a pergunta aberta é se a plataforma **carrega** quem vai em cima |
| **`RigidBody2D`** (23p/5s) | massa, inércia, centro de massa, `gravity_scale`, `freeze_mode`, `custom_integrator`, `contact_monitor`, amortecimentos, forças constantes | `RigidBody` + `MassOverride` · `GravityScale` · `DampingOverride` · `Ccd` · `LockRotation` · `LockPositionX/Y` · `Dominance` · `InitialVelocity` · `MaterialCombine` · `SignalOnHit` | ✅ (mais fino que ele) |
| **`CharacterBody2D`** (13p) | `motion_mode` (Grounded/Floating), `floor_max_angle`, `floor_snap_length`, `platform_on_leave`, `wall_min_slide_angle`, `safe_margin` | `PlatformPlayer` (55 campos) + `TopDownPlayer` + `PlayerMode` + `ph2d-platformer` + `ph2d-topdown` + `ph2d-sweep` | ✅ **muito à frente** — ele dá o alicerce, nós shipamos o controlador |
| **`Area2D`** (19p/8s) | sensor + **overrides locais** de gravidade (ponto/direcção, `gravity_point_unit_distance`), amortecimentos, e **`audio_bus_override`** | `AreaEffector` · `AreaBuoyancy` · `AreaDrag` · `AreaFormDrag` · `AreaTorque` · `AreaFalloff` · `AreaForceWorldAxes` · `SignalOnHit` · `SignalOnLeave` · `SignalTagFilter` | ✅ (mais rica) — ⭐ **menos a zona de áudio** (reverb por região) |
| **`CollisionShape2D`** (6p) | `shape` + `disabled` + **`one_way_collision`** (+ margem e direcção) | `Collider` + `OneWayPlatform` | ✅ |
| **`CollisionPolygon2D`** (6p) | polígono desenhado no viewport, `build_mode` (sólido/segmentos) | **não temos colisor de polígono** (ver §6.1) | ⭐ |
| **`RayCast2D`** (7p) | raio com `collide_with_areas/bodies`, `hit_from_inside`, `exclude_parent` | `RaySensor` + `RaySignals` | ✅ |
| **`ShapeCast2D`** (9p) | varre uma **forma** (não um raio) e devolve **`max_results`** acertos | temos varredura de forma no motor (`ph2d-sweep`) e **não** como sensor autorável | ⭐ |
| **`Joint2D`** (ABST, 4p) | `node_a`/`node_b` · `bias` · `disable_collision` | `PhysicsJoint` + `JointWorldAnchor` | ✅ |
| **`PinJoint2D`** (6p) | pino com **limite angular** e **motor** (`motor_target_velocity`) | `PhysicsJoint` (7 tipos + polia/talha/tambor + pino de mundo) | ✅ |
| **`GrooveJoint2D`** (2p) | corre numa calha (`length`, `initial_offset`) | idem | ✅ |
| **`DampedSpringJoint2D`** (4p) | mola com `rest_length`/`stiffness`/`damping` | idem + `ph2d-spring` | ✅ |
| **`PhysicsBody2D`** (ABST, 0p) | a raiz dos corpos | — | — |

### §3.4 — Partículas, áudio e som

| classe | o que é | PH2D hoje | |
|---|---|---|---|
| **`GPUParticles2D`** (26p/1s) | simulação na placa; **`sub_emitter`**, **`trail_enabled`**/`trail_sections`, `collision_base_size`, `amount_ratio`, `interp_to_end`, `use_fixed_seed`, `explosiveness`, `preprocess`, `visibility_rect` | `ParticleEmitter` (compila para nós reais do Motion; `sim.spawn`/`sim.zone`/`motion.emitter`/`motion.trail` existem) | ✅ — ⭐ comparar depois **`sub_emitter`** e o **`preprocess`** (pré-aquecer) |
| **`CPUParticles2D`** (87p!) | o mesmo modelo com **tudo inline** no nó (rampas de cor/escala, orbit, acelerações radial/tangencial) | idem | ✅ |
| **`AudioStreamPlayer2D`** (14p/1s) | som posicional: `max_distance`, `attenuation`, `panning_strength`, `max_polyphony`, **`bus`**, `area_mask` | `AudioSource2D` | ✅ |
| **`AudioListener2D`** (0p) | o ouvido | `AudioListener2D` (mesmo nome) | ✅ |

### §3.5 — Esqueleto

| classe | o que é | PH2D hoje | |
|---|---|---|---|
| **`Skeleton2D`** (0p/1s) | a raiz da hierarquia de ossos | `ph2d-skeleton` + `ph2d-skeleton-ecs` + o painel próprio | ✅ |
| **`Bone2D`** (1p) | um osso com **`rest`** (pose de repouso) | `Bone` + `BoneRest` + `BoneLimit` + `SmartBone` | ✅ |
| **`PhysicalBone2D`** (5p) | liga um osso a um corpo (`simulate_physics`, `auto_configure_joint`, `follow_bone_when_simulating`) — o **ragdoll** | temos ragdoll pela física + juntas; **não** como componente que casa osso↔corpo | ◑ ⭐ |

### §3.6 — Caminhos

| classe | o que é | PH2D hoje | |
|---|---|---|---|
| **`Path2D`** (1p) | um `Curve2D` desenhado na cena | o caminho é uma **forma do documento** (vector) | ✅ por outra forma |
| **`PathFollow2D`** (7p) | anda no caminho por `progress`/`progress_ratio`, com `h_offset`/`v_offset`, `rotates`, `cubic_interp`, `loop` | `PathFollow` (plano 20) + `ph2d-curve` + `ph2d-arc-length` | ✅ |

### §3.7 — Luz 2D — ⛔ **ADIADO por decisão do dono (2026-08-20)**

`Light2D` (ABST, 15p) · `PointLight2D` (4p) · `DirectionalLight2D` (2p) · `LightOccluder2D` (3p).
A superfície está medida e fica registada para o dia em que a decisão mudar: energia, `blend_mode`,
faixas de `z`/camada, `shadow_filter`/`shadow_filter_smooth`, `range_item_cull_mask`, e o oclusor com
`sdf_collision`. ⚠️ O `ph2d-light` **existe** e é o rig de lâmpadas do Painter e do 3D — é **outra**
luz (relevo por-pixel), não a luz-de-cena com sombras projectadas.

### §3.8 — Navegação — ⭐⭐ **a maior ausência medida**

| classe | o que é |
|---|---|
| **`NavigationRegion2D`** (6p/2s) | a área navegável (`navigation_polygon`, `navigation_layers`, `enter_cost`, `travel_cost`) |
| **`NavigationObstacle2D`** (6p) | obstáculo dinâmico: `radius`, `vertices`, `carve_navigation_mesh`, `avoidance_*` |
| **`NavigationLink2D`** (7p) | atalho entre regiões (salto, escada, teleporte), `bidirectional` |
| **`NavigationAgent2D`** (28p/6s — família `Node`) | A* + **evasão recíproca** (RVO): `neighbor_distance`, `max_neighbors`, `time_horizon_agents`, `avoidance_priority`, `path_postprocessing`, `simplify_path`; sinais `target_reached`, `navigation_finished`, `velocity_computed` |

**Medido do nosso lado:** não há crate `ph2d-nav*`, nem `NavigationAgent`, nem malha de navegação.
⚠️ Uma busca ingénua **acusa três falsos positivos** e leria como «existe»: `fn a_star` é uma
**estrela** (`ph2d-app-field3d/shapes_make.rs`), `a_started_zone` é *«a started zone»*, e
`ph2d-vec-boolean/src/pathfinder.rs` percorre **caminhos vectoriais** numa booleana — nenhum é
navegação de IA. *Uma ausência tem de ser medida com controlo, como uma presença.*

### §3.9 — Fora por decisão

| classe | motivo |
|---|---|
| **`TileMap`** (6p) · **`TileMapLayer`** (12p/1s) | ⛔ **ordem do dono (16/09)**: o tilemap é o projecto `docs/Tilling`, MVP fora daqui |
| **`TouchScreenButton`** (9p/2s) | ⭐ entrada táctil (`bitmask`, `passby_press`, `visibility_mode`, `action`) — **não** é «fora», é candidato sem consumidor hoje |

---

## §4 — Família `Node`: o que não é 2D e ainda assim é do jogo

Das 35, **11 são do editor** (docas, diálogos, `EditorPlugin`) e ficam fora. As que restam:

| classe (props/sinais) | o que é | PH2D hoje | |
|---|---|---|---|
| **`Timer`** (5p/1s) | `wait_time` · `one_shot` · `autostart` · `process_callback` · **`ignore_time_scale`** · sinal `timeout` | `Timers` (o TOP-20 #2) | ✅ — ⭐ menos o `ignore_time_scale` |
| **`CanvasLayer`** (8p/1s) | uma camada com transform próprio que **não segue a câmera** (`layer`, `follow_viewport_enabled`) | `UiCanvas` responde ao caso do HUD; uma camada genérica de mundo, não | ◑ |
| **`AnimationPlayer`** (8p/2s) | keyframes de **qualquer** propriedade + trilhas de método/áudio; `playback_default_blend_time`, **`playback_auto_capture`** | o módulo **Timeline** inteiro (curvas, clips, nesting, expressões, onion, retiming) | ✅ **muito à frente** |
| **`AnimationTree`** (3p) + `AnimationMixer` (10p/7s) | a máquina de estados **de animação**: `root_motion_track`, `deterministic`, `callback_mode_*` | `StateMachine` (o cérebro de **gameplay**) + os nós de Motion | ◑ — a árvore de **mistura** (ver §6.4) não existe |
| **`AudioStreamPlayer`** (10p/1s) | som não-posicional, `mix_target`, `bus`, `max_polyphony` | a rack + o mixer (`ph2d-panel-audio-mixer`) | ✅ |
| **`NavigationAgent2D`** (28p/6s) | ver §3.8 | — | ⭐⭐ |
| **`ParallaxBackground`** (6p) | o pai dos `ParallaxLayer`, com limites de scroll | ver §3.2 | ◑ |
| **`SubViewport`** (6p) | render-to-texture (`render_target_update_mode`, `size_2d_override`) | `ph2d-preview-slot`, `ph2d-thumbnail`, os previews | ◑ como capacidade; ⭐ como componente («desenha esta cena nesta textura») |
| **`WorldEnvironment`** (3p) | aponta um `Environment` (100 propriedades — §6.8) | `ph2d-bloom` e o modo Render do 3D | ◑ |
| **`ResourcePreloader`** (0p) | pré-carrega assets nomeados | `ph2d-asset-index` | ◑ |
| **`HTTPRequest`** (7p/1s) | pedido HTTP como nó | — | ⛔ sem rede |
| **`MultiplayerSpawner`** (2p/2s) · **`MultiplayerSynchronizer`** (5p/3s) | replicação de spawns e de propriedades marcadas na UI | — | ⛔ sem rede (**gap estratégico nomeado**, não dívida) |
| `Window` · `Popup*` · `FileDialog` · `AcceptDialog` · `StatusIndicator` | janelas e diálogos | a nossa moldura é própria | ⛔ |

---

## §5 — Família `Control` (63 no alcance): a UI

O PH2D tem **duas** UIs e elas não se confundem:

1. **A do editor** — `ph2d-editor-core` + **27** painéis. Medido, o `InteractiveState` oferece
   **10** espécies de widget (`Button`, `Checkbox`, `Combobox`, `Dropdown`, `Radio`, `Slider`,
   `Tabs`, `Tag`, `TextInput`, `Toggle`) mais as fileiras ricas (número, cor, curva, gradiente,
   paleta, ficheiro, fonte).
2. **A do JOGO** — o HUD, que tem **quatro** componentes: `UiCanvas` · `UiLabel` · `UiButton` ·
   `Counter`.

⇒ **a lacuna é toda na segunda**, e o catálogo dele diz exactamente o quê:

| grupo | classes | o que falta ao HUD | |
|---|---|---|---|
| **barras** | `ProgressBar` · `TextureProgressBar` (radial/linear, `nine_patch_stretch`) | **a barra de vida** — o widget nº 1 de qualquer jogo | ⭐⭐ |
| **imagem** | `TextureRect` (`expand_mode`, `stretch_mode`) · `NinePatchRect` · `ColorRect` | a moldura 9-slice existe como `SliceNine` (de sprite); no HUD, não | ⭐ |
| **texto rico** | `RichTextLabel` (BBCode, efeitos animados via `RichTextEffect`) · `Label` (`autowrap`, `text_overrun_behavior`) | `UiLabel` é texto simples | ⭐ |
| **arrumação** | `HBoxContainer` · `VBoxContainer` · `GridContainer` · `MarginContainer` · `CenterContainer` · `AspectRatioContainer` · `FlowContainer` · `SplitContainer` · `ScrollContainer` | o HUD posiciona por **âncora** (`VecAnchors`/`Fit`), sem caixas que empilham | ⭐ (o `VecLayout`/`taffy` do vector é o motor que já existe) |
| **entrada** | `LineEdit` · `TextEdit` · `SpinBox` · `OptionButton` · `CheckBox` · `CheckButton` · `HSlider`/`VSlider` · `ItemList` · `Tree` · `TabContainer` | nada disto é alcançável **dentro do jogo** | ⭐ |
| **táctil** | **`VirtualJoystick`** (novo na 4.7) + `TouchScreenButton` | — | ⭐ |
| **vídeo** | `VideoStreamPlayer` | não temos leitor de vídeo | ⭐ |
| **tema** | `Theme` · `StyleBox*` (§6.7) | temos o design system (`ph2d-tokens`) para o editor | ◑ |

⚠️ **Nota de tradução que vale para esta secção inteira:** no Godot um `Control` é um **nó na
árvore**; no nosso modelo o HUD já é *entidade + componente* (`UiCanvas`/`UiLabel`/`UiButton`), logo
cada linha acima entra como **componente**, não como uma segunda árvore.

---

## §6 — `Resource` (242 no alcance): os DOCUMENTOS e as tabelas partilhadas

No vocabulário do dono, um `Resource` da Godot é quase sempre **um objecto/asset** do PH2D — uma
coisa que se guarda, se partilha entre objectos e se edita num painel próprio.

### §6.1 — `Shape2D` (8) — ⭐ a lacuna é concreta e pequena

| dele | nosso |
|---|---|
| `CircleShape2D` | `ColliderShape::Ball` ✅ |
| `RectangleShape2D` | `ColliderShape::Cuboid` ✅ |
| `CapsuleShape2D` | `ColliderShape::Capsule` ✅ |
| `ConvexPolygonShape2D` · `ConcavePolygonShape2D` | **ausente** ⭐ |
| `SegmentShape2D` · `SeparationRayShape2D` · `WorldBoundaryShape2D` | **ausentes** ⭐ |

⇒ **3 formas contra 8**, medido no `enum ColliderShape`. E o polígono é o que falta para colidir com
arte desenhada, que é o caso normal deste app.

### §6.2 — `Texture` (39) — o que é relevante

`AtlasTexture` ✅ (`SpriteRegion`) · `NoiseTexture2D` ✅ (`ph2d-fbm`, `motion.noise`) ·
`GradientTexture1D`/`2D` e `CurveTexture` ✅ (temos curva e gradiente como editores ricos) ·
`PortableCompressedTexture2D`/`CompressedTexture2D` ✅ (`ph2d-asset-ktx2`) · `AnimatedTexture` ◑ ·
**`ViewportTexture`** ⭐ (a saída de um viewport como textura) · **`CameraTexture`** + `CameraFeed`
⭐ (webcam) · **`CanvasTexture`** ⛔ (normal/specular por sprite — é do esquema de luz 2D, adiado) ·
`MeshTexture`, `DrawableTexture2D`, `ExternalTexture`, `DPITexture` — menores.

### §6.3 — `AudioEffect` (27) — ✅ **estamos à frente, com número**

A rack do PH2D tem **42** efeitos (medido na tabela de parâmetros de `ph2d-app-audio`: 38 no `enum
Effect` que preserva o comprimento, mais `Reverb`, `Delay`, `PingPong` e `Convolution`), contra os
**27** dele. E temos análise espectral (`ph2d-audio-spectral`) e *denoise* por ML (`ph2d-audio-ml`),
que ele não tem.

⭐ **O que ELE tem e nós não:** o `AudioBusLayout` — **barramentos** com envios e efeitos por
barramento (o `bus` é uma propriedade de todo player e de toda `Area2D`). Nós temos um mixer; o
*roteamento por barramento nomeado, autorado* é a pergunta aberta.

### §6.4 — `AnimationNode` (18) — ⭐ a árvore de MISTURA de animação

`AnimationNodeBlendTree` · `BlendSpace1D`/`2D` (mistura por um/dois eixos — *andar↔correr* por
velocidade) · `Blend2`/`Blend3`/`Add2`/`Add3`/`Sub2` · `OneShot` · `TimeScale` · `TimeSeek` ·
`Transition` · `StateMachine` (+ `Playback` e `Transition` como recursos).

O nosso `StateMachine` (TOP-20 #15) é o cérebro de **gameplay** — estados, setas, sinais. Isto aqui é
outra coisa: **misturar poses**. O motor de mistura existe do nosso lado (o Timeline compõe clips,
faz crossfade e tem *expressions*), e **não há a superfície** que o artista usa para dizer *«mistura
estas duas animações por este número»*. ⇒ ◑/⭐, e é a metade que o §4 nomeia no `AnimationTree`.

### §6.5 — Curvas, gradientes e ruído — ✅

`Curve` (6p, com `min_domain`/`max_domain`) · `Curve2D` · `Gradient` (4p, `interpolation_color_space`)
· `Noise`/`FastNoiseLite`. Todos têm par nosso: `ph2d-curve`, os editores ricos de curva/gradiente/
paleta (`ph2d-param-editors`), `motion.color_ramp` (com interpolação **por stop**, que ele não tem),
`ph2d-fbm`.

### §6.6 — Tile — ⛔ fora por ordem do dono

`TileSet` (10p) · `TileSetAtlasSource` · `TileSetScenesCollectionSource` · `TileData` ·
`TileMapPattern`. A superfície fica registada (camadas de oclusão, física, terrenos, navegação e
**dados customizados por tile**) porque o projecto `docs/Tilling` existe; não é trabalho desta casa.

### §6.7 — Tema e texto

`Theme` (3p) · `StyleBoxFlat`/`Texture`/`Line`/`Empty` · `FontFile`/`FontVariation`/`SystemFont` ·
`LabelSettings` (12p: contorno, sombra, **`stacked_outline_count`**) · `ColorPalette` ·
`Translation`/`OptimizedTranslation` · `RichTextEffect`.

Nosso: `ph2d-tokens` (design system, zero hex por HR-15) · `ph2d-text` · `ph2d-system-fonts` ·
`ph2d-vector-font` · `ph2d-i18n` (5 386 chaves). ✅ para o editor; ⭐ para **temas do jogo**.

### §6.8 — Material e shader — ⭐⭐ a pergunta estratégica

| dele | nosso |
|---|---|
| **`ShaderMaterial`** + `Shader`/`VisualShader` + os 111 `VisualShaderNode` | **não existe**: medido, nenhuma crate aceita um shader autorado pelo utilizador |
| `CanvasItemMaterial` (blend mode, light mode, partículas) | `BlendMode` ✅ |
| `ParticleProcessMaterial` | o `ParticleEmitter` compila para nós do Motion ✅ |
| `Environment` (100p — glow, ajustes de cor, névoa, tonemap) | `ph2d-bloom` + o modo Render do 3D ◑ |

⚠️ *Um shader por objecto é a porta de escape que todo motor oferece.* Do nosso lado a resposta
natural **não é** um `ShaderMaterial`: é o **nodegraph** (o Motion já cozinha no dispositivo, com
`fx.*` reais). Registado como pergunta, não como dívida.

### §6.9 — `InputEvent` (16) — ✅ com duas faltas nomeadas

`InputEventKey` · `MouseButton` · `MouseMotion` · `JoypadButton` · `JoypadMotion` · `Action` ·
`ScreenTouch` · `ScreenDrag` · **`MagnifyGesture`** · **`PanGesture`** · **`MIDI`** · `Shortcut`.

Nosso: `ph2d-input` + o **Input Map** (acções nomeadas, `dead_zone` e `press_point` **separados** —
a falha pública do Godot que corrigimos). ⭐ faltam **gestos** (pinça/pan de trackpad como evento de
jogo) e **MIDI**.

### §6.10 — Os outros, em uma linha

`PackedScene` ✅ (as nossas instâncias/variantes, F4/F5) · `Animation`/`AnimationLibrary` ✅
(Timeline) · `SpriteFrames` ✅ · `PhysicsMaterial` ✅ (`MaterialCombine`) · `Shortcut` ✅ ·
`Image`/`BitMap` ✅ (`ph2d-imageio`, **16** formatos) · `JSON`/`ConfigFile` ✅ · `Skin` ✅ ·
`NavigationPolygon` + `PolygonPathFinder` + `NavigationMeshSourceGeometryData2D` ⭐ (navegação) ·
`OccluderPolygon2D` ⛔ (luz 2D) · `ButtonGroup` ⭐ (rádio no HUD) · `AudioStream*` — ver abaixo ·
`SceneReplicationConfig` ⛔ (rede) · `VideoStream`/`VideoStreamTheora` ⭐ ·
`SkeletonModification2D*` (8) — ⚠️ **o próprio Godot marca-os EXPERIMENTAIS**; o nosso esqueleto tem
IK com âncora, *Mix*, *Softness* e *Chain*, gateado ✅.

### §6.11 — ⭐⭐ `AudioStream` (10): a MÚSICA ADAPTATIVA

| classe | o que é |
|---|---|
| **`AudioStreamInteractive`** | clips com **transições autoradas** entre eles (o coração da música adaptativa) |
| **`AudioStreamPlaylist`** | lista com *crossfade* |
| **`AudioStreamRandomizer`** | escolhe um de N com variação de pitch/volume — mata o «passo repetido» |
| **`AudioStreamSynchronized`** | N faixas **em sincronia** (camadas de intensidade) |
| `AudioStreamPolyphonic` · `AudioStreamGenerator` · `AudioStreamMicrophone` | polifonia manual, síntese, microfone |

Nosso: `ph2d-audio-stream` (vozes em streaming, residência) + a rack + `ph2d-audio-edit` (que tem
`PickStrategy` para variação!). ⇒ ◑: as peças estão cá e **a autoria da música que reage ao jogo não
existe**. É, com a navegação e a barra de vida, uma das três ausências de maior relevo do documento.

---

## §7 — Servidores e singletons (`Object`/`RefCounted`): capacidades, não componentes

Nada disto vira componente; vira **o que um script pode fazer**.

| dele | nosso |
|---|---|
| `Input` · `InputMap` | ✅ `ph2d-input` + Input Map |
| `Tween` + 6 `Tweener` (`PropertyTweener`, `MethodTweener`, `CallbackTweener`, `IntervalTweener`, `SubtweenTweener`, `AwaitTweener`) | ✅ `Tweens` + `ph2d-tween` — ⭐ e **o nosso é um COMPONENTE**, o dele só existe por código |
| `AStar2D` · `AStarGrid2D` (9p: `jumping_enabled`, `diagonal_mode`, heurísticas) | ⭐ navegação |
| `PhysicsRayQueryParameters2D` · `...Shape...` · `...Point...` · `PhysicsTestMotion*` · `KinematicCollision2D` | ◑ o motor faz; **um script não consulta a física** |
| `Geometry2D` (booleanas de polígono, convex hull, interseções) | ◑ temos `ph2d-vec-boolean`/`ph2d-poly2d` **internos**; não expostos ao script |
| `RandomNumberGenerator` | ✅ (splitmix64 determinista, renasce no rebobinar) |
| `Time` · `OS` · `Engine` · `Performance` | ◑ um relógio/perf visível ao script |
| `AudioServer` (barramentos) | ◑ ver §6.3 |
| `TranslationServer` | ✅ `ph2d-i18n` |
| `ResourceLoader`/`Saver`/**`ResourceUID`** | ✅ `ph2d-asset-index` + `ph2d-asset-id` (blake3) |
| `UndoRedo` | ✅ (fila única por diff, `ProjectState`) |
| **`AccessibilityServer`** (novo na 4.7) | ✅ `ph2d-a11y` — **paridade**, e poucos motores 2D a têm |
| `ThemeDB` · `TextServer` (4) | ✅ `ph2d-tokens`, `ph2d-text` (parley) |
| `FileAccess` · `DirAccess` · `Thread`/`Mutex`/`Semaphore`/`WorkerThreadPool` | ⛔ **de propósito**: o nosso Luau é uma caixa fechada |
| `MovieWriter` (4) | ⭐ gravar o ecrã para vídeo a partir do motor |
| `RenderingServer` · `RenderingDevice` · `PhysicsServer2D` · `NavigationServer2D` | ◑ arquitectura própria (wgpu, rapier) |

---

## §8 — O cruzamento: **157** componentes nossos contra o catálogo dele

Medido **correndo o catálogo** (`ph2d_component_desc::catalog::all()`, não uma varredura de texto):
**157** descritores, nas gavetas que a paleta mostra —

| gaveta | n | | gaveta | n | | gaveta | n | | gaveta | n |
|---|--:|---|---|--:|---|---|--:|---|---|--:|
| Physics | 37 | | Vector | 33 | | Logic | 16 | | Ordering | 11 |
| Model3D | 10 | | Rendering | 10 | | Image | 8 | | Identity | 7 |
| Skeleton | 7 | | Camera | 5 | | Instancing | 4 | | Anchors | 3 |
| Animation | 2 | | Audio | 2 | | Transform | 1 | | Scripting | 1 |

⚠️ **Contar isto por `grep` dá três respostas diferentes e todas erradas** — os `requires:` citam
nomes de outros componentes, e cada família usa um construtor próprio (`D::authored`,
`D::intrinsic`, e helpers de uma letra). *Uma contagem que se tira do texto de um catálogo mede a
sintaxe dele; a que se tira de o correr mede o catálogo.*

### §8.0 — A TABELA, tudo lado a lado

⭐ **Ela mudou-se para [`../25_tabela_comparativa_godot.md`](../25_tabela_comparativa_godot.md)**, por
ordem do dono (22/09), para ficar na raiz de `docs/Components/` e à mão antes de implementar.
⛔ **Não a copie de volta para aqui:** duas cópias de uma tabela divergem na primeira wave que
acrescentar um componente, e a que está errada é sempre a que alguém lê.

### §8.1 — O que ELE tem e NÓS não (a lista candidata, **sem ordem**)

> ⛔ Sem prioridade de propósito: ordenar isto seria decidir, e a decisão é do dono.

**Grandes (assunto próprio, com motor a construir)**
1. **Navegação** — região, agente com evasão, obstáculo, link, `AStarGrid2D`, `NavigationPolygon`. *Nenhuma peça existe.*
2. **Música adaptativa** — `AudioStreamInteractive`/`Playlist`/`Randomizer`/`Synchronized`.
3. **UI de jogo** — barra de progresso, texto rico, contentores, entrada, joystick virtual (§5).
4. **Árvore de mistura de animação** — `BlendSpace1D/2D`, `Blend2/3`, `OneShot` (§6.4).
5. **Rede** — `MultiplayerSpawner`/`Synchronizer`. ⛔ fora do produto hoje.
6. **Shader autorado** — `ShaderMaterial` (§6.8); ⚠️ a nossa resposta provável é o nodegraph.

**Médios (cabe numa wave)**
7. **Colisor de polígono** (§6.1) — e o segmento, a barreira de mundo.
8. **`ShapeCast`** como sensor autorável.
9. **Zona de áudio** (`Area2D.audio_bus_override`) — reverb por região.
10. **Paralaxe** como componente de cena (§3.2).
11. **Rasto** (`Line2D` que apende posições) — o motor é nosso e é melhor.
12. **Ragdoll declarado** (`PhysicalBone2D`: este osso É este corpo).
13. **`CanvasGroup`** — alfa de personagem multi-parte.
14. **`SubViewport`** como componente (desenha esta cena nesta textura).
15. **Vídeo** (`VideoStreamPlayer`).

**Pequenos (um campo, uma linha de painel)**
16. `CanvasModulate` — tint global de cena.
17. `StaticBody2D.constant_linear_velocity` — **esteira rolante**.
18. `Timer.ignore_time_scale`.
19. `RemoteTransform2D` — «eu escrevo a minha pose em ti».
20. `CollisionObject2D.input_pickable` + `mouse_entered/exited` — o rato sobre um corpo como sinal.
21. `TouchScreenButton` / `VirtualJoystick` — táctil.
22. Gestos (`MagnifyGesture`/`PanGesture`) e **MIDI** como eventos de entrada.
23. `ButtonGroup` — rádio no HUD.
24. `GPUParticles2D.preprocess` (pré-aquecer) e `sub_emitter`.

### §8.2 — O que NÓS temos e ELE não

Esta coluna é a que o levantamento existe para produzir, e ela é longa:

- **Controladores prontos**: `PlatformPlayer` (coyote, buffer, curvas de salto) e `TopDownPlayer`
  (8/4/livre, isometria por menu, deslize a velocidade cheia). Ele dá `CharacterBody2D` e o resto é
  script.
- **Abanão da vista** (`ShakeEmitter`/`CameraShake`) — **ele não tem**, e o dossiê de 20/08 já o
  registava como lacuna dele.
- **Fábrica + higiene** (`Factory`, `Lifetime`, `DestroyOutside`) — spawner como componente não
  existe lá.
- **A tabela de acções** (`SignalActions`, `SignalOnAction`, `SignalFrom`, `SignalTagFilter`) — o
  cabeamento *quando · de quem · a quem · o quê* sem uma linha de script. No Godot, ligar um sinal a
  uma reacção **precisa de um método**, logo de um script.
- **Máquina de estados de GAMEPLAY** (`StateMachine`) — lá só existe a de animação; FSM de jogo é
  addon (LimboAI/Beehave).
- **Tags em ÁRVORE** (`Tags`, com dobra ICU) — os `groups` dele são planos e sem UI própria.
- **Arma** (`WeaponFire`: cadência, pente, recarga) e **projéctil arcade** (`ProjectileMotion`).
- **`Counter`/`CounterWatch`**, **`SequencePlayer`**, **`Tweens` como componente**.
- **O grafo de Motion** (≈134 nós, cozimento no dispositivo, 4,19 M objectos a 3,85 ms) — sem par.
- **O motor vectorial** (14 modos, booleana viva, *live path effects*, auto-layout, SVG) — sem par.
- **O Painter** (aquarela, impasto, *wet paint*, sculpt do relevo, liquify) — sem par.
- **Escultura 3D + retopologia em quads** — sem par.
- **O Flip** (animação 2D à Grease Pencil) — sem par.
- **Rack de áudio de 42 efeitos** com espectral e *denoise* por ML — contra 27 sem editor.
- **Esqueleto** com IK ancorada, *smart bones* e a 2.ª mídia (imagem presa aos ossos) — contra um
  `SkeletonModification2D` que o próprio Godot marca **experimental**.
- **Timeline** com clips, *nesting*, retiming, onion, sinais e expressões.
- **Determinismo gateado** — `BTreeMap` por lei, hash de replay igual nos três sistemas operativos.
  O Godot não faz essa promessa.
- **`skew_x` E `skew_y`** (ele tem um `skew` só).
- **Física de zonas**: empuxo, arrasto de forma, torque, eixos de mundo — ele tem gravidade e
  amortecimento.
- **Acessibilidade** (`ph2d-a11y`) em paridade com a novidade da 4.7 dele.

---

## §9 — ⛔ Recusas e fronteiras já MEDIDAS (não as reabra sem ler)

| assunto | estado |
|---|---|
| **Tilemap** | ⛔ ordem do dono (16/09): é o projecto `docs/Tilling`, MVP fora daqui |
| **Luz 2D** (`PointLight2D`, `DirectionalLight2D`, `LightOccluder2D`, `CanvasTexture`, `OccluderPolygon2D`) | ⛔ **adiado pelo dono em 2026-08-20** |
| **Rede** (`Multiplayer*`, 38 classes) | ⛔ fora do produto; o `shells/game`/R1 está adiado |
| **Visual scripting** | ⛔ **o próprio Godot REMOVEU-O na 4.0** (artigo oficial). A lição já está no dossiê antigo e continua: investe-se em UI declarativa por componente e em grafos **de domínio**, não num «GDScript visual» |
| **`FileAccess`/`Thread` no script** | ⛔ de propósito: o Luau do PH2D é caixa fechada |
| **3D de cena** (`Node3D` e 150 vizinhos) | ⛔ o nosso 3D é escultura e modelação |

---

## §10 — O que ficou por medir (para quem continuar)

1. **A prosa do alvo** — as descrições oficiais não vêm no `--doctool` sem o *source tree*. Quem as
   quiser: clonar o repo dele (MIT) e correr `--doctool` contra ele, ou ler `doc/classes/*.xml`.
2. **Comportamento**, em qualquer célula deste documento. Aqui mediu-se **o que existe**, não **o que
   faz** — e esta casa já pagou a diferença oito vezes. Toda célula ⭐ que virar wave começa por uma
   sonda que **corre** o alvo, como as oito de [`ferramentas/`](../ferramentas/).
3. **A comparação de qualidade** onde os dois têm a coisa (ex.: as nossas partículas contra as dele
   com `sub_emitter` e `preprocess`; o nosso Input Map contra os gestos dele).
4. **Os 111 `VisualShaderNode`** — se a resposta ao §6.8 for o nodegraph, esta lista é o catálogo de
   comparação dos nós de shader.
5. **Os outros motores** — o `pesquisa/` desta pasta tem dossiês de Unity, Unreal, Bevy, Cocos/Phaser,
   Construct/GDevelop e GameMaker/Defold, todos **lidos**. Nenhum foi corrido como oráculo.

---

## Ficheiros medidos (reprodutíveis)

- sonda: [`docs/Components/ferramentas/godot_classdb_probe.gd`](../ferramentas/godot_classdb_probe.gd)
- corrida: `godot --headless --script <sonda> --quit -- <saída>` → `censo.tsv` + `superficie.txt`
- binário: Godot **4.7.2** (`arch_linux`, MIT), instalado em `/usr/bin/godot`
- o nosso lado: `crates/ph2d-component-desc/src/catalog/*.rs` (157 nomes) e `crates/` (374 crates)
