# Dossiê — Unity 6 (2D): catálogo de componentes que o usuário ACRESCENTA a um objeto de cena

> Pesquisa em docs oficiais (docs.unity3d.com, docs-multiplayer.unity3d.com), 2026-08-20.
> Modelo de composição da Unity: um **GameObject** é um contêiner vazio; TODO comportamento vem de
> **Components** adicionados via botão "Add Component" no Inspector. É exatamente o modelo que o dono
> do PH2D escolheu (componentes estilo Unity, não herança de nodes do Godot). A força da Unity está em:
> (a) componentes atômicos que compõem entre si; (b) pacotes first-party que entregam sistemas inteiros
> como componentes configuráveis por Inspector; (c) TODO componente expõe suas propriedades numa UI
> uniforme (o Inspector) com curvas, gradientes, dropdowns e drag-de-referência.
>
> Convenção de relevância: **[ALTA]** / **[MÉDIA]** / **[BAIXA]** para uma engine 2D como o PH2D.

---

## 1. Visual / Render

Fonte: https://docs.unity3d.com/6000.1/Documentation/Manual/sprite/sprite-landing.html · https://docs.unity3d.com/6000.1/Documentation/Manual/sprite/9-slice/9-slicing.html · https://docs.unity3d.com/6000.1/Documentation/Manual/class-TrailRenderer.html · https://docs.unity3d.com/6000.1/Documentation/Manual/class-LineRenderer.html · https://docs.unity3d.com/6000.1/Documentation/Manual/sprite/sorting-group/sorting-group-landing.html

### Sprite Renderer **[ALTA]**
- **O que faz:** renderiza um sprite 2D com cor, flip, material e ordenação por camadas.
- **UI:** Sprite (referência), Color (tint), Flip X/Y, **Draw Mode** (Simple / **Sliced** / **Tiled** — os dois últimos são o *nine-patch*: cantos fixos, bordas repetidas, centro esticado ou ladrilhado, com `Size` editável por retângulo), Mask Interaction (visível dentro/fora de máscara), Sprite Sort Point (center/pivot), Sorting Layer + Order in Layer, Material.
- **Elimina:** todo o código de quad + UV + atlas + nine-patch manual; redimensionar painéis/plataformas sem distorcer arte.
- **Compõe com:** Sorting Group, Sprite Mask, Sprite Skin (deform), Animator (anima qualquer propriedade).
- Obs.: o nine-patch NÃO é um componente separado — é um **modo** do Sprite Renderer. Decisão de design que economiza um componente inteiro.

### Sprite Mask **[ALTA]**
- **O que faz:** máscara de estêncil para sprites — esconde ou revela grupos de sprites pela forma de outro sprite.
- **UI:** sprite da máscara, alpha cutoff, faixa de sorting (Custom Range: back/front), interação por camada.
- **Elimina:** shaders de máscara escritos à mão; efeitos de "revelar" (minimapa circular, spotlight, água cortando o personagem).
- **Compõe com:** Sprite Renderer (Mask Interaction), Sorting Group.

### Sorting Group **[ALTA]**
- **O que faz:** agrupa todos os Sprite Renderers filhos e os ordena **como uma unidade** contra o resto da cena. Grupos aninhados são ordenados dentro do pai.
- **UI:** Sorting Layer, Order in Layer, Sort at Root.
- **Elimina:** o pesadelo clássico de personagem multi-sprite (corpo/braço/arma) intercalando com o de OUTRO personagem idêntico — sem isso, cada dev escreve um "z-index manager". Caso de uso oficial: "rendering order of complex 2D multi-Sprite characters".
- **Compõe com:** Sprite Renderer, 2D Animation (rigs multi-sprite), Particle System.

### Line Renderer **[MÉDIA]**
- **O que faz:** desenha uma polilinha (reta ou curva por pontos) no mundo, com largura e cor variáveis ao longo do comprimento.
- **UI:** lista de posições (editável na Scene com ferramentas próprias), **width curve** (curva editável), **color gradient**, corner/end cap vertices (arredondamento), alignment (View/TransformZ), texture mode (stretch/tile), simplify (redução de pontos), use world space.
- **Elimina:** mesh procedural de fita/laser/mira de trajetória/corda visual.
- **Compõe com:** Physics2D raycast (mira), Splines.

### Trail Renderer **[MÉDIA]**
- **O que faz:** rastro atrás de um objeto em movimento; pontos morrem após `Time` segundos.
- **UI:** Time (vida do ponto), Min Vertex Distance, width curve, color gradient (fade no fim), corner/end cap vertices, alignment, **Emitting** (liga/desliga sem apagar o rastro), **AutoDestruct**, texture mode, generate lighting data.
- **Elimina:** todo o buffer circular de posições + mesh strip + fade que qualquer jogo de espada/dash/projétil reescreve.
- **Compõe com:** Rigidbody2D (movimento), Particle System (módulo Trails usa o mesmo motor por partícula).

### Mesh 2D genérico — **NÃO EXISTE como categoria própria.** MeshFilter+MeshRenderer são a rota 3D; em 2D, geometria custom vem via Sprite Shape, Sprite (mesh do sprite editável no Sprite Editor) ou código. Declarado como ausência.

---

## 2. Tilemap & Grid (pacote 2D Tilemap + Extras)

