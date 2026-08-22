# Dossiê — Construct 3 & GDevelop: o catálogo de behaviors que elimina programação

> **Escopo:** levantamento exaustivo dos componentes ("behaviors") que o usuário ACRESCENTA a um objeto de cena
> nas duas engines padrão-ouro de "componente que elimina código", com propriedades de UI, o código que cada um
> elimina, sinergias e relevância para uma engine 2D (PH2D: componentes estilo Unity sobre ECS, decisão já tomada).
>
> **Fontes primárias:** para o Construct 3, além do manual oficial (bloqueado a fetch por Cloudflare, URLs citadas
> por seção), os **arquivos de dados do próprio editor oficial** (release r495.2) foram lidos diretamente:
> `https://editor.construct.net/r495-2/behaviors/behaviorList.json` e
> `https://editor.construct.net/r495-2/loader/lang/precompiled-en-US.json` — que contêm o catálogo completo de
> behaviors com TODAS as propriedades, descrições, conditions, actions e expressions exatamente como aparecem na UI.
> Para o GDevelop, a wiki oficial `https://wiki.gdevelop.io/`. Nada abaixo foi inventado: todo item vem de uma
> dessas fontes.

---

## 0. O modelo das duas engines (por que "behavior" funciona)

As duas engines compartilham a mesma arquitetura de autoria em três camadas:

1. **Objeto** (o "tipo visual": Sprite, Tilemap, Text…) — só desenha.
2. **Behavior** (componente que o usuário ADICIONA ao objeto num diálogo, configura por um painel de
   propriedades e pronto: o objeto passa a se COMPORTAR) — cada behavior traz um pacote fechado de
   propriedades + conditions + actions + expressions.
3. **Event sheet** (a fronteira onde behaviors acabam): tabela visual de `condições → ações`.
   - Construct 3: eventos rodam topo→baixo a cada frame; condições **filtram instâncias** ("object picking") e as
     ações agem só sobre as instâncias filtradas; sub-eventos herdam o picking do pai; há loops (For each,
     Repeat, While), grupos, includes, Functions e **Custom Actions** (métodos definidos pelo usuário sobre um
     objeto/família). Fonte: https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/events
   - GDevelop: idêntico em espírito — condições selecionam instâncias, ações agem sobre a seleção; sub-eventos,
     For each object, For each child variable, While, Repeat, links, grupos, blocos JavaScript e funções.
     Fonte: https://wiki.gdevelop.io/gdevelop5/events/

**O contrato-chave (a lição nº 1 para o PH2D):** o behavior NÃO é uma caixa-preta selada — cada um publica seu
vocabulário no event sheet (conditions: `Is on floor`, `On landed`; actions: `Simulate control`, `Set max speed`;
expressions: `Self.Platform.Speed`). O usuário programa POR CIMA do behavior sem reescrevê-lo. E todo behavior de
movimento tem o par **`Default controls` (bool) + ação `Simulate control`**: com um clique ele anda sozinho com as
setas (demo instantânea), e desligando o bool o MESMO behavior vira motor puro dirigido por eventos (IA, replay,
rede, touch). Terceiro padrão universal: **toda propriedade de editor tem ação `Set ...` correspondente em runtime**
e um bool `Enabled` — nada é só initial-value.

---

## 1. CONSTRUCT 3 — os 32 behaviors oficiais (catálogo completo, r495.2)

Fonte por seção: índice https://www.construct.net/en/make-games/manuals/construct-3/behavior-reference
(páginas individuais em `.../behavior-reference/<nome>`), propriedades extraídas verbatim dos dados do editor
oficial r495.2. Agrupados como no código-fonte do editor: `movements/`, `general/`, `attributes/`, `3d/`.

### 1.1 Movements (14) — controllers e motores de movimento

#### Platform — **o behavior mais famoso do gênero** · Relevância: ALTA
- **Faz:** "Jump and run" de visão lateral completo: corre, pula, cai, aterrissa em Solid/Jump-thru, sobe rampas,
  anda em plataformas móveis, gravidade em qualquer ângulo.
- **Propriedades:** Max speed (px/s) · Acceleration · Deceleration · Jump strength · Gravity ·
  Max fall speed · **Double-jump** (bool) · **Jump sustain** (ms — segurar o botão = pular mais alto) ·
  Default controls · Enabled.
- **Conditions (13):** Is moving · Compare speed · **Is on floor** · Is jumping · Is falling · **Is by wall** ·
  Is double-jump enabled · **On jump · On fall · On stopped · On moved · On landed** · Is enabled.
- **Actions (18):** Simulate control · Set vector X/Y · **Set angle of gravity** (gravidade rotacionável = andar
  em paredes/planetas) · Fall through (descer de jump-thru) · Set ceiling collision · Reset double jump · Set de
  cada propriedade.
- **Elimina:** TODO o character controller de platformer — integração de gravidade, detecção chão/parede/teto,
  coyote-ish sustain, snap a rampas, plataformas móveis que carregam o player, double jump com estado. É o código
  que a indústria inteira reescreve errado; aqui são 10 campos numa UI.
- **Compõe com:** Solid (chão), Jump-thru (plataformas), Sine/Tween em plataformas móveis, Timelines,
  Scroll To (câmera). Animação vem por events (`On landed → Set animation`).

#### 8 Direction — Relevância: ALTA
- **Faz:** movimento top-down 4/8 direções ou só 1 eixo, com aceleração e colisão contra Solids.
- **Propriedades:** Max speed · Acceleration · Deceleration · **Directions** (Up&down / Left&right / 4 / 8) ·
  **Set angle** (No / 90° / 45° / Smooth) · **Allow sliding** (deslizar ao longo de obstáculo em diagonal) ·
  Default controls · Enabled.
- **Actions:** Stop · Reverse · Set vector X/Y · Simulate control · Set ignoring input.
- **Elimina:** o controller top-down inteiro, incluindo o caso chato (deslizar em quinas em vez de travar).
- **Compõe com:** Solid, Scroll To, Line of sight (inimigos), Tile movement como alternativa em grade.

#### Bullet — Relevância: ALTA
- **Faz:** move o objeto para frente no ângulo atual; o "projétil genérico" e também o "movedor genérico barato".
- **Propriedades:** Speed · Acceleration (negativa desacelera) · **Gravity** (arco balístico) ·
  **Bounce off solids** (bool, reflexão pelo ângulo da superfície) · Set angle (bool) ·
  **Step** (sub-passos com trigger `On step` para teste de colisão extra em alta velocidade) · Enabled.
- **Conditions:** Compare speed · **Compare distance travelled** · On step.
- **Actions:** Bounce off object · Set angle of motion · Set vector 3D · Set gravity vector 3D.
- **Elimina:** física de projétil (velocidade+gravidade+ricochete+tunneling), contador de alcance.
- **Compõe com:** Turret (que atira), Timer/Fade (vida útil), Destroy outside (limpeza automática).

