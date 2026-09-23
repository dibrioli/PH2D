# A TABELA COMPARATIVA — Godot 4.7.2 × PH2D, componente a componente

> **Encomenda do dono (2026-09-20/22):** *«crie uma tabela comparativa com tudo que a Godot tem e
> nossas features ao lado»*, guardada em `docs/Components/` para ficar à mão antes de qualquer
> implementação.
>
> **Como foi medida:** o lado dele é o `ClassDB` do **binário instalado** (Godot **4.7.2**, MIT),
> despejado pela sonda versionada
> [`ferramentas/godot_classdb_probe.gd`](ferramentas/godot_classdb_probe.gd) — **1 054** classes,
> das quais **546** no alcance de um editor 2D. O lado nosso são os **157** descritores do catálogo,
> contados a **correr** `ph2d_component_desc::catalog::all()`.
> ⛔ *Três varreduras de texto deram três respostas diferentes e todas erradas* — os `requires:`
> citam nomes alheios e cada família usa um construtor próprio.
>
> **O funil, o método e o porquê de cada corte** vivem no dossiê:
> [`pesquisa/dossie_godot_o_catalogo_medido_2026-09-20.md`](pesquisa/dossie_godot_o_catalogo_medido_2026-09-20.md).
> A lista candidata (o que ele tem e nós não, **sem ordem de propósito**) é o §8.1 de lá.
>
> ⚠️ **Esta tabela envelhece com o PRODUTO, não com o alvo.** O lado dele re-mede-se correndo a
> sonda; o lado nosso muda a cada wave. ⛔ Ao fechar uma wave que traga um componente novo, é esta
> linha que se corrige — e o número de descritores CONTA-SE, nunca se estima.

> **Legenda:** ✅ temos · ◑ metade (a que falta está dita) · ⭐ não temos · ⛔ fora (decisão ou contexto)

## Desenho

| Godot | PH2D | |
|---|---|---|
| `Sprite2D` (atlas, grade de frames, flip, pivô) | `Sprite` + `SpriteRegion` `SpriteGrid` `SpriteSheetRef` `SpritePixels` `SpriteEmissive` `SpriteCornerTint` | ✅ |
| `AnimatedSprite2D` + `SpriteFrames` | `SpriteAnimations` + `SpriteAnimator` | ✅ |
| `NinePatchRect` (moldura 9 fatias) | `SliceNine` | ✅ |
| `Polygon2D` (com pesos de osso) | `ph2d-poly2d` + `Skin` | ✅ |
| `Node2D` — posição, rotação, escala, **1** eixo de inclinação | `Transform` — com inclinação nos **2** eixos | ✅ |
| `CanvasItem` — tint, z, y-sort, filtro, repetição, recorte | `Visibility` `BlendMode` `ZIndexOverride` `ZAsRelative` `YSort` `SortingLayer` `SortingGroup` `OrderInLayer` `TextureFilter` `TextureRepeat` `ShowBehindParent` `TopLevel` `ClipChildren` `VisibilityLayer` | ✅ |
| `Line2D` (linha espessa com perfil de largura, gradiente, pontas) | o motor é nosso e é maior (`VecStrokeProfile`, pontas e juntas do vetorial) — **falta o componente de RASTO** | ◑ |
| `MeshInstance2D` · `MultiMeshInstance2D` | malha 2D e instanciação em massa existem (Motion), não como componente | ◑ |
| `CanvasLayer` (camada que não segue a câmara) | `UiCanvas` cobre o caso do HUD | ◑ |
| `Parallax2D` · `ParallaxLayer` · `ParallaxBackground` | a LEI existe no multiplano do Flip (`depth` ≡ `scroll_scale`, medido); não como componente de cena — ⭐ [pesquisa](23_pesquisa_paralaxe.md) + [plano](24_plano_paralaxe.md) | ◑ |
| `CanvasGroup` (filhos como UM desenho, depois transparência) | — | ⭐ |
| `CanvasModulate` (tinge a cena inteira) | — | ⭐ |
| `BackBufferCopy` (ler o ecrã para um shader) | — | ⭐ |

## Câmara e visibilidade

