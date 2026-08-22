# Dossiê — Unreal Engine 5: catálogo de componentes e gameplay framework (lente: o que traduz para uma engine 2D)

> Pesquisa em docs oficiais (dev.epicgames.com) + doc oficial do plugin PaperZD (Critical Failure Studio).
> Data: 2026-08-20. Contexto do levantamento: PH2D adotou componentes estilo Unity (AddComponent), não herança de nodes.
> Convenção: cada item traz **Nome** · o que faz · propriedades de UI · código que elimina · sinergias · relevância 2D (alta/média/baixa).
> Itens marcados **[API]** foram confirmados só pela referência de API, não por página de manual.

---

## 0) Modelo mental da UE5 — como o "AddComponent" deles funciona

- **Actor** = objeto de cena; **Component** = unidade de comportamento/representação que se ADICIONA ao Actor. Três camadas:
  - **UActorComponent** — base de todos; comportamento abstrato sem presença física (ex.: movimento, inventário). Tem tick próprio, ativação/desativação e replicação.
  - **USceneComponent** — ActorComponent + **Transform** e **attachment** (hierarquia pai-filho DENTRO do actor). Câmeras, spring arms, áudio.
  - **UPrimitiveComponent** — SceneComponent + **geometria** (renderiza e/ou colide). Meshes, sprites, shapes de colisão.
- Diferença chave vs Unity: a UE tem além dos components o **Gameplay Framework** (GameMode/GameState/PlayerState/Controller/Pawn) — classes-papel que resolvem "quem é o dono da regra, quem é o dono do estado do jogador, quem possui o corpo". Isso elimina uma classe inteira de código de arquitetura que todo jogo reescreve.
- **Lição para o PH2D:** o par (components no objeto) + (papéis de jogo prontos fora do objeto) é o que faz a UE "dar jogo de graça". Componentes sozinhos não entregam isso.

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/components-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-framework-in-unreal-engine

---

## 1) Visual / render 2D (Paper 2D)

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/paper-2d-overview-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/paper-2d-components?application_version=4.27 · https://dev.epicgames.com/documentation/en-us/unreal-engine/paper-2d-tile-sets-and-tile-maps-in-unreal-engine · https://dev.epicgames.com/documentation/unreal-engine/paper-2d-flipbooks-in-unreal-engine

- **PaperSpriteComponent** — renderiza UM sprite (asset editável no Sprite Editor: geometria de render, pivô, colisão 2D por sprite). Props: Source Sprite, material override, cor. Elimina: draw + gestão de quad/atlas/colisão do sprite. Sinergia: shapes de colisão, materiais. **Relevância 2D: alta** (é o átomo).
- **PaperFlipbookComponent** — animação por sequência de sprites (asset Flipbook = keyframes {sprite, duração}). Props: SourceFlipbook, Play Rate, direção, looping, vertex color, material override, Mobility=Movable p/ trocar em runtime. Elimina: todo o player de animação por frames (timer + troca de textura + eventos de fim). Sinergia: PaperZD (estado decide QUAL flipbook), Paper2DCharacter (já vem com um). **Alta**.
- **PaperTileMapComponent** — renderiza um Tile Map (asset editado em editor próprio: camadas, pintar/borracha/balde, flip/rotate de tile, dimensões). Tile Set com colisão POR TILE (Box/Circle/Polygon), margem/spacing, import de **Tiled (.json)** que gera Tile Set + Tile Map automaticamente. Elimina: todo o renderer de tilemap + geração de colisão do cenário. **Alta** — e o import de Tiled é um detalhe de adoção que vale copiar.
- **Componentes 3D reutilizados em 2D:** StaticMeshComponent (geometria pronta), SkeletalMeshComponent (malha animada — é como se faz "2.5D" na UE). **Média**.
- **Nine-patch / trail / line:** NÃO há componente de cena dedicado. Nine-slice existe só no UMG (UI); trilhas são feitas com o **Ribbon Renderer** do Niagara; linha de debug é API, não componente. **Categoria declarada: ausente como componente de cena.**

---

## 2) Esqueleto / deformação 2D

**A UE NÃO tem esqueleto 2D nativo.** Paper 2D é sprite/flipbook puro; PaperZD anima ESTADOS, não bones. Quem quer bones 2D na UE usa SkeletalMesh 3D achatado ou plugins de terceiros (fora do escopo desta doc oficial). **Categoria declarada: ausente.** — Isso é um dos maiores buracos da UE para 2D e uma oportunidade direta do PH2D (que já tem sculpt/deform próprios).

---

## 3) Partículas — Niagara

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/creating-visual-effects-in-niagara-for-unreal-engine · https://dev.epicgames.com/documentation/unreal-engine/render-module-reference-for-niagara-effects-in-unreal-engine

