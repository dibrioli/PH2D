# Dossiê — Catálogo de componentes/behaviors: Cocos Creator 3.x e Phaser 3/4

> Levantamento para o PH2D (decisão já tomada: componentes estilo Unity/AddComponent).
> Pesquisa em docs oficiais (docs.cocos.com/creator/3.8, docs.phaser.io, phaser.io/news), 2026-08-20.
> Convenção: nomes de componentes em inglês (originais); relevância medida para uma engine 2D voltada a artistas.
> Iluminação 2D: **adiada por decisão do dono (2026-08-20)** — itens relacionados aparecem marcados "adiado", sem prioridade.

---

# PARTE 1 — COCOS CREATOR 3.8

**Modelo:** é o clone mais fiel do modelo Unity — `Node` (cena em árvore) + `AddComponent` no Inspector. Todo comportamento é um componente; componentes do sistema e componentes de script do usuário aparecem no mesmo menu "Add Component". Scripts TypeScript com `@ccclass`/`@property` expõem campos no Inspector automaticamente.
Fonte (modelo/scripting): https://docs.cocos.com/creator/3.8/manual/en/scripting/setup.html

## 1.1 Visual / Render 2D

Fonte da suíte: https://docs.cocos.com/creator/3.8/manual/en/ui-system/components/editor/base-component.html · páginas individuais citadas por item.

| Componente | O que faz | Propriedades na UI | Código que elimina | Compõe com | Relevância 2D |
|---|---|---|---|---|---|
| **Sprite** | Renderiza uma imagem (SpriteFrame) no node. 4 modos: Simple, **Sliced (nine-patch)**, Tiled, **Filled** (barras de progresso/relógios radiais de graça). | SpriteFrame, Type (Simple/Sliced/Tiled/Filled), FillType (H/V/Radial), FillStart/FillRange/FillCenter, Color, Grayscale, SizeMode (Trimmed/Raw/Custom), Trim, CustomMaterial | Nine-slice manual, tiling de fundo, barra de vida radial/linear (o modo Filled É a barra), dessaturação por shader | UITransform, Widget, Button (troca de sprite por estado) | **Alta** — o Filled e o Sliced são os dois modos que mais eliminam código de UI. Fonte: https://docs.cocos.com/creator/3.8/manual/en/ui-system/components/editor/sprite.html |
| **Label** + **LabelOutline** + **LabelShadow** | Texto; outline e sombra são componentes SEPARADOS que se empilham no mesmo node (composição pura). | Fonte, tamanho, cor; outline: cor/largura; shadow: cor/offset/blur | Render de texto, contorno e sombra por shader | Sprite, Widget, Layout | **Alta** — o padrão "efeito de texto = componente empilhável" é elegante |
| **RichText** | Texto com marcação inline (estilos mistos, imagens embutidas, eventos de clique em trechos) | String com tags BBCode-like | Parser de marcação, layout de texto misto, links clicáveis | Label, Button | Média |
| **Mask** | Recorta a renderização dos filhos (retângulo, elipse, stencil por imagem) | Tipo de máscara, inverted | Stencil/clipping manual | **ScrollView** (a viewport É um Mask), minimapa, retratos circulares | **Alta** |
| **Graphics** | Desenho vetorial imediato por API (linhas, círculos, bézier, fill/stroke) como componente | lineWidth, strokeColor, fillColor | Render de debug, indicadores dinâmicos, mira, laços de seleção | qualquer node | Média |
| **TiledMap** / **TiledLayer** | Exibe mapa TMX (Tiled). Ao atribuir o asset, **gera automaticamente um node filho por layer** com TiledLayer. Oclusão node↔tile por linha (um personagem entre duas fileiras de tiles é ocluído corretamente). | Tmx Asset, EnableCulling | Parser TMX, render por camada, culling de tiles, ordenação de profundidade em mapas "2.5D top-down", `addUserNode()` para inserir sprites entre tiles | Física 2D (colisão por camada é manual no Cocos), Camera | **Alta**. Fonte: https://docs.cocos.com/creator/3.8/manual/en/editor/components/tiledmap.html |
| **MotionStreak** | Trilha/rastro atrás de um node em movimento | FadeTime, MinSeg, Stroke (largura), Texture, Color, FastMode, CustomMaterial, Preview no editor | Geometria de fita gerada por movimento, fade por segmento | qualquer node móvel; partículas | **Alta** (barato de implementar, efeito desproporcional). Fonte: https://docs.cocos.com/creator/3.8/manual/en/editor/components/motion-streak.html |
| **UIMeshRenderer** | Renderiza modelo 3D dentro da hierarquia de UI 2D | mesh/material | Compor 3D em telas 2D (herói 3D no menu) | Camera, Widget | Baixa p/ PH2D (já temos sculpt-3D próprio) |
| **UISkew** | Aplica cisalhamento (skew) ao node | skew X/Y | Matriz de transform manual | qualquer | Baixa/Média |

**Não tem:** trail baseado em spline editável; mesh 2D deformável genérico como componente avulso (só via Spine/DragonBones).

## 1.2 Esqueleto / deform 2D

Fonte: https://docs.cocos.com/creator/3.8/manual/en/editor/components/spine.html · https://docs.cocos.com/creator/3.8/manual/en/editor/components/dragonbones.html