| Godot | PH2D | |
|---|---|---|
| `Camera2D` (suavização, zona morta, limites do nível) | `GameCamera` + `CameraFollow` + `CameraLimits` — **portados dele**, paridade medida a 0,000061 px | ✅ |
| **não tem** | `CameraShake` + `ShakeEmitter` (tremor com distância) | ⭐ só nós |
| `VisibleOnScreenNotifier2D` / `Enabler2D` | `OnScreenEnabler` + `DestroyOutside` | ✅ |
| `Marker2D` (ponto de referência) | objeto vazio + `Name` + `Tags` | ✅ |
| `RemoteTransform2D` (escrevo a minha pose em ti) | `AnchorMount` faz o sentido inverso | ◑ |
| `SubViewport` (desenhar a cena numa textura) | pré-visualizações e miniaturas internas | ◑ |
| `WorldEnvironment` / `Environment` (brilho, ajuste de cor) | `ph2d-bloom` + o modo Render do 3D | ◑ |

## Física

| Godot | PH2D | |
|---|---|---|
| `RigidBody2D` | `RigidBody` + `MassOverride` `GravityScale` `DampingOverride` `Ccd` `LockRotation` `LockPositionX/Y` `Dominance` `InitialVelocity` `MaterialCombine` | ✅ mais fino |
| `StaticBody2D` | `RigidBody` (estático) | ✅ |
| `CharacterBody2D` (o alicerce; o resto é programar) | `PlatformPlayer` (55 campos) + `TopDownPlayer` + `PlayerMode` + `PlayerSignals` + `NoWallCling` | ✅ **muito à frente** |
| `Area2D` (sensor + gravidade/arrasto locais) | `AreaEffector` `AreaBuoyancy` `AreaDrag` `AreaFormDrag` `AreaTorque` `AreaFalloff` `AreaForceWorldAxes` `SignalOnHit` `SignalOnLeave` `SignalTagFilter` | ✅ mais rica |
| `CollisionShape2D` + colisão de um sentido | `Collider` + `OneWayPlatform` | ✅ |
| formas: círculo, retângulo, cápsula | `Ball` `Cuboid` `Capsule` | ✅ |
| `RayCast2D` | `RaySensor` + `RaySignals` | ✅ |
| juntas: pino (com motor e limites), calha, mola | `PhysicsJoint` (7 tipos) + `JointWorldAnchor` + `PulleyWheel` + `WestonAxle` + `RopeStops` | ✅ mais |
| `PhysicsMaterial` (atrito, quique) | `MaterialCombine` | ✅ |
| camadas e máscaras de colisão | `Mask2D` `MaskInteraction` | ✅ |
| `AnimatableBody2D` (plataforma que empurra quem vai em cima) | corpo cinemático + `WalkSurface` + `OneWayPlatform` | ◑ |
| `PhysicalBone2D` (este osso É este corpo) | ragdoll por juntas; não declarado assim | ◑ |
| consultas de física por script (raio/forma/ponto) | o motor faz; um script não pergunta | ◑ |
| formas: **polígono**, segmento, barreira de mundo | — | ⭐ |
| `CollisionPolygon2D` (colidir com arte desenhada) | — | ⭐ |
| `ShapeCast2D` (varrer uma forma, N acertos) | existe no motor, não como sensor | ⭐ |
| esteira rolante (`constant_linear_velocity`) | — | ⭐ |
| rato sobre um corpo como sinal (`input_pickable`) | — | ⭐ |
| reverb por zona (`audio_bus_override` da `Area2D`) | — | ⭐ |

## Partículas

| Godot | PH2D | |
|---|---|---|
| `GPUParticles2D` + `ParticleProcessMaterial` | `ParticleEmitter` (compila para nós reais do Motion) | ✅ |
| `CPUParticles2D` | o mesmo componente | ✅ |
| sub-emissor e pré-aquecimento | por comparar | ◑ |

## Áudio

| Godot | PH2D | |
|---|---|---|
| `AudioStreamPlayer2D` (posicional) | `AudioSource2D` | ✅ |
| `AudioListener2D` | `AudioListener2D` | ✅ |
| **27** efeitos | **42** + análise espectral + limpeza de ruído por IA | ✅ à frente |
| WAV / MP3 / Ogg | `ph2d-audio-decode` + exportação Ogg/Opus | ✅ |
| barramentos com envios (`AudioBusLayout`) | temos mixer; o roteamento autorado não | ◑ |
| **música adaptativa**: clips com transição, playlist, sorteio, camadas sincronizadas | — | ⭐⭐ |
| microfone · síntese | — | ⭐ |

