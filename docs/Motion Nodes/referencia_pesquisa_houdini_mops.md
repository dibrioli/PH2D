# PESQUISA: SideFX Houdini + MOPs → referência para Motion Nodes PH2D

Legenda de fonte: parâmetros com valores exatos vêm das páginas oficiais fetchadas nesta sessão; itens marcados `~` são de conhecimento consolidado do produto (verificáveis na URL citada — docs POPs vivem em `/nodes/dop/pop*.html`, não `/nodes/pop/`).

---

## A) CATÁLOGO

### A.1 POPs — partículas/forças (contexto DOP)
Fonte: https://www.sidefx.com/docs/houdini/nodes/dop/popforce.html · https://www.sidefx.com/docs/houdini/nodes/dop/popattract.html · https://www.sidefx.com/docs/houdini/nodes/dop/popaxisforce.html (+ demais em `/nodes/dop/pop<nome>.html`)

Modelo Houdini: cada POP é um micro-solver que escreve atributos (`force`, `targetv`, `airresist`, `v`, `w`/spin) que o solver integra — idêntico ao nosso "Pure acumula em `accel`, UM integrador aplica".

| nó | categoria | função (1 linha) | status vs PH2D |
|---|---|---|---|
| POP Force | força | força direcional constante + campo de noise embutido (aba Noise) acumulada em `force` | PARCIAL — `force.wind` cobre a constante; falta o **noise embutido em toda força** |
| POP Wind | força | vento como **drag rumo a velocidade-alvo** (`airresist·(targetv−v)`), não força bruta; + aba Noise | PARCIAL — nosso `force.wind`; conferir se é modelo alvo-velocidade (o do Houdini não explode) |
| POP Drag | força | arrasto proporcional a −v, escala global + per-eixo, opcional goal velocity | TEMOS≈ `force.drag` |
| POP Drag Spin | força | arrasto ANGULAR (amortece `w`) | FALTA (não temos velocidade angular física) |
| POP Attract | força | atração a ponto/partículas/pontos-de-geo/superfície; modos Accelerating/Follow/Predict Intercept; janela min/max/reversal | PARCIAL — `force.attractor` cobre Accelerating; faltam Follow/Predict, reversal distance, clustering |
| POP Curl Noise | força | campo divergence-free com cluster de noise + **desvio de colisores** (respeita SDF de objetos) | PARCIAL — `force.curl` sem collision-avoidance |
| POP Axis Force | força | orbit + lift + suction em torno de um eixo (linha ou círculo), com falloff interno soft-edge; modo "treat as wind" | PARCIAL — `force.vortex`+`motion.orbit` separados; o Axis Force unifica os 3 speeds num falloff só |
| POP Curve Force | força | segue/suga para uma CURVA (força tangente + espiral + atração), o "rio de partículas" | **FALTA** — alto valor 2D |
| POP Spin | força | injeta rotação própria (spin sobre eixo, deg/s) na partícula~ | FALTA (rotação como estado integrado) |
| POP Torque | força | torque acumulado no análogo angular de `force`~ | FALTA |
| POP Lookat | força | orienta partícula para alvo/velocidade~ | TEMOS `motion.look_at` |
| POP Follow | força | perseguir/fugir de um líder (outra partícula) com antecipação~ | FALTA |
| POP Interact | força | atração/repulsão inter-partícula por raio (usa `pscale`)~ | PARCIAL — só dentro de `motion.boids` |
| POP Steer Seek / Separate / Align / Cohesion / Wander / Avoid Obstacle / Custom (+ solver de flock) | força | **flocking decomposto**: cada regra é um nó com peso; solver soma ponderado e clampa aceleração~ | PARCIAL — `motion.boids` monolítico; o padrão Houdini é 1 regra = 1 nó empilhável |
| POP Speed Limit | força/estado | clamp min/max de \|v\| (e de spin)~ — estabilizador universal | **FALTA** (barato, alto valor) |
| POP Kill | estado | mata partículas por bbox/regra/expressão~ | TEMOS≈ `motion.cull` + `sim.lifetime` |
| POP Group | estado | cria/mantém grupos por regra (bbox, expressão, aleatório, idade)~ | FALTA como conceito (nosso análogo é falloff/mask) |
| POP Color | aparência | cor por ramp sobre idade/velocidade/atributo~ | TEMOS `motion.color_ramp`/`tint` |
| POP Sprite | aparência | sprite/textura + tamanho por partícula~ | TEMOS (nativo: instância É sprite) |
| POP Property | estado | seta propriedades físicas em massa (mass, pscale, bounce, airresist)~ | PARCIAL — `value.instance_field`/`motion.drive` |
| POP Source / Location | emissor | emissão (birth rate, vida ± variância, velocidade inicial herdada)~ | TEMOS `motion.emitter` (nosso é stateless — superior p/ scrub) |
| POP Replicate | emissor | emite partículas A PARTIR de partículas (fogos, trilhas que emitem)~ | **FALTA** |
| POP Advect by Volumes | força | advecção por campo de velocidade externo~ | FALTA (campo genérico como asset; MOPs Velocity Field idem) |
| POP Collision Detect / Behavior | colisão | detecta colisão, marca grupo, define die/bounce/stick/slide~ | PARCIAL — `sim.collide`/`motion.collide` sem os 4 comportamentos nomeados |
| POP Proximity | estado | escreve nº de vizinhos/dist. mais próximo como atributo~ | FALTA (MOPs Neighbors idem) |
| POP Wrangle | escape | VEX por partícula | TEMOS≈ `motion.expression` (VEX-lite) |
| POP Instance | render | instancia geometria por partícula | TEMOS `motion.clone` |