| Componente | O que faz | Propriedades na UI | Código que elimina | Relevância |
|---|---|---|---|---|
| **sp.Skeleton (Spine)** | Reproduz animação esquelética exportada do Spine (runtime integrado) | SkeletonData, DefaultSkin, Animation, Loop, TimeScale, **AnimationCacheMode** (REALTIME / SHARED_CACHE / PRIVATE_CACHE), Premultiplied Alpha, DebugSlots/DebugBones/DebugMesh, UseTint, EnableBatch, CustomMaterial, **Sockets** | Runtime de esqueleto, mixing entre animações, troca de skin, **Sockets = prender um node externo a um osso** (arma na mão sem uma linha de código), attachments para colisão | **Alta** — o trio (cache modes p/ hordas, sockets, skins) é o que separa "toca animação" de "sistema de personagem" |
| **DragonBones (ArmatureDisplay)** | Idem para o formato DragonBones | análogo ao Spine | idem | Média (formato em declínio; Spine domina) |

O editor NÃO autora esqueletos — só consome dos DCCs. A autoria de rig 2D dentro do editor é território que Cocos cedeu (e onde o PH2D pode atacar).

## 1.3 Partículas

Fonte: https://docs.cocos.com/creator/3.8/manual/en/particle-system/2d-particle/2d-particle.html

**ParticleSystem2D** — lê `.plist` (formato Particle Designer, ecossistema gigante de presets prontos) OU modo Custom com tudo no Inspector:
- Duration (−1 = contínuo), EmissionRate, Life±var, TotalParticles
- StartColor/EndColor ±var (interpolação automática), StartSize/EndSize, StartSpin/EndSpin
- PosVar, **PositionType** (FREE / RELATIVE / GROUPED — partícula fica no mundo ou segue o emissor)
- **EmitterMode Gravity**: Gravity, Speed±var, TangentialAccel, RadialAccel
- **EmitterMode Radius**: StartRadius/EndRadius, RotatePerS (órbita)
- Preview no editor, PlayOnLoad, autodestruição ao terminar

**Elimina:** todo o ciclo de vida de partículas, interpolações, e — via plist — a própria autoria (importa preset pronto). **Relevância: alta**, mas o modelo é datado (sem emission zones geométricas, sem death zones, sem burst configurável rico — ver Phaser, que é muito superior aqui).

## 1.4 Câmera de gameplay

Fonte: https://docs.cocos.com/creator/3.8/manual/en/editor/components/camera-component.html

**Camera** — Priority, **Visibility (layers por bitmask, 32 layers)**, ClearFlags, ClearColor, Projection (ortho/persp), OrthoHeight, Near/Far, Rect (viewport), TargetTexture (**render-to-texture** direto no Inspector), Aperture/Shutter/Iso.

**⛔ NÃO TEM helpers de gameplay:** sem follow, sem lerp, sem deadzone, sem bounds, sem shake/fade/flash/zoom-to — tudo isso é script do usuário no Cocos. (Contraste direto com Phaser, §2.4 — este é um dos maiores buracos do Cocos.)

## 1.5 Física & colisão 2D

Fontes: https://docs.cocos.com/creator/3.8/manual/en/physics-2d/physics-2d.html · physics-2d-rigid-body.html · physics-2d-collider.html · physics-2d-joint.html

Dois backends selecionáveis (built-in leve p/ só-colisão, Box2D p/ física completa) — **o mesmo componente serve os dois**.

**RigidBody2D** — Type (**Static / Kinematic / Dynamic / Animated**), Group (matriz de colisão), EnabledContactListener (callbacks são opt-in por corpo — perf), Bullet (CCD), AllowSleep, GravityScale, LinearDamping, AngularDamping, LinearVelocity, AngularVelocity, FixedRotation, AwakeOnLoad.
- O tipo **Animated** é uma ideia boa e pouco copiada: kinematic que **deriva velocidade da animação** (mover plataforma por keyframe sem teleportar quem está em cima).

**Collider2D** — BoxCollider2D (Size), CircleCollider2D (Radius), PolygonCollider2D (Points + Threshold). Comuns: **Editing (editar a shape na cena)**, Tag (distinguir colliders do mesmo corpo no callback), Group, **Sensor** (callback sem resposta física = trigger/área), Density, Friction, Restitution, Offset.

**Joints (7):** DistanceJoint2D (MaxLength, AutoCalcDistance) · FixedJoint2D (Frequency, DampingRatio — solda elástica) · **HingeJoint2D** (EnableLimit, Lower/UpperAngle, EnableMotor, MaxMotorTorque, MotorSpeed) · RelativeJoint2D (MaxForce/Torque, CorrectionFactor, offsets) · SliderJoint2D (Angle, motor, limites) · SpringJoint2D (Frequency, DampingRatio, Distance) · **WheelJoint2D** (suspensão + motor = veículo sem código).

**Callbacks:** onBeginContact / onEndContact / onPreSolve / onPostSolve. **Raycast** via API do PhysicsSystem2D. Grupos via matriz de colisão nas configurações do projeto (UI de checkboxes).

**Elimina:** integração física inteira, triggers, veículos (WheelJoint), ragdoll/pêndulo (Hinge+Distance), plataformas móveis (Animated body). **Relevância: alta em tudo** — este catálogo é o mínimo que uma engine 2D com física precisa expor como componentes.

