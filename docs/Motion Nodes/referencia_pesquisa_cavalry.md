# CAVALRY (Scene Group / cavalry.studio) — Pesquisa profunda v2.7.2

**Fonte:** docs oficiais em `docs.cavalry.scenegroup.co` → **redireciona 301 para `https://cavalry.studio/docs/`** (usar esse domínio). Versão documentada: **2.7.2** (release notes até 2.7.2). Sitemap completo: `https://cavalry.studio/docs/sitemap.xml` (~800 páginas). Itens marcados **[treino]** vêm de conhecimento prévio não re-verificado no fetch desta sessão; todo o resto foi extraído das páginas citadas.

---

## A) CATÁLOGO COMPLETO

Cavalry divide tudo em **Layers**: *Shapes* (desenham no viewport), *Behaviours* (modificam atributos/geometria de outros layers), *Utilities* (geram valores/dados, não desenham), *Effects* (Filters raster + Shaders), *Distributions* (tipos embutidos consumidos por Duplicator/Connect Shape/etc.). "Shapes are Layers that can be drawn in the viewport and animated using keyframes and/or Behaviours" (`/docs/nodes/shapes/`).

### A.1 Shapes (`https://cavalry.studio/docs/nodes/shapes/`)

| Nó | Função | vs PH2D |
|---|---|---|
| Basic Shape (Rectangle, Ellipse, Polygon, Star, Arc, Arrow, Capsule, Cogwheel, Ring, Super Ellipse, Super Shape) | primitivas procedurais paramétricas | PARCIAL (temos primitivas no vetor, não como fonte no grafo motion) |
| Basic Line (Line, Bezier, Spiral) | linhas procedurais | PARCIAL |
| Editable Shape | path não-procedural editado à mão (pen tool) | PARCIAL (vetor tem, motion não consome) |
| Custom Shape | path por pontos conectáveis (dados→geometria) | FALTA |
| Text Shape | texto completo (ver B) | FALTA (motion não tem texto) |
| **Duplicator** | clona shapes sobre uma Distribution; o coração mograph | TEMOS (motion.clone + distribute_*) |
| **Connect Shape** | liga pontos de distributions com retas/béziers (rede/plexus) | FALTA |
| **Trails** | rastro geométrico do movimento de shapes (experimental, licença Pro) | TEMOS (motion.trail) |
| **Particle Shape** | renderiza/hospeda partículas (par do Particle Emitter; experimental Pro) | TEMOS (emitter+sim.*) |
| **Forge Dynamics Shape** | rígidos 2D Box2D (+ soft body) | PARCIAL (física é módulo ECS separado; sim.zone/collide no grafo) |
| Composition | pre-comp; instâncias com **Pre-comp Overrides** (`/composition/pre-comp-overrides/`) | FALTA (no motion) |
| Component Shape | empacota rig+controles num layer reutilizável [treino: 2.0 "Components"] | FALTA |
| Group | agrupamento com transform | TEMOS (hierarquia) |
| Layout Shape / Layouts (Grid Layout Group/Row, Layout Group, Spacer) | layout responsivo de layers (flex/grid-like) | FALTA |
| Merge | funde paths de vários shapes | PARCIAL |
| Outline | contorno/stroke como geometria | PARCIAL (vetor: Outline Stroke) |
| Convex Hull | casco convexo de pontos/shapes | FALTA |
| Points to Path | pontos (arrays/distributions)→path | PARCIAL (make_point sem path) |
| Segment Path / Chop Path (behaviour) | corta path em segmentos | FALTA |
| Shortest Path | menor caminho entre pontos sobre obstáculos | FALTA |
| Quad Tree Shape | subdivisão quadtree de imagem/shape | FALTA |
| Isolines Shape | linhas de contorno (topográficas) de imagem/ruído | FALTA |
| Rectangle Pattern | padrão de retângulos empacotados | FALTA |
| Ray | raio com colisão (raycast visual) | FALTA |
| Image to Shapes | traça imagem→vetores | FALTA |
| SVG / Footage / Cel Animation / Background / Mesh Shape / Extrude / Corner Pin / Extract Sub-Meshes / Sub-Mesh Bounding Box | import/pintura frame-a-frame/fundo/malha 2.5D/extrusão fake-3D/pin de cantos/acesso a sub-meshes | FALTA (maioria) — four_point_warp ≈ Corner Pin: TEMOS |
| JavaScript Shape | geometria gerada por script | PARCIAL (motion.expression é só valor) |

