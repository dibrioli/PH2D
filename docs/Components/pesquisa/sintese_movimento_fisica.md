# Síntese cruzada — MOVIMENTO & FÍSICA DE GAMEPLAY (cardápio de componentes canônicos PH2D)

> Fontes: dossiês Unity/Godot/Unreal/Construct+GDevelop/GameMaker+Defold/Cocos+Phaser/Bevy +
> `inventario_ph2d.md` (2026-08-20). Escopo deste domínio: corpos e colisão · áreas/sensores/zonas ·
> raycast/shapecast · character controllers prontos · joints · materiais físicos · paths/splines +
> path-follow · effectors · solid/jump-thru · wrap/bound · physics helpers Construct/GDevelop.
> Decisões respeitadas: componentes estilo Unity (AddComponent); iluminação 2D = ADIADO (não aparece
> neste domínio). Prioridade: **P0** = espinha (sem isso não há jogo) · **P1** = diferencial forte de
> facilidade · **P2** = depois. Levantamento EXAUSTIVO — o dono corta.

---

## Leis transversais do domínio (antes do cardápio)

1. **A moeda de troca é o Collider + camadas — nunca um marcador paralelo.** A lição nº 1 do
   Construct: `Solid` funciona porque TODOS os sistemas de movimento o consultam (Platform, Bullet
   bounce, Pathfinding, LOS, Tile movement). No PH2D o equivalente já existe: `Collider` +
   camadas de colisão nomeadas. **Todo mover kinemático novo deste cardápio (TopDownPlayer,
   TileMotion, MoveTo, ProjectileMotion, LineOfSight) deve consultar o MESMO Collider+camada** —
   criar um segundo marcador "Obstacle" seria bifurcar a verdade. O filtro por tags do Solid do
   Construct ("sólido pro inimigo, não pro player") já é expressável por camadas.
2. **Um dono do transform por vez.** Construct documenta (Physics não se mistura com os demais
   movements); o PH2D já pratica (rapier é a verdade; `PlayerMode` Dynamic/Kinematic/Pure). Todo
   componente de movimento novo declara seu modo de posse — e o Inspector deve ACUSAR conflito
   (dois movers ativos no mesmo objeto = badge de aviso, não comportamento indefinido).
3. **`Default controls` + `Simulate control` (padrão Construct, o contrato-chave).** Todo controller
   anda sozinho no primeiro clique (setas/gamepad default) E vira motor puro dirigível por
   sinal/script/timeline desligando um bool. Duas audiências, um componente. Depende do action
   mapping de input (domínio Input) — registrar a dependência cruzada.
4. **Referência durável = `stable_name_id`, nunca Entity bits** (lei do repo). Vale para TODO campo
   "alvo": MoveTo.target, TurretAim.targets, PinTo.host, PathFollow.path, ProjectileMotion.homing.
5. **Micro-movers vs Motion Nodes: MEÇA antes de construir** (lei §5.0 do CLAUDE.md). Sine/Rotate/
   Wiggle/Boids já existem como Motion Nodes GPU-resident. O que falta não é o oscilador — é o
   EMPACOTAMENTO "1 clique no objeto". Antes de escrever qualquer micro-mover, medir se um preset de
   node ligado via bake/bridge já o exprime; se sim, o componente vira um atalho de autoria, não um
   segundo motor.
6. **Config plain-data determinística** (BTreeMap, nunca estado vivo de solver) — o caminho de
   registro (derive → register → PROJECT_SCHEMA → Inspector) já dá persistência+undo de graça;
   o custo real de cada item abaixo é a seção do Inspector (passo 5 do inventário).

---

## A. CORPOS & COLISÃO

### RigidBody + Collider — ✅ JÁ EXISTE
- **Entrega:** corpo dinâmico/estático/kinemático + forma de colisão com restitution/friction;
  o pacote de overrides (GravityScale, Ccd, LockRotation/Position, MassOverride, Dominance,
  DampingOverride, MaterialCombine, InitialVelocity) já cobre o catálogo Unity/Godot inteiro.
- **Equivalentes:** Unity Rigidbody2D+Colliders(8) · Godot RigidBody2D/StaticBody2D+CollisionShape2D ·
  Unreal ShapeComponents+SimulatePhysics · Construct Physics · GDevelop Physics 2.0 · GM "Uses
  Physics" · Defold Collision Object · Cocos RigidBody2D+Collider2D · Phaser Arcade/Matter Body ·
  Bevy rapier/avian RigidBody+Collider.