**Materiais físicos:** não são asset separado no 2D — Friction/Restitution moram no collider (mais simples que Unity, menos reusável).

## 1.6 Character controllers PRONTOS

- **2D: NÃO TEM.** Nenhum platformer/top-down controller de prateleira.
- **3D (3.8+): CharacterController** (Box/Capsule) com SkinWidth, `isGrounded`, `move()` — a doc vende exatamente o argumento: "reduz significativamente o custo de desenvolvimento de personagem". Fonte: https://docs.cocos.com/creator/3.8/manual/en/physics/character-controller/index.html
- Lição: a própria Cocos reconheceu (em 3.8, tardiamente, e só no 3D) que CCT é componente de engine, não script de usuário.

## 1.7 Caminhos / splines

**NÃO TEM** componente de path/path-follow. (O editor de animação anima posição com curvas, mas não existe "objeto path na cena + seguidor".)

## 1.8 Animação

Fonte: https://docs.cocos.com/creator/3.8/manual/en/animation/index.html · tween: https://docs.cocos.com/creator/3.8/manual/en/tween/index.html

- **Animation (componente)** — DefaultClip, Clips[], PlayOnLoad; crossfade, eventos de frame (chamam métodos de componentes no mesmo node, com args).
- **Animation Editor** — keyframa **qualquer propriedade de qualquer componente, inclusive `@property` de scripts do usuário**. Este é o ponto central do modelo Unity-like: animação genérica sobre reflexão de componentes, não um sistema por tipo.
- **Marionette (3.4+)** — animation graph: state machine com transições e blend, autorada visualmente (foco esquelético/3D, mas o conceito é geral).
- **Embedded players** — clips podem embutir players de partículas/outras animações (mini-sequencer dentro do clip).
- **Tween (código-only)** — `tween(node).to/by/delay/repeat/sequence/parallel/call` + easings. Sem autoria no editor.

**Elimina:** todo o interpolador; eventos de frame eliminam o "sincronizar som/hitbox com o frame 7 da animação". **Relevância: alta**; o PH2D já tem timeline própria mais poderosa — o item a roubar é **keyframar propriedade de componente arbitrário** e **eventos de frame que invocam handlers**.

## 1.9 Timeline / sequencer
Não existe sequencer de cutscene separado; o Animation Editor + embedded players fazem esse papel. (PH2D já supera aqui.)

## 1.10 Áudio

Fonte: https://docs.cocos.com/creator/3.8/manual/en/audio-system/overview.html

**AudioSource** — Clip, Loop, PlayOnAwake, Volume; `playOneShot()` para SFX sem interromper música.
**⛔ NÃO TEM** áudio posicional 2D nem AudioListener. Sistema minimalista.

## 1.11 Input

Fonte: https://docs.cocos.com/creator/3.8/manual/en/engine/event/event-input.html

Eventos globais (`input.on` — teclado/mouse/touch/acelerômetro) + eventos por node (touch/mouse com propagação pela hierarquia, capturing/bubbling).
**⛔ NÃO TEM action mapping** (nada de "Jump = Espaço OU botão A"). Nenhum componente de input — é tudo código.

## 1.12 UI in-game (a suíte completa — o maior ativo do Cocos)

Fonte da lista: https://docs.cocos.com/creator/3.8/manual/en/ui-system/components/editor/base-component.html · páginas por componente.