### A.2 Behaviours (`https://cavalry.studio/docs/nodes/behaviours/`)

| Nó | Função | vs PH2D |
|---|---|---|
| **Oscillator** | onda cíclica (ver B) | TEMOS (oscillator/lfo) |
| **Noise** | ruído (valor OU deformer; ver B) | TEMOS (noise/wiggle) |
| **Random** | valores aleatórios estáveis por seed/index | PARCIAL (hash interno, sem nó dedicado) |
| **Wave** | onda como valor e como deformer de path | TEMOS (motion.wave) |
| **Spring** | física de mola sobre atributo (overshoot) | TEMOS (motion.spring) |
| **Stagger** | valores sequenciais min→max por índice (ver B) | TEMOS (motion.stagger) |
| Behaviour Mixer | combina/pesa vários behaviours | TEMOS (motion.mixer) |
| Value / Value2 / Value3 (+ Blends, Solvers) | constantes e blends 1/2/3D; Solver resolve com iterações | TEMOS (debug.const, value.*) |
| Position Blend / Color Blend | interpolação posição/cor | TEMOS (mixer/color_ramp) |
| Blend Shape / Morph | interpola geometrias | TEMOS (motion.morph) |
| Bend Deformer | dobra | TEMOS (motion.bend) |
| Lattice Deformer (+ Lattice Controller util) | gaiola de deformação | TEMOS (motion.lattice) |
| Four Point Warp | warp por 4 cantos | TEMOS |
| Skew / Pinch | cisalha / pinça | FALTA |
| Travel Deformer | deforma shape percorrendo um path | PARCIAL (spline_wrap) |
| Wave (deformer) | ondula path | TEMOS |
| JavaScript Deformer | deformer por script (module `deformer`) | FALTA |
| Squash and Stretch / Motion Stretch | S&S automático por velocidade / esticão por movimento | FALTA (clássico de animação!) |
| Auto-Animate | anima entrada/saída automática de shapes [treino: presets de in/out] | FALTA |
| Look At | orienta para alvo | TEMOS (motion.look_at) |
| Boolean | booleanas de paths | PARCIAL (vetor tem; motion não) |
| Pathfinder | ops de interseção estilo Illustrator | FALTA |
| Path Offset / Path Relax / Resample Path / Reverse Path / Extend Open Paths / Add Divisions / Subdivide / Round / Bevel / Chop Path / Curves to Lines / Path Average / Knot / Stitches | suíte de path-ops procedurais | FALTA no motion (vetor cobre parte: offset vivo, corners) |
| Fill Rule / Flatten Shape Layers / Clean Up / Auto-Crop / Frame / Align | utilidades de geometria/layout | FALTA |
| Sub-Mesh | seleciona/edita sub-meshes (caracteres de texto, letras de duplicator) por índice/range — **é o "text animator"** | PARCIAL (value.instance_field seleciona por índice) |
| Contours to Sub-Meshes / Blend Sub-Mesh Positions / Mesh Solver / Voxelize | contorno→submesh, blend de posições, solver de malha, voxeliza | FALTA |
| Color/Alpha/HSV Material Override, Swap Color, Material Sampler, Number Range to Color, Contrasting Color (util) | recolorir procedural por índice/falloff | PARCIAL (tint, luminance, color_ramp) |
| Number Range | remapeia faixa | TEMOS (value.map_range) |
| Modulate | modula atributo por outro [treino] | TEMOS (drive) |
| Distance / Is Within / Area Range / Get Vector / Measure (util) | medidas geométricas como valores | PARCIAL (value.attribute) |
| **Sound** | áudio→valores por bandas (ver B) | FALTA |
| Visibility Sequence | liga/desliga shapes em sequência temporal | PARCIAL (step/strobe) |
| Stagger/Apply Layout/Apply Distribution | aplica layout/distribution como behaviour | PARCIAL |
| Rubber Hose Limb | membro IK estilo rubber-hose (Duik-like) | TEMOS (rig.rubber_hose!) |
| Manipulator | cria alças de controle no viewport | FALTA |
| Flare | lens flare | FALTA |
| 3D Matrix | transform 3D fake | FALTA |