- **PH2D:** SIM — `ph2d-physics-ecs` (32 components, Inspector §physics_body, hash 3-OS).
- **Prioridade:** P0 (feito). **Nota:** as formas hoje são as do rapier via Collider; conferir se
  polígono livre desenhado (via Vector pen) já vira collider — a ponte "desenhei = colide" é o
  diferencial que nenhuma engine tem com um editor vetorial deste nível.

### SensorZone (trigger com eventos enter/exit) — ⚠️ PARCIAL, fechar
- **Entrega:** região que detecta sobreposição SEM resposta física e publica eventos de
  entrada/saída — todo trigger de gameplay (coletável, dano, checkpoint, porta) vira shape +
  evento, zero código. É o Area2D do Godot, o componente que mais elimina código depois do controller.
- **Equivalentes:** Unity Collider `Is Trigger` · Godot **Area2D** (o modelo mais rico: também
  sobrescreve gravidade/damping local com prioridade) · Unreal Generate Overlap Events · Construct
  (via events de overlap) · GM flag Sensor · Defold tipo Trigger · Cocos Sensor · Phaser
  `overlap()`/Zone · Bevy `Sensor`.
- **PH2D:** PARCIAL — `Collider.is_sensor` existe mas "aguarda consumidor"; `SignalOnHit`/
  `SignalOnLeave` já publicam Signal em CONTATO. Fechar = fazer o par funcionar em modo sensor
  (overlap, não contato resolvido) e garantir que o Signal distingue corpo/sensor.
- **Prioridade:** **P0** — é a metade de "sinais viram gameplay" (R3).
- **Dependências:** nenhuma (rapier já reporta intersecções de sensor). Ordem: primeiro do domínio.
- **Risco:** o consumidor visível hoje é toast/log — o valor só aparece junto com a tabela
  sinal→ação (domínio Runtime/R3). Entregar junto com pelo menos 1 consumidor real.

### PhysicsMaterial (asset compartilhável) — P2
- **Entrega:** atrito/quique nomeado e reusável entre N colliders ("Gelo", "Borracha") — hoje cada
  collider carrega os números soltos; o asset elimina a cópia divergente.
- **Equivalentes:** Unity Physics Material 2D · Godot PhysicsMaterial (+rough/absorbent) · Unreal
  Physical Material · Cocos (inline no collider, sem asset — o contra-exemplo).
- **PH2D:** PARCIAL — friction/restitution no `Collider` + `MaterialCombine` (regra de combinação,
  que Unity nem expõe). Falta só o recorte "asset nomeado".
- **Prioridade:** P2 (conveniência de organização, não capacidade nova).

### KinematicPlatform (corpo animado que EMPURRA e CARREGA) — P1
- **Entrega:** plataforma/porta movida por Timeline/Waypoint que transmite velocidade correta aos
  corpos e ao player em cima — mata o bug notório "plataforma móvel não carrega o jogador"
  (resolvido por design no Godot). A velocidade é DERIVADA do movimento autorado, sem teleporte.
- **Equivalentes:** Godot **AnimatableBody2D** (`sync_to_physics`) · Cocos RigidBody2D tipo
  **Animated** (deriva velocidade da animação — a ideia boa e pouco copiada) · Unity Kinematic
  Rigidbody movido por MovePosition · Unreal InterpToMovement (§G).
- **PH2D:** PARCIAL — o `PlatformPlayer` (cápsula flutuante + ride) já herda plataforma pelo lado
  DELE; falta o lado da plataforma para corpos dinâmicos genéricos (caixa em cima do elevador).
- **Prioridade:** P1. **Dependências:** Timeline (já anima Transform via `SpriteAnimation`) e/ou
  WaypointMotion (§G). **Nota de desenho:** é uma FLAG/modo no RigidBody kinemático
  ("velocity-from-authored-motion"), não necessariamente componente novo — medir composição primeiro.

### TransformInterpolation (anti-jitter de fixed timestep) — P1
- **Entrega:** interpola o visual entre passos da física — mata o tremido física@60Hz vs
  render@144Hz+ com um checkbox por corpo (ou default global).
- **Equivalentes:** Unity Rigidbody2D Interpolate · Godot physics interpolation ·
  avian **TransformInterpolation** (como componente — o idioma a copiar).
- **PH2D:** NÃO (a conferir contra o present-world — pode já haver suavização no `SimRef`).
- **Prioridade:** P1 (qualidade percebida; a workstation 144Hz+ do alvo "extraordinário" denuncia).
- **Risco:** MEDIR antes — se o loop atual já apresenta interpolado, o item evapora.