## Esqueleto

| Godot | PH2D | |
|---|---|---|
| `Skeleton2D` + `Bone2D` (pose de repouso) | `ph2d-skeleton` + `Bone` `BoneRest` `BoneLimit` `SmartBone` | ✅ |
| pesos pintados por osso | `Skin` + malha graduada pelas articulações | ✅ |
| IK — 7 modificadores, **marcados EXPERIMENTAIS por eles** | `IkGoal` `IkTarget` com âncora, *Mix*, *Softness*, *Chain* | ✅ à frente |

## Caminhos, navegação, luz, tiles

| Godot | PH2D | |
|---|---|---|
| `Path2D` + `PathFollow2D` | `PathFollow` + `ph2d-curve` | ✅ |
| `NavigationRegion2D` · `NavigationAgent2D` · `NavigationObstacle2D` · `NavigationLink2D` · `AStar2D` · `AStarGrid2D` · `NavigationPolygon` | **nada** | ⭐⭐ |
| `PointLight2D` · `DirectionalLight2D` · `LightOccluder2D` · `CanvasTexture` | `ph2d-light` é outra luz (relevo do Painter e do 3D) | ⛔ adiado por si (20/08) |
| `TileMap` · `TileMapLayer` · `TileSet` · `TileData` | — | ⛔ é o projeto Tilling |

## Animação

| Godot | PH2D | |
|---|---|---|
| `AnimationPlayer` + `Animation` + `AnimationLibrary` | o módulo Timeline inteiro (curvas, clips, nesting, retiming, onion, expressões) | ✅ **muito à frente** |
| `Tween` + 6 tipos — **só por código** | `Tweens` + `ph2d-tween` — **componente** | ✅ à frente |
| `Curve` · `Gradient` | `ph2d-curve` + editores ricos + rampa com interpolação por parada | ✅ |
| `AnimationTree` + 18 nós de mistura (blend trees, espaços de mistura) | — | ⭐ |

## Lógica de jogo

| Godot | PH2D | |
|---|---|---|
| `Timer` | `Timers` | ✅ |
| sinais ligados no editor (**exigem um script do outro lado**) | `SignalActions` + `SignalOnAction` + `SignalOnHit/Leave` + `SignalTagFilter` — **sem uma linha de script** | ✅ à frente |
| `groups` (planos, sem UI própria) | `Tags` (em árvore, com painel próprio) | ✅ à frente |
| `PackedScene` (prefab) | instâncias, variantes, exceções, aplicar/soltar | ✅ |
| script (`GDScript`) | `LuauScript` com propriedades no Inspector | ✅ |
| **não tem** | `Factory` · `Lifetime` · `DestroyOutside` | ⭐ só nós |
| **não tem** (só addon) | `StateMachine` de jogo | ⭐ só nós |
| **não tem** | `WeaponFire` · `ProjectileMotion` | ⭐ só nós |
| **não tem** | `Counter` · `CounterWatch` · `SequencePlayer` | ⭐ só nós |
| `Timer.ignore_time_scale` | — | ⭐ |
| `MultiplayerSpawner` / `Synchronizer` (rede) | — | ⛔ sem rede |

## Interface DENTRO do jogo

| Godot | PH2D | |
|---|---|---|
| `Label` | `UiLabel` | ✅ |
| `Button` / `TextureButton` | `UiButton` | ✅ |
| `Control` (âncoras e margens) | `UiCanvas` + `VecAnchors` | ◑ |
| caixas que empilham (`HBox` `VBox` `Grid` `Margin` `Center` `Aspect` `Flow` `Split` `Scroll`) | o motor existe (`VecLayout`), no HUD não | ◑ |
| campos, listas, abas, caixas de marcar, sliders, árvore | existem no **editor**, não no jogo | ◑ |
| tema (`Theme` `StyleBox` `Font` `LabelSettings`) | `ph2d-tokens` serve o editor | ◑ |
| **`ProgressBar` / `TextureProgressBar`** | — | ⭐⭐ a barra de vida |
| `TextureRect` · `ColorRect` | — | ⭐ |
| `RichTextLabel` (texto com estilo e efeitos) | — | ⭐ |
| `TouchScreenButton` · `VirtualJoystick` | — | ⭐ |
| `VideoStreamPlayer` | — | ⭐ |
| `ButtonGroup` (rádio) | — | ⭐ |