### A.3 Utilities (`https://cavalry.studio/docs/nodes/utilities/`)

| Nó | Função | vs PH2D |
|---|---|---|
| **Falloff** / Range Falloff | campo de peso espacial (ver B) / falloff por faixa de índices | TEMOS (motion.falloff) / FALTA (por índice) |
| Color Array / Value/Value2/Value3 Array / String Array / Shape Array / Shader Array / Typeface Array / Asset Array | arrays indexáveis de cada tipo (cor por índice do clone etc.) | PARCIAL (color_array sim; demais FALTA) |
| Math / Math2 / Math3 / JS Math | aritmética 1/2/3D e funções JS | TEMOS (value.math) |
| Comparison / Logic / If Else | comparação, lógica booleana, seletor | TEMOS (pulse.compare, value.switch) |
| Accumulator | acumula valor ao longo do tempo | PARCIAL (pulse.counter) |
| Index Context / Length Context / Velocity Context / Velocity Magnitude Context / Position Context | **contexts**: dentro de um consumidor indexado, devolvem índice/contagem/velocidade por elemento | TEMOS (value.instance_field/attribute) |
| Local Time / Animation Control / Sequence / Scheduling Group / Timeline Counter / Seconds to Frames | relógio local por layer, controlador de clipes de animação, sequenciador, agendador de grupos | PARCIAL (motion.time_remap; sem sequenciador) |
| Layer Seed | seed por layer | PARCIAL (hash(seed,id)) |
| **Particle Emitter** / Distribution Emitter / JavaScript Emitter | emissores (ver B) | TEMOS (motion.emitter) |
| Modifiers de partícula: Turbulence, Vortex, Attractor(-Field), Goal, Flow Field, Magnetic, Drag(-Field), Buoyancy(-Field), Direction(-Field), Force, Speed, Path Field, Image Modifier, Data Modifier, Visual Modifier, JavaScript Modifier, Path Modifier | campos/forças de partículas | TEMOS (force.*) / PARCIAL (goal, flow field por imagem, magnetic, path field FALTAM) |
| Constraints (dinâmica): Distance, Pin, Transform, Bounding Box, Bridge, Comp, Component | vínculos entre corpos/layers | PARCIAL (motion.pin_constraint) |
| Collision Events: Color, Impulse, Sticky, Visibility, Body Settings | eventos disparados por colisão que mudam cor/aplicam impulso/grudam/mostram | FALTA (não há eventos de colisão no grafo) |
| **Spreadsheet / Spreadsheet Lookup** | lê CSV/TSV (e Google Sheets Asset); lookup por linha/coluna | FALTA (data-driven) |
| String + String Generator (Block, Date/Time, Formatted, Hash, Hex, Random Date, Random Number, Timecode, Value) + String Manipulator (Case, Join, Regex, Replace, Resize, Shuffle, Sub-String, Transition, Unicode Offset) + String Length / String From Asset / Get Name / Measure Text | suíte completa de strings procedurais (contadores animados, scramble text...) | FALTA |
| Typeface / Apply Font Size / Apply Character Spacing / Apply Typeface / Apply OpenType / Apply Text Material / Apply Font Style | estilização procedural de texto por índice | FALTA |
| Stroke / Fill / Stroke Duplicator | traço (width, dash, trim start/end/travel) e preenchimento como nós; duplica ao longo do stroke | PARCIAL |
| Rig Control / Lattice Controller / Null / Camera / Camera Guide | controles de rig, null, câmera 2D (zoom/pan da comp) | PARCIAL (rig.* sem UI de controle) |
| Bounding Box / Count Sub-Meshes / Get Sub-Mesh Transform / Path Length / Radius | consultas geométricas | PARCIAL |
| Image Sampler / Color Info / HSV Color / Index to Color | amostra pixels de imagem→valores/cores | TEMOS (motion.luminance) / PARCIAL |
| JavaScript Utility | valor arbitrário por script | PARCIAL (motion.expression) |
| Array Manipulator / Data Modifier | reordena/filtra arrays | FALTA |
| Asset From Smart Folder / Audio Smart Folder / Image Smart Folder | assets dinâmicos por pasta (versionamento/variações) | FALTA |