### CompositeCollider (fusão de colliders vizinhos) — P2
- **Entrega:** funde N colliders filhos/de-tile num outline único — remove o "ghost collision"
  nas emendas (o personagem que tropeça na junção de dois boxes).
- **Equivalentes:** Unity **Composite Collider 2D** · Godot (decomposição no CollisionPolygon2D) ·
  Phaser (colisão por tile com faces internas culled).
- **PH2D:** NÃO. **Prioridade:** P2 — sobe para P1 quando o Tilemap (outro domínio) chegar: colisão
  por tile SEM composite reabre o bug clássico. Registrar como dependência do tilemap.

---

## B. CONSULTAS ESPACIAIS (raycast/shapecast como COMPONENTE)

### RaySensor — P0
- **Entrega:** raio PERSISTENTE configurado no editor (origem/direção/alcance/máscara, seta visível
  no viewport) que atualiza todo tick e expõe `is_hitting / point / normal / quem`. Sensor de chão,
  "tem parede à frente?", mira — sem API imperativa, sem código.
- **Equivalentes:** Godot **RayCast2D** (o modelo de UI) · avian **RayCaster** (raycast-como-
  componente, o idioma ECS exato) · Unity/Unreal/GM/Defold/Cocos/Phaser só API · Construct LOS
  `Cast ray` (com normal E REFLEXÃO prontas — copiar as expressions de reflexão: laser que
  ricocheteia em 3 leituras).
- **PH2D:** NÃO (rapier tem as queries; falta o componente + Inspector + gizmo).
- **Prioridade:** **P0** — é a peça de composição que LineOfSight, TurretAim, ledge-check custom e
  todo script Luau vão consumir. **Dependências:** nenhuma. **Ordem:** junto com SensorZone.

### ShapeSensor (shapecast/overlap persistente) — P1
- **Entrega:** varre uma FORMA (não um ponto) — detecção "gorda" de personagem, golpe com área,
  "cabe aqui?". Devolve múltiplos hits + frações safe/unsafe.
- **Equivalentes:** Godot **ShapeCast2D** · avian ShapeCaster · Unity BoxCast/CircleCast (API).
- **PH2D:** NÃO. **Prioridade:** P1. **Dependências:** RaySensor primeiro (mesma infra, mesma seção
  de Inspector generalizada).

### LineOfSight (percepção: alcance + cone + oclusão) — P1
- **Entrega:** "A vê B?" como condição pronta: alcance, cone angular, obstáculos por camada,
  evento ao ganhar/perder visão. A metade de toda IA 2D (stealth, aggro, torreta que só atira se vê).
- **Equivalentes:** Construct **Line of sight** (o desenho de referência) · Unreal AI Perception
  Sight (com memória/afiliação — a versão máxima, fica pro domínio IA) · Godot (compõe RayCast2D).
- **PH2D:** NÃO. **Prioridade:** P1. **Dependências:** RaySensor. Publica `Signal` (ex.:
  `sight.gained/lost`) — casa com o event-sourcing do repo. **Fronteira:** memória de percepção /
  audição / afiliação pertencem à síntese de IA; aqui é só o sensor geométrico.

---

## C. CHARACTER CONTROLLERS PRONTOS
> A lacuna nº 1 confessa de Unity (sem CCT 2D), Godot (CharacterBody2D é só a metade de baixo),
> GameMaker, Defold, Cocos e Bevy-core. Construct/GDevelop provam a demanda; o PH2D JÁ está à
> frente no platformer. Completar a família é o maior diferencial de marketing deste domínio.

### PlatformPlayer — ✅ JÁ EXISTE (o troféu do módulo)
- **Entrega:** platformer completo por componente: cápsula flutuante, walk/jump (coyote+buffer)/
  dash/crouch/ledge/wall/swim/glide/slope/corner-escape/ride/react, 3 modos de posse
  (Dynamic/Kinematic/Pure), eventos entre estados (`PlayerEvent`).
- **Equivalentes:** Construct Platform · GDevelop Platformer character (+ladder/ledge-grab) ·
  bevy-tnua (basis+actions) · Unreal CharacterMovement (o teto de knobs, 3D) · **Unity/Godot/GM/
  Defold/Cocos/Phaser: NÃO TÊM** — o PH2D já entrega o que elas não têm.
- **PH2D:** SIM — `ph2d-platformer` (lei pura) + ponte `bridge/player*` + Inspector §14.
- **Prioridade:** P0 (feito). **Deltas a rever contra os dossiês:** escada como TIPO de superfície
  (GDevelop: Ladder é um tipo de Platform — elegante; hoje há swim/glide mas escada não aparece no
  inventário) · gravidade rotacionável ("Set angle of gravity" do Construct — andar em paredes/
  planetas) · `Simulate control` (lei transversal 3) explícito para IA/replay dirigirem o player.

