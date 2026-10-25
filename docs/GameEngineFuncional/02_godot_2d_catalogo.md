# W2 — O catálogo 2D da Godot, contra o que a PH2D tem

> Pedido do dono (25/10): *«vamos descobrir os objetos e suas funções, listá-los do mais essencial ao mais
> especializado e usar como referência para tornar essa engine totalmente funcional como a Godot em cada
> mínimo aspecto de sua engine 2D […] mesmo usando uma arquitetura baseada em componentes como em Unity»*.

## Fonte e método (oráculo CORRIDO, não lido — CLAUDE.md §0.9)

- **Godot 4.7.2** (pacote Arch `godot 4.7.2-2.1`, **MIT**). Corrido sem janela:
  `PH2D_GPU=1 bash scripts/ph2d-run.sh godot --headless --doctool <dir>` → 1 076 classes em XML.
- Extractor versionado: [`ferramentas/godot_tree.py`](ferramentas/godot_tree.py) `<dir> <saida.json>` — a árvore a
  partir de `Node`, sem o 3D, o editor e o XR. Saída gravada: [`godot/godot_4.7.2_nos_2d.json`](godot/godot_4.7.2_nos_2d.json)
  (**138 classes**: 51 sob `Node2D`, 1 `CanvasItem`, 22 nós sem posição, 64 de interface `Control`), cada uma
  com a cadeia de herança, a descrição curta e as propriedades, métodos e sinais PRÓPRIOS.
- Inventário da PH2D: tipos de objecto (`ObjectKind`, 7), o menu Add (2D · Game · Empty), ~165 componentes
  em `crates/ph2d-component-desc/src/catalog/*.rs`, a API Luau (`crates/ph2d-script/src/api_table.rs`).

## ⭐ O diagnóstico numa frase

**A PH2D não está atrás da Godot em LARGURA — está atrás em ORGANIZAÇÃO.** Dos 51 objetos 2D, a PH2D já
cobre a maioria por componentes (câmara, corpos, juntas, raio, navegação, luz, esqueleto, áudio 2D,
partículas, timers, parallax, HUD). O que a Godot tem e nós não temos é a **porta**: na Godot o artista
escolhe *um objecto com nome e ícone* (`CharacterBody2D`) e ele já vem com as propriedades, as funções do
script (`move_and_slide`, `is_on_floor`) e os sinais (`body_entered`, `timeout`) daquele papel. Na PH2D o
artista tem de saber que `PlatformPlayer` + `Collider` + `RigidBody` + `SignalOnHit` fazem um personagem.

## O catálogo, do essencial ao especializado — e onde a PH2D está

Legenda: ✅ existe · 🟡 parcial / espalhado em componentes · ❌ falta. *(dep.)* = obsoleto na Godot 4.7.

### T0 — Fundação (sem isto nada é jogo)

| Godot | O que é | PH2D |
|---|---|---|
| `Node` | árvore, ciclo de vida (`_ready`, `_process`, `_physics_process`), grupos, sinais, pausa | 🟡 árvore + `Tags` + sinais (`ph2d-runtime`); script só `init`/`update`/`on_signal` |
| `CanvasItem` | visível, `modulate`, `z_index`, ordem de desenho, `y_sort`, material | ✅ `Visibility`, `SortingLayer`, `OrderInLayer`, `YSort`, `BlendMode`, `Mask2D` |
| `Node2D` | posição, rotação, escala, herança de transformação | ✅ `Transform`, `GroupedChildren`, `TopLevel` |
| Cena instanciável (`PackedScene`) | um objecto reutilizável, com variações | ✅ `InstanceOf` / `ObjectInstance`, variantes |
| `Timer` | conta e avisa (`timeout`) | ✅ `Timers` |

### T1 — O jogo mínimo (um plataforma jogável)