### A.2 SOPs — deformers e scattering com análogo 2D
Fonte: https://www.sidefx.com/docs/houdini/nodes/sop/bend.html · https://www.sidefx.com/docs/houdini/nodes/sop/lattice.html · https://www.sidefx.com/docs/houdini/nodes/sop/pathdeform.html · https://www.sidefx.com/docs/houdini/nodes/sop/attribnoise.html · https://www.sidefx.com/docs/houdini/nodes/sop/copytopoints.html (demais: `/nodes/sop/<nome>.html`: scatter, softxform, edit, smooth, mountain, ripple, falloff, attribrandomize, sort, trail, blendshapes)

| nó | categoria | função | status |
|---|---|---|---|
| Bend (unifica bend/twist/taper/length) | deformer | 4 deformações numa **capture region** (origem+direção+comprimento) com limite e simetria | PARCIAL — temos `motion.bend`/`motion.twist` separados; falta capture region explícita + taper + squash&stretch + ramp de perfil |
| Lattice | deformer | grade de controle (modo lattice) OU nuvem de pontos com kernel/raio (modo points) | TEMOS `motion.lattice` (modo lattice); modo points ≈ FALTA (é o "point deform" barato) |
| Path Deform | deformer | mapeia geometria ao longo de curva com offset/stretch, ramps de scale e twist, rigidez por trecho | PARCIAL — `motion.spline_wrap`; faltam ramps ao longo, end behaviors (Extend/Clamp/Clip), rigidity |
| Point Deform | deformer | deforma pela deformação de uma nuvem de pontos de referência (rest→deformed) | FALTA (nosso `rig.skin_deformer` é o parente) |
| Soft Transform / Edit (soft) | deformer | translate/rotate/scale com **soft radius**: métrica (Surface/Edge/Global) + rolloff + visualizar falloff~ | PARCIAL — `motion.move`+`motion.falloff` compõem isso; Houdini embute o cluster soft em TODO gesto de edição |
| Smooth / Relax | deformer | suaviza posições/atributos (iterações + força) | **FALTA** |
| Mountain (= Attrib Noise em P) | deformer | displace por noise com o cluster padrão completo | TEMOS `motion.noise` (raso vs cluster completo — ver B) |
| Ripple | deformer | ondas concêntricas decaindo de um centro | TEMOS≈ `motion.wave` |
| Falloff SOP | utilitário | grava atributo de falloff por proximidade a ponto/curva/superfície | TEMOS `motion.falloff` |
| Scatter | scatter | pontos por densidade/contagem sobre superfície, com relax (blue-noise) e seed | TEMOS `motion.scatter`/`distribute_poisson` |
| Copy to Points | instanciar | copia origem para pontos-alvo honrando a **convenção pscale/orient**; Piece Attribute = variantes | TEMOS `motion.clone`+distribuidores; Piece/variants PARCIAL |
| Copy & Transform | instanciar | N cópias com transform ACUMULATIVO por cópia (o array linear/radial do MoGraph)~ | PARCIAL — `motion.grid`/`distribute_radial` cobrem casos; falta o acumulativo genérico |
| For-Each / copy stamping | meta | variação por-cópia via loop/atributo (stamping foi aposentado em favor de atributos+Piece) | PARCIAL — `value.instance_field` é o caminho moderno (certo) |
| Attrib Randomize | utilitário | valores aleatórios com distribuições nomeadas (uniform/gaussian/direção-esfera) por atributo~ | PARCIAL — cobre-se com expression; falta nó dedicado |
| Attrib Adjust/Remap | utilitário | remapeia atributo com ramp | TEMOS `value.map_range` (+ramp? ver B) |
| Sort | utilitário | reordena pontos (eixo, proximidade, random, expressão) | TEMOS `motion.sort` |
| Trail | utilitário | rastro/eco + **computa `v` por diferença** entre frames | TEMOS `motion.trail`; o "compute velocity" como serviço FALTA |
| Blend Shapes | utilitário | interpola N formas com pesos | TEMOS `motion.morph`/`mixer` |

### A.3 CHOPs — canais de animação processados por nós
Fonte: https://www.sidefx.com/docs/houdini/nodes/chop/lag.html · https://www.sidefx.com/docs/houdini/nodes/chop/spring.html (demais: `/nodes/chop/<nome>.html`)