#### Car — Relevância: MÉDIA (nicho, mas é um veículo pronto)
- **Faz:** carro top-down com aceleração, ré, esterço e DRIFT.
- **Propriedades:** Max speed · Acceleration · Deceleration · **Steer speed** (°/s) ·
  **Drift recover** (taxa com que o ângulo de movimento alcança o ângulo do objeto — baixo = derrapa) ·
  **Friction** (perda ao raspar em Solid, 0–1) · Turn while stopped (bool) · Set angle · Default controls · Enabled.
- **Elimina:** o modelo "bicycle-lite" de veículo arcade com drift — notoriamente difícil de acertar no feel.
- **Compõe com:** Solid (pistas), Pathfinding (IA adversária via Simulate control).

#### Move To — Relevância: ALTA
- **Faz:** vai até um ponto/objeto com aceleração/desaceleração e fila de waypoints; para no destino e avisa.
- **Propriedades:** Max speed · Acceleration (0 = instantânea) · Deceleration · Rotate speed (°/s, 0 = não gira) ·
  Set angle · **Stop on solids** · Enabled.
- **Conditions:** **On arrived** · On hit solid · Is moving.
- **Actions:** Move to position/object (com opção de ENFILEIRAR waypoints) · **Move along Pathfinding path** ·
  **Move along timeline** — três fontes de trajetória, um só executor.
- **Elimina:** easing de chegada, filas de waypoint, "chegou?" com epsilon, integração com pathfinding.
- **Compõe com:** Pathfinding (ele executa o caminho achado), Timelines (segue curva autorada), Tween.

#### Pathfinding — Relevância: ALTA
- **Faz:** A* numa grade derivada automaticamente dos Solids (ou de obstáculos custom), com CUSTOS por região,
  cálculo assíncrono, e movimento embutido ao longo do caminho.
- **Propriedades:** **Cell size** · **Cell border** (folga anti-encostar) · Obstacles (Solids/Custom) ·
  Max speed · Acceleration · Deceleration · Rotate speed · Rotate object (bool) · **Diagonals** (bool) ·
  **Direct movement** (None / To destination / Anywhere along path — corta o zigue-zague da grade quando a área
  está livre) · Enabled.
- **Conditions:** **On path found / On failed to find path** · Is calculating · Is moving along path · On arrived.
- **Actions (21):** Find path · Move along path · Regenerate obstacle map/region/around object ·
  **Set move cost / Add path cost** (campos de custo!) · Start/End path group · Add obstacle · Clear obstacles.
- **Expressions:** NodeCount/NodeXAt/NodeYAt (o caminho é inspecionável) · RabbitX/Y.
- **Elimina:** A* + grid building + rebuild incremental + custo por terreno + steering ao longo do caminho —
  milhares de linhas.
- **Compõe com:** Solid (obstáculos de graça), Move To, Line of sight ("viu → persegue").

#### Physics — Relevância: ALTA
- **Faz:** corpo rígido Box2D completo no objeto.
- **Propriedades:** **Immovable** (massa infinita) · **Collision mask** (polígono de colisão / bounding box /
  círculo) · **Collision filter tags + mode (Inclusive/Exclusive)** · Prevent rotation · Density · Friction ·
  Elasticity · Linear damping · Angular damping · **Bullet** (CCD) · Enabled.
- **Conditions:** Is sleeping · Compare velocity/angular velocity/mass.
- **Actions (32):** Apply force/impulse/torque (no ponto, para posição, no ângulo) · Set velocity/angular velocity ·
  Teleport · Set world gravity · Set stepping mode/iterations · Enable/disable collisions ·
  **Create distance/revolute/limited revolute/prismatic joint** · Remove all joints · Set awake.
- **Expressions:** contatos (contact-count, contact-x/y-at), centro de massa.
- **Elimina:** toda a ponte com o motor físico: criação/destruição de corpos, sincronização transform↔corpo,
  filtros, joints, o "teleport sem explodir a simulação".
- **Compõe com:** os DEMAIS movimentos NÃO se misturam com Physics (regra documentada da casa: um dono do
  transform por vez — mesma lei que o PH2D já pratica no rapier).

#### Custom Movement — Relevância: MÉDIA/ALTA (é a "válvula de escape")
- **Faz:** motor de movimento de baixo nível para o usuário montar o próprio: expõe velocidade, ângulo de
  movimento, aceleração dirigida e sub-passos.
- **Propriedades:** **Stepping mode** (None / Linear / Horizontal-then-vertical / Vertical-then-horizontal) ·
  Pixels per step · Enabled.
- **Actions:** Set angle of motion · Accelerate toward angle/position · Reverse · **Push out solid /
  Push out solid at angle** (resolução de penetração pronta!) · Stop stepping.
- **Elimina:** o esqueleto de qualquer movimento exótico: dt-integração, sub-stepping anti-túnel, depenetração.
- **Lição:** mesmo o "faça você mesmo" é um COMPONENTE com as partes perigosas resolvidas.

#### Follow — Relevância: ALTA (juice + gameplay)
- **Faz:** segue outro objeto reproduzindo o histórico dele por TEMPO ou DISTÂNCIA (trenzinho, sombra com delay,
  replay fantasma).
- **Propriedades:** Mode (Time/Distance) · Delay · Max delay · History rate (amostras/s) · **9 bools de canal:**
  Follow X · Y · Z · width · height · angle · opacity · visibility · **Follow destroyed** · Enabled.
- **Actions:** Follow object · Follow self (replay!) · Rewind history · Clear history · **Load history JSON** ·
  Start following custom property.
- **Expressions:** history-as-json (o histórico é serializável — replay/ghost salvo em arquivo).
- **Elimina:** ring buffer de poses + amostragem + interpolação + replay — por canal escolhível em checkbox.
- **Compõe com:** qualquer movimento; Pin para prender rígido é o irmão sem delay.

#### Orbit — Relevância: MÉDIA/ALTA
- **Faz:** órbita elíptica em torno de um alvo (ponto ou objeto).
- **Propriedades:** Speed (°/s, sinal = sentido) · Acceleration · **Primary radius · Secondary radius**
  (círculo = iguais) · Offset angle (rotação da elipse) · Match rotation (encarar a tangente) · Enabled ·
  **Preview no editor**.
- **Actions:** Set target · Pin/Unpin (a órbita SEGUE um objeto móvel) · Reset total rotation.
- **Elimina:** trigonometria de órbita + acompanhar alvo móvel + contagem de voltas (total-rotation).

#### Rotate — Relevância: MÉDIA
- **Faz:** gira o objeto continuamente.
- **Propriedades:** Speed (°/s) · Acceleration · Rotation type (2D / eixo X / Y / Z) · Enabled · **Preview**.
- **Elimina:** `angle += speed*dt` — trivial, mas com preview no editor e zero eventos: é a peça de cenário viva.