| Godot | PH2D |
|---|---|
| `Sprite2D` | ✅ `Sprite` (+ `SliceNine`, `SpriteRegion`) |
| `AnimatedSprite2D` | ✅ `SpriteAnimator` + `SpriteAnimations` |
| `AnimationPlayer` | 🟡 `ph2d-anim`, Timeline e clips do objecto; tocar no jogo por confirmar |
| `Camera2D` | ✅ `GameCamera`, `CameraFollow`, `CameraLimits`, `CameraShake` |
| `CollisionShape2D` · `CollisionPolygon2D` | ✅ / 🟡 `Collider` |
| `StaticBody2D` | 🟡 sem objecto próprio (corpo fixo dentro do `RigidBody`) |
| `CharacterBody2D` (`move_and_slide`) | 🟡 `PlatformPlayer` / `TopDownPlayer` — controladores prontos, sem a função genérica |
| `RigidBody2D` | ✅ `RigidBody` (+ `Ccd`, `GravityScale`, massa…) |
| `Area2D` (`body_entered`/`exited`) | 🟡 `Area*` + `SignalOnHit`/`SignalOnLeave` |
| `AudioStreamPlayer` · `AudioStreamPlayer2D` | ✅ `AudioSource2D` |
| `CanvasLayer` + `Label`/`Button`/`TextureRect`/`ProgressBar` (HUD) | 🟡 `UiCanvas`, `UiLabel`, `UiButton`, `Counter`, `HealthBar` |
| `TileMapLayer` (`TileMap` *dep.*) | ❌ — projecto à parte `docs/Tilling` (dono, 16/09) |
| Mapa de entrada (InputMap) | 🟡 `ph2d-input` + `key_*`/`mouse_*` no Luau |

### T2 — Mundo e jogabilidade

| Godot | PH2D |
|---|---|
| `RayCast2D` | ✅ `RaySensor` (+ `RaySignals`) |
| `ShapeCast2D` | ❌ |
| `AnimatableBody2D` (plataforma móvel) | 🟡 corpo cinemático por confirmar |
| `Marker2D` | 🟡 objecto Empty |
| `Path2D` + `PathFollow2D` | 🟡 `PathFollow` sem o caminho como objecto |
| `Parallax2D` (`ParallaxBackground`/`Layer` *dep.*) | 🟡 `ScrollFactor`, `ScrollRepeat`, `ScrollMotion` |
| `VisibleOnScreenNotifier2D` / `Enabler2D` | ✅ `OnScreenEnabler` |
| `RemoteTransform2D` | ❌ |
| `CPUParticles2D` / `GPUParticles2D` | 🟡 `ParticleEmitter` (CPU?) — e o Motion Nodes cozinha na GPU |
| `Line2D` / `Polygon2D` | 🟡 `VecShape` (o vetor é mais rico; falta o «objecto linha» simples) |
| `NavigationRegion2D` / `Agent2D` / `Obstacle2D` / `Link2D` | ✅ `NavRegion`, `NavAgent`, `NavLink`, `NavCostArea` |
| `AnimationTree` (máquina de estados) | 🟡 `StateMachine`, `SequencePlayer` |
| `PinJoint2D` · `GrooveJoint2D` · `DampedSpringJoint2D` | ✅ `PhysicsJoint` (+ polia, eixo) |

### T3 — Visual avançado

| Godot | PH2D |
|---|---|
| `PointLight2D` · `DirectionalLight2D` · `LightOccluder2D` · `CanvasModulate` | ✅ Luz 2D (`Luz`, `BloqueiaLuz`, luz ambiente) — mais rica que a Godot |
| `Skeleton2D` · `Bone2D` · `PhysicalBone2D` | ✅ / ✅ / ❌ — esqueleto com IK e pele |
| `CanvasGroup` · `BackBufferCopy` | 🟡 `SortingGroup`, `ClipChildren`, `Mask2D` |
| `MeshInstance2D` · `MultiMeshInstance2D` | ❌ |
| `SubViewport` (minimapa, ecrã dentro do jogo) | ❌ |
| `WorldEnvironment` (glow 2D) | 🟡 efeitos da camada (`ph2d-fx-cells`) |

### T4 — Especializado / infra

`TouchScreenButton` ❌ · `AudioListener2D` ✅ · `HTTPRequest` ❌ · `MultiplayerSpawner`/`Synchronizer` ❌ ·
`ResourcePreloader` ❌ · `ShaderGlobalsOverride` ❌ · `StatusIndicator` ❌ · `Window` ❌ · `InstancePlaceholder` ❌.

### Interface (`Control`, 64 classes — o HUD e os menus do jogo)