## Entrada

| Godot | PH2D | |
|---|---|---|
| `InputMap` — ações nomeadas (com a falha pública de juntar zona morta e ponto de disparo) | Input Map — os **dois separados**, e coagidos na porta | ✅ à frente |
| teclado, rato, comando, ação | `ph2d-input` | ✅ |
| `Shortcut` | ✅ | ✅ |
| toque no ecrã (`ScreenTouch` / `ScreenDrag`) | — | ⭐ |
| gestos de trackpad (pinça, arrasto) | — | ⭐ |
| MIDI | — | ⭐ |

## Assets e recursos

| Godot | PH2D | |
|---|---|---|
| `Image` + formatos | `ph2d-imageio` — **16** formatos | ✅ à frente |
| `AtlasTexture` | `SpriteRegion` | ✅ |
| ruído (`NoiseTexture2D`, `FastNoiseLite`) | `ph2d-fbm`, `motion.noise` | ✅ |
| texturas de gradiente e de curva | editores de curva, gradiente e paleta | ✅ |
| textura comprimida | `ph2d-asset-ktx2` | ✅ |
| `ResourceUID` (identidade de asset) | `ph2d-asset-id` (blake3) | ✅ |
| `Translation` | `ph2d-i18n` | ✅ |
| `ConfigFile` / `JSON` | preferências + `ph2d-expr` | ✅ |
| `ResourcePreloader` | `ph2d-asset-index` | ◑ |
| `AnimatedTexture` | ◑ | ◑ |
| `ViewportTexture` · `CameraTexture` (webcam) | — | ⭐ |

## Shader e material

| Godot | PH2D | |
|---|---|---|
| `ParticleProcessMaterial` | `ParticleEmitter` | ✅ |
| `CanvasItemMaterial` (modo de mistura) | `BlendMode` | ◑ |
| **`ShaderMaterial` + `VisualShader` + 111 nós de shader** | — (a nossa resposta provável é o nodegraph que já temos) | ⭐⭐ |

## Capacidades (o que um script alcança)

| Godot | PH2D | |
|---|---|---|
| `Input` / `InputMap` | ✅ | ✅ |
| sorteio determinista | ✅ (renasce ao rebobinar) | ✅ |
| desfazer/refazer | ✅ fila única por diferença | ✅ |
| **acessibilidade** (novidade da 4.7 deles) | `ph2d-a11y` | ✅ paridade |
| `Geometry2D` (booleanas de polígono, envoltória) | interno; não exposto ao script | ◑ |
| relógio, sistema, medidor de desempenho | ◑ | ◑ |
| ficheiros e threads no script | — | ⛔ de propósito: o nosso script é caixa fechada |
| `MovieWriter` (gravar vídeo do motor) | — | ⭐ |

## O que só NÓS temos (ele não tem equivalente nenhum)

| PH2D | |
|---|---|
| **Motion** — grafo de ~134 nós, cozimento na placa, 4,19 M objetos a 3,85 ms | ⭐ |
| **Vetorial** — 14 modos, booleana viva, efeitos de caminho, auto-layout, exportar SVG | ⭐ |
| **Painter** — aquarela, impasto, tinta molhada, escultura de relevo, liquify | ⭐ |
| **Escultura 3D** + retopologia em quads | ⭐ |
| **Modelador 3D por campo** (a peça continua editável para sempre) | ⭐ |
| **Flip** — animação 2D quadro a quadro | ⭐ |
| **Timeline** com clips, nesting, onion, sinais e expressões | ⭐ |
| **16 ferramentas de imagem** (upscale, remover fundo, equalizar cor e tamanho…) | ⭐ |
| **Determinismo gateado** — o mesmo resultado nos três sistemas, com prova no CI | ⭐ |