#### Sine — Relevância: ALTA (o "oscilador universal", juice barato)
- **Faz:** oscila QUALQUER propriedade do objeto com uma forma de onda.
- **Propriedades:** **Movement** (Horizontal / Vertical / Forwards-backwards / Width / Height / Size / Angle /
  Opacity / Z elevation / **Value only** — vira LFO puro lido por expressão) · **Wave** (Sine / Triangle /
  Sawtooth / Reverse sawtooth / Square) · Period · Period random · Period offset · Period offset random ·
  Magnitude · Magnitude random · Enabled · **Preview**.
- **Elimina:** todo idle-motion (moeda flutuando, planta balançando, pulso de UI) — e os campos `random`
  dessincronizam a cópia N do objeto DE GRAÇA (o bug clássico "todas as moedas dançam em fase" já vem resolvido).
- **Compõe com:** empilha por cima de qualquer movimento (é offset aditivo); Value only alimenta events.

#### Tile movement — Relevância: ALTA para o gênero
- **Faz:** movimento casa-a-casa numa grade 2D (roguelike/puzzle/sokoban), cartesiana ou **isométrica**.
- **Propriedades:** Grid width/height · Grid offset X/Y · Speed X/Y · Default controls · **Isometric** (bool) ·
  Enabled.
- **Conditions:** **Can move to / Can move in direction** (consulta de passabilidade pronta) · Is moving in direction.
- **Actions:** Set grid position · Simulate control · Set grid dimensions.
- **Elimina:** snap à grade + animação entre células + fila de input + bloqueio por Solid.

#### Turret — Relevância: ALTA (tower defense/shooter inteiro num painel)
- **Faz:** detecta alvos num raio, escolhe, gira até eles e dispara em cadência.
- **Propriedades:** Range (px) · **Rate of fire** (s) · Rotate (bool) · Rotate speed · **Target mode**
  (First in range / Nearest) · **Predictive aim** (bool — mira onde o alvo VAI estar) · Projectile speed
  (para a predição) · Use collision cells · Enabled.
- **Conditions:** **On shoot** (aqui o usuário só spawna a bala) · On target acquired · Has target.
- **Actions:** Add object to target (declara O QUE é alvo — ex.: família "Enemies") · Acquire/Unacquire target.
- **Elimina:** aquisição de alvo por distância, priorização, rotação limitada, cooldown, e ATÉ a solução da
  interceptação balística (predictive aim é um sistema de equações que ninguém quer escrever).
- **Compõe com:** Bullet (a bala), famílias (alvos), Line of sight (só atirar se vê).

### 1.2 General (10) — utilitários que grudam em qualquer objeto

#### Anchor — Relevância: ALTA (UI responsiva sem código)
- **Faz:** prende bordas do objeto às bordas do viewport (HUD que sobrevive a resize/fullscreen).
- **Propriedades:** Left edge (Viewport left/right/None) · Top edge (top/bottom/None) ·
  Right edge (Viewport right/None — ancorar as duas horizontais = ESTICA o objeto) · Bottom edge · Enabled.
- **Elimina:** todo o layout reflow de HUD em resolução variável.

#### Bound to — Relevância: MÉDIA
- **Faz:** impede o objeto de sair do layout ou do viewport.
- **Propriedades:** Bound by (Origin/Edge) · Region (Layout/Viewport).
- **Elimina:** o clamp de posição em 4 bordas (e o off-by-half do "pela origem ou pela borda?").

#### Destroy outside — Relevância: ALTA (higiene de memória em 1 clique)
- **Faz:** destrói o objeto ao sair do layout/viewport.
- **Propriedades:** Region (Layout/Viewport).
- **Elimina:** o vazamento clássico de balas/inimigos acumulando fora da tela.

#### Drag & Drop — Relevância: ALTA
- **Faz:** arrastar com mouse OU touch (multitouch: vários objetos ao mesmo tempo).
- **Propriedades:** Axes (Both / Horizontal only / Vertical only) · Enabled.
- **Conditions:** **On drag start · On drop** · Is dragging. **Actions:** Drop programático.
- **Elimina:** hit-test + captura de ponteiro + offset do clique + multi-touch bookkeeping.

#### Fade — Relevância: ALTA (o "aparece-espera-some" universal)
- **Faz:** rampa de opacidade in→wait→out, com autodestruição opcional.
- **Propriedades:** Fade in time · Wait time · Fade out time (0 pula a etapa) · **Destroy** (bool) ·
  Enabled (= começa sozinho) · **Preview**.
- **Conditions:** On fade-out finished / On fade-in finished / On wait finished.
- **Elimina:** temporizador + lerp de alpha + destroy — o ciclo de vida de QUALQUER efeito visual (poeira,
  popup de dano, corpo de inimigo).

#### Flash — Relevância: ALTA (feedback de dano padrão da indústria)
- **Faz:** pisca (alterna visível/invisível) por uma duração.
- **Propriedades:** nenhuma! (tudo na action `Flash(on-time, off-time, duration)`).
- **Conditions:** Is flashing · On flash ended (= fim da invencibilidade).
- **Elimina:** o timer de i-frames + toggle de visibilidade.

#### Line of sight — Relevância: ALTA (a metade de toda IA 2D)
- **Faz:** "A vê B?" com alcance, cone e obstáculos; e RAYCAST genérico com normal e reflexão.
- **Propriedades:** Obstacles (Solids/Custom) · Range (px) · **Cone of view** (°) · Use collision cells · Enabled.
- **Conditions:** Has LOS to object/position/between positions · **Ray intersected**.
- **Actions:** **Cast ray** · Add obstacle · Clear obstacles.
- **Expressions (12):** HitX/Y/UID/Distance · **NormalX/Y/Angle · ReflectionX/Y/Angle** (laser que ricocheteia
  em 3 expressões).
- **Elimina:** raycast contra colisão + oclusão + cone angular — o "percebeu o player?" vira uma condition.
- **Compõe com:** Pathfinding (viu → persegue), Turret (vê → atira), Solid (obstáculos de graça).

#### Pin — Relevância: ALTA
- **Faz:** gruda um objeto noutro mantendo distância/ângulo relativos (arma na mão, barra de vida na cabeça).
- **Propriedades:** **Destroy with pinned object** (bool).
- **Actions:** Pin to object (modos posição+ângulo/só posição/etc.) · **Pin to image point** (segue um ponto
  nomeado DENTRO da animação do pai!) · Pin at distance · Unpin.
- **Elimina:** parent-child transform manual por frame. (Hoje coexiste com as **hierarchies** nativas do editor —
  ver §1.5 — mas segue sendo o jeito dinâmico via events.)

#### Scroll To — Relevância: ALTA (a câmera de gameplay em um checkbox)
- **Faz:** a câmera segue este objeto; com VÁRIAS instâncias marcadas, centra no PONTO MÉDIO delas (co-op local
  de graça).