Conceito: canais (curvas tempo→valor) fluem por nós; qualquer parâmetro pode ser dirigido por um canal (export/channel reference). É o nosso domínio `value.*`/`pulse.*` com identidade própria.

| nó | função | status |
|---|---|---|
| Lag | lag + overshoot em canal; clamp de slope e de aceleração (limitador de velocidade de MUDANÇA) | **FALTA** — irmão barato do spring, essencial p/ follow-through |
| Spring | massa-mola sobre canal (k, massa, damping; entrada = posição OU força) | TEMOS `motion.spring` |
| Filter | suavização por janela (gauss/box/ramp) — o "smooth" temporal | **FALTA** |
| Jiggle | spring 3D posicional~ | TEMOS≈ `motion.spring` |
| Math | aritmética/combinação de canais | TEMOS `value.math` |
| Wave | gerador (sin/tri/square/ramp/pulse) com freq/fase/offset | TEMOS `value.lfo`/`motion.oscillator` |
| Noise | noise 1D temporal por canal (cluster padrão: amp/rough/octaves/seed/pulse) | TEMOS≈ `motion.wiggle` |
| Trigger | ao cruzar limiar dispara envelope **ADSR** (delay/attack/sustain/decay/release)~ | PARCIAL — `pulse.threshold` sem o envelope ADSR |
| Envelope | extrai envelope de amplitude (p/ áudio) | FALTA (pré-requisito de audio-reativo) |
| Logic | combina canais como booleanos (and/or, flip-flop, latch) | PARCIAL — `pulse.compare`/`counter` |
| Lookup | canal A indexa curva B (remap por curva) | TEMOS≈ `value.map_range` (se tiver ramp) |
| Delay | atrasa canal N segundos (eco temporal puro) | PARCIAL — `motion.stagger` é o delay por-instância; falta o por-canal |
| Count | conta cruzamentos | TEMOS `pulse.counter` |
| Constant | valor fixo | TEMOS `debug.const` |
| Audio File/Band EQ | importa áudio, separa bandas | FALTA no grafo (módulo áudio existe fora) |
| Record | grava input ao vivo como canal | TEMOS (Record da timeline, fora do grafo — ok) |
| Fetch/Export | ler/escrever canais⇄parâmetros de qualquer nó | conceito: nosso text-param/`motion.drive` PARCIAL |

### A.4 MOPs — o catálogo completo (SOP-level MoGraph)
Fonte: https://github.com/toadstorm/MOPS/wiki (+ /wiki/Falloffs, /wiki/Modifiers, /wiki/Generators, /wiki/Tools, /wiki/MOPsDOPs) · https://www.motionoperators.com

Espinha do MOPs: TODO efeito é modulado pelo atributo **`f@mops_falloff` ∈ [0,1]** gravado por nós de Falloff; modifiers leem-no como escalar do efeito. Falloffs COMPÕEM entre si (Combine, blend modes de compositor). Instâncias são packed prims com atributos-template (`t`, `r`, `s` extraídos/aplicados por Extract/Apply Attributes).