### A.4 Effects
- **Filters (54)**: blurs (Gaussian/Fast/Box/Directional/Zoom/Luminance/Bilateral/Background), Drop Shadow, Inner Shadow, Glow, RGB Split, Slit Scan, Spherise, Halftone, Pixelate, Pixel Sorting, Dithering, Posterize, Threshold, Levels, HSV, Brightness&Contrast, Gamma, B&W, Tritone, Gradient Map, Invert, Chroma Key, Chromatic Aberration, Edge Detection, Erosion, Distort Edges, Distortion, Bulge, Mirror, Polar Coordinates, Linear/Radial Wipe, Venetian Blinds, Light Sweep, Scan Lines, Scrape, Stripes, Sharpen, Shift Channels, Grain, Vignette, Fill Color, **SkSL Filter** (shader Skia custom). — vs PH2D: TEMOS fx.drop_shadow, fx.rgb_split, motion.slit_scan, motion.spherize; o resto FALTA (pipeline raster pós-shape).
- **Shaders (12)**: Color, Gradient, Multi-Point Gradient, Noise, Checkerboard, Voronoi, Image, Blend, Shape to Shader, SkSL, SLA. — vs PH2D: motion.voronoi/color_ramp cobrem parte; shaders como preenchimento procedural FALTA.

### A.5 Distributions (21 tipos)
**Grid, Circle, Linear, Random, Path, Point, Fibonacci, Rose, Array (posições explícitas), Custom (JS), Math (expressão), Mask (remove pontos fora de shape), Intersections (interseções de paths), Particle (posições de partículas!), Shape Edges, Shape Points, Sub-Mesh, Transform, Shuffle (embaralha IDs), Sort (reordena IDs), Voxelize**. Consumidores: Duplicator, Connect Shape, Distribution Emitter, Apply Distribution. — vs PH2D: TEMOS grid/radial/curve/poisson/fibonacci/scatter; FALTAM mask, intersections, rose, shuffle/sort-como-distribution, shape points/edges, particle-as-distribution, voxelize.

---

## B) PARÂMETROS — top nós

### Duplicator
| Attr | Tipo/Default | Nota |
|---|---|---|
| Input Shapes | lista | transforms do input no nível-pai são **ignorados** (filhos respeitados) |
| Distribution | Distribution (default Grid) | qualquer dos 21 tipos |
| Shape Position/Rotation/Scale | por-cópia | alvos clássicos de Falloff/Stagger/Noise |
| Shape Visibility / Shape Opacity | por-cópia | |
| Auto Id (bool) / Shape Id (int) | cicla entre inputs ou fixa qual clonar | |
| Shape Time Offset | número (frames) | **retima a animação de cada cópia** — par canônico com Stagger |
| Use Index Context / Index Context (out) | bool/out | índice em profundidade p/ duplicators aninhados |
| Skip Invisible Duplicates | bool | remove mesh de cópias invisíveis |

