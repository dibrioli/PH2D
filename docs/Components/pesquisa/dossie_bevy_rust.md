# Dossiê — Bevy 0.15+ e o ecossistema Rust de ECS: o catálogo de componentes-como-dados

> Pesquisa: 2026-08-20 · Fontes: docs oficiais (bevy.org, docs.rs) e READMEs oficiais das crates.
> Contexto PH2D: o Bevy é a engine na posição EXATA da PH2D — um ECS puro (a PH2D já usa `bevy_ecs`
> standalone) que precisou construir, peça a peça, o catálogo de componentes que uma engine completa
> exige. O que o Bevy tem *built-in* mostra o mínimo; o que a comunidade construiu por cima mostra o
> que TODO jogo precisa e a engine não deu. Decisão já tomada pelo dono: componentes estilo Unity
> (AddComponent), não herança de nodes — o modelo do Bevy é exatamente esse, levado ao extremo.

---

## 0. As três mecânicas de plataforma que definem o estado da arte (ANTES do catálogo)

Estas não são componentes — são a infraestrutura que torna componentes-como-dados *utilizáveis por
não-programador*. São o achado mais transferível para a PH2D.

### 0.1 Required Components (Bevy 0.15) — a UX do AddComponent resolvida
Fonte: https://bevy.org/news/bevy-0-15/

- **O que é:** `#[require(Team, Sprite)]` num componente faz os dependentes serem inseridos
  **automaticamente** ao adicionar o principal — e em cascata (`Sprite` requer `Transform` +
  `Visibility`, que vêm juntos). Aceita inicializador custom (`#[require(Team(blue_team))]`) e
  registro em runtime (`app.register_required_components::<Bird, Wings>()`).
- **O que substituiu:** os *bundles* (agregados manuais de componentes) foram **deprecados** na 0.15
  inteira. Spawnar um sprite virou `commands.spawn(Sprite { image, ..default() })` — uma linha.
  `Camera2d` requer `Camera`; `Text` requer `TextFont`+`TextColor`+`TextLayout`; `Node` (UI) puxa
  todo o resto.
- **Código que elimina:** o usuário nunca mais esquece um componente-dependência; o erro "adicionei
  X mas faltou Y e nada renderiza" desaparece por construção.
- **Lição para PH2D:** o "AddComponent" do editor deve inserir o *conceito dirigente* e a engine
  completa as dependências — é isso que faz o modelo Unity funcionar sem o usuário conhecer o grafo
  interno. **Relevância: ALTA (mecânica de plataforma).**

### 0.2 Component Hooks + Observers (Bevy 0.14) — reação a dados sem polling
Fonte: https://bevy.org/news/bevy-0-14/

- **Hooks** (`on_add`/`on_insert`/`on_remove`): funções registradas por TIPO de componente, rodam
  imediatamente no ciclo de vida — para invariantes (ex.: registrar num índice espacial ao inserir).
- **Observers:** sistemas sob demanda que disparam **imediatamente** quando um evento é "triggered",
  inclusive **eventos direcionados a uma entidade** (`trigger_targets()`). Observers são eles mesmos
  entidades com componente `Observer`. Modelo *push* (imediato) vs. o *pull* (polling) dos eventos
  clássicos.
- **Código que elimina:** todos os sistemas "vigia" que varrem o mundo por mudança de estado; a
  lógica "quando ESTE objeto for clicado/atingido/spawnado, faça X" vira um observer local.
- **Lição para PH2D:** é a espinha técnica de qualquer sistema "quando [evento] → [ação]" editável
  por UI (o equivalente do UnityEvent/sinal do Godot num ECS). **Relevância: ALTA.**

### 0.3 States (máquina de estados do APP) — `State`, `OnEnter`/`OnExit`/`OnTransition`, computed states e sub-states
Fontes: https://bevy.org/news/bevy-0-14/ · https://docs.rs/bevy/latest/bevy/prelude/index.html

- Estados globais tipados (Menu/Playing/Paused), com schedules `OnEnter(S)`/`OnExit(S)` e run
  condition `in_state(S)`; 0.14 acrescentou **computed states** (derivados por pattern-match de um
  estado pai) e **sub-states** (existem só enquanto o pai está num valor).
- **Código que elimina:** todo o if-else de fluxo de app espalhado pelos sistemas; pause de jogo
  vira "estes sistemas só rodam em `Playing`".
- **Relevância: ALTA** (mas é estado de APP; estado de *entidade* é o `seldom_state`, §11).

---

## 1. Visual / Render
Fontes: https://bevy.org/news/bevy-0-15/ · https://docs.rs/bevy/latest/bevy/prelude/index.html · https://docs.rs/bevy/latest/bevy/ui/prelude/enum.SpriteImageMode.html · https://bevy.org/examples/2d-rendering/sprite-slice/