| nó | categoria | função | status |
|---|---|---|---|
| Plain Falloff | falloff | valor constante em `mops_falloff` (base p/ blend) | TEMOS≈ (const→falloff) |
| Shape Falloff | falloff | falloff por forma primitiva (linear/esfera/caixa) com transform + inner/outer + noise de quebra | PARCIAL — `motion.falloff` (1 forma?); família completa FALTA |
| Noise Falloff | falloff | noise→falloff, animável, loopável, transform handle | PARCIAL (compomos noise+falloff) |
| Texture Falloff | falloff | textura projetada→falloff por luminância | PARCIAL — temos `motion.luminance` |
| Audio Falloff | falloff | bandas de frequência de áudio→falloff, gain por banda, Auto Distribute | **FALTA** |
| Spline Falloff | falloff | distância a curva OU posição-ao-longo→falloff, com remap | FALTA (temos curva mas não como falloff) |
| Object Falloff | falloff | distância a pontos/geo/volume, "Area of Influence" + remap | PARCIAL |
| Falloff From Attribute | falloff | remapeia atributo existente→falloff (min/max + Auto Range) | TEMOS≈ `value.attribute`+`map_range` |
| Combine Falloffs | falloff | **composita dois falloffs com blend modes de compositor** + strength | **FALTA** — a peça-chave da família |
| Remap Falloff | falloff | min/max + **ramp** sobre falloff existente | PARCIAL |
| Transform Falloff | falloff | handle de transform compartilhado que move outros falloffs em sincronia | FALTA |
| Spread Falloff | falloff | crescimento animável a partir de sementes (Spread + Falloff Width; métrica por conectividade/raio) | **FALTA** — "infection" é assinatura MoGraph |
| Transform Modifier | modifier | T/R/S em espaço local/mundo escalado por falloff | TEMOS `motion.transform`/`move`/`rotate`/`scale` |
| Randomize | modifier | aleatoriza T/R/S e propriedades, per-eixo, seed | PARCIAL |
| Noise Modifier | modifier | noise em posição E orientação; modos Simple/Advect; Aim Weight | TEMOS≈ `motion.noise`+`wiggle` |
| Delay | modifier | **desloca a animação existente no tempo por instância, proporcional ao falloff** | PARCIAL — `motion.stagger`+`time_remap`; o "delay dirigido por falloff espacial" FALTA |
| Spring Modifier | modifier | Hooke (mass/k/damping) sobre transforms; falloff = escalar | TEMOS `motion.spring` |
| Aim / Look At | modifier | orienta para alvo com up vector | TEMOS `motion.look_at` |
| Align | modifier | move pivô relativo ao bbox da instância | FALTA (pivô por instância) |
| Move Along Spline | modifier | empurra instâncias ao longo de curvas (U animável) | PARCIAL — `distribute_curve`+`drive` |
| Color Modifier | modifier | ramps de cor por falloff | TEMOS `motion.color_ramp` |
| Curl Modifier | modifier | curl noise em pos+orient (SOP-level, sem sim) | TEMOS `force.curl` |
| Relax Modifier | modifier | separa instâncias sobrepostas (de-overlap iterativo) | **FALTA** |
| Clip by Attribute | modifier | corta geometria suavemente num limiar de atributo | FALTA |
| Stepper (Plus) | modifier | quantiza transforms/atributos em degraus | TEMOS≈ `motion.step` |
| Filter (Plus) | modifier | suavização temporal de jitter | FALTA (=Filter CHOP) |
| Magnetize (Plus) | modifier | push/pull por pontos de influência | PARCIAL — `force.attractor` |
| Pre/Post-Roll (Plus) | modifier | gera automaticamente entrada/saída da animação | FALTA |
| Instancer | generator | cria/arranja cópias (grid/radial/scatter embutidos) | TEMOS `motion.clone`+`grid`+`distribute_*` |
| Explode | generator | quebra faces em instâncias (shatter+mover por normal) | FALTA (com `motion.voronoi` perto) |
| Trails | generator | trilhas polyline estáveis | TEMOS `motion.trail` |
| Sweep Spline | generator | curva→fita/geometria | PARCIAL (trail/ribbon) |
| Velocity Field (Plus) | generator | falloff/atributo→campo de velocidade p/ advecção | FALTA |
| Apply/Extract Attributes | tool | a API central: intrínsecos⇄atributos-template `t/r/s` | conceito TEMOS (colunas SÃO os atributos) |
| Index From Attribute | tool | atributo→índice de variante da instância | PARCIAL — `value.instance_field` |
| Neighbors | tool | vizinhos como array-atributo | FALTA |
| Noise Patterns | tool | biblioteca de noises escalar/vetor (compartilhada pelos nós de noise) | conceito: **um cluster único reutilizado** — ver B |
| Preview Falloff | tool | **visualiza falloff como sprites coloridos no viewport** | FALTA — UX crítico |
| Reorient | tool | troca eixos locais sem girar a instância | FALTA |
| Visualize Frame | tool | desenha eixos up/normal/side por instância | FALTA (debug visual) |
| Waveforms (Plus) | tool | gera/modifica atributos por formas de onda periódicas | TEMOS `value.lfo` |
| Sequencer (Plus) | tool | atributos rítmicos por interface de tracker | PARCIAL — `pulse.beat` |
| MOPs DOPs (Aim/Noise/Spline/Falloffs em sim) | dop | os mesmos modifiers dentro da simulação | análogo: nossos nós já operam no laço da sim (`sim.zone`) — TEMOS o desenho |

### A.5 Convenções transversais (o que padroniza o Houdini)
Fonte: https://www.sidefx.com/docs/houdini/nodes/sop/attribnoise.html · https://www.sidefx.com/docs/houdini/copy/instanceattrs.html · https://www.sidefx.com/docs/houdini/network/parms.html