- **Propriedades:** Enabled.
- **Actions:** **Shake** (magnitude, duração, decaimento — screen-shake oficial embutido) · Set enabled.
- **Elimina:** camera-follow + média de alvos + shake com decay.
- **Compõe com:** propriedades de LAYER (parallax, escala) para o resto do trabalho de câmera.

#### Timer — Relevância: ALTA (o "setTimeout do designer")
- **Faz:** N timers NOMEADOS por instância, one-shot ou periódicos.
- **Propriedades:** nenhuma; tudo por actions.
- **Conditions:** **On timer "tag"** · Is timer running/paused.
- **Actions:** Start timer (duração, once/regular, tag) · Stop · Pause/resume (por tag ou todos).
- **Expressions:** CurrentTime · TotalTime · Duration · **normalized-progress** (0–1, pronto para barra de recarga).
- **Elimina:** toda a contabilidade de cooldowns/spawners/delays por instância — a alternativa é um dicionário de
  acumuladores dt que todo iniciante escreve errado.

#### Tween — Relevância: ALTA (a animação procedural inteira)
- **Faz:** anima propriedades do objeto por curvas de easing, com tags, em paralelo.
- **Propriedades do painel:** só Enabled — o poder todo está nas actions:
- **Actions (24):** Tween one property (**X, Y, Z, Size, Width, Height, Depth, Angle, Opacity, Color, X/Y/Z
  Scale**) · Tween two/three properties (**Position, Size, Scale** juntos) · **Tween value** (número arbitrário
  lido por expressão) — cada uma com: End value · Time · **Ease** (biblioteca completa de easings) ·
  **Destroy on complete** · **Loop** · **Ping pong** · Repeat count · Tags. Mais: Pause/Resume/Stop (por tag/todos),
  Set time/ease/end value em voo, Set playback rate.
- **Conditions (14):** On finished (por tag) · On any finished · On looped · On ping-pong · Is playing…
- **Elimina:** o motor de interpolação inteiro + agendamento + composição — é o `DOTween` sem uma linha de código.

#### Wrap — Relevância: MÉDIA
- **Faz:** saiu por um lado, entra pelo outro (Asteroids).
- **Propriedades:** Wrap to (Layout/Viewport). **Condition:** On wrap.

### 1.3 Attributes (5+2) — behaviors-MARCADOR (zero código, mudam como os sistemas tratam o objeto)

- **Solid** — marca o objeto como impassável para TODOS os behaviors de movimento (Platform, 8-Dir, Bullet
  bounce, Pathfinding obstacles, LOS, Tile movement…). Propriedades: Enabled · **Tags / Use instance tags +
  filtro por tags** (um Solid pode ser sólido para o inimigo e não para o player). Relevância: ALTA — é a
  **moeda de troca entre behaviors**: um marcador que dezenas de sistemas consultam.
- **Jump-thru** — plataforma atravessável por baixo, pisável por cima (só o Platform a respeita; com a action
  `Fall through` no Platform). Relevância: ALTA no gênero.
- **Persist** — a instância LEMBRA seu estado ao revisitar o layout (posição, vida, o que foi coletado…), em vez
  de resetar. Zero propriedades. Relevância: ALTA — persistência de mundo em 1 clique.
- **No save** — exclui o objeto do sistema de savegame (cenário estático → saves menores/rápidos). Zero
  propriedades. Relevância: MÉDIA/ALTA — o opt-out da serialização é ele mesmo um componente.
- **Shadow caster** — o objeto projeta sombra 2D a partir de um objeto "Shadow light" (Height define o comprimento;
  Tag filtra quais luzes). **[ADIADO — iluminação 2D foi adiada por decisão do dono, 2026-08-20; listado sem
  prioridade.]**
- (No editor r495.2 há ainda 2 de fronteira:) **Billboard** (3D: encara a câmera; offsets X/Y/Z) — BAIXA para 2D
  puro; e o par **NoSave/Persist** acima já contado.

### 1.4 Os OBJETOS (plugins) que completam o quadro — checklist de categorias
Fonte: `plugins/pluginList.json` + lang do editor r495.2; manual em
https://www.construct.net/en/make-games/manuals/construct-3/plugin-reference