### Falloff
| Attr | Valores |
|---|---|
| Strength | multiplicador do resultado do Graph |
| Shape Type | **Circle · Rectangle · Linear · Sweep (radar, com Repetitions + Angles) · Shape** (usa Input Shapes) |
| Size | p/ Circle/Rectangle/Linear |
| Path Mode (Shape) | Filled Path (força cheia dentro) · Path Edges (decai da borda, com Distance) |
| Falloff Graph | curva custom de decaimento |
| Probability (+Seed) | converte o valor em binário 0/1 com probabilidade % |
| Enabled | desligado ⇒ **constante 1** |
| Custom Color/Color | cor do gizmo no viewport |

### Oscillator
Type: Sine/Cosine/Tangent · Wave Style: Normal/Square/Triangle/Sawtooth/**Custom (Graph)** · Minimum/Maximum · Value Offset · **Stagger** (defasa no tempo por duplicate) · Separate Channels · Time Mode: Seconds/**BPM** · Frequency · Time (auto-conectado; desconectável) · Time Offset (s) · Time Scale · Strength Fade to Zero · deformer: Use Normals, Number of Waves. Sem seed.

### Stagger
Minimum · Maximum · Offset · **Graph** (X=índices, Y=min→max). Saída `stagger.id`. Ex.: 5 cópias, min −100/max 100 ⇒ −100,−50,0,50,100. Receita retiming: `Stagger → Shape Time Offset`, min negativo. Min>Max auto-invertido.

### Noise
Minimum/Maximum/Offset · Frequency · Separate Channels · **Seed + Use Layer as Seed** · Stagger · **Looping + Loop Length (frames)** · Time (desconectar ⇒ estático) · Time Scale · Noise Position/Rotation/Scale · Use Position Context · Use Index Context · Strength Fade to Zero · Use Normals. Tipos: **Cellular** (Jitter, Distance Function Euclidean/Manhattan/Natural, Cellular Type) · **Cubic/Simplex/Value** (Octaves, Lacunarity, Gain, **Curl Noise + Curl Amount**).

### Connect Shape
Mode: **Within Range / Nearest / Point Index** · Search Distance · Connection Limit · Per Point Connection Limit · Skip Points/Skip Offset · Line Type: Linear/Auto Bézier · Auto Bézier Mode: Smooth/Shoulder Target/Shoulder Source/Projection Vectors · Source Distribution · Target Distribution · Ignore Target Distribution · Time Offset (trim animado por conexão).

### Trails
Input Shapes · Limit Length + Length · Start Frame · Path Type: Béziers/Lines · Time Offset · Use Particle Color · Use Index Context. **Experimental, Pro.**

### Forge Dynamics (Box2D)
Mundo: Start Frame · Gravity · Ground Mode + Ground Friction · Velocity/Position Iterations · Fields · Collision Events · caps globais · Initial Position/Rotation Strength (força de retorno à pose inicial!) · World Scale · Time Step · **Cache Solver → `.sdcache`**. Por corpo: Body Type **Dynamic/Still/Kinematic/Kinematic Hybrid** · Friction · Bounce · Density · **Gravity Scale** · Sensor · Collision Shape Type: Box/Circle/Simple/Complex Polygon/Chain/**Soft Body** (stiffness, damping, pressure, shape matching, particle radius) · Starting Velocity · Damping · **Collision Groups + Collides With (strings)** · Lifespan · Sleep. Colisão→**Collision Events** (Color/Impulse/Sticky/Visibility/Body Settings) e **Constraints**.

### Particle Emitter
Emitter Shape: Point/Circle/Square/**Input Shape/Perimeter** (+Size, Margin, Use Path Normals, Normal Angle) · Use Single Position · Seed · Emitter Type: **Time/Distance** (Rate p/s · Particles per Pixel) · Maximum Particles · Duration · Interval · Probability · Initial Direction/Type (Angle/Inwards/Outwards) · Initial Speed (aceita Behaviours per-particle) · Use Emitter Velocity + Strength · Override Lifespan · Time.

### Sound
File · Volume Adjustment · Pan · Frequency Scale: **Logarithmic/Mel/Bark/Linear** · Frequency Range Hz · Frequency Bands (nº) · Equaliser (toggle por banda) · Band Weighting Graph · Volume Clipping · A-Weighting · Sub Steps · Smoothing Frames · Value Offset · Frame Offset · Play Audio · Outs: Maximum dB, File Length, Active Bands · Use Index Context (bandas→duplicates!) · Strength.

### Text Shape
String (+Generators) · Font (variable fonts com eixos!) · Typeface · Font Size · Style · Char/Word/Line/Paragraph Spacing · Alignment · Text Box + Auto Width/Height + **Shrink to Fit** · Word Breaks · Orphans · Monospacing · **Text Path** (+Loop/Travel/Push) · **Background Shape** (Document/Paragraph/Line/Word/Character; Specific Indices) · Formatting Inputs `{0}` · nome do layer herda a String. **Animação por caractere = Sub-Mesh behaviour + Apply* utilities.**

### Resumo demais
- **Random**: Min/Max, Seed, Use Layer as Seed, por índice — estável.
- **Spring**: Strength/Damping — follow-through de qualquer atributo.
- **Number Range**: in/out min/max com clamp.
- **Stroke/Fill**: Width, Cap, Join, Dash, **Trim Start/End/Travel** (o "escreve-on" nativo — Lottie exporta trim sem bake).

---

## C) ARQUITETURA DE UI

**O modelo mental:** Cavalry NÃO é canvas-de-nós primário. Primário = **Scene Window (árvore + timeline) + Attribute Editor**; grafo de nós = **vista secundária editável** (Dependency Graph). Tudo é o mesmo grafo por baixo.

### Scene Window
4 partes: **Timeline** · **Scene Tree** · **Time Editor** (retimar layers — barras de clip) · **Graph Editor** (curvas). Composições = tabs. Atributos animados aparecem como sub-linhas do layer — keyframe nasce no Attribute Editor; Scene Tree mostra só o que já é animado. Time Markers.

### Attribute Editor
- Control Rows por layer; múltiplos UIs; **Search Bar** filtra atributos e edita em massa cross-layers; **Live Mode** vs **pin**; Dependency Graph button no header (preview não-editável do subgrafo).
- **Connection Icons**: entrada dirigida = ícone amarelo à esquerda; saída dirigindo = roxo à direita. Clique = popover com conexões; duplo-clique revela o conectado; ✕ desconecta.
- Right-click: reset, deletar animação, **Add to Control Centre**, filtros de contexto.
- Widgets ricos: **Graph attribute** (mini curve-editor na row!), **Gradient**. User Presets por layer.

### O gesto de conexão
- **No Attribute Editor:** hover no atributo → **Connection Anchor** azul → arrasta até outro atributo (azul se compatível) → solta.
- **Na Scene Tree:** arrasta anchor até outro **layer** → **popup pergunta qual atributo-alvo**. Alt = vários layers. Cmd-drag = substituir.
- Regras: 1 input por atributo, N outputs; **curva de keyframes conta como input** (conectar substitui a animação); incompatíveis esmaecidos.

### Dependency Graph
Vista esquemática interativa — **nível layer E atributo**: bloco com header (ícone+cor) e rows de atributos com **porta in esq / out dir**; "Nodes are colour coded based on type". **Totalmente editável**: clique sequencial OU drag; reconectar substitui. Scroll-zoom, **F=fit**, busca, snap, mini-map, 3 estilos de fio (Bézier/reta/ortogonal). Arrasta layers da Scene Tree pra dentro; duplo-clique carrega no AE. Lacunas: self-connections, comp settings não aparecem. ⇒ **árvore = autoria primária; grafo = espelho completo editável sob demanda.**

### Control Centre
Promover atributos importantes → acesso rápido; right-click → **Add to Control Centre**; row ganha ponto azul; tab por composição; uso: rigging/compartilhar cenas.

### Animação
- **Graph Editor**: Linear/Bézier/**Step**; handles com Angle/Weight Locking, break/join; **X colapsa handle**; transform tool p/ escalar seleções de keys.
- **Magic Easing**: ease de um clique sem abrir o Graph Editor (2.0+).
- **Time Editor**: barras/clipes por layer p/ retimar, trim, loop.

### Reuso
- **Composition** = pre-comp com tabs; **Pre-comp Overrides** por instância.
- **Component Shape**: empacota rig num layer com UI própria.
- **Assets Window**: media, comps, **Google Sheets asset**, **Smart Folders**, Referencing.
- **Presets** de atributos por layer; **Shelf** (scripts/botões); Workspaces; Command Search.

### Scripting/dados
JS em 2 planos: **Scripting API** (módulos api/cavalry/ctx/deformer; Script UIs; Render Scripts; **Web APIs** REST) e **JavaScript Layers** (Shape/Deformer/Emitter/Modifier/Utility + expressões).

### Render/Export
Render Manager; formatos PNG/JPEG/SVG seq, MP4, ProRes, GIF, WebM + **Lottie** (exporta nativo sem bake position/rotation/alphas/stroke width/**trim**/path animation; **baka** Deformers, Duplicator; **não exporta** Filters, Noise shaders; knob "Lottie Baking" Still/Animated/Nuclear). **Dynamic Rendering**: re-renderiza em lote dirigido por dados (CSV/Sheets). **Render Tokens**. **Cavalry Player** + **Web Player** (API JS).