Fonte: https://docs.unity3d.com/6000.0/Documentation/Manual/tilemaps/work-with-tilemaps/tilemap-reference.html · https://docs.unity3d.com/6000.0/Documentation/Manual/tilemaps/grid-reference.html · https://docs.unity3d.com/6000.3/Documentation/Manual/tilemaps/work-with-tilemaps/tilemap-renderer-reference.html · https://docs.unity3d.com/Packages/com.unity.2d.tilemap.extras@4.1/manual/RuleTile.html

### Grid **[ALTA]**
- **O que faz:** define o layout de células (retangular, **hexagonal**, **isométrico**, isométrico z-as-y) que os Tilemaps filhos usam; também serve de guia de snapping.
- **UI:** cell size, cell gap, cell layout, cell swizzle.
- **Elimina:** matemática de grid→mundo e mundo→grid (inclusive hex e iso, que quase ninguém acerta de primeira).

### Tilemap **[ALTA]**
- **O que faz:** armazena e gerencia os Tile assets pintados nas células; repassa a informação ao renderer e ao collider. Pintura via janela Tile Palette (pincel, balde, retângulo, picking).
- **UI:** animation frame rate, color (tint), tile anchor, orientation.
- **Elimina:** a estrutura de dados de nível inteira (chunks, serialização, lookup por célula) + o editor de nível em si.
- **Compõe com:** Tilemap Renderer, Tilemap Collider 2D, Rule Tiles, NavMesh (3D), Composite Collider 2D.

### Tilemap Renderer **[ALTA]**
- **O que faz:** renderiza o tilemap em chunks batched.
- **UI:** sort order, **mode (Chunk/Individual)** — Individual permite sprites intercalarem com tiles (personagem atrás de árvore do tilemap), detect chunk culling, sorting layer/order, mask interaction.
- **Elimina:** batching manual e culling de milhares de tiles.

### Tilemap Collider 2D **[ALTA]**
- **O que faz:** gera colisão automaticamente de cada tile pintado (cada tile define seu Collider Type: none/sprite/grid).
- **UI:** usado por composite (checkbox), max tile change count, extrusion factor.
- **Elimina:** colocar N BoxColliders à mão no nível; sincronizar colisão↔arte quando o nível muda.
- **Compõe com:** **Composite Collider 2D** (funde os retângulos por tile em um outline único — mata o bug de "ghost collision" nas emendas).

### Rule Tile / Animated Tile / Rule Override Tile (2D Tilemap Extras) **[ALTA]**
- **O que faz:** **auto-tiling**: o tile escolhe o próprio sprite pela vizinhança (matriz 3×3 de regras: "se tem chão à direita, use este sprite"); saída pode ser sprite fixo, **aleatório** ou **animado**; a regra pode fixar também collider e GameObject. Rule Override Tile reusa as regras de outro Rule Tile trocando só os sprites (retheming instantâneo).
- **UI:** editor visual da matriz de regras por tile, com setas/checks clicáveis.
- **Elimina:** TODO o código de auto-tiling (o cálculo de bitmask de 47 casos que todo dev de jogo de tile já escreveu) e a re-pintura manual quando o terreno muda.
- **Relevância:** ALTA — junto com o Tilemap, é dos maiores redutores de trabalho da Unity 2D.

---

## 3. Sprite Shape (terreno orgânico por spline)

Fonte: https://docs.unity3d.com/Packages/com.unity.2d.spriteshape@10.0/manual/index.html

### Sprite Shape Controller + Sprite Shape Renderer **[ALTA]**
- **O que faz:** terreno/plataforma/estrada 2D desenhado como **spline** (aberta ou fechada); um **Sprite Shape Profile** (asset) mapeia FAIXAS DE ÂNGULO → sprites de borda (chão, parede, teto, quinas), e um fill texture preenche o interior de formas fechadas. Os sprites deformam e trocam sozinhos conforme o ângulo da curva.
- **UI:** profile, edição de pontos da spline na Scene (modo contínuo/quebrado por ponto, altura por ponto, sprite variant por ponto), collider offset/detail, e no profile: faixas de ângulo, sprites por faixa, corner sprites, fill.
- **Elimina:** geração procedural de mesh de terreno + UV mapping + troca de sprite por inclinação + colisão acompanhando a curva. É o "desenhe o nível com uma caneta".
- **Compõe com:** Edge/Polygon Collider 2D (gerado da spline), Cinemachine Confiner (mesma spline pode limitar câmera).

---

## 4. Esqueleto / deformação 2D (pacote 2D Animation)

Fonte: https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/SpriteSkin.html · https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/2DIK.html · https://docs.unity3d.com/Packages/com.unity.2d.animation@15.0/manual/SpriteSwapIntro.html

### Sprite Skin **[ALTA]**
- **O que faz:** deforma o mesh do sprite pelos **bones** rigados no Skinning Editor (editor visual de bones + pesos dentro da Unity; importa PSD com camadas via PSD Importer). Deformação em CPU **ou GPU** (2D Animation 10+, URP).
- **UI:** Always Update (anima fora da tela), Auto Rebind, Root Bone, lista de bones, botões Create Bones / Reset Bind Pose.
- **Elimina:** todo o pipeline de skinning 2D (matrizes de bone, pesos por vértice, update do mesh) — o usuário riga na UI e anima os Transforms dos bones com o Animator/Timeline comum.
- **Compõe com:** Animator (bones são Transforms animáveis), IK, Sprite Library.