- **Campo `Group` no topo de TODO SOP/POP** (string; vazio = tudo): padrão de escopo universal com dropdown de grupos existentes e seleção interativa por viewport. Nosso análogo natural: mask/falloff de entrada em todo nó.
- **Cluster de noise padrão** (repetido em POP Force/Wind/Curl, Attrib Noise, Noise CHOP, MOPs Noise Patterns): `noise type` (menu ~15 tipos: Perlin, Simplex, Worley F1/F2−F1 nas 3 métricas, Alligator, Sparse Convolution, Flow, Clouds) · `element size`/frequency (+scale vec) · `offset` (vec) · `animate/pulse duration` · `fractal: octaves(1-16, def 4) + lacunarity(2.0) + roughness(0.5)` · `amplitude` + modos de range (Positive/Zero-Centered/Min-Max/Middle±) · `seed` · remap ramp opcional + bias/gain.
- **Ramps como TIPO de parâmetro** (float ramp e color ramp): editor inline no painel de params, pontos com interpolação por-ponto (constant/linear/catmull-rom/bezier/bspline), usados para perfil de taper, cor por idade, remap de falloff. É o parâmetro que nosso canal de TEXT PARAM (`motion.expression`) pode transportar serializado.
- **Convenção de instância**: `pscale` (uniforme), `scale` (vec), `orient` (quat, vence), senão `N`+`up`, `rot` (quat extra), `trans`, `pivot` — prioridade documentada e honrada por Copy to Points, Path Deform, instancers, MOPs. Recomendação direta para as colunas do PH2D.
- **`seed` em todo nó estocástico** + estabilidade por `id` (não por índice) — igual ao nosso `hash(seed,id,lane)`.
- **Espaço local/world explícito** (menu) em deformers/modifiers.
- **Falloff como ATRIBUTO componível** (MOPs `mops_falloff`, soft radius dos SOPs): efeito = lerp(identidade, efeito_cheio, falloff).

---

## B) PARÂMETROS (~27 nós; clusters padrão destacados)

Fonte por bloco: URLs citadas em A; tabelas de POP Force/Attract/Axis, Bend, Lattice, Path Deform, Attrib Noise, Copy to Points, Lag, Spring extraídas das páginas oficiais nesta sessão.

### POP Force (`dop/popforce.html`) — fetchado
| param | tipo | default | função |
|---|---|---|---|
| Activation | float | 1 | liga/desliga (aceita expressão) — **padrão de todo POP** |
| Group | str | "" | escopo — padrão de todo POP |
| Force | vec3 | 0,0,0 | força total |
| Ignore Mass | toggle | off | multiplica pela massa p/ anular divisão do solver |
| **Noise (cluster)**: Amplitude | float | 0 | 0 = noise desligado por default |
| Swirl Size | float | 1 | tamanho do padrão |
| Swirl Scale | vec3 | 1,1,1 | anisotropia |
| Offset | vec3 | 0,0,0 | deslocamento do campo |
| Pulse Length | float | 1 | período da frequência mais baixa (animação temporal) |
| Attenuation | float | 1 | expoente pós-noise |
| Roughness | float | 0.5 | ganho por oitava |
| Turbulence | int | 1 | nº de oitavas |
| Seed | int | 0 | semente |

### POP Wind (`dop/popwind.html`) ~
| param | tipo | default | função |
|---|---|---|---|
| Wind | vec3 | 0,0,0 | velocidade-ALVO do ar |
| Air Resistance | float | 1 | rigidez do arrasto rumo ao alvo (força = airresist·(targetv−v)) |
| Ignore Mass | toggle | off | idem |
| Noise (cluster) | — | amp 0 | mesmo cluster do POP Force sobre a velocidade-alvo |

### POP Drag (`dop/popdrag.html`) ~
| param | tipo | default | função |
|---|---|---|---|
| Scale | float | 1 | coeficiente global de arrasto |
| Directional Scale | vec3 | 1,1,1 | arrasto anisotrópico |
| Goal Velocity (Use Wind) | vec3 | 0,0,0 | arrasta rumo a esta velocidade em vez de zero |
| Ignore Mass | toggle | off | idem |

### POP Curl Noise (`dop/popcurlnoise.html`) ~
| param | tipo | default | função |
|---|---|---|---|
| Amplitude | float | 1 | magnitude do campo |
| Swirl Size | float | ~0.5–1 | escala dos vórtices |
| Grain | float | ~0.25 | detalhe fino relativo |
| Attenuation / Roughness / Turbulence / Pulse / Offset | cluster | 1 / 0.5 / 2 / 1 / 0 | o MESMO cluster de noise |
| Add Collision Objects | toggle+lista | off | o campo CONTORNA SDFs de colisores (curl respeita paredes) |
| Collision Distance/Scale | float | — | alcance da deflexão |

### POP Attract (`dop/popattract.html`) — fetchado
| param | tipo | default | função |
|---|---|---|---|
| Attraction Type | menu | Position | Position / Particles / Points / Surface Points |
| Goal | vec3 | 0,0,0 | alvo no modo Position |
| Match Method | menu | Average Position | ou Point per Particle (1-para-1 por id) |
| Number of Clusters | int | 1 | k-means dos alvos |
| Force Method | menu | Accelerating | **Accelerating / Follow / Predict Intercept** |
| Force Scale | float | 1 | ganho |
| Reversal Distance | float | 0 | dentro disso a força INVERTE (repele) — evita orbitar o alvo |
| Peak Force Distance | float | 0 | platô de força |
| Min/Max Distance | float | 0 / 1e30 | janela de atuação |
| Ambient Speed / Speed Scale | float | 0 / 1 | modos follow: velocidade de cruzeiro |
| Ignore Mass | toggle | off | idem |