| Componente | O que faz | Propriedades (UI) | Código que elimina | Compõe com | Relevância 2D |
|---|---|---|---|---|---|
| **Sprite** | Desenha uma imagem 2D na posição do Transform. | `image`, `color` (tint), `flip_x/flip_y`, `custom_size`, `anchor`, `rect`, `texture_atlas`, `image_mode` | Todo o pipeline de quad+material+bind group | Requer `Transform`+`Visibility` (automático via required components) | **Alta** |
| **TextureAtlas / TextureAtlasLayout** | Recorta um frame de uma spritesheet; o layout (grade de rects) é asset compartilhado. | `layout` (handle), `index` (frame atual) | O cálculo de UVs por frame | `Sprite`, crates de animação (§2) | **Alta** |
| **SpriteImageMode::Sliced(TextureSlicer)** | **Nine-patch/9-slice**: fatia a textura em 9 regiões; cantos não escalam, bordas/centro escalam ou tileiam. Existe o gêmeo para UI (`NodeImageMode`). | `border` (BorderRect), `center_scale_mode`, `sides_scale_mode`, `max_corner_scale`; variantes `Auto`/`Scale`/`Tiled` | Toda a malha de 9 quads e a matemática de bordas para painéis/botões redimensionáveis | `Sprite`, `ImageNode` (UI) | **Alta** |
| **Mesh2d + MeshMaterial2d\<ColorMaterial\>** | Malha 2D arbitrária com material (cor/textura/shader custom). | mesh (handle), material (cor, textura) | Pipeline de render de geometria custom | `Transform`; shaders custom | **Média** |
| **Text2d / Text** (mundo vs UI) | Texto como String-newtype; rich text via entidades-filhas `TextSpan`. | `TextFont` (fonte, tamanho), `TextColor`, `TextLayout` (alinhamento, quebra) | Layout de glifos, atlas de fonte | Required components puxam font/color/layout | **Alta** |
| **Gizmos** (immediate mode) | Linhas, círculos, retas, grades, AABBs desenhados por frame — debug visual. `GizmoConfig` por grupo. | por chamada (não é componente de cena); `ShowAabbGizmo` é componente marker | Renderer de linhas de debug inteiro | qualquer sistema | **Alta** (debug) |
| **Tilemap — AUSENTE no core.** A comunidade padroniza em `bevy_ecs_tilemap` (§1.1). | | | | | |
| **Trail/line renderer — AUSENTE no core** (gizmos são só debug, por-frame). Sem componente de trail retido built-in. | | | | | |
| **Animated sprite — AUSENTE no core** (existe `AnimationPlayer` genérico, mas o idioma 2D-spritesheet vem de crates — §2). | | | | | |

### 1.1 bevy_ecs_tilemap — tilemap como entidades
Fonte: https://github.com/StarArawn/bevy_ecs_tilemap

- **Modelo:** CADA TILE É UMA ENTIDADE (`TilePos`, `TileTextureIndex`, `TileColor`, `TileFlip`), e o
  mapa é outra entidade (`TilemapSize`, `TilemapTexture`, `TilemapTileSize`, `TilemapType`). Por
  baixo, chunking em meshes para a GPU; animação de tiles na GPU.
- **Tipos de mapa:** quadrado, **hexagonal** (flat/pointy) e **isométrico** (diamond/staggered);
  camadas e mapas esparsos.
- **Código que elimina:** o renderer de tilemap com chunking, e — o diferencial — mexer num tile é
  query ECS comum ("dar dano a um tile" é editar um componente), não uma API monolítica de grid.
- **Relevância: ALTA.** A lição: *tile-como-entidade* é caro em contagem mas unifica o modelo mental
  — tudo na cena é entidade+componentes, sem um segundo sistema de dados.

---

## 2. Animação de sprite / esqueleto / deform 2D

### 2.1 bevy_spritesheet_animation
Fonte: https://github.com/merwaaan/bevy_spritesheet_animation

- **`SpritesheetAnimation`** (componente) + animações como **assets reutilizáveis**
  (`Assets<Animation>`): clips encadeáveis com duração por frame ou por repetição, repeat count,
  direção (inclui **ping-pong**), easing; **eventos em frames marcados** (markers) e ao completar.
- **Código que elimina:** o eterno `Timer` + `atlas.index += 1` escrito à mão em todo jogo, e o
  sistema de "no frame 7 do ataque, spawne a hitbox" (vira marker + evento).
- **Relevância: ALTA** — este é o "AnimatedSprite" que o core do Bevy não tem.

### 2.2 bevy_spine — esqueleto 2D (runtime oficial Spine via rusty_spine)
Fonte: https://github.com/jabuwu/bevy_spine

- Carrega `.json`/`.skel` + atlas; **`AnimationState`** para play/mix de animações; eventos do Spine
  expostos como eventos Bevy; **ossos viram entidades com Transform sincronizado** — dá para
  pendurar física/queries em ossos.