- **NiagaraComponent** (adiciona um System ao actor) — o sistema é composto de **Emitters**, cada um uma pilha de **Modules** (spawn, forças, vida, cor…) 100% editáveis/parametrizáveis no editor; usuários avançados criam módulos novos no Niagara Script Editor sem tocar C++.
- **Renderers por emitter:** **Sprite Renderer** · **Ribbon Renderer** (trilhas/rastros — o "trail component" da UE na prática) · **Mesh Renderer** · **Light Renderer** (partícula emite luz). Um emitter pode ter vários renderers.
- Extras: fluids, debugger com análise de perf em tempo real, Debug Drawing.
- Elimina: o motor de partículas inteiro E a linguagem de scripting de efeitos — o artista compõe comportamento por pilha de módulos com parâmetros expostos.
- Sinergia: Sequencer (dispara/anima), eventos de gameplay, PaperZD notifies (spawn de efeito em frame X).
- **Relevância 2D: alta** (sprite+ribbon cobrem 90% do VFX 2D). Nota: o PH2D já mediu 4,19M partículas na GPU — o diferencial da UE não é throughput, é a **UI de pilha de módulos**.

---

## 4) Câmera de gameplay

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/using-spring-arm-components-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/camera-shakes-in-unreal-engine

- **CameraComponent** — o ponto de vista; projeção/FOV; vira a view quando o Controller "possui" o pawn. **Alta**.
- **SpringArmComponent** — o "follow" da indústria: mantém a câmera a TargetArmLength do alvo, **testa colisão** (bDoCollisionTest, ProbeSize) pra nunca atravessar parede, e aplica **lag** configurável (suavização de posição/rotação com velocidade própria), SocketOffset. Elimina: todo o código de câmera-que-segue com suavização e anti-clipping. Em 2D vira: follow com deadzone/lag + limites. **Alta** (adaptado).
- **Camera Shakes (CameraShakeBase)** — assets de shake SEM código: padrões **Perlin Noise** (explosão), **Sinusoidal** (balanço), **Sequence** (autorado à mão) e **Composite** (camadas); amplitude/frequência POR EIXO, multiplicadores, blend in/out. **Alta**.
- **UCameraShakeSourceComponent** — shake POSICIONAL: raio interno/externo, falloff (Linear/Quadratic), auto-start — treme mais perto da fonte, sem uma linha de código. **Alta**.
- Transições entre câmeras: `SetViewTargetWithBlend` (API do PlayerController, com curva/duração) — não é componente. Zonas de câmera e limites de mundo: **ausentes como componente** (cada jogo escreve o seu — buraco da UE que Cinemachine/Unity preenche; oportunidade PH2D).

---

## 5) Física & colisão

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/components-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/physics-constraint-component-user-guide-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/physics-constraint-reference-in-unreal-engine · https://docs.unrealengine.com/5.0/en-US/API/Runtime/Engine/PhysicsEngine/URadialForceComponent/ (família de physics components)

- **Shape components (BoxComponent, CapsuleComponent, SphereComponent)** — colisão invisível; cada primitive tem presets de colisão por canal (Block/Overlap/Ignore), **Generate Overlap Events** (o "sensor/área" da UE: eventos OnBeginOverlap/OnEndOverlap por checkbox), Simulate Physics, Physical Material. Elimina: broadphase manual, eventos de trigger. **Alta**.
- **PhysicsConstraintComponent** — junta genérica configurável que vira QUALQUER joint só por UI:
  - Linear: XMotion/YMotion/ZMotion = Free/Limited/Locked + Limit; soft constraint (Stiffness/Damping/Restitution), **Linear Plasticity**, breakable com threshold.
  - Angular: Swing1/Swing2/Twist = Free/Limited/Locked + ângulos; soft por eixo; **Angular Plasticity**; breakable por torque.
  - **Motores/drives**: linear (alvo de posição/velocidade + força máx) e angular (modos SLERP ou Twist-and-Swing, alvo de orientação/velocidade).
  - Disable Collision entre os corpos ligados; projeção linear/angular; Shock Propagation p/ correntes.
  - Elimina: hinge/spring/motor/rope/ragdoll codificados um a um — UMA superfície de UI cobre a família toda. **Alta** (o rapier2d do PH2D tem os joints; a lição é a UI unificada).
- **RadialForceComponent** — força/impulso radial "fire-and-forget" (explosão empurrando tudo no raio). **Alta**.
- **PhysicsThrusterComponent** — força CONTÍNUA no -X do componente (foguete); auto-activate. **Média**.
- **PhysicsHandleComponent** — "agarrar" um corpo físico e movê-lo mantendo a física viva (gravity gun, drag&drop físico com mouse). Elimina: o spring-para-o-cursor que todo jogo físico reescreve. **Alta**.
- **Raycast/shapecast:** API (Line Trace / Sweep por canal), não componente. **Physical Materials:** asset de fricção/restituição aplicado por material. **Declarados: existem, não são componentes.**