### POP Axis Force (`dop/popaxisforce.html`) — fetchado
| param | tipo | default | função |
|---|---|---|---|
| Type | menu | Sphere | Sphere (eixo-linha) / Torus (eixo-círculo) |
| Center | vec3 | 0,0,0 | centro |
| Axis Direction | vec3 | 0,0,1 | eixo |
| Radius | float | 1 | raio de falloff |
| Height | float | 2 | extensão axial |
| **Orbit Speed** | float | 1 | tangencial (vórtice) |
| **Lift Speed** | float | 0 | ao longo do eixo (fonte/chaminé) |
| **Suction Speed** | float | 0 | radial (sucção) |
| Soft Edge | float 0–1 | 0.25 | onde começa a rampa do falloff interno |
| Inner/Outer Strength | float | 1 / 0 | força dentro/na borda |
| Treat as Wind + Air Resistance | toggle+float | off / 1 | modo alvo-velocidade (estável) vs força bruta |

### POP Speed Limit (`dop/popspeedlimit.html`) ~
Min Speed (0) · Max Speed (~10) · toggles por-limite · limites de spin angular. Clamp pós-integração.

### POP Interact (`dop/popinteract.html`) ~
Magnitude (+atrai/−repele) · raio por `pscale` ou explícito · falloff exponencial · O(n²)/grade.

### POP Steer* (família flock) ~
Cada nó: `Weight` (1.0) + params da regra (Seek: goal; Separation/Cohesion/Align: raio+ângulo de visão; Wander: freq/força; Obstacle: lookahead). Solver clampa aceleração máx. Decomposição = mixável por pesos animáveis.

### Bend SOP (`sop/bend.html`) — fetchado
| param | tipo | default | função |
|---|---|---|---|
| Limit Deformation to Capture Region | toggle | on | só dentro da cápsula |
| Deform in Both Directions | toggle | off | simétrico pelo origin |
| Bend | float deg | 0 | dobra (>180° ok); modo Angle ou Goal Direction |
| Twist | float deg | 0 | torção no eixo de captura |
| Length Scale + Preserve Volume | float+toggle | 1 / off | esticar com squash&stretch |
| Taper + Squish + Squish Pivot | float ×2 + 0–1 | 1 / 1 / 0.5 | afunilar ponta e meio |
| Profile Ramp | ramp | — | perfil arbitrário de taper ao longo |
| **Capture (cluster)**: Origin / Direction / Length | vec3+vec3+float | — | a cápsula; botão "Set Capture Region" interativo |
| Up Vector (+Angle) | menu+vec3+deg | — | orientação do plano de dobra |

### Lattice SOP (`sop/lattice.html`) — fetchado
Divisions (vec3i, casa com a grade) · Order 2–11 (suavidade) · Interpolation Linear/Bezier/NURBS · Falloff além da borda · **modo Points**: Kernel (Wyvill/Elendt/Blinn/Links/RenderMan/Hart) + Radius + Normalize Threshold + visualizar raios.

### Path Deform SOP (`sop/pathdeform.html`) — fetchado (resumo do relevante 2D)
Map Length Using (fração da curva/da geo/distância) · Position Offset (U animável = mover ao longo) · Start/End Behavior (**Extend/Clamp/Clip**) · Uniform Scale + Preserve Volume · **Scale Ramp** ao longo · **Rotation/Twist base + Ramp** ao longo · Rigidity (grupo rígido + soften radius + falloff ramp) · Forward/Up axis · honra `N/up/orient/pscale...` da curva · outputs opcionais: atributo U + índice de curva.

### Soft Transform / Edit (`sop/softxform.html`, `sop/edit.html`) ~ — o cluster "soft"
| param | tipo | default | função |
|---|---|---|---|
| Soft Radius | float | 1 | alcance |
| Distance Metric | menu | Surface | **Surface (geodésica) / Edge / Global (reta)** |
| Rolloff / Soft Type | menu | Smooth | linear/quadrático/cúbico/smooth (+ tangentes ajustáveis) |
| Visualize Falloff | toggle | off | pinta o peso no viewport |
| T/R/S + Pivot | vec3×3 | — | o gesto |

### Attrib Noise (`sop/attribnoise.html`) — fetchado — **o cluster de noise canônico completo** (ver A.5; destaques):
Noise Type (15 opções) · Element Size 1 + Scale vec · Offset · Animate + Pulse Duration 1 · Fractal None/Standard/Terrain/Hybrid · Octaves 4 (1–16) · Lacunarity 2 · Roughness 0.5 · Amplitude 1 + 6 modos de range · Bias/Gain 0.5/0.5 · Remap Ramp · Operation (Set/Add/Sub/Mul/Min/Max) · Blend 0–1 + Blend Attribute · Warp (lattice/gradient) · clamps · VEXpression de override.

### Scatter (`sop/scatter.html`) ~
Generate By (density/count) · **Force Total Count 1000** · Density Attribute + Scale · Relax Iterations (~10, blue-noise) · Seed · outputs prim/uv de origem.