### IK Manager 2D + solvers (Limb, Chain CCD, Chain FABRIK) **[ALTA]**
- **O que faz:** cinemática inversa — arrasta-se um **target** e a cadeia de bones o segue. Limb = 2 bones (braço/perna); CCD e FABRIK = cadeias de N bones.
- **UI:** por solver: Effector, Target, Chain Length, Iterations, Tolerance, Constrain Rotation, **Weight** (0–1, mistura FK↔IK), Always Update; manager com lista ordenada de solvers e Restore Default Pose.
- **Elimina:** os algoritmos de IK inteiros + o blend FK/IK; pés que colam no chão e mãos que seguram objetos viram "arraste o alvo".

### Sprite Library (asset + component) + Sprite Resolver **[ALTA]**
- **O que faz:** troca de sprite estruturada por **Categoria + Label** ("Mouth"/"happy"). O Sprite Library (component) aponta um Sprite Library Asset (com herança de variantes); o Sprite Resolver, no GameObject de cada parte, escolhe o label — e o label é **animável por keyframe**. O Resolver sobe a hierarquia para achar a Library (uma library raiz serve o rig todo).
- **UI:** dropdown visual com thumbnails de categoria/label.
- **Elimina:** sistemas de "skin"/troca de equipamento/lip-sync feitos com dicionários de sprite à mão. Um rig, N personagens (troca de library asset).

---

## 5. Partículas

Fonte: https://docs.unity3d.com/6000.1/Documentation/Manual/class-ParticleSystem.html

### Particle System **[ALTA]**
- **O que faz:** sistema completo de partículas, 100% configurável por Inspector, organizado em **módulos** ativáveis: Emission (rate, bursts), Shape (emissão por forma — inclui **sprite** e borda de sprite), Velocity/Limit Velocity/Force over Lifetime, **Color over Lifetime** (gradiente), **Size/Rotation over Lifetime** (curvas), Noise, **Collision** (com o mundo 2D/3D), **Sub Emitters** (partícula que spawna partículas ao nascer/colidir/morrer), **Texture Sheet Animation** (flipbook), **Trails**, Lights, Renderer (billboard/mesh/sorting). Toda propriedade numérica aceita constante, curva, aleatório-entre-duas-curvas ou gradiente.
- **UI:** dezenas de módulos colapsáveis + preview com Simulate/Resimulate/Show Bounds na Scene.
- **Elimina:** um motor de VFX inteiro; explosões, fumaça, chuva, faíscas, folhas, sangue — zero código. É citado com frequência como o padrão-ouro de "poder sem programar".
- **Compõe com:** Timeline (Control Track dá scrub/sequenciamento), Sorting Group, Physics 2D (colisão).
- Nota: existe também o **VFX Graph** (GPU, milhões de partículas, node-based) — first-party, foco 3D/URP-HDRP, relevância MÉDIA para 2D.

---

## 6. Câmera de gameplay (pacote Cinemachine 3)

Fonte: https://docs.unity3d.com/Packages/com.unity.cinemachine@3.1/manual/index.html · .../CinemachineCamera.html · .../Cinemachine2D.html · .../CinemachinePositionComposer.html · .../CinemachineConfiner2D.html · .../CinemachineImpulse.html

### CinemachineCamera (câmera virtual) **[ALTA]**
- **O que faz:** câmeras "virtuais" procedurais; a Unity Camera real é possuída pelo **Cinemachine Brain**, que ativa a virtual de maior prioridade e faz **blend** suave entre elas ao trocar (transição de sala = ativar outra vcam, zero código).
- **UI:** Tracking Target / Look At Target (arrastar), Lens (ortho size, near/far, dutch), Priority, Blends (curvas de transição, custom por par de câmeras), e **behaviors** plugáveis de posição/rotação.
- **Behaviors de posição relevantes p/ 2D:** **Position Composer** (o follow 2D canônico: mantém o alvo numa posição de tela com **dead zone** — zona morta onde a câmera não se move —, **soft zone**, damping por eixo, **lookahead** que antecipa o movimento); Follow (offset fixo); Orbital; **Spline Dolly** (câmera sobre trilho/spline).
- **Elimina:** o "camera follow script" que TODO jogo 2D escreve e reescreve (com damping, dead zone, lookahead, clamps), e toda a lógica de troca/transição de câmeras.

### Cinemachine Confiner 2D (extensão) **[ALTA]**
- **O que faz:** confina a **janela visível** (não só o centro) dentro de um polígono/Collider2D — limites de fase perfeitos, com corte de cantos calculado para a janela caber.
- **UI:** Bounding Shape 2D (qualquer Collider2D), Damping, Slowing Distance, Max Window Size.
- **Elimina:** clamps manuais de câmera por retângulos + os bugs quando a sala não é retangular.

### Cinemachine Impulse (Impulse Source / Collision Impulse Source / Impulse Listener) **[ALTA]**
- **O que faz:** **screen shake físico e propagado**: fontes emitem um sinal (forma, amplitude, frequência, dissipação por distância); câmeras com Listener tremem quando o sinal as alcança. Collision Impulse Source dispara sozinho em colisão.
- **UI:** perfis de sinal prontos (recoil, bump, explosion, rumble) + curvas custom.
- **Elimina:** o codigo de camera-shake (coroutine de Perlin) e o acoplamento "explosão → achar câmera → chamar Shake()".