---

## 6) Character controllers PRONTOS

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/movement-components-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/GameFramework/UCharacterMovementComponent

### CharacterMovementComponent — o controller mais rico da indústria
Locomoção SEM rigid body (kinemático com colisão própria), embutido em todo `Character`.

- **Modos de movimento (EMovementMode):** `Walking` · `NavWalking` (anda projetado no navmesh, colisão simplificada) · `Falling` · `Swimming` · `Flying` · `Custom` (N sub-modos do usuário).
- **Knobs por grupo (nomes reais):**
  - Chão: `MaxWalkSpeed`, `MaxWalkSpeedCrouched`, `MaxAcceleration`, `BrakingDeceleration*` (por modo), `GroundFriction`, `MaxStepHeight` (sobe degrau), `SetWalkableFloorAngle` (rampa andável), `bCanWalkOffLedges`, `PerchRadiusThreshold` (ficar "empoleirado" na beirada).
  - Pulo/queda: `JumpZVelocity`, `AirControl` + `AirControlBoostMultiplier`, `FallingLateralFriction`, `GravityScale`, `bApplyGravityWhileJumping`.
  - Água/voo: `Buoyancy`, `MaxSwimSpeed`, `MaxFlySpeed`.
  - Agachar: `CrouchedHalfHeight` (troca a cápsula, com eventos).
  - Interação física: `bEnablePhysicsInteraction`, `PushForceFactor`, `RepulsionForce`, `StandingDownwardForceScale` — empurra caixas SEM ser rigid body.
  - Rotação: `RotationRate`, `bOrientRotationToMovement` (vira pra onde anda), `bUseControllerDesiredRotation`.
  - Root motion (a animação manda no deslocamento).
  - **Rede:** predição cliente + reconciliação EMBUTIDAS (`NetworkSmoothingMode` etc.) — multiplayer de personagem sai de graça.
  - **Multidão:** `bUseRVOAvoidance`, `AvoidanceConsiderationRadius` (desvio local entre agentes).
- **Elimina:** literalmente o maior arquivo de gameplay de um projeto — máquina de estados de locomoção + colisão + degrau/rampa/beirada + coyote de rede. É o argumento nº 1 de "engine que dá jogo de graça".
- **Tradução 2D (platformer):** os mesmos knobs mapeiam 1:1 — walk speed/accel/fricção, jump velocity, air control, gravity scale, step height (degrau de tile), walkable angle (rampa), ledge/perch, empurrar caixa, agachar, nadar, voar, modos Custom (dash, escada, corda). **Relevância: ALTÍSSIMA — é o blueprint do "PlatformerController" que o PH2D já começou no ph2d-platformer.**

### Outros
- **Paper2DCharacter** — Character com PaperFlipbookComponent no lugar do skeletal mesh (o template 2D pronto). **Alta**.
- **FloatingPawnMovement [API]** — movimento simples sem gravidade p/ qualquer Pawn (top-down/8-direções na prática: MaxSpeed, Acceleration, Deceleration). **Alta** para top-down.
- **Veículo 2D:** ausente (ChaosVehicles é 3D). **Declarado.**

---

## 7) Caminhos / splines + movers utilitários

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/blueprint-spline-components-overview-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/GameFramework/UInterpToMovementComponent · https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/GameFramework/UProjectileMovementComponent

- **SplineComponent** — caminho editável NO VIEWPORT (add/remover/duplicar pontos, tangentes por ponto, loop fechado); consultas get-location/rotation-at-distance/time; editável por Construction Script e animável em runtime. Elimina: estrutura de dados de curva + editor de curva no mundo. Uso: patrulha, trilho de câmera, distribuir N objetos ao longo do caminho. **Alta**.
- **SplineMeshComponent** — deforma um mesh entre 2 pontos de spline (canos, cabos, cercas). Em 2D: equivale a deformar um sprite/nine-slice ao longo de curva. **Média**.
- **Path-follow pronto:** NÃO existe componente oficial "SplineFollow" — todo tutorial monta Timeline + get-location-at-distance. **Declarado ausente** (buraco pequeno e famoso da UE; PH2D pode entregar `PathFollow2D` de fábrica).
- **ProjectileMovementComponent** — bala/granada/flecha por UI: `InitialSpeed`, `MaxSpeed`, `ProjectileGravityScale`, `Velocity`, `bInitialVelocityInLocalSpace`, `bRotationFollowsVelocity` (a flecha aponta pra onde voa), quicar (`bShouldBounce`, `Bounciness`, `Friction`, `BounceVelocityStopSimulatingThreshold`), **homing** (`bIsHomingProjectile`, `HomingTargetComponent`, `HomingAccelerationMagnitude`), `bInterpMovement`; eventos `OnProjectileBounce`/`OnProjectileStop`. Elimina: balística + ricochete + perseguição. **Alta**.
- **RotatingMovementComponent** — rotação contínua a taxa fixa com pivô offsetável (serra, moinho, órbita). Sem colisão durante rotação. **Alta** (todo platformer tem serra giratória).
- **InterpToMovementComponent** — plataforma móvel por UI: `ControlPoints` (waypoints), `Duration`, `BehaviourType` = OneShot / OneShot_Reverse / Loop_Reset / **PingPong**, `bPauseOnImpact`; eventos OnInterpToReverse/Stop/Wait/Reset. Elimina: o script de plataforma que vai-e-volta — o clássico nº 1 de tutorial. **Alta**.