### Copy to Points (`sop/copytopoints.html`) — fetchado
Source/Target groups · **Piece Attribute** (variantes por atributo int/str casando origem↔alvo) · Pack & Instance + Pivot (Centroid) + Display As · Transform Using Target Point Orientations (honra a pilha `orient→N/up→pscale→scale→rot→trans→pivot`) · multiparm de transferência de atributos (Copy/Mult/Add/Sub por padrão de nomes).

### Lag CHOP (`chop/lag.html`) — fetchado
Method (Value/Amplitude/Magnitude) · **Lag (par: subida, descida)** = tempo p/ alcançar 90% · **Overshoot (par)** · Clamp Slope + Max Slope (par) · Clamp Acceleration + Max Accel (par) · comuns: Scope `*`, Units (s/frames/samples), Time Slice.

### Spring CHOP (`chop/spring.html`) — fetchado
Spring Constant (freq↑) · Mass (freq↓ amp↑) · Damping · **Input Effect: Position | Force** · condições iniciais do canal ou manuais.

### Trigger CHOP (`chop/trigger.html`) ~
Threshold 0.5 (borda de subida) · retrigger delay · envelope **Delay→Attack(len+shape)→Peak→Decay→Sustain(level)→Release(len+shape)**.

### MOPs Shape Falloff (wiki/Falloffs) — fetchado (nível de precisão do wiki)
Shape (gradiente linear/esfera/cubo…) · transform próprio ou por Transform Falloff · inner/outer (zona cheia→zero) · Remap ramp · aba Noise de quebra · escreve `f@mops_falloff`.

### MOPs Noise Falloff — fetchado
Noise (via MOPs Noise Patterns: múltiplas funções escalar/vetor) · frequência/offset/velocidade · loopável · rest attribute opcional · remap.

### MOPs Combine Falloffs — fetchado
Input A ∘ B com **blend modes de compositor** (add/sub/mult/screen/max/min/average~) + Blend Strength — falloffs empilham como camadas de Photoshop.

### MOPs Spread Falloff — fetchado
Spread (ANIMÁVEL — frente de crescimento) · Falloff Width (fade da frente) · métrica: conectividade ou raio · pontos-semente por grupo.

### MOPs Transform Modifier (wiki/Modifiers) — fetchado+~
T/R/S (+Uniform Scale) · espaço **World/Local (eixos da instância)** · ordem de transform · efeito ×`mops_falloff` (com invert) · Group.

### MOPs Randomize ~
por canal (T/R/S/pscale/cor): min/max ou centro±, per-eixo, **Seed**, id-estável; distribuição uniforme (gauss em versões novas).

### MOPs Delay — fetchado+~
desloca a ANIMAÇÃO da instância no tempo: Delay (frames) × `mops_falloff` (falloff espacial vira offset temporal — a "onda de dominó"); requer histórico (time-shift interno).

### MOPs Spring Modifier — fetchado
Mass · Spring Constant k · Damping · alvo = transform de entrada; falloff escala o efeito; versão rotacional incluída.

---

## C) UI/UX
Fonte: https://www.sidefx.com/docs/houdini/network/index.html · /network/flags.html · /network/badges.html · /network/organize.html · /network/parms.html · /basics/ladder.html · /model/geometry_spreadsheet.html · /shelf/ e /assets/ (HDAs) · /basics/radialmenus.html

**O nó no canvas.** Corpo retangular arredondado uniforme; a IDENTIDADE visual vem de: ícone do optype no centro, cor de fundo (paleta atribuível pelo usuário, tecla C), forma opcional custom (H17+: chevron, círculo, etc. — usada por convenção, ex. nós de saída). **Flags são FAIXAS clicáveis nas bordas do nó** (SOPs): esquerda = Bypass (amarelo, atalho Q) e Template (rosa — "ghost" da geometria como referência); direita = Display/Render (azul/roxo, atalho R; decide O QUE o viewport mostra — o cook segue o display flag, nosso análogo do `motion.output`); Lock (trava cache). Em DOPs/CHOPs o conjunto muda (CHOP tem Export flag laranja = escreve nos parâmetros). **Badges/anéis**: erro = anel/círculo VERMELHO com X; warning = triângulo amarelo; comentário = balão (texto opcional exibido sob o nó); relógio = time-dependent; cadeado = HDA travado. Subnet com erro dentro ganha contorno vermelho — o erro BORBULHA. Fios: cinza; grossura/estilo diferem por contexto; **dots** reroteiam; wires podem ser escondidos por nó.

**No nó vs no painel.** No nó: nome (editável, duplo-clique), ícone, flags, badges, inputs/outputs nomeados no hover (tooltip com tipo), anel de seleção, preview thumbnail opcional (COPs/imagens). TODO o resto (parâmetros) vive no **Parameter Pane** — sem knobs no canvas. Painel: abas/folders por grupo (Force | Noise | Bindings…), ramps inline, campos com **nome verde = keyframe, cor de canal = expressão** (clicar no label alterna valor⇄expressão), sliders com range "soft" (digitável além), menus, campos-grupo com seta de seleção interativa no viewport.