O essencial: `Control`, `Label`, `RichTextLabel`, `Button`, `TextureButton`, `CheckBox`, `TextureRect`,
`NinePatchRect`, `ColorRect`, `Panel`, `ProgressBar`, `TextureProgressBar`, `HSlider`, `LineEdit`, e os
contentores `HBox`/`VBox`/`Grid`/`Margin`/`Center`/`Panel`/`ScrollContainer`, `TabContainer`. A PH2D tem
`UiCanvas`/`UiLabel`/`UiButton` + `VecWidget`/`VecLayout` (vetor) — ❌ os contentores de layout para o jogo.

## O que falta FORA do catálogo (é aqui que a Godot ganha)

1. **A janela de jogo e o Play**: hoje o jogo corre no próprio canvas (`Playhead::is_playing`, F5); não há
   janela de jogo separada nem build exportado (`shells/desktop` é só o editor).
2. **O ciclo de vida do script**: só `init`/`update(dt)`/`on_signal`; faltam `ready`, `physics`, colisão
   (`body_entered`), entrada e o fim (`exit_tree`).
3. **Funções e sinais POR TIPO de objecto**: na Godot o tipo é a documentação (`CharacterBody2D.move_and_slide`,
   `Timer.timeout`, `AnimationPlayer.animation_finished`); o Luau da PH2D é genérico (`ph2d.get/set`).
4. **Ligar sinais no editor** (o painel *Node ▸ Signals* da Godot): escolher o sinal de um objecto e a função de outro.

## Proposta: componentes por dentro, OBJECTOS por fora

A arquitectura não muda (ECS, ADR-0075). Muda a porta: **cada objecto do catálogo Godot vira um TIPO no menu
Add, que é um Empty + um conjunto FIXO de componentes**, com o nome, o ícone, a secção do Inspector, as funções
de script e os sinais desse papel. Os componentes continuam combináveis à mão (a liberdade da Unity), mas o
caminho óbvio é o objecto (a clareza da Godot).

Fases propostas (cada uma fecha com smoke do dono):

| Fase | O quê | Por que nesta ordem |
|---|---|---|
| **F1 — O catálogo T0+T1 como objectos** | menu Add com os objectos do T1 (Sprite, Animated Sprite, Camera, Static/Character/Rigid Body, Area, Collision Shape, Audio Player, Timer, Label, Button…), cada um um conjunto fixo de componentes; Inspector por objecto | é o que mais muda a facilidade de uso, sobre o que JÁ existe |
| **F2 — Play numa janela de jogo** | Play (F5) abre o jogo numa janela própria, a partir da cena; pausa e paragem devolvem o editor intacto | sem isto não é uma engine, é um editor que anima |
| **F3 — O ciclo de vida e os sinais por objecto** | `ready`/`process`/`physics`/colisão no Luau; cada tipo publica os seus sinais e funções; ligar sinais no editor | é o «conjunto de funções reconhecidas como ligadas ao objecto» do pedido |
| **F4 — As lacunas T1/T2** | `CharacterBody` genérico (`move_and_slide`), `StaticBody`, `ShapeCast`, `RemoteTransform`, `Path2D` como objecto, `Line2D` simples | completa o jogo mínimo e o mundo |
| **F5 — Interface do jogo** | contentores de layout (HBox/VBox/Grid/Margin), `TextureRect`, `ProgressBar`, temas | menus e HUD de verdade |
| **F6 — Exportar o jogo** | um executável que corre a cena sem o editor | fechar o ciclo |
| depois | TileMapLayer (projecto `docs/Tilling`), SubViewport, MeshInstance2D, toque, rede | especializados |

## Perguntas ao dono (decidem a F1)

1. **Os nomes:** usar os nomes da Godot (`CharacterBody2D`, `Area2D`) ou nomes próprios mais simples
   (`Personagem`, `Zona`)? *Recomendação: nomes simples em inglês (HR-15), com a correspondência Godot no
   tooltip — mais fácil que a Godot, sem perder quem vem dela.*
2. **A janela de jogo:** janela separada (como a Godot) ou dentro do canvas em ecrã cheio? *Recomendação:
   janela separada.*
3. **A ordem das fases** acima.