- **Visual/render:** Sprite (animações por frames + **mesh distortion** — grade de pontos deformável por
  actions/scripting, cf. blog oficial "Have you heard about Meshes?" https://www.construct.net/en/blogs/construct-official-blog-1/heard-meshes-1567) ·
  9-patch (margens L/R/T/B, tile/stretch por borda) · Tiled Background · **Tilemap** (com suporte a TMX) ·
  **Particles** (25 propriedades: rate, spray cone, spray/one-shot, imagem OU **qualquer objeto como partícula**,
  speed/size/opacity/grow + randomizadores de cada um, aceleração, gravidade, destroy mode, timeout, **preview no
  editor**) · Text · Sprite font · SVG Picture · Drawing canvas (desenho procedural). — Trail/line oficial:
  **NÃO EXISTE** (se faz com Particles ou Drawing canvas).
- **Esqueleto/deform 2D:** **NÃO EXISTE oficial** (mesh distortion do Sprite é o parcial; Spine/Spriter são
  addons de terceiros).
- **Câmera:** não é objeto — Scroll To (behavior) + propriedades de layer (parallax/escala/ângulo) + action Shake.
- **Áudio:** plugin Audio central (não posicional por objeto na cena, mas com pan/posicionamento e tags por som;
  análise/efeitos). Listener implícito.
- **Input:** Keyboard · Mouse · Touch (+orientação/motion) · Gamepad — plugins globais; **não há um asset de
  "action mapping"** oficial: o mapeamento é feito nos event sheets ou pelos Default controls dos behaviors.
- **UI in-game:** Button · Text input · List · Slider bar · Progress bar · File chooser · iframe · HTML Element —
  são controles DOM sobrepostos ao canvas (limitação honesta documentada).
- **Dados/persistência:** Local storage · Array · Dictionary · JSON · XML · CSV · Binary data + system save/load
  (com Persist/No save nos objetos).
- **Rede:** Multiplayer (WebRTC DataChannels, rooms; sem sincronizador drop-in por objeto — o par de peers troca
  mensagens; bem mais manual que o do GDevelop) · WebSocket · AJAX.
- **Timeline/sequencer:** **Timelines** é cidadão do EDITOR (project primitive): keyframes de propriedades de
  instâncias, master keyframes, easing por keyframe, tracks de posição usáveis como PATH (Move To
  "Move along timeline"), tudo controlado em runtime pelo plugin **Timeline controller**.
  https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/timelines
- **State machine visual:** **Flowcharts** + plugin Flowchart controller (diálogos, FSMs de alto nível).
- **Navegação/AI:** Pathfinding + LOS (behaviors); **behavior tree oficial: NÃO EXISTE**.
- **Spawn/factory/pooling:** **NÃO EXISTE como componente** — `System: Create object` nos events; pooling é
  interno/automático do runtime.

### 1.5 Composição e o "componente próprio" no Construct 3
- **Hierarchies (scene graph nativo):** no editor, arrastar um objeto sobre outro cria parent-child com herança
  seletiva POR CANAL (transform X, Y, ângulo, largura, altura, opacidade, visibilidade, z-elevation — checkboxes),
  e `Create object with hierarchy` instancia a árvore inteira. (Blog oficial "Let's talk about the Scene Graph".)
- **Containers:** objetos compostos — criar/selecionar/destruir uma peça pega/spawna as irmãs juntas.
  https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/objects/containers
- **Families:** "traits" — um behavior/variável/efeito posto na família vale para todos os membros; conditions
  visam a família ("Bullet colide com Enemies"). https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/objects/families
- **Custom Actions:** métodos definíveis em event sheets sobre um objeto/família (a meio-caminho entre eventos e
  behavior próprio). https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/events/custom-actions
- **Behavior próprio de verdade = Addon SDK (JavaScript/TypeScript):** `behavior.js` declara as propriedades
  (aparecem no MESMO painel de UI dos oficiais), aces.json declara conditions/actions/expressions, runtime em JS;
  distribui como .c3addon. Ou seja: **usuário final compõe por UI; estender o CATÁLOGO exige código** — diferença
  crucial para o GDevelop abaixo. https://www.construct.net/en/make-games/manuals/addon-sdk

---

## 2. GDEVELOP — behaviors oficiais + extensões revisadas

Fontes: índice https://wiki.gdevelop.io/gdevelop5/behaviors/ · lista completa
https://wiki.gdevelop.io/gdevelop5/behaviors/all-behaviors/ · páginas individuais citadas por item.
O GDevelop tem DOIS anéis: (a) behaviors **embutidos** (core), (b) **extensões revisadas** — behaviors da
comunidade PROMOVIDOS a oficiais após revisão, instaláveis de dentro do editor (mais um anel "community/
experimental" que NÃO listo como oficial aqui, exceto onde marcado).

### 2.1 Core (embutidos)

#### Platformer character + Platform — Relevância: ALTA
https://wiki.gdevelop.io/gdevelop5/behaviors/platformer/
- **Par de behaviors:** o personagem leva `Platformer character`; o chão leva `Platform`, cujo tipo é
  **Platform / Jumpthru / Ladder** (escada é um TIPO de plataforma — elegante).
- **Propriedades do character:** Gravity · Max falling speed · Jump speed · **Jump sustain** · Acceleration ·
  Deceleration · Max speed · **Air control** · **Slope max angle** (~60° default) · **Ladder climbing speed** ·
  **Can grab platform ledges** (+ offset Y e tolerância X do agarrão!) · Can go down from jumpthru ·
  Default controls.
- **Elimina:** o mesmo do Platform do C3, MAIS escada e ledge-grab de fábrica.
- **Compõe com:** Platformer keyboard/gamepad mapper (remapeio por UI), Platformer character animator
  (troca animação por estado SEM eventos), Advanced platformer movements (coyote time, dash…), Smooth
  platformer camera.

#### Top-down movement (4/8 direções) — Relevância: ALTA
https://wiki.gdevelop.io/gdevelop5/behaviors/topdown/
- **Propriedades:** Acceleration · Deceleration · Max speed · Rotation speed · Angle offset · Rotate object ·
  **Allows diagonals** · **Viewpoint: Top-Down / Isometry 2:1 / True Isometry 30° / Custom** (o MESMO controller
  serve jogo isométrico mudando um dropdown!) · Default controls · Movement angle snapping.
- **Elimina:** controller top-down + toda a matemática de projeção isométrica de input.

#### Physics Engine 2.0 — Relevância: ALTA
https://wiki.gdevelop.io/gdevelop5/behaviors/physics2/
- **Propriedades:** **Body type (Dynamic/Static/Kinematic)** · Bullet (CCD) · Fixed rotation · Can sleep ·
  **Shape (Box/Circle/Edge/Polygon com offset e escala)** · Density · Friction · Restitution · Linear damping ·
  Angular damping · **Gravity scale** (por corpo!) · **Layers e Masks (16×16)**.
- **Joints (11 tipos!):** Distance · Revolute · Prismatic · **Pulley · Gear · Mouse · Wheel · Weld · Rope ·
  Friction · Motor** — todos criados por ACTIONS de evento.
- **Elimina:** a ponte Box2D inteira, com um catálogo de joints maior que o do C3.
- **Nota:** também há **3D physics engine + 3D physics character + 3D physics car** core (fora de escopo 2D,
  registrados por completude).

#### Pathfinding (grid) + Pathfinding obstacle — Relevância: ALTA
https://wiki.gdevelop.io/gdevelop5/behaviors/pathfinding/
- **Par personagem/obstáculo.** Personagem: Acceleration · Max speed · Rotate object + Rotate speed +
  Angle offset · Virtual cell width/height · Grid offset X/Y · Extra border · Allow diagonals · Smoothing max
  cell gap. Obstáculo: **Impassable (bool)** · **Cost** (multiplicador — pântano custa 4×).
- **Diferença para o C3:** obstáculos são MARCADOS por behavior (não derivados de Solid — GDevelop não tem Solid
  global!), e o custo mora NO OBSTÁCULO.

#### NavMesh pathfinding (character + floor/obstacle) — Relevância: ALTA
https://wiki.gdevelop.io/gdevelop5/behaviors/nav-mesh-pathfinding/
- Malha de navegação gerada automaticamente dos obstáculos, movimento contínuo em qualquer direção,
  **multidão com esquiva mútua** (Avoidance sight range + Radius por agente), rebuild automático quando
  obstáculos mudam.
- **Elimina:** navmesh generation + funnel + local avoidance — o degrau que separa engine "de brinquedo" de
  engine séria.

#### Tween — Relevância: ALTA
https://wiki.gdevelop.io/gdevelop5/behaviors/tween/
- Tudo que o do C3 faz; alvos: posição (X/Y/Z), ângulo, escala (**interpolação EXPONENCIAL por default** — escala
  anima "certo" multiplicativamente), width/height/depth, opacidade, **cor em HSL (default) ou RGB**, tamanho de
  fonte, **parâmetros de EFEITO (shader)**, **variáveis de objeto/cena**, valores arbitrários; easings de
  easings.net; tweens nomeados (restart por nome), encadeamento por condição "tween finished".

#### Draggable — ALTA · https://wiki.gdevelop.io/gdevelop5/behaviors/draggable/
- Propriedade única: **Check collision mask** (arrasto só em pixel não-transparente). Topo do Z-order ganha o
  drag. Conditions: is being dragged / drag started / stopped.

#### Anchor — ALTA · https://wiki.gdevelop.io/gdevelop5/behaviors/anchor/
- Por borda (4): **None / Window side / Window center / Proportional** — mais rico que o do C3 (centro e
  proporcional); ancorar bordas opostas ESTICA (mesma semântica).

#### Destroy when outside of the screen — ALTA · https://wiki.gdevelop.io/gdevelop5/behaviors/destroyoutside/
- Idêntico ao Destroy outside do C3.

#### Multiplayer object — Relevância: ALTA (é o sincronizador drop-in que o C3 não tem)
https://wiki.gdevelop.io/gdevelop5/all-features/multiplayer/
- Behavior no objeto → sincroniza AUTOMATICAMENTE posição/rotação/escala, animações, efeitos, **variáveis do
  objeto**, forças/física e timers entre até 8 jogadores; **ownership por host/jogador com transferência por
  action**; lobbies prontos (auth, matchmaking, conexão) sem servidor do usuário.
- **Elimina:** TODA a rede: replicação, interpolação, predição, gestão de conexão. O behavior mais agressivo em
  "código eliminado" das duas engines.

#### Light Obstacle + objeto Light — **[ADIADO — iluminação 2D adiada por decisão do dono; registrado sem
prioridade]** (behavior marca obstáculos de luz 2D; objeto Light emite).

#### Save state configuration (+ sistema Save state) — Relevância: ALTA
https://wiki.gdevelop.io/gdevelop5/all-features/save-state/
- O sistema salva TUDO (objetos, variáveis, sons, efeitos, timers, ações assíncronas) com actions
  `Save/Load game to device storage` em slots; o behavior por objeto configura **Persisted / Do not save** +
  perfis nomeados (checkpoint vs. save completo).
- **Elimina:** serialização inteira do jogo. (É o par Persist/No save do C3, elevado a sistema com slots.)

### 2.2 Extensões REVISADAS (community → oficial) — as que definem categoria
Lista completa (nome + 1 linha) em https://wiki.gdevelop.io/gdevelop5/behaviors/all-behaviors/ — abaixo as de
maior peso conceitual; TODAS confirmadas na lista oficial:

- **Fire bullets** · https://wiki.gdevelop.io/gdevelop5/extensions/fire-bullet/ — a ARMA como componente:
  Firing cooldown (0.1) · Firing arc (45°) · **Heat/overheat** (increase per shot, linear+exponential cooling,
  overheat duration) · **Reloading** (duration, shots per reload, automatic) · **Ammo** (max, starting,
  unlimited) · Bullets per shot · Angle/speed variance · Rotate bullets. Actions: fire at angle/object/position.
  Elimina: cooldown+ammo+reload+overheat+spread — o sistema de tiro completo por UI. Relevância: ALTA.
- **Health** · https://wiki.gdevelop.io/gdevelop5/extensions/health/ — vida/escudo/armadura como componente:
  max health, over-heal, shield (pontos/duração/regen), armadura plana e %, regen com delay, damage cooldown
  (i-frames), dodge chance. Conditions: Is dead · Is just damaged · Is just dodged. Elimina: o RPG-combat-core.
  Relevância: ALTA.
- **Sticker** · https://wiki.gdevelop.io/gdevelop5/extensions/sticker/ — o Pin do GDevelop (segue posição e
  opcionalmente rotação; destroy junto opcional). Relevância: ALTA.