---

## 8) Animação (estado + tween)

**Fonte:** https://www.criticalfailure-studio.com/paperzd-documentation/ · https://dev.epicgames.com/documentation/en-us/unreal-engine/timelines-in-unreal-engine

### PaperZD (plugin padrão de facto para 2D na UE)
- **AnimBP 2D** com compilador próprio e Animation Graph; **AnimSequences** (metadados sobre flipbooks) em Animation Sources.
- **State Machine visual**: estados, transições com condições sobre variáveis (velocidade, direção), **Transitional States** (fluem sozinhos pro estado estável — ex.: "aterrissando"→"parado"), **JumpNodes** (interrupção de fluxo p/ dano/pulo de qualquer estado).
- **AnimNotifies / NotifyStates**: eventos em frame exato da animação (som do passo, spawn de efeito, janela de hitbox) — reutilizáveis entre sequências via asset.
- **Animações direcionais**: uma sequência guarda N variantes por direção; o sistema escolhe o flipbook pelo vetor de direção (top-down/isométrico de graça).
- **AnimPlayer component**: playback centralizado, callbacks ("Playback Complete"). Integra com Sequencer.
- Elimina: a máquina de estados de animação inteira + o dispatch "qual sprite pra qual direção" + eventos por frame. **Relevância: ALTÍSSIMA — é o espelho exato do que um módulo de animação 2D precisa ter.**

### TimelineComponent (o tween da UE)
- Vive DENTRO do Blueprint do actor: tracks **float/vector/color/event** com curvas e keyframes editadas inline; Play/PlayFromStart/Reverse/Stop, loop, length; dispara eventos em pontos da curva; o valor sai por pino e o usuário liga no que quiser (posição da porta, intensidade da luz).
- Elimina: TODO lerp manual com estado (porta abrindo, luz piscando, fade, "sine"). É o componente que os usuários da UE mais usam sem perceber. **Alta** — o equivalente PH2D seria um "TweenComponent" com curva editável no inspector.

### AnimBP 3D (Skeletal) — existe e é rico (state machines, blend spaces), mas é 3D; relevância 2D **média** (só via 2.5D).

---

## 9) Timeline / Sequencer (cinemáticas)

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-sequencer-movie-tool-overview · https://dev.epicgames.com/documentation/unreal-engine/sequencer-track-list-in-unreal-engine

- **Level Sequence Asset** (dados) + **Level Sequence Actor** (na cena; auto-play, loop, playback rate, restore state, desligar input do player, esconder HUD) — cutscene sem código, disparável em runtime.
- **Tracks (lista completa da doc 5.8):** Object Binding (liga actors/components e expõe suas propriedades) · Transform & **Property Tracks** (anima QUALQUER propriedade: floats, cores…) · Animation · Audio · **Event Track** (Trigger/Repeater — chama lógica do jogo em frame X) · **Camera Cut** (qual câmera está ativa) · Fade (tela → cor) · Level Visibility · Material Tracks (parâmetros de material) · Media · Time Dilation (câmera lenta) · Subsequences (colaboração) · Folder · Geometry Cache · Console Variable · **Customizable Sequencer Track** (o USUÁRIO cria tipos de track por Blueprint).
- Elimina: sistemas de cutscene, scripts de "anima essa propriedade por 2s", sincronização som/animação/câmera.
- Sinergia: Camera Shake tracks, PaperZD, Niagara. **Alta** — o PH2D já tem timeline própria; o delta da UE é (1) Property Track genérica sobre qualquer propriedade de componente, (2) Event Track chamando gameplay, (3) tracks CUSTOMIZÁVEIS pelo usuário.

---

## 10) Áudio

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/audio-components?application_version=4.27 · https://dev.epicgames.com/documentation/en-us/unreal-engine/sound-attenuation-in-unreal-engine