### Cinemachine Pixel Perfect (extensão) **[MÉDIA]** — resolve o conflito Cinemachine × Pixel Perfect Camera (ambos mexem no ortho size). Fonte: https://docs.unity3d.com/Packages/com.unity.2d.pixel-perfect@5.0/manual/index.html

### Pixel Perfect Camera (pacote 2D Pixel Perfect) **[ALTA]**
- **O que faz:** mantém pixel art nítida e estável em qualquer resolução: calcula o zoom inteiro correto a partir de Assets Pixels Per Unit + Reference Resolution.
- **UI:** Assets PPU, Reference Resolution, **Crop Frame** (barras/pillarbox), **Grid Snapping / Pixel Snapping** (snap dos sprites à grade no render, sem tocar nos transforms), Upscale Render Texture (render na resolução de referência + upscale — pixels sem rotação nem aliasing).
- **Elimina:** toda a matemática de viewport/zoom inteiro/snapping subpixel que jogos de pixel art exigem.

---

## 7. Física & colisão 2D (Box2D integrado)

Fonte: https://docs.unity3d.com/6000.4/Documentation/Manual/2d-physics/rigidbody/body-types/rigidbody-2d-body-types-landing.html · https://docs.unity3d.com/6000.2/Documentation/Manual/2d-physics/collider/collider-2d-landing.html · https://docs.unity3d.com/6000.1/Documentation/Manual/2d-physics/effectors/effectors-2d-landing.html · https://docs.unity3d.com/6000.1/Documentation/Manual/2d-physics/joints/2d-joints-landing.html · https://docs.unity3d.com/6000.3/Documentation/Manual/2d-physics/collider/custom-collider/custom-collider-2d-reference.html

### Rigidbody2D **[ALTA]**
- **O que faz:** entrega o objeto à simulação. **Body Type** com 3 modos: **Dynamic** (forças, gravidade, colide com tudo), **Kinematic** (move-se por velocidade, ignora forças — para plataformas móveis e controllers scriptados), **Static** (imóvel, custo mínimo).
- **UI:** body type, material, mass, linear/angular damping, **gravity scale** (por objeto!), collision detection (discrete/continuous), interpolation, **constraints** (freeze X/Y/rotação), sleeping mode, use auto mass.
- **Elimina:** integração, resposta a colisão, sleeping, CCD.

### Colliders 2D — 8 formas **[ALTA]**
- **Box Collider 2D** (retângulo, com edge radius arredondando cantos), **Circle Collider 2D**, **Capsule Collider 2D** (sem quinas — o corpo de personagem que não engancha), **Polygon Collider 2D** (forma livre; auto-gerada do contorno do sprite), **Edge Collider 2D** (linha aberta — chão/rampa sem volume), **Composite Collider 2D** (FUNDE boxes/polygons filhos num outline só — remove emendas), **Tilemap Collider 2D** (ver §2), **Custom Collider 2D** (N formas primitivas via API `PhysicsShapeGroup2D` — colliders procedurais editáveis em runtime sem custo de recriação).
- **UI comum:** **Is Trigger** (vira sensor/área: eventos OnTriggerEnter2D/Exit sem resposta física — a "Area" do 2D), offset, material físico por collider, **Used By Effector**, edição visual de vértices na Scene.
- **Elimina:** broadphase, narrowphase, triggers/sensores.

### Physics Material 2D (asset) **[ALTA]** — friction + bounciness por collider. Elimina resposta de quique/atrito manual. Fonte: https://docs.unity3d.com/6000.1/Documentation/Manual/2d-physics/physics-material-2d.html

### Effectors 2D — 5 componentes **[ALTA — categoria inteira é um diferencial]**
Anexam-se a um collider (geralmente trigger) e aplicam forças/comportamento a quem entra em contato:
1. **Platform Effector 2D** — plataformas **one-way** (atravessa por baixo, pousa por cima), com arco de superfície configurável, one-way grouping, atrito/quique laterais opcionais. *Elimina o hack de ligar/desligar collider do plataformer.*
2. **Area Effector 2D** — força com ângulo/magnitude (± variância aleatória) em quem está dentro: vento, correnteza, esteiras de ar.
3. **Point Effector 2D** — atração/repulsão radial de um ponto (ímã, gravidade local, explosão), com drag e modos de escala por distância.
4. **Buoyancy Effector 2D** — **água pronta**: nível de superfície, densidade, empuxo, fluxo linear/angular, arrasto. Objetos boiam ou afundam conforme a própria densidade.
5. **Surface Effector 2D** — **esteira rolante**: velocidade tangencial na superfície, escala de força, use contact force.
- **UI comum:** máscara de camadas, force magnitude/variation, use collider mask.
- **Elimina:** cada um substitui um script clássico de gameplay (one-way platform, zona de vento, ímã, água, esteira) por um checkbox + 3 números.