- **Object spawner** — spawna objetos periodicamente (a fábrica como behavior). Relevância: ALTA.
- **Câmera (categoria SEM core, coberta por extensões):** **Smooth Camera** (follow com lerp) ·
  **Smooth platformer camera** (estabiliza no pulo) · Third person camera · Scrollable viewport (pan por drag) ·
  Shake object / 3D camera shake. Relevância: ALTA.
- **Input mappers (categoria própria!):** Platformer keyboard/gamepad mapper · Top-down keyboard/gamepad
  mapper · Multitouch joystick (virtual) · Cursor · Swipe/Pinch gesture · Konami Code — o REMAPEIO de controles
  é um behavior anexável, não código. Relevância: ALTA (modelo interessante: o mapper é um componente SEPARADO
  do controller).
- **Animators:** Platformer character animator · Top-down movement animator — trocam a animação do sprite pelo
  ESTADO do controller (idle/run/jump/fall) sem um único evento. Relevância: ALTA (fecha o ciclo
  controller→animação por UI pura).
- **Movimento utilitário:** Boids movement (flocking!) · Homing projectile · Advanced projectile · Boomerang ·
  Bounce (forces) · Ellipse/Rectangular/Curved/Linear movement · Back-and-forth (timed/animated) · Screen wrap ·
  Stay on screen · Speed restrictions · Pixel perfect movement · **Make objects orbit** · Turret 2D movement ·
  2D top-down physics car · Explosion force · Magnetic effect (experimental) · Face forward · Travel to random
  positions. Relevância: MÉDIA/ALTA (é a cauda longa que faz "tem um behavior pra isso" ser verdade).