- **AudioComponent** — instância de som no actor (Sound Wave/Cue como sub-objeto): play/stop, fade in/out, volume/pitch em runtime, override de attenuation. **Alta**.
- **Sound Attenuation asset** (compartilhável entre N sons — a jogada de UX):
  - **Formas espaciais:** Sphere (default) · **Capsule** (fonte linear: cano, rio) · **Box** (sala) · **Cone** (alto-falante direcional) — com zona interna a volume máximo.
  - **Falloff:** Linear · Logarithmic · Inverse · Log Reverse · **Custom (curva do usuário)**.
  - **Spatialization:** panning padrão ou binaural (plugin); **Non-Spatialized Radius** — perto da fonte o som vira 2D suavemente (mata o "pan pulando" quando o player passa por cima da fonte — problema CLÁSSICO de 2D).
  - Air Absorption (filtro por distância) · **Listener Focus** (volume/prioridade pelo ângulo de atenção) · **Occlusion** (parede abafa) · Reverb Send por distância · Priority (limite de canais) · Submix routing por distância.
- Listener: segue a câmera por default (PlayerCameraManager). Elimina: todo o middleware de som posicional que times 2D acabam escrevendo (distância→volume, pan, oclusão). **Alta — o rack de 42 efeitos do PH2D não cobre ESPACIALIZAÇÃO por objeto de cena; é categoria nova.**

---