- **Código que elimina:** runtime de esqueleto 2D inteiro (bind pose, skinning, mixing) e a ponte
  osso→entidade.
- **Relevância: ALTA** para o requisito "esqueleto/deform 2D". Nota: o core do Bevy **NÃO tem**
  skinning 2D próprio; a categoria é 100% terceirizada (Spine é pago no editor; alternativa comum é
  integração Aseprite para frame-a-frame).

### 2.3 AnimationPlayer / AnimationGraph (core, genérico)
Fonte: https://bevy.org/news/bevy-0-14/ · prelude

- Player genérico de `AnimationClip` com **blending por grafo** (`AnimationGraph`: blend nodes +
  clip nodes) e `AnimationTransitions` para crossfade. Hoje é dirigido a código (grafo montado
  programaticamente); editor visual é plano futuro declarado.
- **Relevância 2D: Média** (o idioma 2D usa mais as crates acima), mas a arquitetura
  *grafo-de-blend-como-asset* é a referência.

---

## 3. Partículas — AUSENTE no core; duas escolas na comunidade

### 3.1 bevy_hanabi (GPU, compute)
Fonte: https://github.com/djeedai/bevy_hanabi

- **`EffectAsset`** (asset declarativo) + **`ParticleEffect`** (componente que instancia). Sistema de
  **modifiers** por fase: *spawn* (rate constante, burst único, bursts repetidos), *init* (posição
  em formas: círculo/esfera/cone/plano; velocidade; lifetime; cor/tamanho por partícula), *update*
  (aceleração/gravidade, forças radiais/tangenciais, **colisão** com planos/esferas/depth-buffer),
  *render* (billboard, **gradientes de cor e tamanho ao longo da vida**, trails/ribbons, texturas).
  Suporte 2D.
- **Código que elimina:** simulação por-frame, shaders de partícula, sync CPU-GPU, pipeline de
  render, interpolação de gradientes — tudo vira asset declarativo.
- **Relevância: ALTA.**

### 3.2 bevy_enoki (CPU+instancing, 2D-first) — E TEM EDITOR
Fonte: https://github.com/Lommix/bevy_enoki

- **`ParticleSpawner`** + asset **`Particle2dEffect` em RON com hot-reload**; **curvas** de
  scale/cor/velocidade ao longo da vida; spritesheet como material; SIMD na CPU + GPU instancing
  (zero compute shader ⇒ roda em web/mobile). **Editor web oficial** para autorar o efeito e salvar
  o RON.
- **Lição para PH2D:** o par *asset-declarativo + editor visual + hot-reload* é o que transforma
  partícula de "código" em "conteúdo". **Relevância: ALTA.**

---

## 4. Câmera de gameplay
Fontes: prelude · https://github.com/johanhelsing/bevy_pancam · https://github.com/johanhelsing/bevy_trauma_shake · https://github.com/bevyengine/bevy/pull/21520