| Componente | O que faz / elimina | Propriedades-chave |
|---|---|---|
| **Canvas** | Raiz da UI; adapta à resolução de design | design resolution, fit width/height |
| **UITransform** | Tamanho + âncora do retângulo de UI (separado do Transform) | ContentSize, AnchorPoint |
| **Widget** | **Ancoragem responsiva a bordas do pai** — px ou %; esticar ligando bordas opostas | Top/Bottom/Left/Right, H/VCenter, Target, AlignMode (ONCE/ALWAYS/ON_WINDOW_RESIZE). Elimina TODO o código de adaptação a resolução. Fonte: .../widget.html |
| **Layout** | Auto-arranjo dos filhos | Type (NONE/HORIZONTAL/VERTICAL/GRID), ResizeMode (NONE/**CHILDREN**/**CONTAINER**), Paddings, SpacingX/Y, direções, Constraint FIXED_ROW/COL + ConstraintNum, AffectedByScale. Elimina posicionamento manual de listas/grades/inventários. Fonte: .../layout.html |
| **Button** | Clique + feedback visual por estado | Interactable, Transition (NONE/COLOR/SPRITE/**SCALE**), estados Normal/Pressed/Hover/Disabled, Duration, ZoomScale, **ClickEvents[] (node+componente+método+customEventData) ligados NO INSPECTOR** — zero listener em código. Fonte: .../button.html |
| **ScrollView** | Rolagem com física | Content, Horizontal/Vertical, **Inertia, Brake, Elastic, BounceDuration**, ScrollBar refs, CancelInnerEvents; eventos (scroll-to-top, scrolling…). Compõe com Mask (viewport) + Layout (conteúdo). Elimina toda a física de scroll com momentum e bounce. Fonte: .../scrollview.html |
| **ScrollBar** | Indicador/controle da rolagem | direção, auto-hide |
| **PageView** + **PageViewIndicator** | Rolagem paginada com snap + bolinhas de página | threshold de virada, indicador auto-sincronizado |
| **EditBox** | Campo de entrada de texto (teclado nativo em mobile) | placeholder, máscara de senha, limites, eventos |
| **Toggle** + **ToggleContainer** | Checkbox / grupo radio | isChecked, checkMark, grupo exclusivo |
| **Slider** | Controle deslizante | direção, progress, handle, eventos |
| **ProgressBar** | Barra de progresso | Mode (H/V/Filled), TotalLength, Progress, Reverse |
| **UIOpacity** | Opacidade em cascata para a subárvore | opacity |
| **BlockInputEvents** | Bloqueia input de atravessar (fundo de diálogo modal) — **componente vazio, só presença** | — |
| **SafeArea** | Ajusta ao notch/safe-area de celulares | — |
| **UICoordinateTracker** | Converte/rastreia coordenadas 3D→UI (nametag sobre personagem) | target, offset, eventos |
| **VideoPlayer** | Vídeo (URL/local) com callbacks | clip, playOnAwake, keepAspect. Fonte: .../videoplayer.html |
| **WebView** | Página web embutida | url, callbacks |

**Relevância: alta.** A suíte de UI é o argumento nº 1 de produtividade do Cocos: um menu completo, responsivo, com scroll e navegação, sem UMA linha de código de layout.

## 1.13 Navegação & AI
**NÃO TEM** (sem navmesh, sem agente, sem behavior tree). Terceirizado a plugins/loja.

## 1.14 Spawn / factory / pooling
- **Prefab** (fonte: https://docs.cocos.com/creator/3.8/manual/en/asset/prefab.html): arrastar node → Assets vira prefab; instâncias verdes; **modo de edição do prefab** (duplo-clique) propaga a todas as instâncias; **nested prefabs**; **overrides por instância** com botões revert/apply/unlink no Inspector. É o coração do workflow (idêntico ao Unity).
- **NodePool** (API, não componente): pool por tipo de node, `get()`/`put()`. Fonte: https://docs.cocos.com/creator/3.8/api/zh/class/NodePool
- Sem "Spawner" visual.

## 1.15 Timers
**Scheduler por componente** (fonte: https://docs.cocos.com/creator/3.8/manual/en/scripting/scheduler.html): `this.schedule(cb, interval, repeat, delay)`, `scheduleOnce`, `unschedule` — timers que morrem com o componente (sem leak). Não é componente visual.

## 1.16 Rede / multiplayer
**NÃO TEM** componentes (sem spawner/synchronizer de rede).

## 1.17 Persistência / save
`sys.localStorage` (Web Storage-like; sqlite no nativo). Fonte: https://docs.cocos.com/creator/3.8/manual/en/advanced-topics/data-storage.html — API, não componente.

## 1.18 Parallax / scrolling
**NÃO TEM** componente de parallax (contraste: Phaser tem scrollFactor por objeto).

## 1.19 Componente próprio do usuário
TypeScript: classe `extends Component` + `@ccclass` + `@property` → campos tipados aparecem no Inspector (números, cores, refs a nodes/assets/outros componentes, arrays). Lifecycle: onLoad/onEnable/start/update/lateUpdate/onDisable/onDestroy. **Sem visual scripting nativo** (só de terceiros).
Fonte: https://docs.cocos.com/creator/3.8/manual/en/scripting/setup.html

---

# PARTE 2 — PHASER 3 / PHASER 4

**Modelo:** framework code-first (sem editor de cena oficial no core; Phaser Editor é produto separado). NÃO tem "AddComponent" — o catálogo aqui é de **GameObjects prontos + plugins de Scene** (physics, tweens, time, input, cameras, sound). O valor para o PH2D: Phaser mostra **quais comportamentos o dev 2D usa tanto que merecem vir de graça** — e a granularidade certa deles.
**Phaser 4** saiu em **10/04/2026**: renderer RenderNode, sistema unificado de **Filters** (substitui FX+masks; empilháveis em qualquer objeto/câmera), **SpriteGPULayer** (milhões de sprites, 1 draw call) e **TilemapGPULayer** (tilemap inteiro em 1 quad, custo fixo por pixel). Fontes: https://phaser.io/news/2026/05/phaser-3-vs-phaser-4 · https://phaser.io/news/2026/04/phaser-4-renderer-faster-cleaner-and-built-for-modern-games

## 2.1 Catálogo de GameObjects

Fonte: https://docs.phaser.io/phaser/concepts/gameobjects

| GameObject | O que faz | Relevância 2D |
|---|---|---|
| **Sprite** | textura + animações | alta |
| **Image** | textura sem animação (mais leve) | alta (a distinção leve/completo é boa ideia) |
| **TileSprite** | textura repetida com scroll de UV próprio (`tilePositionX/Y`) — fundo infinito em 1 propriedade | **alta** |
| **NineSlice** | nine-patch redimensionável | alta |
| **Rope** | textura ao longo de uma curva de pontos (bandeiras, cobras, cabos) | média/alta |
| **Mesh** / **Plane** | vértices custom / plano 3D com textura | média |
| **Blitter** (+**Bob**) | render em lote de sprites estáticos baratíssimos | média (superado por SpriteGPULayer no v4) |
| **Container** | agrupamento hierárquico de transform | alta |
| **Layer** | subdivisão do display list (ordenação em bloco) | média |
| **Group** | coleção lógica + **pooling** (§2.7) | **alta** |
| **Graphics** + **Shape**s | desenho vetorial / formas prontas (Rectangle, Arc, Star, Polygon…) | média |
| **Text** / **BitmapText** / **DynamicBitmapText** | texto canvas / bitmap font (barato) / com efeito por caractere | alta |
| **Video** | vídeo como textura | média |
| **Zone** | área invisível interativa (drop zone, trigger de input, spawn area) | alta |
| **RenderTexture** | superfície de desenho offscreen | alta |
| **Shader** / **DOMElement** | shader custom como objeto / HTML sobreposto | média/baixa |
| **ParticleEmitter** | §2.3 | alta |
| **PathFollower** | §2.8 | alta |

**Mixins compartilhados** (o "component set" implícito de todo GameObject): Transform, Alpha (por canto!), BlendMode, Tint, **ScrollFactor (parallax por objeto — 0 = fixo na tela, 0.5 = fundo lento)**, Depth, Mask, Visible, **Data Manager** (key-value com eventos `changedata-…` — "vida mudou → atualiza HUD" sem acoplamento).

## 2.2 Física Arcade — o "character controller barato"

Fonte: https://docs.phaser.io/phaser/concepts/physics/arcade

O **Body** arcade (AABB/círculo, sem rotação de shape) é o achado central do Phaser: um corpo simples com TANTOS knobs declarativos que o platformer/top-down sai quase sem código:

- **Movimento:** velocity, acceleration, **drag (com opção damping exponencial)**, gravity (mundo + por corpo + `allowGravity`), **maxVelocity/maxSpeed**, angular velocity/acceleration/drag.
- **Resposta:** **bounce**, mass, friction (carona em plataforma imóvel!), **slideFactor**, **pushable** (pode ser empurrado?), **immovable**, "direct control mode" p/ objetos movidos por tween/drag.
- **Mundo:** `collideWorldBounds` + evento `worldbounds` + **`wrap(padding)`** (asteroides em 1 chamada) + custom bounds por corpo.
- **Estado de contato (o coração do platformer):** `body.blocked.down/up/left/right`, `body.touching.*`, `body.wasTouching.*` — "está no chão?" é UMA leitura de flag, não um raycast escrito à mão.
- **Colisão declarativa:** `physics.add.collider(a, b, callback, processCallback)` — colisores PERSISTENTES entre objetos/grupos/tilemap layers; `overlap()` = trigger sem separação. Categorias/máscaras por bitmask.
- **Helpers de perseguição:** `moveTo(x,y,speed)`, `moveToObject(obj,speed)`, `accelerateTo(...)`, busca closest/furthest.
- **Debug:** desenho de corpos e vetores de velocidade com um flag.

**Elimina:** o inteiro "primeiro dia de qualquer jogo 2D" — mover, cair, quicar, ficar em pé em plataforma, pegar moeda (overlap), sair da tela e voltar do outro lado. **Relevância: altíssima** — é o melhor modelo do mercado de "física como conjunto de knobs declarativos por objeto". (Matter.js existe como segundo backend para corpos rotacionados/joints.)

## 2.3 Partículas

Fonte: https://docs.phaser.io/phaser/concepts/gameobjects/particles

**ParticleEmitter** — muito mais rico que o do Cocos. Config: speed/speedX/Y radial ou por eixo, gravity, acceleration, maxVelocity, **bounce**, lifespan, delay, hold, quantity, **frequency (−1 = explode)**, duration, stopAfter; scale/alpha/tint/rotate/color com **start→end + easing**; **emitZone** (random OU edge sobre círculo/retângulo/polígono/curva, com yoyo), **deathZone** (entra/sai de geometria → morre), **particleBounds** com colisão por borda; **follow** (emissor gruda num objeto com offset), **gravity wells** (atratores), animação de frames NA partícula, sort, `onEmit`/`onUpdate`. Todo valor aceita: número, range, array random, callback, stepped, curva interpolada.
**Elimina:** praticamente qualquer VFX de gameplay sem shader. **Relevância: altíssima — é o benchmark de API de partículas 2D.**

## 2.4 Câmera de gameplay (o benchmark)

Fonte: https://docs.phaser.io/phaser/concepts/cameras

- **`startFollow(target, roundPixels, lerpX, lerpY, offsetX, offsetY)`** — seguir com suavização por eixo.
- **`setDeadzone(w,h)`** — zona morta onde o alvo anda sem mover a câmera.
- **`setBounds(x,y,w,h)`** — trava nos limites do mundo/fase.
- **Efeitos com duração e callback:** **Shake**, **Fade** (in/out), **Flash**, **Pan** (viaja até um ponto), **Zoom** (animado), **Rotate**. Não se interrompem entre si salvo `force`.
- **`camera.ignore(objs)`** — câmera de UI que não vê o mundo (e vice-versa); múltiplas câmeras (split-screen, minimapa); roundPixels p/ pixel art; culling.
- Parallax não é da câmera: é o **scrollFactor de cada objeto** (design elegantíssimo).

**Elimina:** o segundo maior bloco de código repetido de jogo 2D depois do controller. **Relevância: altíssima.** O contraste com Cocos (que não tem NADA disso) é a prova de que "camera de gameplay" precisa ser produto, não script.

## 2.5 Tweens

Fonte: https://docs.phaser.io/phaser/concepts/tweens

Um objeto de config anima qualquer propriedade de qualquer objeto JS: targets (array!), props com duração/ease individuais, duration, ease (Linear/Cubic/Elastic/Bounce/Back…), delay, **hold**, repeat, **loop**, yoyo, **stagger (com grid!)** — cascatas "cada moeda some 50 ms depois da anterior" em 1 linha; callbacks (onStart/Update/Repeat/Yoyo/Loop/Complete/Stop); **chains** (`tweens.chain()` sequencial); **addCounter** (tween de número puro p/ HUD); valores relativos `'+=600'`, `random(10,100)`, por callback; `updateTo` ao vivo.
**Elimina:** interpolação manual e coreografia de UI/feedback ("juice"). **Relevância: altíssima.**

## 2.6 Tilemaps

Fontes: https://docs.phaser.io/api-documentation/class/tilemaps-tilemap · https://docs.phaser.io/api-documentation/class/tilemaps-tilemaplayer

Import de Tiled (JSON)/CSV; camadas de tile + camadas de objeto; **`setCollisionByProperty({collides:true})`** — o artista marca `collides` no PRÓPRIO Tiled e a engine colide, zero código por fase; setCollision por índice/faixa/exclusão; callbacks por tile; **`createFromObjects(layer, {name/gid/type, classType})`** — a camada de objetos do Tiled vira sprites (da classe que você indicar!) automaticamente; integração direta com Arcade (`collider(player, tileLayer)`); culling. No v4, **TilemapGPULayer** dá custo fixo por pixel.
**Elimina:** o pipeline fase-do-editor-externo→jogo inteiro. **Relevância: alta.**

## 2.7 Groups & pooling (first-class)

Fonte: https://docs.phaser.io/phaser/concepts/gameobjects/group

- **Pooling nativo:** `group.get(x,y)` / `getFirstDead(true, x, y)` reutiliza um membro inativo OU cria (até `maxSize`); **`killAndHide(obj)`** desativa p/ reuso; `getTotalFree()`.
- `createMultiple({key, quantity, setXY: {stepX}})` — 50 moedas espaçadas em 1 chamada.
- **`runChildUpdate`** — chama `update()` de cada membro automaticamente.
- **Ações em lote:** setAll, incXY, playAnimation, propertyValueInc sobre o grupo.
- **Physics groups:** membros já nascem com body; grupos são não-exclusivos (um objeto em N grupos).

**Elimina:** o padrão bala/inimigo/moeda inteiro (spawn, reciclagem, limite, update). **Relevância: altíssima** — pooling como conceito de PRIMEIRA classe da API, não técnica avançada.

## 2.8 Caminhos / splines

Fontes: https://docs.phaser.io/api-documentation/class/gameobjects-pathfollower · https://docs.phaser.io/api-documentation/class/curves-path

**Path** (composto de linhas, béziers quadr./cúbicas, splines Catmull-Rom, elipses) + **PathFollower** (Sprite que anda no path): `startFollow({duration, from, to, rotateToPath, rotationOffset, yoyo, repeat, ease, delay, startAt})`, pause/resume/stopFollow, pathOffset.
**Elimina:** patrulha de inimigo, plataformas em trilho, projéteis em arco, cutscene de movimento. **Relevância: alta.**

## 2.9 Animação de sprites

Fonte: https://docs.phaser.io/phaser/concepts/animations

AnimationManager global (anims compartilhadas) ou locais por sprite; `generateFrameNumbers`/`generateFrameNames` (padrões de atlas); config: frameRate OU duration, repeat (−1), yoyo, delay, repeatDelay, **skipMissedFrames**, showOnStart/hideOnComplete; `play`, `chain` (fila), `playAfterDelay/AfterRepeat`, eventos por animação E por frame.
**Elimina:** troca manual de frames e sincronização. **Relevância: alta.**

## 2.10 Timeline/sequencer
Sem timeline visual (é framework). `tweens.chain` + Timeline class (código) cobrem sequências. **Declarado: não tem no sentido de editor.**

## 2.11 Áudio (com posicional 2D!)

Fonte: https://docs.phaser.io/phaser/concepts/audio

WebAudio/HTML5 auto; `sound.play(key)`; volume/mute/rate/**detune (±1200 cents)**/pan/seek; **markers** (seções nomeadas num arquivo) e **audio sprites**; **áudio espacial** (3.60+): `setListenerPosition(x,y)`, posição por som, `follow` de objeto, modelos de pan/distância (equalpower/HRTF); analyser p/ visualização; unlock de autoplay.
**Relevância: alta** — pan por distância da câmera é barato e o Cocos não tem.

## 2.12 Input

Fonte: https://docs.phaser.io/phaser/concepts/input

`setInteractive()` com hit areas geométricas ou **pixel-perfect**; eventos pointer unificados mouse/touch (com coordenadas de MUNDO por câmera); **drag & drop declarativo**: `setDraggable` + eventos dragstart/drag/dragend + **drop zones** (Zone + dragenter/dragover/drop); teclado (`addKey`, **`createCursorKeys()`**, combos); gamepad completo; mouse wheel.
**⛔ Sem action mapping** (igual ao Cocos — buraco da categoria inteira; só Godot/Unity têm).
**Elimina:** normalização de dispositivos e TODO o protocolo de drag&drop. **Relevância: alta.**

## 2.13 UI in-game
**Declarado: NÃO TEM suíte de UI** (sem button/scroll/layout — só Text/BitmapText/NineSlice/Zone como matéria-prima; ecossistema supre via rexUI, de terceiros). Contraste absoluto com Cocos.

## 2.14 Navegação & AI
**NÃO TEM** (sem pathfinding/navmesh/behavior tree no core).

## 2.15 Rede / multiplayer
**NÃO TEM** no core.

## 2.16 Persistência / save
**NÃO TEM** API própria (localStorage do browser, manual). Tem **Data Manager** (key-value com eventos) por objeto/scene/registry global — excelente p/ estado observável, não persistente.

## 2.17 Timers
Fonte: https://docs.phaser.io/phaser/concepts/time — `time.addEvent({delay, loop, repeat, callback, paused, timeScale})`, `delayedCall`; Clock por Scene sincroniza tweens/anims/som; **timeScale por timer** (slow-motion seletivo); pause/resume/progress.
**Relevância: alta** (elimina acumulação de dt à mão).

## 2.18 FX / Filters (pós-processo por OBJETO)

Fontes: https://docs.phaser.io/phaser/concepts/fx · https://phaser.io/news/2026/05/phaser-3-vs-phaser-4

- **3.60:** `obj.preFX.addGlow()` / `obj.postFX.addBloom()` — 14 efeitos prontos por chamada única: Barrel, Bloom, Blur, Bokeh/TiltShift, Circle outline, ColorMatrix (sépia/grayscale/…), Displacement, **Glow**, Gradient, Pixelate, **Shadow**, **Shine**, Vignette, **Wipe/Reveal** — em qualquer objeto ou câmera.
- **Phaser 4:** vira o sistema unificado **Filters** (empilháveis, qualquer objeto/câmera, +16× perf mobile; inclui color grading e **image-based lighting** — este último: *iluminação → adiado por decisão do dono*).

**Elimina:** shaders de feedback (hit flash, glow de selecionado, transições de tela). **Relevância: altíssima para "juice" sem código.**

## 2.19 Actions (operações em lote)

Fonte: https://docs.phaser.io/phaser/concepts/actions

Funções estáticas sobre ARRAYS de objetos: **GridAlign**, AlignTo, **PlaceOnCircle/Ellipse/Line/Rectangle/Triangle**, RandomCircle/Rectangle/…, Rotate/RotateAround, IncX/Y/XY, SetScale/Alpha/Tint/Depth/BlendMode/HitArea, Shuffle, SmoothStep, Spread, **ShiftPosition** (movimento-cobra), ToggleVisible.
**Elimina:** loops de arranjo (menu circular, formação de inimigos, grade de cartas). **Relevância: média/alta** — barato de implementar, ótimo retorno.

## 2.20 Scenes
Fonte: https://docs.phaser.io/phaser/concepts/scenes — cenas paralelas (gameplay + UI + pause), launch/sleep/wake/pause, transições, registry global, cada cena com seus plugins (cameras/tweens/time/input/physics próprios). Modelo de estruturação de jogo melhor que o do Cocos (uma cena só). **Relevância: alta como arquitetura de referência.**

## 2.21 Componente próprio
Sem sistema de componentes: herda-se de GameObject/Sprite ou compõe-se em classes JS/TS. Phaser Editor (produto irmão) fornece prefabs visuais + "script nodes" de terceiros. **Declarado: sem visual scripting no core.**

---

# MATADORES DE CÓDIGO — os 15 que mais eliminam programação só com UI/config

Ordenados por (código eliminado × frequência de uso num jogo 2D):

1. **Arcade Body (Phaser)** — drag/bounce/maxVelocity/gravity/`blocked.down`/wrap/collideWorldBounds como knobs: o "character controller barato" que resolve o primeiro dia de TODO jogo 2D sem uma linha de física.
2. **Camera gameplay (Phaser)** — startFollow(lerp) + deadzone + bounds + shake/fade/flash/pan/zoom prontos: o segundo maior bloco de código repetido do 2D, inteiro de graça. (Cocos não tem — e dói.)
3. **Widget (Cocos)** — ancoragem responsiva px/% a qualquer borda do pai: elimina TODO o código de adaptação de resolução de UI.
4. **ScrollView + Mask + ScrollBar (Cocos)** — scroll com inércia, brake e bounce elástico por checkbox: a física de scroll que todo mundo escreve errado, pronta.
5. **Layout (Cocos)** — HORIZONTAL/VERTICAL/GRID + ResizeMode CHILDREN/CONTAINER: inventários, hotbars e menus sem posicionar um único filho à mão.
6. **Button com ClickEvents no Inspector (Cocos)** — estados visuais (color/sprite/scale) + handler apontado por UI (node→componente→método→arg): o listener que nunca foi escrito.
7. **Group/pooling (Phaser)** — `get()`/`killAndHide()`/`maxSize`/`createMultiple`/`runChildUpdate`: o padrão bala/inimigo/moeda como primitiva da engine.
8. **ParticleEmitter (Phaser)** — emitZone/deathZone/follow/gravity wells/explode-flow, tudo config: VFX de gameplay inteiro sem tocar em código de partícula.
9. **Tween com stagger/chain/yoyo (Phaser)** — coreografia de "juice" (cascatas, pulsos, contadores de HUD) num objeto de config.
10. **setCollisionByProperty + createFromObjects (Phaser Tilemaps)** — o artista marca `collides` e nomeia spawns NO Tiled; a fase colide e se popula sozinha.
11. **Sprite modo Filled + Sliced (Cocos)** — barra de vida linear/radial e nine-patch são um dropdown do componente de sprite, não uma feature.
12. **PathFollower (Phaser)** — patrulhas, trilhos e arcos: `startFollow({duration, rotateToPath, yoyo})` sobre um path desenhável.
13. **preFX/postFX / Filters (Phaser 3.60/4)** — glow/shadow/bloom/pixelate/wipe por objeto em UMA chamada: o feedback visual que exigiria shader.
14. **Prefab com nested + overrides (Cocos)** — template vivo com edição propagada e variação por instância: o multiplicador de todo o resto do editor.
15. **Joints 2D prontos, WheelJoint à frente (Cocos/Box2D)** — veículo com suspensão e motor, ponte de corda, pêndulo: física composta por AddComponent.

Menções honrosas: **MotionStreak** (Cocos — trail em 6 propriedades) · **Sockets do Spine** (Cocos — arma na mão do osso sem código) · **BlockInputEvents** (Cocos — modal correto por PRESENÇA de um componente vazio) · **Actions.PlaceOnCircle/GridAlign** (Phaser) · **Data Manager com eventos** (Phaser — HUD reativo) · **RigidBody2D tipo Animated** (Cocos — plataforma por keyframe sem teleporte) · **body.wrap()** (Phaser — asteroides em 1 chamada) · **áudio espacial 2D** (Phaser).

---

# BURACOS DAS DUAS (categorias sem dono — oportunidade PH2D)

- **Action mapping de input:** NENHUMA das duas tem ("Jump = Espaço OU A do gamepad"). Godot/Unity têm; num modelo de componentes, seria asset + componente leitor.
- **Character controller 2D pronto:** nenhuma tem (Cocos só ganhou o de 3D em 3.8, confessando a demanda). O Arcade Body do Phaser chega a 80% mas ainda exige o script de "ler input → aplicar velocity".
- **Navegação/AI (navmesh, agente, behavior tree):** zero nas duas.
- **Rede (spawner/synchronizer):** zero nas duas.
- **Save/persistência como sistema** (slots, o-que-persiste-por-objeto): zero — só localStorage cru.
- **Autoria de esqueleto 2D no editor:** as duas só CONSOMEM Spine/DragonBones.
- **Sequencer de cutscene:** nenhuma (o PH2D com timeline + sinais já está à frente).
- **Iluminação 2D:** Phaser tem Light2D/LightManager (diffuse+normal maps) e o v4 traz image-based lighting nos Filters; Cocos 2D não tem sistema de luz 2D dedicado no manual — **[adiado por decisão do dono, 2026-08-20; listado sem prioridade]**.

# LEITURA CRUZADA (o que cada uma prova)

- **Cocos prova:** a suíte de UI COMPLETA como componentes (Widget/Layout/ScrollView/Button…) é o maior redutor de código de um editor Unity-like; e que "keyframar qualquer @property de qualquer componente" é a cola entre animação e componentes.
- **Phaser prova:** os cinco sistemas que o dev 2D usa em TODO jogo — body arcade declarativo, câmera de gameplay, tween/stagger, pooling, partículas ricas — devem ser produto da engine, não scripts de usuário; e que efeitos por objeto (FX/Filters) são o "juice" mais barato por linha de API.
- **A soma (Cocos ∪ Phaser) ≈ o catálogo mínimo** que um artista espera "de graça" numa engine 2D moderna; os buracos comuns (input mapping, CCT 2D, AI, save, rede) são onde uma engine nova diferencia.

---

## Fontes principais
- Cocos Creator 3.8 Manual: https://docs.cocos.com/creator/3.8/manual/en/ (UI: ui-system/components/editor/… · Física 2D: physics-2d/… · Animação: animation/ · Partículas: particle-system/2d-particle/ · TiledMap/Spine/MotionStreak/Camera: editor/components/… · Prefab: asset/prefab.html · Scheduler: scripting/scheduler.html · Storage: advanced-topics/data-storage.html · CCT 3D: physics/character-controller/)
- Phaser Docs: https://docs.phaser.io/phaser/concepts/… (gameobjects, physics/arcade, cameras, tweens, animations, audio, input, time, fx, actions, scenes, gameobjects/particles, gameobjects/group) · API: docs.phaser.io/api-documentation (PathFollower, Tilemap/TilemapLayer)
- Phaser 4: https://phaser.io/news/2026/05/phaser-3-vs-phaser-4 · https://phaser.io/news/2026/04/phaser-4-renderer-faster-cleaner-and-built-for-modern-games