### Joints 2D — 9 componentes **[ALTA]**
Conectam dois Rigidbody2D (ou um ao mundo). Todos com **break force/torque** (quebram sob estresse) e âncoras editáveis na Scene:
1. **Distance Joint 2D** — distância fixa (ou máxima) entre dois corpos; pêndulo, corrente.
2. **Fixed Joint 2D** — solda (via mola rígida); objetos grudados que podem quebrar.
3. **Friction Joint 2D** — resiste a movimento/rotação relativos (amortecedor genérico).
4. **Hinge Joint 2D** — dobradiça com **motor** (velocidade/torque) e **limites de ângulo**: portas, alavancas, rodas motorizadas, serras.
5. **Relative Joint 2D** — mantém offset relativo com força configurável (câmera-física, escudo orbitando).
6. **Slider Joint 2D** — trilho linear com motor e limites: pistões, elevadores.
7. **Spring Joint 2D** — mola (distância, frequência, damping).
8. **Target Joint 2D** — puxa para um PONTO no mundo (não outro corpo) com mola: **drag-and-drop físico com o mouse** é o caso de uso oficial.
9. **Wheel Joint 2D** — suspensão + motor: **veículos 2D prontos** (com Hinge para direção).
- **Elimina:** solvers de restrição inteiros; um carro 2D é 1 corpo + 2 Wheel Joints, zero física escrita.

### Raycast/queries — **não é componente**: API `Physics2D.Raycast/BoxCast/CircleCast/OverlapArea…`. O que existe como componente é o **Physics 2D Raycaster** (na câmera), que leva os eventos de ponteiro do Event System a objetos 2D com collider (clicar/arrastar objetos do mundo sem escrever picking). Fonte: https://docs.unity3d.com/Packages/com.unity.ugui@2.0/manual/EventSystem.html

---

## 8. Character controllers PRONTOS — **A UNITY NÃO TEM (2D)**