- **Core:** `Camera2d` (requer `Camera`), `OrthographicProjection` (escala/viewport), `ClearColor`.
  **Só isso.** Follow, limites, zonas, transições: **AUSENTES no core** (um pan-cam 2D oficial está
  em scaffolding — PR #21520 — sinal de que a lacuna é reconhecida).
- **bevy_pancam:** pan por arrastar + zoom por scroll para câmera ortográfica (estilo editor de
  mapa); botões e limites configuráveis. **Relevância: Média** (é câmera de editor, não de jogo).
- **bevy_trauma_shake:** componente **`Shake`** na câmera + `add_trauma(0.3)`; modelo
  trauma-com-decay (GDC/Squirrel Eiserloh): `ShakeSettings { amplitude, trauma_power,
  decay_per_second, frequency, octaves }`. "Três linhas de código" — zero config para o caso comum.
  **Código que elimina:** o shake caseiro que sempre fica ou brusco ou flutuante. **Relevância: ALTA.**
- **Veredito da categoria:** a MAIOR lacuna do ecossistema Bevy vs. engines de produto. Uma câmera
  de gameplay 2D completa (follow com deadzone, lookahead, limites por região, zonas de zoom,
  transição entre salas) é oportunidade aberta para a PH2D entregar como componentes prontos.

---

## 5. Física & colisão — bevy_rapier2d e avian2d (a dupla dominante)
Fontes: https://rapier.rs/docs/user_guides/bevy_plugin/rigid_bodies · https://docs.rs/avian2d/latest/avian2d/

Modelo idêntico ao que a PH2D já pratica (rapier2d + ponte ECS): física é um SACO DE COMPONENTES
PEQUENOS sobre a entidade, não um objeto monolítico.

| Componente | O que faz | Relevância 2D |
|---|---|---|
| **RigidBody** (`Dynamic`/`Fixed`(Static)/`KinematicPositionBased`/`KinematicVelocityBased`) | Define quem integra o movimento: a física, ninguém, ou o usuário (por posição ou por velocidade) | **Alta** |
| **Collider** | Forma de colisão (círculo, cápsula, cuboid, polyline, trimesh…) | **Alta** |
| **Sensor** | Colisor que só REPORTA sobreposição, sem resposta física — o "Area2D/Trigger" | **Alta** |
| **Velocity / LinearVelocity + AngularVelocity** | Velocidades lidas/escritas como dado simples | **Alta** |
| **ExternalForce / ExternalImpulse** | Força contínua vs. impulso instantâneo, como componentes | **Alta** |
| **Friction / Restitution** (avian) — material físico por-entidade | Atrito e quique como componentes separados, não um "PhysicsMaterial" opaco | **Alta** |
| **GravityScale** | Multiplicador de gravidade por corpo (0 desliga, negativo inverte) | **Alta** |
| **Damping** | Arrasto linear/angular ("atrito com o ar") | **Média** |
| **LockedAxes** | Trava eixos de translação/rotação (ex.: personagem que não tomba) | **Alta** |
| **Ccd** | Continuous collision detection anti-tunneling para objetos rápidos | **Média** |
| **Dominance** | Grupos de dominância: o mais alto ignora força de contato do mais baixo | **Baixa** |
| **Sleeping** | Corpos inativos dormem (economia) | **Média** |
| **CollisionLayers** (avian) | Filtro camada×máscara de quem colide com quem | **Alta** |
| **CollisionEventsEnabled** (avian) | Opt-in de eventos de colisão por entidade | **Média** |
| **Joints:** `FixedJoint` · `RevoluteJoint` (dobradiça) · `PrismaticJoint` (trilho) · `DistanceJoint` | Restrições entre dois corpos, como dados | **Alta** |
| **RayCaster / ShapeCaster** (avian) | Raycast/shapecast PERSISTENTE como COMPONENTE: a entidade carrega um raio que atualiza sozinho todo frame (line-of-sight, sensor de chão) — em vez de chamada de API imperativa | **Alta** — idioma notável |
| **TransformInterpolation** (avian) | Interpola o visual entre passos do fixed timestep — mata o jitter de física@50Hz vs render@144Hz com UM componente | **Alta** |
| **PhysicsPickingPlugin** (avian) | Clique/hover via colisores integrado ao bevy_picking | **Média** |

**Código que elimina (conjunto):** integração numérica, resolução de contato, broadphase, e —
crucial para "UI reduz programação" — *cada tuning vira um campo editável* (gravidade daquele objeto,
quique, camadas) em vez de código.

---

## 6. Character controller PRONTO — bevy-tnua
Fonte: https://github.com/idanarye/bevy-tnua

- **`TnuaController`**: controlador "flutuante" (personagem paira a `float_height` do chão — atrito
  e degraus deixam de ser problema). **Basis** `TnuaBuiltinWalk` (`float_height`,
  `desired_velocity`, `desired_forward`) + **actions** empilháveis: `TnuaBuiltinJump`,
  `TnuaBuiltinCrouch`, `TnuaBuiltinDash`, knockback.
- **O que vem de graça (o ouro):** **coyote time**, **jump buffering**, contador de ações aéreas
  (**double jump**), rampas/escadas, **plataformas móveis**, wall-slide/climb — os detalhes que
  separam plataformer "de game jam" de plataformer com game-feel.
- Funciona com rapier 2D/3D E avian 2D/3D via crates de integração (o controller é agnóstico).
- **Código que elimina:** o personagem-de-plataforma inteiro — a peça que TODO iniciante escreve
  errado três vezes (ground check, timing de pulo, plataforma móvel).
- **AUSENTES como controllers prontos:** top-down, 8-direções, veículo — não há crate dominante;
  ficam a cargo do usuário (com tnua servindo de base).
- **Relevância: ALTA.** Nota PH2D: já existe o `ph2d-platformer` — o diferencial do tnua a copiar é
  o desenho *basis+actions empilháveis* e a lista de game-feel acima como CHECKLIST de paridade.

---

## 7. Caminhos / splines
Fonte: https://docs.rs/bevy_math/latest/bevy_math/cubic_splines/index.html

- **bevy_math (core):** `CubicBezier`, `CubicBSpline`, `CubicCardinalSpline` (Catmull-Rom),
  `CubicHermite`, `CubicNurbs` (círculos/elipses perfeitos), `LinearSpline` → todos geram
  **`CubicCurve`** amostrável em posição, **velocidade e aceleração**, com iteração por pontos.
- **Path-follow como componente: AUSENTE no core** — confirmado; são structs de matemática, não
  gameplay. (bevy_tweening cobre o caso "mover ao longo de curva no tempo" via lens; mas um
  `PathFollow { path, speed, orient_to_path, loop }` de editor não existe no ecossistema como padrão.)
- **Relevância: Alta** como fundação; a categoria "componente de seguir caminho" é lacuna → oportunidade PH2D.

---

## 8. Tween / easing — bevy_tweening
Fonte: https://github.com/djeedai/bevy_tweening

- **`Animator<T>`** (componente) executa **`Tween`** (easing entre dois valores: `EaseFunction`,
  duração, `RepeatCount::Finite(n)|Infinite`, `RepeatStrategy::MirroredRepeat` = ping-pong),
  **`Sequence`** (encadeia com `.then()`) e **`Delay`**.
- **Lenses** dizem QUAL campo animar sem copiar o componente: `TransformPositionLens`,
  `TransformRotationLens`, `TransformScaleLens`, `SpriteColorLens`, `TextColorLens`,
  `UiPositionLens`, `ColorMaterialColorLens` — e `Lens` custom para qualquer campo.
- **Eventos/one-shot systems ao completar** — encadeia gameplay ("ao fim do fade, troque de cena").
- **Código que elimina:** toda interpolação manual frame-a-frame, gestão de estado de animação,
  easing repetido.
- **Relevância: ALTA.** Para a PH2D: o conceito de *lens* é como um tween genérico vira UI — a lista
  de lenses É o dropdown "propriedade a animar".

## 8-bis. Timeline / sequencer — **AUSENTE**
Não há timeline/dope-sheet no Bevy nem crate dominante de cutscene/sequência (o `AnimationGraph` é
blend, não sequência; editores visuais são "futuro" declarado). A PH2D com timeline própria +
sinais (ADR-0143) está À FRENTE do ecossistema aqui.

---

## 9. Áudio
Fonte: https://docs.rs/bevy/latest/bevy/audio/index.html

- **`AudioPlayer`** (componente: tocar um som na entidade) + **`PlaybackSettings`**: modo
  (`Once`/`Loop`/**`Despawn`**/**`Remove`** — os dois últimos LIMPAM a entidade/componente sozinhos ao
  terminar: fire-and-forget), `volume`, `speed`, `paused`, `spatial`, `spatial_scale`.
- **`SpatialListener`** — áudio posicional 2D/3D: a posição vem do **Transform** (pan/atenuação
  automáticos); `SpatialScale`/`DefaultSpatialScale` calibram unidades→ouvido; **`GlobalVolume`**.
- **`AudioSink` / `SpatialAudioSink`** — controle durante playback (volume, speed, pause) como
  componente na mesma entidade.
- **Código que elimina:** mixagem manual, panning por distância, ciclo de vida de sons one-shot.
- **Relevância: ALTA.** (Comunidade usa muito `bevy_kira_audio` para mais controle de mix, mas o
  modelo de componentes é o mesmo.)

---

## 10. Input — leafwing-input-manager (o padrão de facto)
Fonte: https://github.com/Leafwing-Studios/leafwing-input-manager

- O usuário define um **enum de AÇÕES semânticas** (`Jump`, `Run`, `Move`) com derive `Actionlike`;
  **`InputMap`** (componente!) liga teclado+gamepad+mouse à ação: dual-axis com deadzone/
  sensibilidade/clamp, **virtual D-pad**, **chords** (combinações), `ClashStrategy` para desambiguar
  chord vs tecla solta.
- **`ActionState`** (componente) é o que o gameplay lê: `pressed()`, `just_pressed()`, `value()`,
  `axis_pair()` — nunca mais `KeyCode` no código de jogo.
- **Por ser componente POR-ENTIDADE:** multiplayer local = 2 entidades com InputMaps diferentes;
  rede = ActionState serializável; testes = input mockável.
- **Código que elimina:** o mega-sistema de input com match de teclas espalhado, o remapeamento de
  controles (vira EDITAR UM MAPA — i.e., uma tela de settings de graça), e a duplicação
  teclado-vs-gamepad.
- **Relevância: ALTA — provavelmente o componente com melhor razão poder/tamanho do ecossistema.**

---

## 11. Estado de ENTIDADE — seldom_state (state machine como componente)
Fonte: https://github.com/Seldom-SE/seldom_state

- **`StateMachine`** (componente) + **cada estado É um componente** (`Grounded`, `Airborne`,
  `Stunned`) que a máquina insere/remove mantendo exatamente UM ativo. Transições declarativas:
  `trans::<Grounded, _>(just_pressed(Action::Jump), Airborne)`; **30 triggers built-in** (input,
  `done`, eventos, composição booleana); `trans_builder` passa DADOS do estado velho para o novo;
  `on_enter`/`on_exit` para efeitos (ex.: trocar animação).
- **Sinergia-chave:** estado-como-componente significa que OUTROS sistemas fazem query por estado
  (`Query<&Transform, With<Airborne>>`) — a máquina de estados vira filtro de query.
- **Código que elimina:** o enum de estado + match gigante + flags booleanas que dessincronizam; e o
  acoplamento animação↔lógica (on_enter cuida).
- **Relevância: ALTA.** Para PH2D: é O candidato a "componente que reduz programação via UI" — uma
  tabela estado×trigger×próximo-estado é 100% editável visualmente.

---

## 12. AI

### 12.1 big-brain — utility AI declarativa
Fonte: https://github.com/zkat/big-brain

- **`Thinker`** (componente, montado por builder) + **Scorers** (componentes que produzem `Score`
  0..1 — os "olhos") + **Actions** (componentes com máquina `Requested → Executing →
  Success/Failure` + `Cancelled`). **Pickers** (`FirstToScore(0.8)`, Highest) escolhem a ação pelo
  score.
- O usuário escreve sistemas pequenos e ISOLADOS (um scorer = uma medição; uma action = um
  comportamento); o Thinker compõe tudo declarativamente.
- **Código que elimina:** árvores de decisão à mão, tracking manual de estado de execução,
  cancelamento de comportamento interrompido.
- **Relevância: ALTA** para NPCs; o desenho (medir→pontuar→escolher) é UI-izável (sliders de peso).

### 12.2 Navegação — vleue_navigator + Polyanya
Fonte: https://github.com/vleue/vleue_navigator

- **Navmesh 2D auto-atualizável:** `NavMeshSettings` + `NavMeshUpdateMode`; **obstáculos declarados
  por componente** (`NavMeshObstacle`/`CachableObstacle` de primitivas — OU direto dos colliders
  avian/rapier!); quando um obstáculo se move, o mesh **regenera sozinho**. Pathfinding **Polyanya**
  ("fast and optimal", melhor que A* em grid); `NavMeshDebug` para ver o mesh.
- **Código que elimina:** geração de navmesh, sync colisão↔navegação, algoritmo de pathfinding,
  atualização dinâmica de obstáculos.
- **AUSENTE:** componente "NavAgent" com steering/avoidance pronto (o usuário ainda move a entidade
  ao longo do path); behavior tree dominante também não há (big-brain é utility, não BT).
- **Relevância: ALTA** (a dupla obstáculo-é-o-collider + auto-update é o estado da arte 2D).

---

## 13. UI in-game (bevy_ui) + picking
Fontes: https://docs.rs/bevy/latest/bevy/prelude/index.html · https://bevy.org/news/bevy-0-15/

- **`Node`** é o componente-raiz (layout **flexbox/grid via Taffy** — a MESMA taffy que a PH2D já
  usa no auto layout do Vector, ADR-0153); `Button`, `Text`, `ImageNode` (com `NodeImageMode` 9-slice),
  `BackgroundColor`, `BorderColor`, `BorderRadius`, `Outline`, `ScrollPosition`, `GlobalZIndex`,
  gradientes (`BackgroundGradient`/`BorderGradient`). Required components: adicionar `Button` puxa
  `Node` e tudo de que precisa.
- **bevy_picking (core desde 0.15):** `Pickable` + eventos-observer por entidade: `Over`, `Enter`/
  `Leave`, `Press`/`Release`, `Click`, **`Drag`**, **`DragDrop`** — hover/clique/arrastar em QUALQUER
  entidade (sprite, mesh, UI) via observers, com backends intercambiáveis (inclusive físico, §5).
- **Código que elimina:** hit-test manual, state de hover/press, drag&drop caseiro.
- **Relevância: ALTA** — e a dupla *picking-genérico + observers* é o mecanismo pelo qual "clicável"
  vira um checkbox na UI do editor.

---

## 14. Level editor externo — bevy_ecs_ldtk
Fonte: https://github.com/Trouv/bevy_ecs_ldtk

- O arquivo do **LDtk** (editor de níveis 2D gratuito) é um ASSET; `LdtkWorldBundle` spawna o mundo.
  O usuário registra `app.register_ldtk_entity::<GoblinBundle>("Goblin")` e deriva
  **`#[derive(LdtkEntity)]`** com `#[sprite_sheet]`, `#[grid_coords]` — **cada entidade posta no
  editor vira entidade Bevy com os componentes certos, automaticamente**. `LdtkIntCell` faz o mesmo
  para camadas de int-grid (ex.: colisão); `LevelSelection` escolhe o nível; hot-reload do arquivo.
- **Código que elimina:** parser de formato de nível, loops de spawn, tradução dado-do-editor →
  componente.
- **Relevância: ALTA como PADRÃO DE DESENHO para a PH2D:** "objeto autorado no editor → entidade com
  componentes registrados por identificador" é exatamente o contrato cena↔runtime que a PH2D vai
  precisar (e a PH2D tem o editor DENTRO, vantagem estrutural).

---

## 15. Rede / multiplayer — bevy_replicon
Fonte: https://github.com/projectharmonia/bevy_replicon

- **Replicação server-autoritativa por marcação:** componente **`Replicated`** na entidade +
  `app.replicate::<C>()` por tipo → o servidor sincroniza mudanças aos clientes automaticamente
  (serialização via reflection). **Eventos remotos** cliente↔servidor; visibilidade/interesse por
  relações ECS; agnóstico de transporte (renet, quinnet, matchbox).
- A MESMA lógica de jogo roda single, client, dedicated e listen-server.
- **Código que elimina:** serialização de estado, protocolo de sync, bifurcação single/multi.
- **Relevância: Média** (para a PH2D hoje; o padrão "replicação = marker + registro por tipo" é o
  que importa guardar).

---

## 16. Persistência / save — moonshine_save
Fonte: https://github.com/Zeenobit/moonshine_save

- **`Save`** (componente marker): só entidades marcadas entram no save; filtros por componente
  (allow/block); `commands.trigger_save(SaveWorld::default_into_file(path))` /
  `trigger_load(...)` — pipelines via observers; serialização por `Reflect`.
- **A tese:** separar **model** (estado salvável) de **view** (sprites, UI — reconstruíveis) — o
  save é subconjunto mínimo e vira "fonte única da verdade" do estado de jogo.
- **Código que elimina:** serialização manual e o inchaço de salvar a cena inteira.
- **Relevância: Média-Alta**; a tese model/view espelha a decisão da PH2D de undo/save sobre o mesmo
  snapshot canônico.

---

## 17. Parallax — bevy-parallax
Fonte: https://github.com/Corrosive-Games/bevy-parallax

- **`LayerData`** por camada: `speed`, `path` (imagem), `tile_size`, `cols/rows`, `scale`, `z`;
  **`ParallaxCameraComponent`** na câmera; repetição/wrap automático de tiles;
  `ParallaxMoveEvent` move o conjunto.
- **Código que elimina:** a matemática velocidade-relativa-à-câmera + wrap de tiles + z-ordering.
- **Relevância: ALTA** (2D side-scroller); é uma tabelinha de camadas — UI pura.

---

## 18. Scripting / componente próprio
Fontes: https://bevy.org/news/bevy-0-15/ · https://github.com/makspll/bevy_mod_scripting

- **O caminho nativo:** `#[derive(Component)]` numa struct QUALQUER — é TODO o custo de criar um
  componente; `#[derive(Reflect)]` o torna inspecionável/serializável (a base de um inspector
  genérico de editor); `#[require(...)]` declara dependências. Comportamento = sistemas (funções
  soltas com queries). **Visual scripting: AUSENTE** no ecossistema (nada dominante).
- **bevy_mod_scripting:** **Lua** (5.1-5.4, LuaJIT, Luau) e **Rhai**; scripts anexados a entidades,
  callbacks (`on_update`, `on_init`), **hot-reload sem recompilar**, acesso a queries/componentes
  via reflection com bindings gerados; um só registro de bindings serve todas as linguagens.
- **Código que elimina:** o ciclo de recompilação Rust para iterar gameplay; abre modding e
  contribuição de não-programadores.
- **Relevância: Média-Alta** — para a PH2D (alvo: ARTISTAS), a lacuna de visual scripting no
  ecossistema Rust é a oportunidade número um; os Motion Nodes já são meio caminho.

---

## 19. Categorias SEM resposta no ecossistema (declaração explícita de ausência)

| Categoria pedida | Situação no Bevy/ecossistema |
|---|---|
| **Camera follow/limites/zonas/transições de gameplay** | AUSENTE como componente pronto (só pancam=editor e shake; PR de pan-cam 2D em scaffolding) |
| **Timeline/sequencer/cutscene** | AUSENTE (AnimationGraph é blend por código; editor visual "futuro") |
| **Character controllers top-down / 8-dir / veículo** | AUSENTE (só plataforma, via tnua) |
| **Path-follow como componente** | AUSENTE (só a matemática de splines no core) |
| **Spawner/factory/pooling como componente** | AUSENTE (spawn é código; pooling manual) |
| **Trail/line renderer retido** | AUSENTE no core (gizmos = só debug imediato) |
| **NavAgent com steering/avoidance** | AUSENTE (navmesh+path existem; mover a entidade é do usuário) |
| **Behavior tree dominante** | AUSENTE (big-brain é utility AI; BTs existem mas sem crate padrão) |
| **Visual scripting** | AUSENTE (só scripting textual Lua/Rhai) |
| **Utilitários "Construct-like"** (pin, wrap, fade, flash, sine, move-to, solid/jumpthru como behaviors soltos) | AUSENTE como catálogo — o ecossistema resolve caso a caso (tween cobre fade/sine/move-to; física cobre solid) |
| **Timer como componente de cena** | Parcial: `Timer`/`Time` são structs excelentes (repeating, elapsed, just_finished) mas o usuário os embute em componentes seus — não há "TimerComponent" de editor |
| **Esqueleto 2D nativo** | AUSENTE no core (terceirizado ao Spine via bevy_spine) |

**Estas ausências são o mapa das oportunidades da PH2D**: são exatamente as categorias em que uma
engine com EDITOR integrado (que o Bevy não tem) pode converter cada lacuna em um componente com UI.

---

## ⭐ MATADORES DE CÓDIGO — os 15 que mais eliminam programação só com UI/dados

Ordenados por (código eliminado × frequência de uso × quão editável-por-UI é):

1. **InputMap + ActionState** (leafwing) — remapeamento, gamepad+teclado, chords, deadzones e tela
   de controles viram UMA TABELA editável; o gameplay nunca vê tecla.
2. **TnuaController + TnuaBuiltinWalk/Jump/Crouch/Dash** — o personagem de plataforma com coyote
   time, jump buffering, double jump e plataforma móvel vira um painel de ~10 sliders.
3. **StateMachine** (seldom_state) — a tabela estado×trigger×transição substitui o match gigante;
   on_enter/on_exit conecta animação sem código.
4. **RigidBody + Collider + Sensor + camadas/materiais** (rapier/avian) — todo o motor de movimento
   e triggers de gameplay em componentes de meia dúzia de campos.
5. **ParticleEffect / ParticleSpawner** (hanabi/enoki) — efeito = asset declarativo com curvas e
   gradientes, autorado em editor com hot-reload; zero shader, zero sim manual.
6. **SpritesheetAnimation** — clips, ping-pong, easing e eventos-em-frame sem o Timer+index de sempre.
7. **Animator + lenses** (bevy_tweening) — qualquer campo animável por dropdown (lens) + easing +
   repeat; fade/pulse/move-to declarativos.
8. **LdtkEntity registrado por identificador** (bevy_ecs_ldtk) — o nível desenhado no editor spawna
   entidades já componentizadas; zero loop de spawn.
9. **NavMesh auto-atualizável** (vleue_navigator) — obstáculo É o collider; o mesh se refaz sozinho;
   path ótimo por Polyanya sem uma linha de algoritmo.
10. **Shake (trauma)** — game-feel de impacto em `add_trauma(0.3)`; 5 números de configuração.
11. **AudioPlayer + PlaybackSettings::DESPAWN + SpatialListener** — som posicional 2D fire-and-forget;
    o ciclo de vida se limpa sozinho.
12. **Thinker + Scorers + Pickers** (big-brain) — NPC utility-AI composto por pesos e thresholds —
    sliders, não árvore de ifs.
13. **SpriteImageMode::Sliced (9-slice)** — painéis e botões redimensionáveis com 1 enum + 4 bordas.
14. **Replicated + app.replicate::<C>()** (replicon) — multiplayer server-autoritativo por marcação
    de tipo, sem protocolo manual.
15. **Required Components** (`#[require]`) — o meta-matador: faz TODOS os anteriores serem
    adicionáveis com UM clique sem quebrar dependências (e por isso é o que a UI de AddComponent da
    PH2D deve implementar primeiro).

Menções honrosas: **RayCaster-como-componente** (line-of-sight declarativo, avian) ·
**TransformInterpolation** (mata o jitter do fixed timestep com 1 componente) ·
**bevy-parallax** (fundo em camadas = tabela) · **Save marker** (persistência por marcação) ·
**Pickable + Drag/DragDrop observers** (drag&drop sem hit-test manual).

---

## Síntese para a PH2D (leitura do pesquisador)

1. **O modelo componente-como-dado está validado no extremo:** no Bevy TUDO é componente pequeno e
   ortogonal (até raycast, até shake, até "replicável", até o estado da state machine). A decisão
   Unity-style da PH2D é compatível — e o `#[require]` do 0.15 é a peça de UX que falta na própria
   Unity: adicionar o conceito principal puxa as dependências.
2. **O que o ECS puro precisou ADICIONAR para virar engine** (a posição da PH2D): input semântico,
   character controller, state machine de entidade, tween, partículas, navmesh, tilemap, 9-slice,
   spatial audio, picking, replicação, save. Essa lista É o catálogo mínimo de um AddComponent
   competitivo — e cada item acima tem o desenho de referência citado.
3. **Onde a PH2D pode passar o ecossistema inteiro:** câmera de gameplay completa, timeline (já
   tem!), path-follow, spawner/pooling, utilitários Construct-like, visual scripting sobre os
   Motion Nodes, e o inspector que edita tudo isso — porque o Bevy NÃO TEM EDITOR, e toda a
   comunidade dele está programando o que a PH2D pode entregar como UI.
4. **Padrões técnicos a adotar:** observers/hooks para "quando X → Y" autorável; estado-de-entidade
   como componente (queries filtram por estado); asset declarativo + hot-reload + editor para
   conteúdo (partículas enoki); marker components para capacidades transversais (Save, Replicated,
   Pickable); reflection para inspector genérico.