### TopDownPlayer (top-down / 4 / 8 direções / isométrico) — P0
- **Entrega:** o SEGUNDO controller canônico: aceleração/desaceleração/max-speed, modos de direção
  (2-eixos / 4 / 8 / livre), **deslizar ao longo de obstáculo em diagonal** (o caso chato que trava
  iniciante), rotação (nenhuma/90°/45°/suave) e — o golpe do GDevelop — **dropdown de viewpoint:
  Top-down / Isometria 2:1 / Isometria 30° / Custom**, que reprojeta o input sem trocar de componente.
- **Equivalentes:** Construct **8 Direction** · GDevelop **Top-down movement** (com isometria) ·
  Godot CharacterBody2D FLOATING (metade) · Unreal FloatingPawnMovement · Phaser Arcade body+knobs
  (80%, falta o input) · demais: não têm.
- **PH2D:** NÃO. **Prioridade:** **P0** — sem ele, metade dos gêneros (RPG, twin-stick, sokoban-like
  livre) exige script no dia 1.
- **Dependências:** nenhuma dura (consome Collider como obstáculo — lei 1; modos de posse — lei 2;
  action mapping quando existir — lei 3). **Ordem:** primeiro controller novo; reusar o desenho
  modular do `ph2d-platformer` (lei pura + ponte) que já foi validado.

### ProjectileMotion (bullet genérico) — P0
- **Entrega:** move para frente no ângulo atual com aceleração, gravidade (arco), **ricochete em
  sólido pela normal**, distância-percorrida (alcance), sub-passos anti-túnel, e os campos de
  **homing** (alvo + aceleração de perseguição — Unreal embute no mesmo componente, não é
  componente separado). "A flecha aponta pra onde voa" = 1 bool.
- **Equivalentes:** Construct **Bullet** · Unreal **ProjectileMovementComponent** (o superset:
  bounce+homing+rotation-follows-velocity) · GDevelop Homing/Advanced projectile (extensões) ·
  GM built-ins speed/direction/gravity (o embrião histórico) · demais: código.
- **PH2D:** NÃO (motion nodes têm forças, mas projétil de GAMEPLAY colide e ricocheteia — é outro laço).
- **Prioridade:** **P0** — bala/pickup-que-voa/inimigo-que-avança é o mínimo arcade; a alternativa
  é todo usuário escrever integração + ricochete.
- **Dependências:** Collider como sólido (lei 1). Par natural com DespawnOutside (§F) e com o
  spawner (domínio Spawn) — o trio Construct clássico Turret→Bullet→Destroy outside.

### VehiclePlayer (carro arcade top-down + veículo físico lateral) — P1
- **Entrega:** dois recortes num componente com modo: (a) **arcade top-down** com esterço, ré e
  DRIFT (drift-recover — o feel notoriamente difícil de acertar, resolvido no Car do Construct);
  (b) **físico lateral** = corpo + 2 WheelJoints com motor/suspensão (os joints o PH2D JÁ tem —
  aqui é só o empacotamento "AddComponent Vehicle" que monta o rig).
- **Equivalentes:** Construct **Car** (arcade) · Unity 2 Wheel Joints (receita, não componente) ·
  GDevelop 2D physics car (extensão) · Cocos WheelJoint2D · demais: não têm.
- **PH2D:** PARCIAL — `PhysicsJoint::Wheel` + `WestonAxle` existem; falta o arcade e o montador.
- **Prioridade:** P1. **Dependências:** joints (feito); TopDownPlayer antes (compartilha input/posse).

### TileMotion (movimento casa-a-casa em grade) — P1
- **Entrega:** roguelike/puzzle/sokoban: snap à grade, animação entre células, fila de input,
  bloqueio por sólido, condição pronta "posso mover para lá?", modo isométrico.