- O componente **CharacterController** built-in é **exclusivamente 3D** (cápsula, move-and-slide). **Não existe** um platformer/top-down controller 2D pronto no core ou em pacote first-party de produção — todo projeto 2D escreve o seu sobre Rigidbody2D + Capsule Collider, ou baixa da Asset Store/GitHub (Brackeys etc.). É a lacuna MAIS reclamada da Unity 2D (fonte da ausência: https://docs.unity3d.com/6000.1/Documentation/Manual/class-CharacterController.html + discussões oficiais).
- **Oportunidade direta para o PH2D:** um `PlatformerBody2D` e um `TopDownBody2D` de primeira classe (coyote time, jump buffer, one-way, rampas, 8-direções) seriam um diferencial imediato sobre a Unity — o Godot (CharacterBody2D) e o GameMaker provam a demanda.

---

## 9. Caminhos / Splines (pacote Splines)

Fonte: https://docs.unity3d.com/Packages/com.unity.splines@2.7/manual/index.html · https://docs.unity3d.com/Packages/com.unity.splines@2.9/manual/animate-spline.html

### Spline Container + Spline Animate **[MÉDIA-ALTA]**
- **O que faz:** Spline Container guarda a(s) spline(s) (knots/tangentes, edição visual na Scene). **Spline Animate** move+rotaciona um GameObject ao longo da spline: método (tempo/velocidade), easing, loop (once/loop/ping-pong), alinhamento, start offset, play on awake.
- **Elimina:** interpolação de Bézier, arc-length parameterization, "patrol path" scripts.
- O pacote inclui ainda (per docs, casos de uso oficiais): **Spline Instantiate** (instancia prefabs ao longo da spline — cercas, moedas, florestas) e **Spline Extrude** (mesh 3D por extrusão). Cinemachine usa a mesma spline no **Spline Dolly** (trilho de câmera).
- **Compõe com:** Cinemachine, LineRenderer.

---

## 10. Animação

Fonte: https://docs.unity3d.com/6000.1/Documentation/Manual/class-Animator.html · https://docs.unity3d.com/6000.1/Documentation/Manual/AnimatorControllers.html

### Animator + Animator Controller (asset) **[ALTA]**
- **O que faz:** o Animator toca um **Animator Controller**: máquina de estados visual com **states** (clips), **transitions** (condições sobre **parameters**: float/int/bool/**trigger**; exit time; duração de blend), **Blend Trees** (1D/2D — mistura idle/walk/run por velocidade; os 4 clips direcionais do top-down por vetor), **layers** (avatar mask + blend: tronco atira enquanto pernas correm), **sub-state machines**. Qualquer propriedade de qualquer componente é animável (Animation Window grava keyframes; sprites incluídos — flipbook é arrastar N sprites para a janela).
- **UI:** Controller, Apply Root Motion, Update Mode (Normal/Physics/Unscaled), Culling Mode; e o editor gráfico do controller inteiro.
- **Elimina:** máquina de estados de animação, blending, sincronização sprite-flipbook, eventos de frame (**Animation Events** chamam métodos num frame exato — footstep, spawn de hitbox).
- **Compõe com:** Sprite Skin/bones, Sprite Resolver (label animável), Timeline, física (Animate Physics).
- Nota 2D: para flipbooks simples o Animator é considerado pesado — mas é O caminho oficial; não há "AnimatedSprite" leve como no Godot (**ausência** digna de nota).

### Tween — **NÃO EXISTE built-in.** Nenhum componente/sistema de tween first-party (DOTween/LeanTween são third-party onipresentes — outra lacuna que engines menores preenchem com componente próprio). Declarado como ausência.

---

## 11. Timeline / Sequencer (pacote Timeline)

Fonte: https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/index.html · https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/trk-add.html

### Playable Director + Timeline asset **[ALTA]**
- **O que faz:** sequenciador multi-track para cutscenes/sequências de gameplay. O **Playable Director** (componente) toca um Timeline asset e faz o **binding** de cada track a objetos da cena; wrap mode, play on awake, initial time.
- **Tracks oficiais:** **Animation Track** (anima objetos; grava direto na timeline), **Activation Track** (liga/desliga GameObjects por clipe), **Audio Track**, **Control Track** (controla o TEMPO de Particle Systems, sub-timelines, prefabs instanciados — scrub de partículas!), **Signal Track** (ver abaixo), **Playable Track** (clipes custom por script).
- **Signals:** **Signal Emitter** (marker na timeline) + **Signal Receiver** (componente: mapeia signal asset → UnityEvents) — a timeline dispara gameplay sem acoplamento.
- **Elimina:** todo scripting de cutscene (sequência de coroutines com WaitForSeconds), sincronização de câmera+animação+som, e dá **scrub/preview no editor**.
- **Compõe com:** Cinemachine (Cinemachine Shot track — troca de vcams com blend na timeline), Animator, Audio.

---

## 12. Áudio

Fonte: https://docs.unity3d.com/6000.1/Documentation/Manual/class-AudioSource.html · https://docs.unity3d.com/6000.4/Documentation/Manual/class-AudioListener.html · https://docs.unity3d.com/6000.3/Documentation/Manual/class-AudioReverbZone.html · https://docs.unity3d.com/6000.4/Documentation/Manual/class-AudioEffect.html

### Audio Source **[ALTA]**
- **O que faz:** toca um AudioClip no espaço.
- **UI:** clip, **Output** (Audio Mixer Group), mute, Play On Awake, loop, priority, volume, pitch, stereo pan, **Spatial Blend (0=2D … 1=3D)** — o posicional 2D usa curvas de rolloff/distância customizáveis —, doppler, min/max distance.
- **Elimina:** mixagem de vozes, prioridade, atenuação por distância.

### Audio Listener **[ALTA]** — o "microfone" (1 por cena, tipicamente na câmera). Sem propriedades.

### Audio Reverb Zone **[MÉDIA]** — área esférica que aplica reverb gradualmente a tudo que o listener ouve dentro dela (caverna, hall). Presets + min/max distance.

### Audio Filters (componentes) **[MÉDIA]** — **Audio Low Pass / High Pass / Echo / Distortion / Reverb / Chorus Filter**: empilham-se no MESMO GameObject do Source ou Listener; a ordem dos componentes é a ordem da cadeia de efeitos. Elimina DSP de gameplay simples (abafar som embaixo d'água = LowPass no listener).

### Audio Mixer (asset, não componente) **[ALTA]** — grupos hierárquicos, efeitos, **snapshots** (estados de mixagem com transição — "combate"/"exploração"), ducking. Sources roteiam por dropdown.

---

## 13. Input (pacote Input System)

Fonte: https://docs.unity3d.com/Packages/com.unity.inputsystem@1.14/manual/PlayerInput.html

### PlayerInput **[ALTA]**
- **O que faz:** liga um **Input Actions asset** (editor visual: Action Maps → Actions → bindings por dispositivo, com **composites** tipo WASD-vector2, processors e **interactions** hold/tap/multi-tap) ao objeto do jogador. Troca de dispositivo automática (teclado↔gamepad), **control schemes**.
- **UI:** Actions asset, Default Scheme, Default Map, **Behavior**: Send Messages / Broadcast / **Invoke Unity Events** (cada action vira um evento configurável no Inspector) / C# events.
- **Elimina:** citado verbatim na doc: "meant primarily as an easy, out-of-the-box setup that eliminates much of the need for custom scripting" — polling de teclas, rebinding, pareamento de dispositivos.
- **PlayerInputManager** (componente irmão): **multiplayer local automático** — "join when button pressed", spawn de prefab por jogador, pareamento de dispositivo por jogador e **split-screen automático** (divide viewports sozinho).

---

## 14. UI in-game

Fonte: https://docs.unity3d.com/Packages/com.unity.ugui@2.0/manual/EventSystem.html · https://docs.unity3d.com/6000.3/Documentation/Manual/UIElements.html · https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-get-started-with-runtime-ui.html

### uGUI (GameObject-based) **[ALTA]**
- **Canvas** (Screen Space / **World Space** — UI presa a objetos do mundo, healthbars), **Canvas Scaler** (escala por resolução de referência), **Rect Transform** (âncoras/pivôs responsivos), **Event System** + input module + **Graphic Raycaster** / **Physics 2D Raycaster** (eventos de ponteiro para UI E para objetos do mundo com collider — interfaces `IPointerClickHandler`, `IDragHandler`, `IDropHandler`…).
- **Controles prontos:** Image (sprite, **filled** — cooldowns radiais), Text/TextMeshPro, **Button, Toggle, Slider, Scrollbar, Scroll View, Dropdown, Input Field** — todos com **UnityEvents no Inspector** (OnClick arrasta-objeto-escolhe-método, zero código) e sistema de navegação por gamepad automático.
- **Layout Groups** (Horizontal/Vertical/Grid + Content Size Fitter): auto-layout.
- **Elimina:** toda a infraestrutura de UI (hit-testing, foco, navegação, layout responsivo, drag&drop com `IDragHandler`).

### UI Toolkit (UIDocument) **[MÉDIA para jogos 2D; ALTA para tooling]**
- Componente **UIDocument** carrega UXML (markup) + USS (stylesheets tipo CSS); no Unity 6, **runtime data binding** liga propriedades do jogo a elementos sem boilerplate. Caminho recomendado para UI de editor e UI de jogo "web-like"; uGUI segue melhor para UI no mundo.

---

## 15. Navegação & AI

Fonte: https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavMeshAgent.html · https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/index.html

### AI Navigation (pacote): NavMeshSurface, NavMeshAgent, NavMeshObstacle, NavMeshLink, NavMeshModifier **[MÉDIA em 2D — ver ressalva]**
- **NavMeshSurface** — baked navmesh da geometria; **NavMeshAgent** — agente que navega sozinho (speed, angular speed, acceleration, stopping distance, auto braking, **obstacle avoidance** com priority entre agentes, area mask de custos); **NavMeshObstacle** — obstáculo móvel que **esculpe (carve)** o navmesh; **NavMeshLink** — atalhos/pulos entre superfícies; **NavMeshModifier** — custo/área por objeto.
- **Elimina:** A*, steering, avoidance multi-agente, replan dinâmico.
- ⚠️ **RESSALVA 2D:** o sistema é **3D-only** — não entende Tilemap/sprites nativamente; a comunidade usa o workaround NavMeshPlus (third-party). **A Unity NÃO tem pathfinding 2D de grade/tilemap first-party** — lacuna notória (Godot tem Navigation2D + TileMap; oportunidade para o PH2D).
- **Behavior tree:** não há behavior tree no core historicamente (o pacote "Unity Behavior" é recente e não foi verificado nesta pesquisa; o que existe consolidado é a **State Machine do Visual Scripting** para FSM de AI). Declarado.

---

## 16. Spawn / factory / pooling — **SEM COMPONENTE**

- Instanciação = `Instantiate(prefab)` em código; pooling = API `ObjectPool<T>` (sem componente, sem UI). **Prefabs** (+ nested prefabs + variants) são o sistema de factory da Unity — poderosíssimo, mas o ATO de spawnar é sempre código. Não há "Spawner" ou "MultiMesh" de Inspector. Declarado como ausência (Particle System e PlayerInputManager são os únicos "spawners" configuráveis).

## 17. Timers — **SEM COMPONENTE.** `Invoke`/coroutines/`Time.time` em código. Declarado.

## 18. Rede / multiplayer (pacote Netcode for GameObjects)

Fonte: https://docs-multiplayer.unity3d.com/netcode/1.12.0/basics/networkobject/ · https://docs.unity3d.com/Packages/com.unity.netcode.gameobjects@2.9/manual/components/helper/networkrigidbody.html

- **NetworkManager** (singleton de sessão), **NetworkObject** (identidade replicada; spawn em rede), **NetworkTransform** (sincroniza pos/rot/scale com interpolação e thresholds configuráveis; variante client-authoritative), **NetworkAnimator** (replica estado/triggers do Animator; server- ou owner-authoritative), **NetworkRigidbody2D** (põe o corpo em kinematic em todos os peers não-autoritativos — física 2D em rede por checkbox).
- **Elimina:** replicação de transform/animação/física, ownership, spawn em rede. Relevância: MÉDIA (só para jogos multiplayer) mas é o modelo first-party a copiar: *sincronizar = adicionar um componente*.

## 19. Persistência / save — **SEM COMPONENTE.** `PlayerPrefs` (chave-valor) + serialização própria em código. Nenhum "save game" de Inspector. Declarado.

## 20. Parallax / scrolling — **SEM COMPONENTE.** Todo projeto 2D escreve o próprio parallax script (ou usa material scroll). Lacuna clássica; Godot tem ParallaxBackground/Parallax2D. Declarado.

## 21. Utilitários de transform: Constraints **[MÉDIA-ALTA]**

Fonte: https://docs.unity3d.com/6000.5/Documentation/Manual/constraint-components.html

- **Position / Rotation / Scale / Parent / Aim / Look At Constraint** — prendem o transform ao de outro(s) objeto(s), com **peso** por fonte, freeze de eixos e offsets. Parent Constraint = "pin" a outro objeto **sem** reparentar (item na mão); Aim = torre que mira o player sem código. Multi-fontes fazem média ponderada.
- **Elimina:** os scripts de "follow/pin/aim" de 5 linhas que se multiplicam aos milhares — e rodam na ordem certa do pipeline de animação, coisa que o script de usuário sempre erra.
- **Demais utilitários da checklist** (line-of-sight, drag&drop de mundo, wrap, fade, flash, sine, move-to, anchor de tela, solid/jumpthru): **não existem como componentes na Unity** — viram código ou recursos citados acima (drag&drop físico = Target Joint 2D; jumpthru = Platform Effector 2D; anchor = RectTransform em Canvas; LOS = `Physics2D.Linecast` em código). Declarado por item.

## 22. Como o usuário cria um componente PRÓPRIO

Fonte: https://docs.unity3d.com/6000.1/Documentation/Manual/CreatingAndUsingScripts.html · https://docs.unity3d.com/Packages/com.unity.visualscripting@1.9/manual/vs-graph-machine-types.html

1. **MonoBehaviour (C#):** classe herda `MonoBehaviour` → o arquivo JÁ é um componente adicionável. **Todo campo público/`[SerializeField]` aparece automaticamente no Inspector** com o editor certo pro tipo (curvas, gradientes, referências arrastáveis, listas) — esta é a alma do ecossistema: escrever um componente com UI custa ZERO UI. Atributos (`[Range]`, `[Header]`, `[Tooltip]`, `[RequireComponent]`) refinam; custom editors/PropertyDrawers vão além. Callbacks: `Awake/Start/Update/FixedUpdate/OnTriggerEnter2D/…`.
2. **Visual Scripting (pacote):** **Script Machine** executa um Script Graph (fluxo de nós: eventos → ações, com acesso a TODA a API por reflexão); **State Machine** executa um State Graph (estados com transições — FSM visual). Variáveis em escopos (Object/Scene/App/**Saved** — persistência simples embutida). Graph asset compartilhado ou embed. **UnityEvents** no Inspector são o "cola-código-sem-código" onipresente.

## 23. Iluminação 2D — **ADIADO por decisão do dono (2026-08-20); listado sem priorizar**

Fonte: https://docs.unity3d.com/6000.4/Documentation/Manual/urp/Lights-2D-intro.html · https://docs.unity3d.com/6000.0/Documentation/Manual/urp/ShadowCaster2D.html

- (adiado) **Light 2D** (URP 2D Renderer): tipos Freeform/Sprite/Spot/Global, blend styles, falloff, normal maps.
- (adiado) **Shadow Caster 2D** + **Composite Shadow Caster 2D**: sombras coplanares 2D pela silhueta do sprite.

---

# ⭐ MATADORES DE CÓDIGO — os componentes Unity que mais eliminam programação só com UI

1. **Rule Tile (+ Tilemap)** — auto-tiling por matriz de regras visual; elimina o algoritmo de bitmask/47-casos e TODA a re-pintura manual de terreno.
2. **Cinemachine (CinemachineCamera + Position Composer + Confiner 2D + Impulse)** — follow com dead zone/lookahead, limites de fase, screen-shake e transições entre câmeras: quatro scripts obrigatórios de todo jogo 2D reduzidos a componentes.
3. **Particle System** — um motor de VFX inteiro dirigido por curvas/gradientes no Inspector; scrub no editor; zero código para 95% dos efeitos.
4. **Platform Effector 2D** — plataforma one-way (o hack mais reescrito do gênero platformer) num checkbox.
5. **Buoyancy Effector 2D** — água com empuxo/fluxo/arrasto funcionais por 6 campos numéricos.
6. **Wheel Joint 2D (+ Hinge/Slider/Spring/Target Joints)** — veículos, pontes de corda, elevadores e drag-físico-com-mouse sem uma linha de solver.
7. **PlayerInput (+ Input Actions asset + PlayerInputManager)** — mapeamento visual de ações multi-dispositivo, rebinding, e multiplayer local com split-screen automático.
8. **Animator Controller (Blend Trees + layers + Animation Events)** — FSM de animação visual; o blend 2D direcional do top-down é um gráfico, não um switch-case.
9. **Timeline (Playable Director + Signal Track + Control Track)** — cutscenes multi-track com scrub; substitui as cadeias de coroutines; Signals disparam gameplay sem acoplamento.
10. **Sprite Shape** — nível orgânico DESENHADO como spline, com sprites trocando por ângulo e colisão automática.
11. **2D IK (IK Manager + solvers)** — pés no chão e mãos em objetos arrastando um target; o algoritmo inteiro vem pronto.
12. **Sprite Library / Sprite Resolver** — skins, equipamento e lip-sync por dropdown Categoria/Label animável — o "dicionário de sprites" que todo RPG escrevia.
13. **Pixel Perfect Camera** — a matemática de pixel art (zoom inteiro, snapping, upscale) inteira num componente.
14. **UnityEvents + Inspector auto-gerado de MonoBehaviours** — o meta-matador: qualquer campo público vira UI de edição de graça, e qualquer evento liga-se a qualquer método por drag&drop. É o que faz TODOS os outros componentes (e os do usuário) configuráveis sem código.
15. **Constraints (Parent/Aim/Position…)** — os micro-scripts de follow/pin/mira eliminados, rodando na ordem certa do pipeline.

---

# Lacunas da Unity 2D (oportunidades diretas para o PH2D)

- **Character controller 2D pronto** (platformer/top-down): NÃO existe — a ausência mais sentida.
- **Pathfinding 2D nativo** (grade/tilemap/polígono): NavMesh é 3D-only.
- **Tween engine first-party**: inexistente (DOTween domina como third-party).
- **Parallax**: sem componente.
- **Spawner/pooling com UI**: sem componente.
- **Timer com UI**: sem componente.
- **Save/persistência estruturada**: só PlayerPrefs.
- **AnimatedSprite leve** (flipbook sem Animator): inexistente.
- **Behavior tree consolidada no core**: inexistente (só FSM do Visual Scripting).