**Criação/busca: TAB menu.** Tecla TAB no network → caixa de busca fuzzy + árvore por categorias (contexto-sensível: só nós válidos ali); digitar filtra por nome E por sinônimos; Enter → o nó nasce colado ao cursor e **auto-conecta** se um nó estava selecionado; arrastar um fio e soltar no vazio também abre o TAB menu já filtrado por compatibilidade. Recentes no topo. Shelf tools criam redes prontas (presets de setup).

**Anotação/organização.** Network boxes (caixas coloridas com título, minimizáveis, arrastam o conteúdo junto); sticky notes (texto rico, cor, tamanho); comentários por nó (badge); nomear nós é cultura (o nome é o id de referência `ch("../wind1/amp")`). Layout automático (L), alinhamento, snap. **Quickmarks** (Ctrl+1..5 salva enquadramento+seleção; 1..5 volta — navegação em redes grandes).

**Erros.** Nó vermelho no canvas + MMB no nó abre **info popup**: mensagem de erro completa, tempo de cook, contagem de pontos/prims, lista de atributos com tipos. O erro aponta o parâmetro culpado. Warnings não bloqueiam o cook.

**Presets/galleries.** Menu de engrenagem do painel de params: Save/Load Preset (por optype), **Gallery** = biblioteca navegável de presets nomeados com thumbnail (ex.: dezenas de materiais/setups); Revert to Defaults por-parâmetro (RMB) ou nó inteiro. **HDA (Houdini Digital Asset)**: selecionar subgrafo → Collapse to Subnet → Create Digital Asset → **promover parâmetros** (arrastar params internos para a interface do asset, com ranges/labels/condições de visibilidade `hidewhen`), embutir ícone+help+exemplos; o HDA vira um TIPO no TAB menu, versionável — o nosso alvo para "empacotar subgrafo".

---

## D) ARTISTA
Fonte: https://www.sidefx.com/docs/houdini/basics/ladder.html · /network/parms.html · /model/geometry_spreadsheet.html · /expressions/ e /vex/snippets.html · /nodes/ (exemplos embutidos por nó)

**Defaults.** Filosofia: o nó recém-criado FAZ algo visível e neutro-útil — Scatter nasce com 1000 pontos; Copy to Points instancia na hora; noise nasce amplitude 1/size 1 (exceto DENTRO de forças, onde nasce 0 = desligado); ranges soft 0–1 ou 0–10 digitáveis além. Activation=1 e Group="" em todo POP: escopo total por default, restrição opt-in.

**Scrubbing: value ladder.** MMB-segurar em qualquer campo numérico abre a **escada de valores** (degraus 0.001 · 0.01 · 0.1 · 1 · 10 · 100): mover vertical escolhe o degrau, horizontal incrementa naquele passo — edição grossa e fina no MESMO gesto, sem digitar. Funciona em vec3 (por componente ou ligado). Alt+clique no campo = keyframe ali mesmo.

**Spreadsheet.** Geometry Spreadsheet = tabela viva pontos×atributos do nó com display flag (colunas P, v, pscale, orient, mops_falloff…), filtrável por grupo/padrão de nome; é O instrumento de debug de motion (ver o número, não adivinhar o pixel). Par com o viewport: visualizers de atributo (cor falsa, setas de v, texto por ponto) ligáveis por atributo. MOPs adiciona Preview Falloff (sprites coloridos = falloff) e Visualize Frame (eixos por instância).

**Expressões/referências.** Qualquer parâmetro aceita expressão HScript (`$F` frame, `$T` segundos, `$FF` frame fracionário, `sin()`, `rand()`, `fit01()`) ou referência de canal `ch("../outro_no/param")` — criada por RMB Copy Parameter → Paste Relative References (é o nosso `motion.drive` como GESTO de UI). Campos de expressão ficam coloridos; clicar no label alterna exibir valor/expressão. Escape hatch universal: Wrangle (VEX) com `@atributo` — nosso `motion.expression` é exatamente isso.

**Aprendizado.** F1 em qualquer nó → página de help com **arquivos de exemplo embutidos** (botão instala a cena de exemplo no projeto); tooltips por parâmetro; help descreve CADA parâmetro. Galleries/shelf dão setups prontos que o artista disseca. MOPs replica o modelo: wiki por nó + cenas de exemplo + tutoriais oficiais em 7 partes (github.com/toadstorm/MOPS/wiki/MOPs-Tutorial); a lição do MOPs para o PH2D: uma API interna de 2 nós (Extract/Apply Attributes) + UM atributo-contrato (`mops_falloff`) sustenta ~90 nós de superfície — família > monólito, falloff componível > parâmetro de força, e todo efeito é escalável por um campo espacial que o artista VÊ.