- **Equivalentes:** Construct **Tile movement** (com isométrico) · demais: não têm.
- **PH2D:** PARCIAL — `ph2d-grid` já dá 11 grids + snap math + A* determinístico ("any gameplay
  code can share"); falta só o componente que o consome no objeto.
- **Prioridade:** P1 (barato: a matemática já existe; gênero inteiro destravado).
- **Dependências:** ph2d-grid (feito) · Collider como sólido.

### MoveTo (vai-até com chegada, fila e executor de caminhos) — P1
- **Entrega:** "vá até X" com aceleração/desaceleração, rotação opcional, **fila de waypoints**,
  parada em sólido e evento **On arrived** — e a jogada do Construct: o MESMO executor consome
  três fontes de trajetória (ponto direto · caminho do A* do grid · trilha autorada/timeline).
  É o músculo de "clique para mover", cutscene de deslocamento e IA simples.
- **Equivalentes:** Construct **Move To** (o desenho de referência) · Unreal AI MoveTo (com navmesh)
  · GM mp_grid→path→follow · demais: código.
- **PH2D:** NÃO. **Prioridade:** P1 (quase P0 — é o helper mais consumido pelos outros).
- **Dependências:** Collider (parada) · ph2d-grid A* (fonte de caminho) · PathFollow (§E, fonte de
  caminho). **Fronteira:** NavAgent com navmesh/avoidance é do domínio IA — aqui é o EXECUTOR.

### OrbitMotion — P2
- **Entrega:** órbita elíptica em torno de ponto/objeto (raio primário/secundário, offset, sentido,
  encarar tangente, seguir alvo móvel) — escudo orbitando, lua, item girando.
- **Equivalentes:** Construct **Orbit** (com preview no editor) · GDevelop Make objects orbit ·
  Unreal RotatingMovement (pivô offsetado).
- **PH2D:** PARCIAL — Motion Nodes exprimem órbita (medir composição, lei 5).
- **Prioridade:** P2. **Nota:** candidato nº 1 a "preset de Motion Node empacotado como componente".

---

## D. JOINTS

### PhysicsJoint — ✅ JÁ EXISTE (validado por todas as engines)
- **Entrega:** joint = ENTIDADE (autorável no Inspector sem widget novo — o eixo do slider é a
  rotação do Transform da entidade-joint), 9 kinds (Pin, Spring, Rope, Weld, Slider, Rod, Wheel,
  Pulley, Custom) + polia/talha/tambor + pino de mundo + motores + limites + break.
- **Equivalentes:** Unity 9 joints 2D · Godot 3 (Pin com motor) · Unreal PhysicsConstraint (a lição
  da UI unificada — que o kind `Custom` do PH2D já espelha) · GDevelop 11 (Box2D) · Cocos 7 ·
  Construct 4 · Bevy rapier/avian 4.
- **PH2D:** SIM. **Prioridade:** P0 (feito).
- **Deltas de catálogo contra o mercado (extensões de `JointKind`, não componentes novos):**
  **Gear** (GDevelop/Box2D — engrenagem ligando duas revolutas, P2) · **Friction** (amortecedor
  genérico Unity/GDevelop, P2 — conferir se `Custom` já exprime) · **Target/Mouse** (puxar para um
  PONTO com mola — Unity TargetJoint2D; ⚠️ MEDIR primeiro: `Spring` + `JointWorldAnchor` pode já
  compor exatamente isso; se sim, o que falta é só o §PhysicsGrab abaixo).

### PhysicsGrab (agarrar/arrastar corpo físico com o ponteiro) — P1
- **Entrega:** o gesto "pega e arrasta mantendo a física viva" (gravity-gun, drag-and-drop físico,
  puzzle de empilhar): pressiona → mola até o cursor → solta. O spring-para-o-cursor que todo jogo
  físico reescreve.
- **Equivalentes:** Unreal **PhysicsHandleComponent** · Unity Target Joint 2D (caso de uso oficial)
  · GDevelop Mouse joint · Construct Drag&Drop (o primo sem física).
- **PH2D:** NÃO. **Prioridade:** P1. **Dependências:** provável composição Spring+WorldAnchor
  (medir) + input de ponteiro do shell. **Nota:** a variante SEM física (arrastar objeto de cena,
  eixos restritos, multitouch) é do domínio Input/UI — não duplicar aqui.

---

## E. EFFECTORS & ZONAS (o pacote que a Unity vende como diferencial — PH2D já tem 80%)

### Família Area* — ✅ JÁ EXISTE
- **Entrega:** `AreaEffector` (vento/correnteza) · `AreaDrag` · `AreaBuoyancy` (água com empuxo) ·
  `AreaFormDrag` · `AreaTorque` · `AreaFalloff` · `AreaForceWorldAxes` — campos de força/arrasto/
  empuxo/torque com falloff, por Inspector.
- **Equivalentes:** Unity Area/Point/Buoyancy Effector 2D · Godot Area2D (gravidade/damping
  override) · Unreal RadialForce/Thruster · Defold particle modifiers (só partícula).
- **PH2D:** SIM. **Prioridade:** P0 (feito).

### ConveyorSurface (esteira rolante) — P1
- **Entrega:** velocidade tangencial na SUPERFÍCIE de um sólido imóvel — esteiras, plataformas
  giratórias que transmitem, água corrente no chão. Um número + direção.
- **Equivalentes:** Unity **Surface Effector 2D** · Godot StaticBody2D `constant_linear_velocity`
  (transmite SEM se mover — o desenho mais barato, a copiar).
- **PH2D:** NÃO (é o buraco visível da família Area*; `WalkSurface` é o irmão certo para pendurar —
  já é "material de superfície p/ o player").
- **Prioridade:** P1. **Dependências:** nenhuma. **Nota:** decidir se afeta só o player
  (via WalkSurface) ou qualquer corpo dinâmico (via fricção do contato) — as engines fazem os dois.

### AreaAttractor (atração/repulsão radial — ímã, poço gravitacional) — ⚠️ PARCIAL, fechar
- **Entrega:** força radial de um ponto com falloff e modos por distância — ímã de moedas,
  gravidade local, repulsor.
- **Equivalentes:** Unity **Point Effector 2D** · Godot Area2D gravity_point (+unit_distance
  inverso-quadrado) · Unreal RadialForce · GDevelop Magnetic effect · Phaser gravity wells.
- **PH2D:** PARCIAL — o campo de atração existe mas **não alcança um player de pose própria**
  (item aberto do §5: força sustentada pede canal por-tique no player). Fechar essa costura É o item.
- **Prioridade:** P1 (a metade que falta é exatamente a que o usuário nota: "o ímã não puxa o herói").

### AreaGravityOverride (zona que SUBSTITUI gravidade/damping, com prioridade) — P2
- **Entrega:** dentro da zona a gravidade é OUTRA (direção/escala) ou zero — água profunda, espaço,
  gravidade invertida de puzzle; empilhamento por prioridade.
- **Equivalentes:** Godot Area2D `gravity_space_override` + `priority` (o único com o modelo completo).
- **PH2D:** NÃO como override (as Area* SOMAM força; substituir ≠ somar).
- **Prioridade:** P2. ⚠️ MEDIR composição primeiro: `AreaEffector` anti-gravidade + `GravityScale`
  pode aproximar; o que não exprime é a semântica "replace com prioridade".

### OneWayPlatform — ✅ JÁ EXISTE
- Unity Platform Effector 2D · Godot `one_way_collision` · Construct Jump-thru (+ ação
  `Fall through` — conferir se o drop-through por input já existe no PlatformPlayer).
- **PH2D:** SIM. P0 (feito).

### WalkSurface (material de superfície para o player: gelo, lama) — ✅ JÁ EXISTE
- Sem equivalente direto por componente nas outras (Unreal usa Physical Material + NavModifier).
  **PH2D:** SIM (+`NoWallCling`). P0 (feito). Vantagem a manter.

---

## F. FRONTEIRAS DE MUNDO (higiene que o Construct provou valer um componente)

### DespawnOutside — P1
- **Entrega:** destrói o objeto ao sair da região (mundo/câmera + margem) — fecha o vazamento
  clássico de balas/inimigos acumulando fora da tela. Zero propriedades além da região.
- **Equivalentes:** Construct **Destroy outside** · GDevelop Destroy when outside · Godot
  VisibleOnScreenNotifier2D (+ script de 1 linha).
- **PH2D:** NÃO. **Prioridade:** P1 — par obrigatório do ProjectileMotion e de qualquer spawner.
- **Nota:** o primo "desligar processamento fora da tela" (`OnScreenEnabler` JÁ EXISTE no ph2d-ecs)
  cobre a metade perf; este cobre a metade ciclo-de-vida.

### ScreenWrap — P2
- **Entrega:** saiu por um lado, entra pelo outro (Asteroids), com padding.
- **Equivalentes:** Construct **Wrap** · GDevelop Screen wrap · Phaser `body.wrap(padding)`.
- **PH2D:** NÃO. P2 (trivial, gênero-específico, mas "tem um behavior pra isso" soma na percepção).

### RegionBound — P2
- **Entrega:** clampa o objeto dentro de uma região (mundo/câmera), por origem ou por borda —
  o player que não sai da tela em 1 checkbox.
- **Equivalentes:** Construct **Bound to** · GDevelop Stay on screen · Phaser `collideWorldBounds`
  (+ evento worldbounds — copiar o EVENTO ao tocar a borda).
- **PH2D:** NÃO. P2.

---

## G. HELPERS DE MOVIMENTO (a cauda Construct/GDevelop que faz "tem um componente pra isso" ser verdade)

### PathFollow (seguir um caminho VETORIAL desenhado) — P1 ⭐ diferencial PH2D
- **Entrega:** o objeto anda por um path desenhado com a caneta do módulo Vector: progresso 0–1
  por comprimento de arco (a matemática que "quase ninguém acerta"), orientação pela tangente,
  offset lateral, loop/ping-pong, easing. Patrulha, trilho de plataforma, arco de projétil,
  cutscene de deslocamento = animar UM float na Timeline.
- **Equivalentes:** Godot Path2D+**PathFollow2D** · GM Path asset+follow embutido (speed % por
  ponto — copiar!) · Phaser Path+PathFollower · Unity Spline Animate · Unreal (ausente — monta-se
  à mão toda vez) · Construct (timeline track como path) · Bevy (só a matemática).
- **PH2D:** PARCIAL — os paths JÁ SÃO entidades de primeira classe (`VecPathRef`, família Vec*,
  kurbo por baixo = arc-length disponível); falta SÓ o seguidor. Nenhuma engine tem um editor de
  path deste nível acoplado ao seguidor — **"desenhe a patrulha com a caneta" é um demo matador**.
- **Prioridade:** P1 (primeiro da fila P1 — custo baixo, efeito de vitrine).
- **Dependências:** ponte VecPathRef (feita). Consumidores: MoveTo (fonte), KinematicPlatform,
  câmera em trilho (domínio Câmera), Spawner-along-path (domínio Spawn — anotar para o Unity
  Spline Instantiate).

### WaypointMotion (plataforma vai-e-volta por pontos) — P1
- **Entrega:** waypoints no viewport + duração + modo (OneShot / Loop / **PingPong**) + pause-on-
  impact + eventos — o clássico nº 1 de tutorial (plataforma móvel, porta, serra em trilho) sem
  timeline nem código.
- **Equivalentes:** Unreal **InterpToMovementComponent** (o desenho exato) · GDevelop
  Back-and-forth/Linear movement · Godot (AnimationPlayer ou PathFollow — exige autoria).
- **PH2D:** PARCIAL — a Timeline exprime (autoria mais pesada); o componente é o atalho de 30 s.
- **Prioridade:** P1. **Dependências:** KinematicPlatform (para carregar o player). **Nota:** medir
  se um preset de Timeline "colável" não entrega o mesmo com menos superfície (lei 5).

### PinTo (prender a outro objeto SEM reparentar) — P1
- **Entrega:** segue posição/ângulo (por canal: X, Y, ângulo, escala, opacidade — checkboxes) de
  outro objeto ou de um PONTO NOMEADO da animação dele (arma na mão do frame) — o "parent-child de
  gesto", sem os bugs de reparenting, com destroy-junto opcional.
- **Equivalentes:** Godot **RemoteTransform2D** · Construct **Pin** (pin to image point!) ·
  GDevelop Sticker · Unity Parent Constraint (com peso) · Cocos Spine Sockets (o caso osso).
- **PH2D:** PARCIAL — hierarquia real existe (`GroupedChildren`, Transform propagation); o que
  falta é o vínculo NÃO-hierárquico (undo/sorting/grupo não viajam junto) via `stable_name_id`.
- **Prioridade:** P1. **Risco:** definir a ordem no frame (depois do produtor, antes do render) —
  mesmo problema da janela de Signal, já resolvido uma vez no repo.

### TurretAim (aquisição de alvo + rotação + cadência + mira preditiva) — P1
- **Entrega:** detecta alvos num raio (por camada/tag), prioriza (primeiro/mais próximo), gira com
  velocidade limitada, dispara `Signal on-shoot` em cadência — e resolve a **interceptação
  balística** (mira onde o alvo VAI estar, dado o speed do projétil): o sistema de equações que
  ninguém quer escrever. Tower defense/shooter num painel; o usuário só spawna a bala.
- **Equivalentes:** Construct **Turret** (único com o pacote completo) · GDevelop Turret 2D
  movement (parcial).
- **PH2D:** NÃO. **Prioridade:** P1.
- **Dependências:** ProjectileMotion (o que ele dispara) · LineOfSight (opcional: "só se vê") ·
  Signal→ação (o On-shoot precisa de consumidor autorável ou spawner).

### DelayFollow (seguir com histórico: trenzinho, sombra, ghost) — P2
- **Entrega:** reproduz o histórico de outro objeto com atraso por tempo/distância, POR CANAL
  (posição/ângulo/opacidade…), com replay ("follow self") e histórico serializável (ghost de
  time-trial salvo em arquivo).
- **Equivalentes:** Construct **Follow** (único; o histórico-como-JSON é a cereja).
- **PH2D:** NÃO — mas o `TapeWire` (corrida gravada → bake) é meio caminho da infra de histórico.
- **Prioridade:** P2 (nicho, mas o ghost-replay é feature de vitrine barata sobre o TapeWire).

### ExplosionImpulse (impulso radial one-shot) — P2
- **Entrega:** "explodiu aqui": impulso radial com raio/falloff em tudo no alcance, fire-and-forget
  — acionável por Signal.
- **Equivalentes:** Unreal **RadialForceComponent** · GDevelop Explosion force · Construct Physics
  "apply force towards".
- **PH2D:** NÃO (Area* são campos SUSTENTADOS; explosão é impulso INSTANTÂNEO — outra semântica).
- **Prioridade:** P2. **Dependências:** tabela sinal→ação (o gatilho natural).

### ConstantThrust (força contínua local — foguete) — P2
- **Equivalentes:** Unreal PhysicsThruster · Godot `constant_force/torque` no RigidBody2D.
- **PH2D:** PARCIAL (InitialVelocity ≠ força contínua; motion nodes têm forças — medir, lei 5).
- **Prioridade:** P2 — provavelmente um CAMPO no RigidBody (constant_force), não componente novo.

### KinematicMotor (a válvula de escape: motor de movimento custom) — P2
- **Entrega:** para o usuário montar movimento exótico SEM perder as partes perigosas: integração
  com dt, sub-passos anti-túnel, e **push-out-solid** (depenetração pronta). O "faça você mesmo"
  como componente.
- **Equivalentes:** Construct **Custom Movement** (`Push out solid` é a joia) · Godot
  CharacterBody2D `move_and_slide` (a API, não componente).
- **PH2D:** PARCIAL — o Luau + rapier dão o cru; falta a depenetração/sub-passo exposta como serviço.
- **Prioridade:** P2 (sobe se o script ganhar UI — é o que torna script de movimento seguro).

### BoidsFlock — P2
- **Entrega:** flocking (coesão/separação/alinhamento) no objeto de cena.
- **Equivalentes:** GDevelop Boids movement (único).
- **PH2D:** PARCIAL — boids JÁ EXISTEM como Motion Node GPU. É o caso puro da lei 5: empacotar o
  node como componente/preset, nunca reescrever. P2.

---

## Ordem de construção sugerida (dependências resolvidas)

**Wave A (P0 — fecha a espinha):**
1. `SensorZone` (consumir `is_sensor` + eventos→Signal) — destrava triggers de gameplay.
2. `RaySensor` — destrava LineOfSight/TurretAim e todo ground-check custom.
3. `TopDownPlayer` — o segundo controller canônico (reusa o desenho lei-pura+ponte do platformer).
4. `ProjectileMotion` (+homing embutido) — o mover genérico.
   *(PlatformPlayer, RigidBody/Collider, Joints, Area*, OneWay, WalkSurface: já entregues.)*

**Wave B (P1 de composição — cada item multiplica os anteriores):**
5. `PathFollow` (vitrine: caneta→patrulha) → 6. `MoveTo` (executor de 3 fontes) →
7. `WaypointMotion` + `KinematicPlatform` (plataformas que carregam) →
8. `LineOfSight` + `TurretAim` → 9. `PhysicsGrab` → 10. `ConveyorSurface` +
fechar `AreaAttractor`→player → 11. `PinTo` · `DespawnOutside` · `ShapeSensor` ·
`TransformInterpolation` (medir antes) · `TileMotion` · `VehiclePlayer`.

**Wave C (P2):** OrbitMotion · ScreenWrap · RegionBound · DelayFollow · ExplosionImpulse ·
ConstantThrust · KinematicMotor · BoidsFlock · PhysicsMaterial · CompositeCollider ·
AreaGravityOverride · JointKind::Gear/Friction/Target.

**Riscos transversais:** (a) cada componente = seção artesanal no Inspector (o custo real, passo 5
do inventário) — os P0/P1 deste domínio somam ~15 seções novas: é o argumento mais forte a favor de
um derive/reflect de painel genérico ANTES da Wave B; (b) controllers exigem action mapping
(domínio Input) para o padrão Default-controls/Simulate; (c) tudo que dispara/reage pede a tabela
sinal→ação (R3, domínio Runtime) — sem ela, TurretAim/SensorZone/LineOfSight publicam num vazio.