## 11) Input — Enhanced Input

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine (lista de triggers/modifiers confirmada em https://unrealdirective.com/articles/enhanced-input-what-you-need-to-know/)

- **Input Action** (asset) — a intenção ("Jump", "Fire"); tipos de valor: bool, Axis1D, Axis2D, Axis3D. Estados reportados: Started / Ongoing / Triggered / Completed / Canceled.
- **Input Mapping Context** (asset) — mapeia teclas/botões→ações para UM contexto (a pé, dirigindo, menu); empilháveis em runtime com prioridade (resolve conflito de tecla automaticamente).
- **Triggers embutidos (9):** **Chorded Action** (só dispara junto com outra ação — Shift+X) · **Combo** (sequência de ações numa janela de tempo — fighting game por asset!) · **Down** · **Hold** (segurou T segundos) · **Hold And Release** · **Pressed** · **Pulse** (re-dispara a cada intervalo enquanto segura) · **Released** · **Tap** (apertou-e-soltou rápido).
- **Modifiers embutidos (11):** Dead Zone · FOV Scaling · Modifier Collection · **Negate** · Response Curve Exponential · Response Curve User Defined · **Scalar** · Scale by Delta Time · **Smooth** · **Swizzle Input Axis Values** (WASD vira eixo 2D com 1 binding) · To World Space.
- Elimina: TODO o parsing de input (detecção de tap vs hold vs double-tap, combos, deadzone, curvas de resposta, troca de contexto ao abrir menu). O gameplay recebe "ação disparou com valor V" — nunca lê tecla.
- **Relevância: ALTÍSSIMA e 100% portável para 2D.** É o sistema mais copiável da lista: puro dado + pipeline valor→modifiers→triggers.

---

## 12) UI in-game — UMG

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/umg-ui-designer-for-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/widget-components-in-unreal-engine

- **Widget Blueprint** — editor visual de UI (canvas, âncoras, hierarquia de widgets, animações internas de widget, bindings de propriedade). É UI de jogo sem código de layout.
- **WidgetComponent** — põe um Widget Blueprint NO MUNDO: modos **World Space** (é um quad na cena, sofre oclusão — barra de vida sobre a cabeça, terminal, placa) e **Screen Space** (projetado na tela, nunca ocluído). Props: Widget Class, Draw Size, Draw at Desired Size, Pivot, redraw manual/automático, focusable. Elimina: projeção mundo→tela + render-to-texture + roteamento de clique para UI de cena. **Alta** (health bar flutuante é o caso nº 1 de jogo 2D).
- **WidgetInteractionComponent [API]** — ponteiro virtual que injeta hover/clique em widgets world-space. **Média**.
- Drag & drop de UI: suportado no framework UMG (eventos de drag). **Média**.

---

## 13) Navegação & AI

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/navigation-system-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/behavior-trees-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-behavior-tree-node-reference · https://dev.epicgames.com/documentation/en-us/unreal-engine/environment-query-system-in-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/ai-perception-in-unreal-engine

### Navegação
- **NavMeshBoundsVolume** — volume que delimita a geração do NavMesh (gerado da colisão do level, em tiles→polígonos). Modos: Static / Dynamic / Dynamic Modifiers Only.
- **NavModifierVolume / NavModifierComponent** — muda CUSTO ou área de navegação numa região (lama anda-se devagar, lava proibida).
- **NavLinkProxy** — liga áreas não-contíguas (pular a vala, descer da plataforma) — o "jump link".
- Evitamento: **RVO** (no CharacterMovement) e **Detour Crowd Manager**.
- Elimina: pathfinding + custo por área + links especiais. Em 2D o análogo é navgrid/plataforma-graph; a UI (volumes + links visuais + custo por área) traduz direto. **Alta**.

### Behavior Trees + Blackboard (AI sem código)
- **Blackboard** = memória chave-valor da AI; o BT lê/escreve keys e decide ramos.
- **Composites:** Selector · Sequence · Simple Parallel.
- **Tasks embutidas:** Move To · Wait · Wait Blackboard Time · Play Sound · Play Animation · Rotate to Face BB Entry · Run Behavior (sub-árvore) · Run EQS Query · Make Noise · Push Pawn Action · Set Tag Cooldown · Finish With Result.
- **Decorators (condições):** Blackboard · Compare BB Entries · Composite · Conditional Loop · Cone Check · Cooldown · Does Path Exist · Force Success · Is At Location · Is BB Entry Of Class · Keep In Cone · Loop · Set/Tag Cooldown · Time Limit.
- **Services (a cada N seg num ramo ativo):** Default Focus · Run EQS.
- Elimina: a máquina de decisão da AI inteira — patrulha-persegue-ataca-foge configurável por árvore visual. **Alta**.

### EQS (Environment Query System)
- Consultas espaciais data-driven: **Generators** (pontos em grade/círculo, actors de classe, etc.) geram candidatos; **Tests** (distância, trace/linha-de-visão, dot, overlap, pathfinding) FILTRAM e PONTUAM; **Contexts** dão o referencial; o melhor item vai pro Blackboard. EQS Testing Pawn visualiza no editor. Elimina: "ache o melhor ponto de cobertura/flanco/fuga" codificado à mão. **Média-alta** (em 2D: posicionamento tático, spawn points).

### AI Perception
- **AIPerceptionComponent** (no AIController) + **AIPerceptionStimuliSourceComponent** (no alvo; Auto Register + p/ quais sentidos).
- **Sentidos:** **Sight** (SightRadius, LoseSightRadius, PeripheralVisionHalfAngleDegrees, AutoSuccessRange, offsets de POV) · **Hearing** (HearingRange; ouve "Report Noise Event") · **Damage** · **Touch** (esbarrou) · **Team** · **Prediction** (posição prevista do alvo em T seg).
- Comuns: Max Age (esquecer), Detection by Affiliation (Enemies/Neutrals/Friendlies), Dominant Sense, Starts Enabled, Debug Color (visualizador de cones no jogo: ' + Numpad4).
- Eventos: OnPerceptionUpdated, OnTargetPerceptionUpdated (com struct: idade, força, localização do estímulo, sensed com sucesso). Funções: GetCurrentlyPerceivedActors, GetPerceivedHostileActors, Forget All…
- Elimina: cone de visão + memória do inimigo + "ouviu o barulho" — o coração de stealth/combate, todo por dropdown. **Alta — cone de visão 2D + ruído é feature de UI puríssima.**

---

## 14) Gameplay Ability System (GAS)

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-ability-system-for-unreal-engine · https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-effects-for-the-gameplay-ability-system-in-unreal-engine

- **AbilitySystemComponent** — o hub: concede/ativa habilidades, aplica efeitos, possui atributos.
- **Gameplay Attributes / AttributeSets** — valores de jogo (vida, mana, armadura) com base/atual e clamping centralizados.
- **Gameplay Abilities** — habilidade ativa/passiva que coordena "mecânica + VFX + animação + som"; **Ability Tasks** para trabalho assíncrono no meio da habilidade.
- **Gameplay Effects (asset de UI pura):** duração **Instant / Has Duration / Infinite**; **Modifiers** (soma/multiplica/override de atributo, ex.: "+5% da armadura base"); **Periodic** (dano por tick = veneno); **Stacking** rico (acumula até X, reseta/estende tempo, instâncias independentes); componentes de efeito que **concedem tags** ao alvo e **exigem tags** para aplicar; Executions para fórmulas complexas.
- **Gameplay Tags** — vocabulário hierárquico (`State.Stunned`, `Damage.Fire`) que ativa/bloqueia tudo por comparação de tags (parte do framework; cues cosméticos reagem a tags).
- Replicação e predição embutidas no framework.
- Elimina: o sistema de RPG inteiro — buffs/debuffs/DoT/cooldown/custo/stack/imunidade viram ASSETS que designer preenche. **Média-alta para 2D** (action-RPG/roguelike 2D usam idêntico; a lição-chave para o PH2D: atributos + efeitos-como-asset + tags são UI pura, motor pequeno).

---

## 15) Spawn / factory / pooling

- Spawn: `SpawnActor` (API/nó Blueprint) a partir de classe/Blueprint (o "prefab" da UE é o Blueprint Class). **ChildActorComponent** — spawna um actor-filho quando o componente registra (prefab aninhado dentro de outro). Fonte: https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/ChildActorComponent?application_version=5.0
- **Pooling genérico: AUSENTE** — não há object pool oficial de gameplay. **Declarado.** (Oportunidade: um SpawnerComponent com pool embutido é pedido recorrente.)

## 16) Timers

- **NÃO é componente:** `FTimerManager` global (no World e no GameInstance); `SetTimer` (delay, loop, first delay), handles com Pause/UnPause/Clear, IsTimerActive, GetTimerRemaining, SetTimerForNextTick; exposto a Blueprint por nós de timer. Elimina: contadores manuais em tick. **Alta** (mas a UX "componente Timer no inspector" — estilo Godot — a UE não tem; **declarado**). Fonte: https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-timers-in-unreal-engine

## 17) Rede / multiplayer

**Fonte:** https://dev.epicgames.com/documentation/en-us/unreal-engine/networking-overview-for-unreal-engine
- Cliente-servidor autoritativo. Por **checkbox** no Blueprint: `Replicates` (o actor existe nos clientes; criar/destruir replica sozinho) e `Replicate Movement` (posição/rotação/velocidade sincronizadas). Por propriedade: `Replicated` e `RepNotify` (callback na chegada — recomendado sobre RPC). **RPCs**: Server/Client/Multicast, reliable/unreliable. Ownership, relevância e prioridade configuráveis. CharacterMovement traz predição pronta (ver §6); GameState/PlayerState replicam por design (ver §0).
- Elimina: a camada de sync de estado; o que fica para o usuário é marcar O QUE replica. **Média para o PH2D hoje; o desenho "checkbox por componente/propriedade" é a referência de UX.**

## 18) Persistência / save

- **USaveGame**: classe-contêiner; o usuário marca variáveis e chama SaveGameToSlot/LoadGameFromSlot (+ versões **async** p/ não travar frame); slots nomeados + user index; arquivos `.sav`. **NÃO há serialização automática de cena — declarado** (o usuário escolhe o que salvar). Fonte: https://dev.epicgames.com/documentation/en-us/unreal-engine/saving-and-loading-your-game-in-unreal-engine — Nota PH2D: nosso ProjectState snapshot-based já é MAIS automático que a UE aqui.

## 19) Parallax / scrolling

- **AUSENTE como componente** — em UE faz-se com material (offset por profundidade) ou câmera; nada oficial "ParallaxLayer". **Declarado.** (Oportunidade direta PH2D: camada com fator de parallax é trivial e todo jogo 2D usa.)

## 20) Utilitários (a régua da lista Construct-like)

- line-of-sight → AI Perception Sight / trace por canal. **Tem.**
- drag&drop físico → PhysicsHandleComponent. **Tem.** (drag&drop de UI → UMG.)
- pin → PhysicsConstraint (Locked). **Tem.**
- fade de tela → Fade Track (Sequencer) / Camera Manager. **Tem (não é componente).**
- flash/sine/wiggle → TimelineComponent + curvas; shake → CameraShake. **Tem.**
- move-to → AI MoveTo (com navmesh) / InterpToMovement (sem). **Tem.**
- wrap (Pac-Man), anchor de cena, solid/jumpthru configurável, scroll-to → **AUSENTES como componentes prontos** (jumpthru faz-se com colisão unidirecional custom). **Declarados.**

## 21) Como o usuário cria componente/behavior PRÓPRIO

- **Blueprint Class de ActorComponent/SceneComponent** — componente novo SEM C++, com variáveis expostas no inspector (Instance Editable), eventos (BeginPlay/Tick) e funções; adicionável a qualquer actor pelo painel Components. Visual scripting = **Blueprints** (o gameplay inteiro pode ser feito neles).
- **C++** para performance/engine — mesma superfície (UPROPERTY expõe no editor).
- Extensões data-driven sem código de engine: Input Modifiers/Triggers custom, módulos Niagara, **Customizable Sequencer Tracks**, tasks/decorators/services de BT em Blueprint.
- **Lição PH2D:** a dupla "componente-de-usuário com propriedades auto-expostas no inspector" + "pontos de extensão data-driven em CADA sistema" é o que faz o ecossistema escalar.

---

## ⭐ MATADORES DE CÓDIGO — os 15 que mais eliminam programação só com UI

1. **CharacterMovementComponent** — 6 modos de locomoção + degrau/rampa/beirada/agachar/empurrar/predição de rede; o maior arquivo de gameplay que ninguém precisa escrever.
2. **Enhanced Input (Actions + Contexts + 9 Triggers + 11 Modifiers)** — tap/hold/combo/chord/deadzone/curvas + troca de contexto, tudo asset; o gameplay nunca vê tecla.
3. **TimelineComponent** — todo lerp com estado (porta, fade, pulso, sine) vira curva desenhada.
4. **Behavior Tree + Blackboard** (com as tasks/decorators/services embutidas) — patrulha→persegue→ataca sem uma linha.
5. **AI Perception (Sight/Hearing + afiliação + memória)** — cone de visão e "ouviu barulho" por dropdown; stealth de graça.
6. **ProjectileMovementComponent** — balística + ricochete + homing por 10 campos.
7. **InterpToMovementComponent + RotatingMovementComponent** — plataforma móvel ping-pong e serra giratória: os 2 clássicos de tutorial, por UI.
8. **PhysicsConstraintComponent** — TODA a família de joints (limites, molas, motores, quebra) numa superfície só.
9. **Sequencer (Property/Event/CameraCut/Fade tracks)** — cutscene e "anima qualquer propriedade" sem código, com eventos chamando o jogo.
10. **Sound Attenuation asset + AudioComponent** — som posicional completo (formas, falloff, oclusão, non-spatialized radius) compartilhado por asset.
11. **SpringArmComponent** — câmera-follow com lag e anti-clipping por 5 campos.
12. **Camera Shake (+ Shake Source posicional)** — trauma/juice autorado por asset com falloff espacial.
13. **Gameplay Effects (GAS)** — buff/debuff/DoT/stack/imunidade como asset preenchível.
14. **Tile Set/Tile Map + import de Tiled** — level 2D com colisão por tile pintado, sem código.
15. **PaperZD AnimBP** — máquina de estados de animação 2D + notifies por frame + variantes direcionais, visual.

---

## Apêndice — buracos da UE em 2D (= oportunidades diretas do PH2D)

1. **Esqueleto/bones 2D nativo: não existe** (nem deform de sprite).
2. **Câmera de gameplay 2D** (limites de mundo, zonas, deadzone 2D, transições): nada pronto — SpringArm é 3D-cêntrico.
3. **Path-follow pronto** sobre spline: monta-se à mão toda vez.
4. **Parallax**: sem componente.
5. **Pooling/spawner**: sem componente.
6. **Timer como componente de inspector**: só manager por código/nós.
7. **Nine-patch de cena, trail 2D dedicado, wrap/jumpthru/scroll-to**: ausentes.
8. **Save automático de cena**: manual (o snapshot do PH2D já supera).
9. Paper 2D está semi-abandonado (docs 4.27; física 2D "experimental — use a 3D") — o mercado 2D da UE vive de plugin (PaperZD). Uma engine 2D-first com o NÍVEL DE KNOBS do CharacterMovement/Enhanced Input/Attenuation não tem concorrente dentro da UE.
10. Iluminação 2D: a UE não tem pipeline 2D de luz (usa a 3D). **[adiado por decisão do dono — listado sem prioridade]**

## Fontes principais (por categoria)
- Componentes/base: https://dev.epicgames.com/documentation/en-us/unreal-engine/components-in-unreal-engine
- Gameplay framework: https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-framework-in-unreal-engine
- Movement components: https://dev.epicgames.com/documentation/en-us/unreal-engine/movement-components-in-unreal-engine · API UCharacterMovementComponent / UProjectileMovementComponent / UInterpToMovementComponent
- Paper 2D: https://dev.epicgames.com/documentation/en-us/unreal-engine/paper-2d-overview-in-unreal-engine · tile maps: .../paper-2d-tile-sets-and-tile-maps-in-unreal-engine
- PaperZD: https://www.criticalfailure-studio.com/paperzd-documentation/
- Enhanced Input: https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine (+ unrealdirective.com para a lista integral de triggers/modifiers)
- GAS: https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-ability-system-for-unreal-engine · effects: .../gameplay-effects-for-the-gameplay-ability-system-in-unreal-engine
- AI: behavior-trees-in-unreal-engine · unreal-engine-behavior-tree-node-reference · environment-query-system-in-unreal-engine · ai-perception-in-unreal-engine · navigation-system-in-unreal-engine
- Niagara: creating-visual-effects-in-niagara-for-unreal-engine · render-module-reference-for-niagara-effects-in-unreal-engine
- Sequencer: unreal-engine-sequencer-movie-tool-overview · sequencer-track-list-in-unreal-engine
- Timelines: timelines-in-unreal-engine · Timers: gameplay-timers-in-unreal-engine
- Física: physics-constraint-component-user-guide-in-unreal-engine · physics-constraint-reference-in-unreal-engine · API RadialForce/PhysicsThruster/PhysicsHandle
- Câmera: using-spring-arm-components-in-unreal-engine · camera-shakes-in-unreal-engine
- Áudio: sound-attenuation-in-unreal-engine · audio-components (4.27)
- UI: umg-ui-designer-for-unreal-engine · widget-components-in-unreal-engine
- Save: saving-and-loading-your-game-in-unreal-engine · Rede: networking-overview-for-unreal-engine