---

## D) O ARTISTA

- **Mograph clássico em ~5 gestos:** 1) Rectangle; 2) **Duplicator** + arrasta o shape pra dentro (input transforms ignorados de propósito — o clone "encaixa" sem pulo); 3) Distribution=Grid, Count no viewport; 4) **Falloff**, arrasta a âncora → `Shape Scale`/`Shape Position` (gizmo do falloff arrastável na tela, cor própria); 5) **Stagger → Shape Time Offset** (min negativo) p/ delay em cascata. Tudo procedural, scrub sempre-vivo.
- **Defaults inteligentes:** Falloff desligado devolve 1 (nunca "mata" a cena); Stagger auto-inverte min/max; Duplicator ignora transform do input; Noise com Use Layer as Seed evita gêmeos; Oscillator com BPM; Probability no Falloff = máscara binária com um clique.
- **Retiming como material:** `Shape Time Offset` + contexts fazem "delay por elemento" ser um número conectável.
- **Contexts** são a alma: behaviour "pergunta" índice/posição/velocidade do elemento que o avalia — mesmo desenho do nosso EvalCtx/instance_field.
- **Presets & onboarding:** Quick Start + Example Files oficiais; User Presets por atributo/layer; Shelf; Command Search. Free vs **Professional** (partículas, Trails, Forge avançado, Lottie, Dynamic Rendering, Web APIs = Pro).
- **Rigging Duik-like:** sem esqueleto IK completo; **Rubber Hose Limb**, **Constraints**, **Manipulator** (alças no canvas), **Rig Control**, **Component + Control Centre** — rig = atributos promovidos, não bones.
- **Dados:** CSV/Sheets → Spreadsheet/Lookup → strings e valores; Dynamic Rendering fecha o ciclo (n variações de uma cena).
- **Sinal p/ PH2D:** maiores buracos vs Cavalry: (1) **Connect Shape** + distributions compostas (mask/intersections/shuffle/sort), (2) **Sound behaviour** (áudio→bandas→valores; `ph2d-audio-spectral` já tem FFT), (3) **texto + strings procedurais**, (4) **path-ops procedurais no grafo**, (5) **Squash & Stretch/Motion Stretch/Auto-Animate**, (6) **data-driven** (CSV→colunas), (7) **eventos de colisão→ações**, (8) padrão de UI "árvore primária + grafo-espelho editável + âncoras de conexão com popup".
