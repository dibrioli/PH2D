Pesquisa completa (22 chamadas). Segue o dump denso.

---

# Motion-por-nós fora do canvas puro — pesquisa para PH2D (2026-07-24)

## A) Arquiteturas de exposição, por sistema

### A1. Unreal Niagara — **stack-in-graph** (3 níveis, cada um com paradigma próprio)

**Hierarquia:** Module (grafo) → Emitter (stack) → System (canvas de stacks + timeline).
- **System Editor**: canvas onde cada Emitter aparece como uma **coluna vertical (stack)**; o System tem stack próprio; um Sequencer/timeline embaixo controla loop/burst timing dos emitters.
- **Stack = fases do ciclo de vida**, executadas topo→fundo, grupos na ordem: `System Spawn → System Update → Emitter Spawn → Emitter Update → Particle Spawn → Particle Update → Events/Simulation Stages → Render`. "Particle simulation in Niagara operates as a stack — simulation flows from the top to the bottom, executing programmable code blocks called modules in order" ([Key Concepts](https://dev.epicgames.com/documentation/unreal-engine/key-concepts-in-niagara-effects-for-unreal-engine)).
- **Estados de execução** do emitter/system: `Active / Inactive / InactiveClear / Complete` ([System & Emitter Module Reference](https://dev.epicgames.com/documentation/en-us/unreal-engine/system-and-emitter-module-reference-for-niagara-effects-in-unreal-engine)).
- **Namespaces com permissão por grupo** (o mecanismo que disciplina o stack): System group lê System/Engine/User e escreve **só** System; Emitter group escreve Emitter; Particle group escreve `Particles.*`. Todo o stack é um *parameter map* que flui módulo a módulo (`Particles.Velocity`, `Particles.NormalizedAge`, `Engine.DeltaTime`, `User.*` = exposto ao jogo). É o equivalente estrutural do `accel` transiente do PH2D — mas generalizado: **qualquer atributo é uma coluna nomeada em namespace**, e forças escrevem em `Physics.Force` que UM solver (`Solve Forces and Velocity`) integra — literalmente a semântica Houdini do PH2D.
- **Module = grafo interno**: cada module é um visual script (gera HLSL) com `Map Get`/`Map Set` nas pontas. O artista comum **nunca abre** o grafo; vive no stack.
- **Dynamic Inputs (a joia)**: todo input de módulo tem um dropdown que troca o valor literal por um **mini-grafo tipado sem canvas** — `Multiply Float`, `Random Range Float`, `Float from Curve`, `Lerp`, sample de textura… e eles **encadeiam recursivamente** (um Dynamic Input tem inputs que aceitam Dynamic Inputs). O parâmetro vira uma árvore de expressão renderizada como indentação no próprio stack. São assets próprios (você escreve os seus). Ex. documentado: "scaled using a Dynamic Input, simply Multiply Float by some constant and Link Inputs > Particles > Normalized Age" ([ibbles/Niagara.md](https://github.com/ibbles/LearningUnrealEngine/blob/master/Niagara.md)).
- **Curvas inline**: `Float from Curve`/`Color from Curve` desenham o **editor de curva embutido no stack**, com sample index default = `Particles.NormalizedAge` (curva-sobre-vida sem nenhum fio).
- **Scratch Pad**: módulo/dynamic-input **local ao asset** — `+ → New Scratch Pad Module` põe o módulo no stack e abre o shell do grafo; promove a asset reusável depois ([docs 4.27/5.x](https://docs.unrealengine.com/4.27/en-US/RenderingAndGraphics/Niagara/EmitterEditorReference)).
- **Herança + versioning**: emitters herdam de emitter-asset pai (overrides destacados por cor); módulos têm **versões explícitas** com banner de upgrade no stack e scripts python de migração de parâmetros.
- **Eventos**: Location/Death/Collision geram event data; o receptor ganha um grupo **Event Handler** no stack (modo: Spawned/Every Particle) — spawn de filhos dirigido por evento.
- **Simulation Stages (GPU)**: múltiplos passes Spawn/Update nomeados em sequência, iterando sobre partículas ou data interfaces (fluidos em grid) ([Key Concepts](https://dev.epicgames.com/documentation/unreal-engine/key-concepts-in-niagara-effects-for-unreal-engine)).
- **Debugging** ([Niagara Debugger UE5.8](https://dev.epicgames.com/documentation/unreal-engine/niagara-debugger-for-unreal-engine), [Debugging 4.27](https://docs.unrealengine.com/4.27/en-US/RenderingAndGraphics/Niagara/DebuggingInNiagara)): **Debug HUD** com "printout per particle in the Viewport with the attributes that you selected"; **Attribute Spreadsheet** — "see the inputs to the simulation as well as the per-particle computed values" numa tabela viva filtrada, com captura de frame; playback global de FX (pause/step/slow); **isolate** (solo) por emitter; per-module debug draw (forças desenham vetores); custo por módulo no stack em builds de profiling.

### A2. Unity VFX Graph — **híbrido perpendicular** (espinha vertical de Contexts + grafo horizontal de Operators)

([GraphLogicAndPhilosophy](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@12.1/manual/GraphLogicAndPhilosophy.html), [Blocks](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@12.1/manual/Blocks.html), [Subgraph](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@12.1/manual/Subgraph.html))
- **System = cadeia vertical** `Spawn → Initialize → Update → Output` ligada por *flow slots*; vários systems no mesmo asset. Spawn roda todo frame e computa quantos nascem; Initialize no nascimento; Update por frame ("Forces and Collisions"); Output define a forma/render.
- **Blocks**: "Nodes that you can stack into a Context. Every Block is in charge of one operation… Unity executes the Blocks in a Context from top to bottom." Criados com spacebar/right-click dentro do context, **arrastáveis entre contexts compatíveis**, checkbox enable/disable por block (exige recompile). É o stack do Niagara **desenhado dentro de um nó do grafo**.
- **Dualidade**: fluxo **vertical = processamento** (vida do sistema), fluxo **horizontal = propriedade** (Operators de matemática/sampling alimentando ports de Blocks/Contexts). O idioma: lógica de dados entra de lado, ciclo de vida desce.
- **Blackboard**: painel de propriedades com flag **Exposed** → aparecem no componente `VisualEffect` no Inspector (bindáveis por C#/Timeline), com categorias, tooltips, range. Nos subgraphs, o Blackboard define inputs/outputs e a categoria de menu ([Subgraph](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@12.1/manual/Subgraph.html)).
- **Subgraphs em 3 sabores**: System Subgraph, **Block Subgraph** (blocks empacotados usados como UM block — "all Blocks inside the Block Subgraph Context execute in order"), Operator Subgraph.
- **Eventos**: Spawn contexts recebem eventos nomeados (`OnPlay`/`OnStop` + custom via C#/Timeline) com **payload de atributos**; blocks `Trigger Event On Die/Always/Rate` geram **GPU events** que alimentam o Initialize de um sistema filho (trails, splashes) — parenting partícula→partícula 100% GPU.
- **Blocks canônicos** (lista de treino, conferível no manual): Spawn: `Constant Spawn Rate / Single Burst / Periodic Burst / Variable Spawn Rate`; Initialize: `Set Attribute` (Position/Velocity/Lifetime/Color/Size…), `Position (Sphere/AABox/Circle/Torus/Line/Mesh/Skinned Mesh/SDF/Sequential)`, `Velocity from Direction & Speed (Random/Spherical/Tangent/New Direction)`; Update: `Force`, `Gravity`, `Drag`, `Linear Drag`, `Turbulence` (Value/Perlin/Cellular, octaves/frequency/intensity), `Vector Force Field`, `Conform to Sphere`, `Conform to SDF`, `Collide (Sphere/AABox/Cylinder/Plane/SDF/Depth Buffer)` com bounce/friction/lifeloss, `Kill (AABox/Sphere/Plane)`, `Trigger Event`; Output: orient (`Face Camera/Along Velocity/Fixed Axis`), flipbook UV, shader-graph.
- **Recepção pelos artistas** (síntese de discurso comunitário, não de doc): o stack dentro do context lê bem ("Niagara-like"); a crítica recorrente é o spaghetti horizontal quando a lógica cresce e o recompile-para-toggle; o elogio é a espinha vertical tornar o ciclo de vida auto-evidente.

### A3. Superluminal Stardust — **grafo de painel único DENTRO de host de timeline** ([user guide](https://superluminal.tv/user-guide), [visão geral](https://superluminal.tv/stardust-new-version-6))

- **1 layer AE = 1 efeito = 1 grafo**: o painel Stardust (Window menu) mostra os nós; **cada parâmetro de cada nó é um parâmetro de efeito no Effect Control Window do AE** → keyframe, expression, parent, driver de áudio — o host inteiro vira o motor de animação dos parâmetros do grafo. Renomear o controle no ECW renomeia o nó no painel.
- **Conexão**: arrastar conectores-círculo entre nós; Shift = multi-conexão; **Alt+drag destaca um nó da cadeia religando os vizinhos** (splice). Fluxo: `Emitter → Particle` mínimo; Fields/Replica/Force penduram; **força solta no canvas = global** (aplica a todos os sistemas), força conectada entre emitter/particle afeta só propriedades de nascimento.
- **Node set completo** (do user guide): **Emitter** (Point, Sphere, Box, Spline via luzes AE, OBJ vértice/face/volume/superfície, Text/Mask, Grid retangular/esférico, Light, Ring, Layer, Auxiliary=emitir de outro sistema); **Particle** (Circle, Rectangle, Cloud, Texture, Face, Model 3D); **Fields/space deformers** (Sphere/Box/OBJ pull-push-enclose, Gradient/Radial, Bend/Stretch/Twist/Spiral, Maps por layer AE com projeção esférica/planar, Black Hole); **Turbulence** (noise suave sobre position/color/size, modos normal/esférico/axial, falloff esférico); **Replica** (Offset exponencial, Grid, Sphere, Linear, Corners, Path, Spline, Scatter, **Flow** — réplica guiada por turbulência); **Force** (gravity/spin/wind com ajuste over-life); **Physics** (Physical Properties Dynamic/Kinematic + Forças: Directional, Spherical com swirl, Noise, Path-follow com colisão, **Connect** = correntes/molas entre partículas); **Model/Material/Deform** (OBJ/primitivas/texto extrudado/VDB; PBR com SSS/refraction/emissive; deform por noise/bend/twist/maps/path); **Volume** (malha OpenVDB de partículas com boolean/smooth/advect); **Volumetric Lighting**; utilitários: **Transform, Motion (paths pré-prontos: circular/follow/orient-to-light), Shading, Clone (ramifica sistema com time-shift+reseed), Source, Group, Light, Aux**. (O papel do "Form" do Trapcode é coberto por Grid/OBJ emitter com velocidade zero; **Mirror** entra como modo de réplica/simetria nas versões novas.)
- **Curvas over-life e over-path embutidas** por propriedade (mini-gráficos), + time-remap do efeito inteiro por um parâmetro.
- **PRESETS são o centro do produto**: presets serializam **a árvore de nós inteira + render settings + solids com máscara + text layers + footage/OBJ + luzes + keyframes lineares + expressions**; **Smart Presets** = one-click; browser com busca indexada por keyword ("Fire", "Grid", "Replica"); user presets em pasta versionável. A biblioteca É o onboarding e o marketing ([CG Channel 1.6](https://www.cgchannel.com/2020/07/superluminal-releases-stardust-1-1-for-after-effects/)).
- **Higiene de grafo**: por nó, switches **Visibility / Solo / Shy** (esconde do ECW) / **Helpers** (gizmos de viewport por nó) / Locate (centra no painel). Cache de física e volume com estados Off/On/**Freeze**.

### A4. Autograph (Left Angle → Maxon, grátis desde 2026.0) — **"nós sem canvas": Generators + Modifier stack por PARÂMETRO**

([CG Channel 2023](https://www.cgchannel.com/2023/01/check-out-next-gen-motion-design-and-vfx-tool-autograph/), [2023.11](https://www.cgchannel.com/2023/12/left-angle-releases-autograph-2023-11/), [2025](https://www.cgchannel.com/2025/01/left-angle-releases-autograph-2025/), [free/Maxon 2026](https://www.cgchannel.com/2026/04/maxon-releases-motion-graphics-software-autograph-for-free/), [AE vs Autograph](https://www.vfxer.com/adobe-after-effects-vs-maxon-autograph/))
- Timeline de camadas estilo AE (layer stack + dope sheet) + modo 3D USD com renderer Filament PBR.
- **Generators** = "a simpler alternative to a node graph": fontes procedurais não-destrutivas de **imagem, texto, número e objeto 3D** — o nó-fonte vive como camada/entrada, não num canvas.
- **Modifiers** = "a simpler alternative to expressions": **pilha não-destrutiva anexada a QUALQUER parâmetro** ("multiply a number, shorten a word, blur an image, or warp a 3D object") — "Modifiers act as a non-destructive stack, similar to the logic found in 3D packages like Blender". Exemplos citados: noise procedural sobre shapes, modifiers **audio-reativos** (frequências do mix, EQ paramétrico). A pilha É um grafo linear ancorado no ponto de uso — zero fio, zero canvas, zero código ("without writing a single line of code").
- **Instancer/data**: liga **CSV/planilha** a generators pela UI → variantes de design/idioma de um projeto único (localization em massa).
- **Responsive design**: 1 arquivo → N resoluções/aspect ratios (parâmetros relativos ao formato) — o diferencial de produto declarado.
- Lição de UX: eles apostaram que o público de motion design **rejeita o canvas**; a resposta não foi stack-in-graph (Niagara) e sim **stack-no-parâmetro** (o lugar onde o artista já está olhando).

### A5. Rive (rápido) — **state machine como grafo de CONTROLE, não de dados**

([Inputs](https://help.rive.app/editor/state-machine/inputs), [Layers](https://help.rive.app/editor/state-machine/layers), [runtimes](https://help.rive.app/runtimes/state-machines), [blog](https://rive.app/blog/how-state-machines-work-in-rive))
- O grafo tem **estados = animações (clips)** e transições com condições sobre **Inputs** de 3 tipos: `number / boolean / trigger` — "Inputs are the contract between designers and developers".
- **Layers**: máquinas paralelas — "only one state can be active per layer… you can have multiple layers running simultaneously" (mão acena ENQUANTO corpo anda).
- **Blend states**: 1D blend entre animações dirigido por number input (o blend-tree de Unity em miniatura).
- **Listeners**: eventos de ponteiro (hover/click/drag) mapeados a mudanças de input **dentro do editor**, sem código.
- **Constraints** no rig (IK, distance, transform, follow-path) compõem com a máquina — ex. joystick = bone com Distance Constraint.
- Relevância PH2D: é o modelo de **runtime control plane sobre clips** — casa com a composição de clips/containers da Timeline, não com o Motion cook.

### A6. Trapcode Particular (rápido) — **Designer: blocks-como-presets sobre efeito monolítico**

([Maxon product](https://www.maxon.net/en/product-detail/red-giant/particles-and-3d/trapcode-particular), [Red Giant 2024](https://www.cgchannel.com/2023/09/maxon-releases-red-giant-2024/))
- **Não há canvas nem "Flow"** (Flow é o tipo de Replica do *Stardust*): Particular é UM efeito AE com árvore de parâmetros monolítica; o **Designer** é um builder modal **block-based** — "add adjustable blocks with preset behaviors and styles for emitters, particles, physics, and aux particles" + browser de **355+ presets**; blocks são presets grossos aplicados aos parâmetros, refinados depois no ECW. Suporta cadeias parent/child de emitters (aux systems). Lição: blocks = **atalho de autoria**, não modelo de execução.

---

## B) CatálOGO consolidado (nó/module · sistema · função · status vs PH2D)

Status julgado contra os 87 nós declarados (forças/deformers/distribuição/sim/rig/value/pulse/color + zone + trail + expression). `(?)` = conferir no registry.

| Nó/Module | Sistema | Função | vs PH2D |
|---|---|---|---|
| Curl Noise Force | Niagara | força divergence-free por campo de curl | **TEMOS** (`force.curl`, Bridson) |
| Drag | Niagara/VFXG/Stardust | arrasto linear (+rotacional no Niagara) | **TEMOS** (`force.drag`); drag ROTACIONAL FALTA(?) |
| Gravity Force | Niagara/VFXG | aceleração constante (massa-independente no VFXG) | **PARCIAL** (expressável via wind/linear; nó dedicado com preset −g FALTA) |
| Point Attraction / Point Force | Niagara/Stardust(Spherical) | atração a ponto c/ falloff, kill radius, swirl (Stardust) | **TEMOS** (`force.attractor`); kill-radius + swirl PARCIAL |
| Line Attraction Force | Niagara | atrai ao ponto mais próximo de um segmento | **FALTA** |
| Vortex Force / Vortex Velocity | Niagara/Stardust | velocidade tangencial em torno de eixo + pull à origem | **TEMOS** (`force.vortex`); "vortex velocity no spawn" PARCIAL |
| Wind Force | Niagara | vento que **satura na velocidade do vento** (air resistance/massa) | **PARCIAL** (`force.wind` existe; a semântica saturante é o diferencial) |
| Linear/Acceleration Force | Niagara/VFXG(Force) | vetor de força/aceleração em espaço escolhido | **TEMOS** (wind/linear) |
| Vector Noise Force | Niagara | jitter randômico por partícula (não-curl) | **PARCIAL** (curl ≠ jitter; hash por id existe p/ emitter) |
| Limit Force / Speed Limit | Niagara | clampa \|força\|/\|vel\| acima de teto | **FALTA** (barato e útil p/ estabilidade) |
| Turbulence (displacement) | Stardust/VFXG | noise aplicado a **posição/cor/tamanho** direto (não via accel) | **PARCIAL** (deformers cobrem posição; noise-sobre-atributo genérico?) |
| Vector Force Field / Sample Vector Field | VFXG/Niagara | campo vetorial de textura/asset dirigindo força | **FALTA** |
| Conform to Sphere / Conform to SDF | VFXG | força de sucção+aderência a superfície/SDF | **FALTA** (a dupla attract+stick; SDF!) |
| Collision (planos/depth/SDF/scene) | Niagara/VFXG | colisão com bounce/friction/lifeloss | **TEMOS** (`sim.collide`; variedade de proxies FALTA) |
| Kill (volume box/sphere/plane, invert) | Niagara/VFXG | mata dentro/fora de volume | **PARCIAL** (lifetime/collide existem; kill-volume dedicado?) |
| Spawn Rate | Niagara/VFXG | contínuo, partículas/seg + probability | **TEMOS** (emitter rate) |
| Spawn Burst Instantaneous / Single+Periodic Burst | Niagara/VFXG | N no tempo T; periódico | **PARCIAL** (pulse.* pode dirigir rate; burst declarativo de 1ª classe?) |
| Spawn Per Unit | Niagara | spawn por **distância percorrida** do emitter | **FALTA** (o nó de trilha-de-movimento) |
| Spawn Per Frame | Niagara | N por frame | PARCIAL |
| Initialize Particle | Niagara | bloco único: lifetime/pos/massa/cor/size/rotação | **PARCIAL** (colunas nascem no emitter; um "init consolidado" é UX, não motor) |
| Location: Box/Sphere/Cylinder/Cone/Torus/Grid | Niagara/VFXG | distribuição de nascimento em primitivas | **TEMOS/PARCIAL** (M3 distribuições — cone/torus?) |
| Static/Skeletal Mesh Location | Niagara | nascimento em superfície/osso/vértice de malha | **PARCIAL** (2D: nasc. em spline/shape vetorial = o análogo; skeleton TEMOS) |
| Jitter Position | Niagara | re-jitter posicional com delay timer | FALTA (trivial) |
| Rotate Around Point / Orbit | Niagara/Stardust(Motion) | órbita paramétrica | **PARCIAL** (vortex ≈; órbita determinística por idade?) |
| Add Velocity (linear/from point/in cone) | Niagara | velocidade inicial modal | **TEMOS/PARCIAL** |
| Inherit Velocity | Niagara | herda velocidade da fonte/emitter | **FALTA** (emitter stateless torna isso interessante: é função do playhead) |
| Mass/Rotational Inertia by Volume | Niagara | massa a partir de densidade×bounds | FALTA (rígido já tem no physics; motion não) |
| Replica (Offset/Grid/Sphere/Linear/Corners/Scatter/Flow) | Stardust | **replicação do sistema inteiro** com transform incremental | **PARCIAL** (kaleidoscope/mirror ≈ 2 modos; a família toda FALTA) |
| Aux / Clone / parent-child emitters | Stardust/Particular | sistema filho emitido de partículas do pai | **FALTA** (= GPU events do VFXG) |
| Trigger Event On Die/Always + GPU Event | VFXG/Niagara | evento por partícula → spawn em sistema filho | **FALTA** (o buraco grande: parenting evento→spawn) |
| Fields/space deformers (Bend/Twist/Spiral/Maps/BlackHole) | Stardust | deformação do ESPAÇO das partículas | **TEMOS** (deformers GPU) — Maps-por-layer PARCIAL |
| Physics Connect (chains/springs) | Stardust | correntes/molas entre partículas | **TEMOS** (`motion.spring`, verlet_rope) |
| Path force / follow path | Stardust | segue caminho colidindo | **PARCIAL** (rig/spline? follow-path de partícula FALTA) |
| Force over-life graphs | Stardust | toda força modulada por curva de idade | **PARCIAL** (value.*+expression fazem; curva INLINE é UX que falta) |
| Ribbon/Strip renderer | Niagara/VFXG | trilha geométrica ligando partículas por idade | **PARCIAL** (`motion.trail` = eco; ribbon contínuo?) |
| Sprite renderer + orient | Niagara/VFXG | billboard/along-velocity | TEMOS (instâncias 2D nativas) |
| Blackboard / User.* params | VFXG/Niagara | contrato de parâmetros exposto ao host | **PARCIAL** (`Graph::set_text_param` + params no Graph = a fundação; painel de exposição FALTA) |
| State machine (states/transitions/inputs/listeners/blend) | Rive | grafo de controle dirigindo clips | **FALTA** (candidato: camada sobre Timeline/containers, não sobre Motion) |
| Constraints (IK/distance/follow) | Rive | rig constraints | **TEMOS** (ik/fabrik/rubber_hose) |
| Audio-reactive modifier | Autograph/Stardust(via AE) | freq do áudio → parâmetro | **FALTA** (o módulo de áudio já tem análise espectral! `ph2d-audio-spectral`) |
| Data/CSV instancer | Autograph | planilha → variantes | FALTA (fora de escopo motion) |
| Presets que serializam a árvore | Stardust/Particular | biblioteca pesquisável one-click | **FALTA** (o formato textual v2 do Graph já é o serializado — falta o browser) |

---

## C) Parâmetros dos ~15 modules maduros (Niagara sobretudo)

⚠️ Fonte da estrutura: [Particle Update Group Reference](https://dev.epicgames.com/documentation/en-us/unreal-engine/particle-update-group-reference-for-niagara-effects-in-unreal-engine), [Particle Spawn Group Reference](https://dev.epicgames.com/documentation/en-us/unreal-engine/particle-spawn-group-reference-for-niagara-effects-in-unreal-engine), [Emitter Update Group Reference](https://dev.epicgames.com/documentation/en-us/unreal-engine/emitter-update-group-reference-for-niagara-effects-in-unreal-engine) — as páginas listam módulos/opções mas **não publicam defaults numéricos**; os defaults abaixo marcados `≈` vêm de conhecimento de editor (treino) — conferir in-engine antes de copiar como literal. Unidades UE: cm, cm/s, cm/s².

1. **Gravity Force** (Particle Update): `Gravity: vec3` default **(0, 0, −980)** cm/s² (o −9,8 m/s² canônico). Escreve em `Physics.Force` (massa-aware).
2. **Drag**: `Drag: float` ≈1.0 (adimensional; `v *= 1/(1+drag·dt)` na prática); `Rotational Drag: float` ≈0.0; checkboxes p/ afetar velocity/rotational separadamente.
3. **Curl Noise Force**: `Noise Strength: float` ≈300; `Noise Frequency: float` (escala espacial do campo; ≈ centenas de cm por célula); `Pan Noise Field: vec3` (advecção do campo, default 0); offset/seed. Divergence-free (mesma família do `force.curl` PH2D).
4. **Point Attraction Force**: `Attraction Strength: float` ≈100–1000 típico; `Attraction Radius: float` ≈alcance com falloff; `Falloff Exponent`; `Kill Radius: float` (mata ao chegar — o detalhe que o attractor PH2D não tem); `Attractor Position: vec3` (aceita Dynamic Input → seguir outro objeto).
5. **Vortex Force**: `Vortex Axis: vec3` (0,0,1); `Vortex Amount: float` (vel. tangencial); `Origin Pull Amount: float` (componente radial p/ dentro — o que evita o espalhamento); `Vortex Origin: vec3`.
6. **Wind Force**: `Wind Speed: vec3` (alvo de velocidade — **satura**: só acelera até a partícula igualar o vento); `Air Resistance: float`/massa opcional. Semântica ≠ Linear Force (que empurra sem teto).
7. **Linear Force**: `Force: vec3`; `Coordinate Space: Local|World|Simulation`; `Force Scale: float`.
8. **Limit Force**: `Force Limit: float` — clampa \|Physics.Force\| acumulada (roda DEPOIS das forças, ANTES do solve — a posição no stack é o parâmetro).
9. **Vector Noise Force**: `Noise Strength: float`; `Noise Frequency: float` — ruído branco/valor por partícula (jitter caótico, não-curl).
10. **Spawn Rate** (Emitter Update): `SpawnRate: float` part/s; `Spawn Probability: 0..1` =1; `Spawn Group: int` =0. Aceita Dynamic Input (rate por curva/áudio/distância).
11. **Spawn Burst Instantaneous**: `Spawn Count: int`; `Spawn Time: float s` =0 (relativo ao loop do emitter); `Spawn Probability`; `Spawn Group`.
12. **Spawn Per Unit**: `Spawn Per Unit: float` (part/cm percorrido); `Use Movement Tolerance: bool` + `Movement Tolerance` (ignora micro-movimento); `Max Movement Threshold` (teleporte não jorra partículas — o clamp que evita o burst no jump-cut!).
13. **Emitter State** (Emitter Update): `Life Cycle Mode: System|Self`; `Inactive Response: Complete|Kill|Continue`; `Loop Behavior: Once|Multiple|Infinite`; `Loop Duration: float s` + `Loop Duration Mode`; `Loop Delay`; `Scalability Mode: System|Self` + culling por distância/visibilidade; `Reset Age on Awaken` ([ref](https://dev.epicgames.com/documentation/en-us/unreal-engine/emitter-update-group-reference-for-niagara-effects-in-unreal-engine)).
14. **Initialize Particle** (Particle Spawn): `Lifetime Mode: Direct|Random(min,max)` default ≈5s; `Color: linear RGBA` =branco; `Position Mode: Simulation|Direct`; `Mass` =1; `Sprite Size Mode: Uniform|Random Uniform|NonUniform` ≈10–50 px; `Sprite Rotation Mode: Random|Direct`; `Mesh Scale`. Um módulo, todos os atributos com enable individual.
15. **Sphere Location**: `Sphere Radius` ≈100; `Hemisphere X/Y: bool`; `Surface Only Band Thickness` (0=volume, >0=casca); distribuição Random|Uniforme; offset + coordinate space. (Irmãos Box/Cylinder/Cone/Torus/Grid com o mesmo padrão modal.)
16. **Add Velocity in Cone**: `Cone Axis: vec3`; `Cone Angle: graus`; `Velocity Strength: float|range`; `Inner Cone %`; falloff angular.
17. **Collision** (Particle Update): `Collision Mode: Scene Depth (GPU)|Distance Fields|Ray Traced (CPU)|Analytical Planes`; `Restitution` ≈0.15–0.3; `Friction` ≈0; `Radius Calculation` (auto do sprite size + scale); `Restitution Randomness`; kill-on-collide opcional.
18. **VFX Graph Turbulence** (Update block, p/ contraste): `Mode: Absolute|Relative`; `Noise Type: Value|Perlin|Cellular`; `Intensity`; `Frequency`; `Octaves 1..8`; `Roughness`; `Lacunarity`; transform do campo ([manual](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@12.1/manual/Blocks.html)).

---

## D) As 10 ideias de UX que valem roubo (ranqueadas para o PH2D)

1. **Dynamic Inputs (Niagara)** — o dropdown que troca qualquer parâmetro escalar por uma árvore de mini-nós **sem abrir canvas**, recursiva, indentada no próprio painel. É a maior alavanca: o PH2D já tem `motion.expression` + text params — a UX que falta é *promover qualquer param do painel a expressão/curva/random com 1 clique*, mantendo o grafo limpo. Mata 80% dos nós utilitários de valor.
2. **Attribute Spreadsheet + Debug HUD (Niagara)** — tabela viva partícula×coluna com captura de frame, + atributos impressos sobre a instância no viewport, + solo por emitter. O PH2D tem probe; a **planilha de colunas do Stream** é o degrau seguinte e é barata (as colunas já são colunas). ([debugger docs](https://dev.epicgames.com/documentation/unreal-engine/niagara-debugger-for-unreal-engine))
3. **Curvas inline no stack (Niagara `Float from Curve`)** — o editor de curva mora DENTRO do parâmetro, default samplado por idade normalizada. Curva-sobre-vida sem fios é o que separa "dá pra fazer" de "artistas fazem".
4. **Fases do ciclo como GRUPOS visuais (Niagara stack / VFXG contexts)** — Spawn/Update/Render como seções rotuladas com permissão de escrita por seção. O PH2D já TEM a semântica (Pure→accel→UM integrador); expor o grafo agrupado por fase transforma a regra invisível em geografia visível — e o erro "força depois do integrate" vira impossível de desenhar.
5. **Presets que serializam a árvore inteira (Stardust Smart Presets)** — preset = grafo + assets + keyframes, browser pesquisável, one-click. O formato textual v2 do Graph JÁ é a serialização; falta só o browser + pasta de user presets. É onboarding, marketing e teste de regressão de graça.
6. **Blackboard/User params (VFXG/Niagara `User.*`)** — painel de propriedades EXPOSTAS do grafo com flag, categoria e range, bindável pelo host (Timeline!). Fundação PH2D existente: params no `Graph`. O ganho: a Timeline keyframa o grafo pelo contrato, não pelo nó interno.
7. **Evento→spawn filho (VFXG GPU Events / Stardust Aux / Particular parent-child)** — `on die/collide/rate` da partícula pai alimenta o Initialize do sistema filho com payload de atributos. É a capacidade estrutural que o catálogo B mostra faltando por inteiro no PH2D — e casa com o desenho stateless (filho = função do evento registrado no cook).
8. **Modifier-stack-no-parâmetro (Autograph)** — a pilha não-destrutiva ancorada no parâmetro, não num canvas: noise/audio/math empilhados onde o artista já olha. Para o PH2D é o mesmo gesto do item 1 visto de outro ângulo — e o caso **audio-reativo** é imediato (`ph2d-audio-spectral` já analisa).
9. **Herança + versioning de módulos (Niagara)** — emitter herda de asset-pai com overrides destacados; módulo tem versão com banner de upgrade e script de migração. Para uma biblioteca de 87+ nós que evolui, é a resposta a "mudei o nó, o que acontece com os grafos salvos".
10. **Solo/Shy/Helpers por nó + Freeze de cache (Stardust)** — solo de render por nó, shy esconde do painel de params, helper = gizmo de viewport por nó, caches com estado Freeze. Higiene barata que faz grafo grande continuar operável.

*Menções honrosas:* Scratch Pad (módulo local promovível a asset — o caminho "protótipo→biblioteca" sem fricção); Rive **layers de state machine + inputs/listeners** como plano de controle interativo sobre os containers da Timeline (é outro produto, mas o modelo de `inputs = contrato` é o certo); Particular Designer (blocks-como-presets grossos: autoria em 2 tempos — bloco grosso, refino fino).

**Fontes principais:** [Niagara Key Concepts](https://dev.epicgames.com/documentation/unreal-engine/key-concepts-in-niagara-effects-for-unreal-engine) · [Overview](https://dev.epicgames.com/documentation/unreal-engine/overview-of-niagara-effects-for-unreal-engine) · [System/Emitter Module Ref](https://dev.epicgames.com/documentation/en-us/unreal-engine/system-and-emitter-module-reference-for-niagara-effects-in-unreal-engine) · [Particle Update Ref](https://dev.epicgames.com/documentation/en-us/unreal-engine/particle-update-group-reference-for-niagara-effects-in-unreal-engine) · [Particle Spawn Ref](https://dev.epicgames.com/documentation/en-us/unreal-engine/particle-spawn-group-reference-for-niagara-effects-in-unreal-engine) · [Emitter Update Ref](https://dev.epicgames.com/documentation/en-us/unreal-engine/emitter-update-group-reference-for-niagara-effects-in-unreal-engine) · [Niagara Debugger](https://dev.epicgames.com/documentation/unreal-engine/niagara-debugger-for-unreal-engine) · [ibbles Niagara notes](https://github.com/ibbles/LearningUnrealEngine/blob/master/Niagara.md) · [VFXG Graph Logic](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@12.1/manual/GraphLogicAndPhilosophy.html) · [VFXG Blocks](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@12.1/manual/Blocks.html) · [VFXG Subgraph](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@12.1/manual/Subgraph.html) · [Stardust User Guide](https://superluminal.tv/user-guide) · [Stardust new version](https://superluminal.tv/stardust-new-version-6) · [CG Channel Stardust 1.6](https://www.cgchannel.com/2020/07/superluminal-releases-stardust-1-1-for-after-effects/) · [Autograph 2023](https://www.cgchannel.com/2023/01/check-out-next-gen-motion-design-and-vfx-tool-autograph/) · [Autograph 2023.11](https://www.cgchannel.com/2023/12/left-angle-releases-autograph-2023-11/) · [Autograph 2025](https://www.cgchannel.com/2025/01/left-angle-releases-autograph-2025/) · [Autograph grátis/Maxon](https://www.cgchannel.com/2026/04/maxon-releases-motion-graphics-software-autograph-for-free/) · [AE vs Autograph](https://www.vfxer.com/adobe-after-effects-vs-maxon-autograph/) · [Rive Inputs](https://help.rive.app/editor/state-machine/inputs) · [Rive Layers](https://help.rive.app/editor/state-machine/layers) · [Rive runtimes](https://help.rive.app/runtimes/state-machines) · [Rive blog](https://rive.app/blog/how-state-machines-work-in-rive) · [Trapcode Particular](https://www.maxon.net/en/product-detail/red-giant/particles-and-3d/trapcode-particular) · [Red Giant 2024](https://www.cgchannel.com/2023/09/maxon-releases-red-giant-2024/)