- **Juice/visual:** Flash object/layer · Shake object · Sway · Shock wave · Slice object into pieces ·
  Tween into view · **YSort** (depth por Y — o painter's order de top-down em um behavior!) · Object masking ·
  Rolling counter · Animated shadow clones · Typewriter text. Relevância: ALTA (YSort e typewriter em especial).
- **UI (dezenas):** Button states · Labeled button · Slider · Toggle switch · Resource bar (contínua/unidades) ·
  Star rating · Pop-up · Game over dialog · Two choices dialog · Volume settings · Time formatting · Player
  avatar · Animated score counter — a biblioteca de HUD como behaviors + prefabs. Relevância: MÉDIA/ALTA.
- **Rede/serviços:** Advanced HTTP · WebSocket client · MQTT · P2P avançado · Multiplayer custom lobbies ·
  Internet connectivity. Relevância: MÉDIA.
- **Gameplay avulso:** Idle tracker · Is on screen · Link path finding (grafo de waypoints LIGADOS — pathfinding
  por links!) · Object stack · Repeat every X seconds (timer periódico como behavior) · FPS displayer ·
  Pathfinding painter. Relevância: MÉDIA.

### 2.3 Objetos (checklist visual/render) — https://wiki.gdevelop.io/gdevelop5/objects/
Sprite (animações + collision mask + points) · Tiled Sprite · **Panel Sprite (9-patch)** · **Tilemap** (nativo +
**externo LDtk/Tiled**) · Text · Bitmap Text · BBText · Particle Emitter · Shape Painter (desenho procedural) ·
Video · Light **[ADIADO]** · **Spine** (esqueleto 2D OFICIAL — categoria que o C3 não tem!) · 3D Box/Model/Light ·
UI prefabs (Button, Text Input, Slider, Toggle, Resource Bar, Multitouch Joystick) · **Custom Objects (prefabs:
objetos compostos de objetos, com eventos próprios — o "prefab com script embutido")**.
- **Trail/line oficial: NÃO EXISTE** (Shape Painter cobre na mão). **Mesh 2D deformável: NÃO EXISTE**
  (Spine cobre o caso esqueleto/deform). **Timeline/sequencer de editor: NÃO EXISTE** (o C3 tem; no GDevelop
  anima-se por Tween/eventos). **Behavior tree: NÃO EXISTE** nas duas.

### 2.4 Onde os behaviors ACABAM: o event sheet (e os timers)
- Toda lógica de LIGAÇÃO ("quando X colide com Y, tire vida") é events: conditions filtram instâncias, actions
  agem, sub-eventos refinam. https://wiki.gdevelop.io/gdevelop5/events/
- **Timers:** cena e POR OBJETO, nativos dos events (start/check/reset), respeitam time scale global.
  https://wiki.gdevelop.io/gdevelop5/all-features/timers-and-time/
- **Áudio:** por events (canais, música/efeitos), sem componente de cena; posicional só via extensão experimental
  "Sound volume based on distance". Declaração honesta: **áudio posicional 2D como componente: NÃO EXISTE core.**

### 2.5 Criar o próprio behavior SEM CÓDIGO — o diferencial estrutural do GDevelop
https://wiki.gdevelop.io/gdevelop5/behaviors/events-based-behaviors/
- Behaviors são criáveis DENTRO do editor com os MESMOS event sheets do jogo: propriedades tipadas (número,
  string, cor, dropdown, bool, layer) que viram o painel de UI automaticamente; lifecycle
  (`onCreated`, `doStepPreEvents`, `doStepPostEvents`, `onDestroy`, `onActivate/onDeActivate`); funções custom
  viram conditions/actions/expressions no vocabulário do event sheet; **required behaviors** (um behavior pode
  DEPENDER de outro — composição declarada); distribuição pela galeria com processo de revisão → é EXATAMENTE o
  funil que transformou 200+ behaviors da comunidade no catálogo revisado da §2.2.
- **Contraste:** no Construct 3, estender o catálogo exige JavaScript (Addon SDK). No GDevelop, o catálogo é
  auto-alimentado pela mesma ferramenta visual — por isso a cauda longa deles é 6× maior.

---

## 3. Checklist de categorias × as duas engines (declaração explícita)

| Categoria | Construct 3 | GDevelop |
|---|---|---|
| Sprite/animado | Sprite (plugin) + mesh distortion | Sprite; Spine para esqueleto |
| Tilemap | Tilemap (TMX) | Tilemap nativo + LDtk/Tiled |
| Nine-patch | 9-patch | Panel Sprite |
| Trail/line | **NÃO TEM oficial** | **NÃO TEM oficial** (Shape Painter manual) |
| Mesh 2D/deform | Sprite mesh distortion (parcial) | **NÃO TEM** |
| Esqueleto 2D | **NÃO TEM oficial** (3rd party) | **Spine (oficial)** |
| Partículas | Particles (25 props, preview) | Particle Emitter |
| Câmera gameplay | Scroll To + Shake + layers | **Sem core**; extensões Smooth Camera etc. |
| Física/colisão | Physics (Box2D, 4 joints) + Solid/Jump-thru | Physics 2.0 (Box2D, **11 joints**, layers/masks) |
| Raycast | LOS (Cast ray + normal/reflexão) | **Sem raycast genérico core** (LOS não é core) |
| Controller platformer | Platform | Platformer character (+ladder, ledge grab) |
| Controller top-down | 8 Direction · Tile movement | Top-down (+isometria) |
| Veículo | Car | 3D car core; 2D physics car (extensão) |
| Caminhos/splines | Timeline position track como path | **NÃO TEM** (Curved/Ellipse movement aproximam) |
| Animação (tween) | Tween · Timelines | Tween |
| Timeline/sequencer | **Timelines + Timeline controller (editor!)** | **NÃO TEM** |
| State machine visual | Flowcharts | **NÃO TEM** (FSM é community, não revisado) |
| Áudio | Audio plugin central | Events; posicional = experimental |
| Input mapping | Default controls + Simulate control; **sem asset de mapeamento** | **Mappers como behaviors** (remap por UI) |
| UI in-game | Form controls DOM + 9patch/SpriteFont | Prefabs + dezenas de behaviors de UI |
| Pathfinding | A* grid (custos, grupos) | Grid **e NavMesh (crowd avoidance)** |
| Behavior tree | **NÃO TEM** | **NÃO TEM** |
| Spawn/pooling | **NÃO TEM componente** (System.Create) | Object spawner (extensão revisada) |
| Timers | Timer behavior (tags, por instância) | Scene/object timers (events) + Repeat every X s |
| Multiplayer | Plugin WebRTC (manual) | **Multiplayer object (sync automático + lobbies)** |
| Save/persistência | Persist · No save · Local storage | **Save state + behavior de configuração por objeto** |
| Parallax | Propriedade de LAYER | Layers + extensões parallax de Tiled Sprite |
| Line-of-sight | LOS behavior | **Não core** (não listado como revisado) |
| Drag&drop / Pin / Wrap / Fade / Flash / Sine / Move-to / Scroll-to / Anchor / Solid | **TODOS behaviors oficiais** | Draggable/Sticker/Screen wrap/…; **Solid NÃO EXISTE** (marcadores por sistema: Platform, Obstacle etc.) |
| Iluminação 2D | Shadow caster + Shadow light **[ADIADO]** | Light + Light Obstacle **[ADIADO]** |
| Componente próprio | Addon SDK (JS/TS) + Custom Actions + Families | **Events-based behaviors (sem código!)** + prefabs |

---

## 4. MATADORES DE CÓDIGO — os 15 que mais eliminam programação só com UI

Ordenados pelo produto (código eliminado × frequência de uso), com a engine de referência:

1. **Platform / Platformer character** (C3/GD) — o character controller de plataforma inteiro: gravidade, pulo
   com sustain, double jump, rampas, plataformas móveis, ladder e ledge-grab (GD) — ~10 campos de UI substituem
   o sistema mais reescrito (e mais errado) do gamedev.
2. **Physics** (C3/GD) — corpo rígido completo por painel: shape, density/friction/restitution, damping, CCD,
   filtros por camada, e juntas criadas por action (11 tipos no GD). Elimina a ponte física INTEIRA.
3. **Pathfinding + NavMesh** (C3/GD) — A* com custos e rebuild incremental; navmesh com crowd avoidance (GD).
   "Clique → o inimigo te acha" sem uma linha.
4. **Tween** (C3/GD) — o motor de animação procedural (easing, loop, ping-pong, tags, destroy-on-complete,
   cor em HSL, escala exponencial, variáveis e parâmetros de shader como alvos).
5. **Multiplayer object** (GD) — behavior no objeto = replicado (transform, animação, variáveis, física) com
   ownership e lobbies. Zero código de rede. O teto da categoria.
6. **Turret** (C3) — aquisição de alvo, priorização, rotação limitada, cadência e MIRA PREDITIVA num painel;
   o usuário só spawna a bala em `On shoot`.
7. **Line of sight** (C3) — percepção de IA (alcance+cone+oclusão) e raycast com normal/reflexão como conditions
   e expressions.
8. **Timer** (C3) + timers de objeto (GD) — cooldowns/spawns/delays POR INSTÂNCIA sem contabilidade de dt;
   `normalized-progress` alimenta a barra de recarga de graça.
9. **Solid / Jump-thru** (C3) — o marcador que TODOS os sistemas de movimento respeitam, com filtro por tags.
   Um checkbox transforma "desenhei uma parede" em "É uma parede".
10. **Fire bullets + Health** (GD) — o par tiro/dano completo (cooldown, ammo, reload, overheat, spread ×
    vida, escudo, armadura, regen, i-frames, dodge) inteiramente por propriedades.
11. **Scroll To + Shake** (C3) / Smooth Camera (GD) — câmera de gameplay: follow (com média multi-alvo p/ co-op)
    e screen-shake com decay, um checkbox e uma action.
12. **Persist / No save** (C3) e **Save state** (GD) — savegame e persistência de mundo como marcador por
    objeto; GD salva o jogo INTEIRO em slots com 1 action.
13. **Sine** (C3) — oscilador de qualquer propriedade com randomização embutida: o "juice de idle" de um jogo
    inteiro sem eventos, com preview no editor.
14. **Fade + Flash** (C3/GD) — ciclo de vida visual (aparecer/esperar/sumir/autodestruir) e i-frames piscantes:
    os dois patterns de feedback mais escritos do 2D.
15. **Pin / Sticker + Hierarchies** (C3/GD) — attach com herança POR CANAL (posição/ângulo/…, até image point
    animado) — o scene-graph "de gesto", sem reparenting em código.

**Menções honrosas:** Anchor (HUD responsivo), Drag & Drop, Destroy outside (higiene), Move To (waypoints +
executor de paths), Follow (replay/ghost com histórico serializável), YSort (GD), Animators de controller (GD),
Input mappers como behaviors (GD), Tile movement isométrico (C3), Custom Movement com `Push out solid` (C3).

---

## 5. Os 7 padrões de design que fazem o catálogo funcionar (síntese para o PH2D)

1. **Behavior = propriedades + vocabulário.** Cada componente publica conditions/actions/expressions no sistema
   de scripting/eventos. O painel configura; o vocabulário compõe. (Um componente sem vocabulário é um beco.)
2. **`Default controls` + `Simulate control`.** Todo controller anda sozinho no primeiro clique (demo/teste) E
   vira motor puro dirigível por IA/rede/replay desligando um bool. Duas audiências, um componente.
3. **Marcadores são componentes** (Solid, Jump-thru, Persist, No save, Obstacle, Light obstacle): zero
   propriedades, mas mudam como N sistemas tratam o objeto. Baratíssimos de implementar, enormes em alavancagem.
4. **Pares consumidor/produtor** (Platform↔Solid; Pathfinding↔Obstacle; Turret↔"Add target"; Shadow
   caster↔Shadow light): o behavior no agente CONSOME o marcador no cenário. Colisão de responsabilidade nunca
   acontece porque cada metade mora num objeto diferente.
5. **Tudo que é propriedade tem `Set` em runtime, `Enabled` togglável e (no C3) `Preview` no editor** — o
   componente é vivo na autoria e mutável no jogo, sem exceção.
6. **Randomização embutida onde a cópia denuncia** (Sine period/magnitude random; particles randomizers;
   Fire bullets variance): o catálogo já resolve o bug estético "100 instâncias em fase".
7. **O funil comunidade→oficial do GDevelop** (behaviors feitos SEM código com a própria ferramenta, revisados e
   promovidos) é o que produz a cauda longa de 200+ — o catálogo cresce sem custo do time core. O C3, preso ao
   SDK JS, tem catálogo oficial menor porém mais profundo por item.

---

## 6. Fontes

**Construct 3** (manual bloqueia fetch automatizado; conteúdo de propriedades extraído dos dados do editor oficial):
- Dados do editor r495.2 (fonte primária das §1.1–1.4): https://editor.construct.net/r495-2/behaviors/behaviorList.json · https://editor.construct.net/r495-2/loader/lang/precompiled-en-US.json · https://editor.construct.net/r495-2/plugins/pluginList.json
- Índice de behaviors: https://www.construct.net/en/make-games/manuals/construct-3/behavior-reference — páginas individuais: `/platform`, `/8-direction`, `/bullet`, `/car`, `/move`, `/pathfinding`, `/physics`, `/tween`, `/turret`, `/custom-movement` etc.
- Events: https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/events (+ `/sub-events`, `/conditions`, `/how-events-work`, `/custom-actions`)
- Timelines: https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/timelines
- Containers / Families: https://www.construct.net/en/make-games/manuals/construct-3/project-primitives/objects/containers · `/families`
- Scene graph (hierarchies): https://www.construct.net/en/blogs/construct-official-blog-1/lets-talk-scene-graph-1569 · Meshes: https://www.construct.net/en/blogs/construct-official-blog-1/heard-meshes-1567
- Addon SDK (behavior custom): https://www.construct.net/en/make-games/manuals/addon-sdk

**GDevelop:**
- Índice: https://wiki.gdevelop.io/gdevelop5/behaviors/ · lista completa: https://wiki.gdevelop.io/gdevelop5/behaviors/all-behaviors/
- Platformer: https://wiki.gdevelop.io/gdevelop5/behaviors/platformer/ · Top-down: https://wiki.gdevelop.io/gdevelop5/behaviors/topdown/ · Physics: https://wiki.gdevelop.io/gdevelop5/behaviors/physics2/ · Pathfinding: https://wiki.gdevelop.io/gdevelop5/behaviors/pathfinding/ · NavMesh: https://wiki.gdevelop.io/gdevelop5/behaviors/nav-mesh-pathfinding/ · Tween: https://wiki.gdevelop.io/gdevelop5/behaviors/tween/ · Anchor: https://wiki.gdevelop.io/gdevelop5/behaviors/anchor/ · Draggable: https://wiki.gdevelop.io/gdevelop5/behaviors/draggable/
- Events: https://wiki.gdevelop.io/gdevelop5/events/ · Timers: https://wiki.gdevelop.io/gdevelop5/all-features/timers-and-time/
- Multiplayer: https://wiki.gdevelop.io/gdevelop5/all-features/multiplayer/ · Save state: https://wiki.gdevelop.io/gdevelop5/all-features/save-state/
- Fire bullets: https://wiki.gdevelop.io/gdevelop5/extensions/fire-bullet/ · Health: https://wiki.gdevelop.io/gdevelop5/extensions/health/ · Sticker: https://wiki.gdevelop.io/gdevelop5/extensions/sticker/
- Objetos: https://wiki.gdevelop.io/gdevelop5/objects/ · Behaviors custom sem código: https://wiki.gdevelop.io/gdevelop5/behaviors/events-based-behaviors/
