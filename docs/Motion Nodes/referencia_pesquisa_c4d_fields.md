# Cinema 4D MoGraph + Fields — pesquisa profunda (help.maxon.net)

Fontes primárias fetched nesta sessão estão citadas inline. Onde a página devolveu texto, os trechos entre aspas são verbatim da doc. Itens marcados `[conhecimento consolidado]` vêm do meu conhecimento do produto (defaults de fábrica do app), não de fetch desta sessão — confiáveis, mas confira no app se for load-bearing.

---

## A) CATÁLOGO — `item · categoria · função · status vs PH2D`

### A1. Cloner e geradores

| Item | Categoria | Função | Status vs PH2D |
|---|---|---|---|
| Cloner: Linear mode | gerador | N clones com offset P/S/R por passo ou endpoint, com Step cumulativo (espiral) | **TEMOS-PARCIAL** — `motion.clone` + `motion.move/rotate/scale`; falta o "step cumulativo" declarado no gerador |
| Cloner: Radial mode | gerador | círculo com plane/align/start-end angle/offset | **TEMOS** — `motion.distribute_radial` |
| Cloner: Grid Array | gerador | grade 3D com Form (Cube/Sphere/Cylinder/Object) e Fill (casca oca) | **PARCIAL** — `motion.grid` existe; Form-constraint e Fill (shell) faltam |
| Cloner: Honeycomb | gerador | grade 2D com fileiras alternadas deslocadas (tijolo/colmeia), Form Square/Circle/Spline | **FALTA** |
| Cloner: Object mode (vertex/edge/poly/surface/volume) | gerador | clones sobre geometria alvo | **PARCIAL** — `motion.scatter`/`distribute_poisson`/`distribute_curve`; falta per-vertex/per-edge/volume de mesh arbitrário |
| Cloner: Object mode (spline) | gerador | Count/Step/Even/Spline Point, rail spline, offset/rate (auto-anima!) | **TEMOS-PARCIAL** — `distribute_curve` + `spline_wrap`; falta Rate (fluxo automático sem keyframe) e rail |
| Clones: Iterate/Random/Blend/Sort | política de fonte | qual filho vira cada clone; **Blend** interpola primitivas paramétricas entre clones | **FALTA** (multi-fonte por instância; Blend é único no mercado) |
| Instance/Render Instance/Multi-Instance | perf | dedup de memória | **TEMOS por construção** — streams com colunas `Arc` são "multi-instance" nativo (ADR-0138) |
| Fix Clone / Fix Texture | detalhe | clones herdam matrix do cloner; textura fixa ou atravessada | n/a (2D; equivalente seria UV-lock de tile) |
| Transform tab do Cloner | autoria | P/S/R aplicados a TODO clone + Color por clone + Time offset | **PARCIAL** — cor via `motion.tint/color_*`; time offset via `motion.stagger`/`time_remap` |
| MoSpline / Matrix / Fracture / Voronoi Fracture | geradores-irmãos | fontes MoGraph alternativas | Matrix ≈ streams puros TEMOS; Fracture 2D **FALTA** (temos `motion.voronoi` p/ células) |

### A2. Effectors (15)

| Effector | Função | Status vs PH2D |
|---|---|---|
| Plain | só o falloff vira o valor; o "hello world" de Fields — transform constante modulado pelo campo | **PARCIAL** — `motion.move`+`motion.falloff` fazem isso em 2 nós; o desenho C4D é 1 gesto |
| Random | valor aleatório por clone; modos Random/Gaussian/Noise/Turbulence/Sorted; Synchronized/Indexed/Seed | **TEMOS-PARCIAL** — `motion.wiggle` (noise animado), `motion.noise`, hash(seed,id); falta o pacote único com modos + Indexed + Synchronized |
| Shader | grayscale de textura/shader → valor por clone (via UV do arranjo) | **TEMOS-PARCIAL** — `motion.luminance` (por imagem); falta amostrar procedurals arbitrários no espaço do arranjo |
| Delay | suaviza TEMPORALMENTE a saída dos effectors anteriores: Average/Blend/**Spring** | **PARCIAL** — `motion.spring` é físico por-instância; falta o *modificador genérico* "qualquer propriedade chega com mola/atraso" |
| Formula | fórmula matemática → valor por clone (vars id, count, t, posição…) | **TEMOS** — `motion.expression` (VEX-lite) |
| Inheritance | clones assumem posição/ANIMAÇÃO de um objeto de referência (com atraso por índice); morph entre arranjos | **FALTA** (é o "clones viram a animação de outro objeto", motion-graphics puro) |
| Push Apart | separa clones sobrepostos: Push Apart/Scale/Hide, Radius, Iterations | **PARCIAL** — `motion.collide`/`sim.collide` resolvem overlap físico; falta o modo Scale/Hide (resolver por escala ou visibilidade) |
| ReEffector | meta-effector: reescala/restringe a influência de OUTROS effectors via seu próprio campo | **FALTA** (meta-camada sobre a pilha de modificadores) |
| Sound | espectro de áudio → clones (probes de frequência/amplitude no gráfico; Iterate/Distribute/Blend) | **FALTA** — temos módulo de áudio inteiro e nenhuma ponte áudio→motion |
| Spline | puxa clones para uma spline / arranja ao longo dela | **TEMOS** — `motion.spline_wrap`, `distribute_curve` |
| Step | rampa 0→1 pelo índice do clone, com spline de forma | **TEMOS** — `motion.step` + `motion.stagger` |
| Target | orienta clones para alvo/câmera (sem Parameter tab) | **TEMOS** — `motion.look_at` |
| Time | valor que acumula com o tempo (rotação contínua sem keyframe) | **PARCIAL** — `value.lfo`/`motion.drive`? (acúmulo contínuo por dt é sim-like; verificar `motion.drive`) |
| Volume | limita efeito ao VOLUME de um mesh arbitrário | **PARCIAL** — nosso falloff tem formas; mesh-as-mask arbitrário falta |
| Group | agrupa N effectors sob um falloff/strength comum | **FALTA** |

### A3. Fields (o sistema, R20+)

| Item | Categoria | Status vs PH2D |
|---|---|---|
| Fields list (pilha de camadas com blending por camada) | infra | **FALTA** — nosso `motion.falloff` é 1 nó, 1 forma |
| Field objects: Linear, Spherical, Box, Cylinder, Cone, Torus, Capsule, Radial | forma espacial | **PARCIAL** — depende das formas do nosso falloff; Radial (setor angular) e Capsule provavelmente faltam |
| Field objects: Random | procedural | **PARCIAL** (`motion.noise` é nó separado, não camada de falloff) |
| Field objects: Shader, Sound, Formula, Python, Group | amostradores | Shader PARCIAL (`luminance`), Sound FALTA, Formula TEMOS (`expression`), Python n/a (Luau futuro), Group FALTA |
| Objetos arrastados como field (mesh → distância/proximal, emitter → partículas) | amostrador | **FALTA** |
| Modifier layers: Solid, Curve, Step, Quantize, Clamp, Colorize/ColorRemap, Decay, Delay, Freeze, Invert, Rangemap, Formula, Time | pós-processo escalar na pilha | **FALTA** quase tudo — `value.map_range` cobre Rangemap; `pulse.sample_hold` lembra Freeze |
| Blending por camada: Normal, Add, Subtract, Multiply, Screen, Overlay, Min, Max, Clip (mask) | infra | **FALTA** |
| Remapping por field (Inner Offset, Contour, Min/Max, Clamp, Invert) | infra | **PARCIAL** — se nosso falloff tem curva; o pacote completo falta |
| 3 canais amostrados: **Value + Color + Direction** | infra | **FALTA** — nosso falloff só produz escalar |
| Máscara por camada (qualquer sub-pilha de fields vira mask de outra camada) | infra | **FALTA** |
| Sub-fields (campos dentro de PARÂMETROS de outra camada) | infra | **FALTA** |

### A4. Infra MoGraph

| Item | Função | Status vs PH2D |
|---|---|---|
| Effector como **Deformer** (aba Deformer: Off/Object/Point/Polygon) | o MESMO effector deforma pontos de mesh em vez de clones | **PARCIAL** — nossos deformers (bend/twist/…) são nós próprios; a dualidade "um nó, dois domínios (instâncias OU pontos)" falta |
| MoGraph Selection (tag; ferramenta de pintar seleção de clones) | restringe effector a um subconjunto | **PARCIAL** — `motion.cull`/`value.instance_field` aproximam; falta seleção AUTORADA persistente por clique/pincel |
| MoGraph Weightmap (peso 0–100% por clone, pintável; effectors podem ESCREVER peso via Weight Transform) | modulação autorada + computada | **FALTA** — é uma coluna `weight` autorável que effectors leem E escrevem |
| MoGraph Cache tag | baka o resultado (scrub, render farm) | **TEMOS outra metade** — `Cook::checkpoint`/CheckpointRing dá scrub bit-exato; bake-to-disk persistente falta (physics já tem bake-to-timeline como precedente) |
| Falloff legado (pré-R20: shapes + spline) | o ancestral do nosso `motion.falloff` | TEMOS o equivalente ao LEGADO — a lição é que a Maxon o substituiu inteiro |

---

## B) PARÂMETROS DETALHADOS

### B1. Cloner ([OMOGRAPH_CLONER-ID_OBJECTPROPERTIES.html, r21](https://help.maxon.net/c4d/r21/us/html/OMOGRAPH_CLONER-ID_OBJECTPROPERTIES.html))

**Top-level:**

| Param | Opções/Range | Nota |
|---|---|---|
| Mode | Object · Linear · Radial · Grid Array · Honeycomb Array | default Linear `[conhecimento consolidado]` |
| Clones | **Iterate** (sequencial do Object Manager) · **Random** (com Seed) · **Blend** (interpola primitivas paramétricas OU meshes entre clone 1 e N) · **Sort** (sequência dirigida por Effector, exige Modify Clone > 0%) | |
| Fix Clone | bool | clones assumem P/S/R matrix do Cloner |
| Fix Texture | Straight · Off · Alternate X/Y | textura fixa ao clone vs clone "atravessa" a textura |
| Instance Mode | Instance · Render Instance · **Multi-Instance** ("single internal object, minimal memory, fastest") | |
| Viewport Mode | Off · Points · Matrix · Bounding Box · Object | display-only |
| Seed | 0..2147483647 | |

**Object mode (mesh):** Distribution = Vertex · Edge (com Offset/Scale on Edge/Edge Scale) · Polygon Center · Surface (com Count) · Volume (Random/Surface sub-modos) · Axis. Mais: Selection (tag restringe), Enable Scale/Scale (proporcional à área do polígono, com Scale Spline), Up Vector (trava rotação Z aleatória), Fix Clone alinha eixo X à normal.

**Object mode (spline):** Mode = **Count** (espaçamento por interpolação da spline, não-uniforme) · **Step** (intervalo fixo) · **Even** (uniforme por arco) · **Spline Point** (1 por vértice) · Axis; Per Segment; Align Clone (tangente); Rail spline (+ Target/Scale); Smooth Rotation; **Offset** (desloca todos ao longo); Offset Variation; Start/End; **Loop** (reinsere no começo); **Rate** ("auto-animates clone movement per frame, no keyframe required") + Rate Variation + Random Seed; Volume Spread (tubo em volta do rail).

**Linear mode:** Count · Offset · Mode (**Per Step** | **End Point**) · Amount [-∞..+∞%, multiplicador global de P/S/R — "growth effects"] · P.XYZ [m] · S.XYZ [%] · R.HPB [°] · **Step Mode** (Cumulative = espiral | Single Value) · Step Size [%] · Step Rotation HPB. Defaults de fábrica: Count 3, P.Y 50 cm `[conhecimento consolidado]`.

**Radial mode:** Count · Radius · Plane (XY/XZ/YZ) · Align (X tangencial) · Start/End Angle · Offset · Offset Variation.

**Grid Array:** Count XYZ (3×3×3 default `[conhecimento consolidado]`) · Mode (Per Step | Endpoint — o que o Size significa) · Size XYZ · **Form** (Cube · Sphere · Cylinder · **Object** — mesh arbitrário como volume) · **Fill** [0..100%] ("0% = hollow shell only" — casca!).

**Honeycomb:** Orientation (XY/XZ/YZ) · Offset Direction (Height|Width) · Offset [0..100%, 50% = colmeia clássica] · Offset Variation · Perpendicular Variation · Seed · Count W/H · Mode (Endpoint|Per Step) · Size W/H · **Form** (Square · Circle · **Spline** como boundary).

**Transform tab:** P/S/R aplicados a todo clone antes dos effectors + **Color** (por clone, gradiente/aleatória) + **Time offset** (filhos animados começam defasados). **Effectors tab:** lista ordenada de effectors com checkbox — a ORDEM é a ordem de aplicação.

### B2. Abas comuns a TODO effector

**Aba Effector** ([OEPLAIN-ID_MG_BASEEFFECTOR_GROUPEFFECTOR.html, s22](https://help.maxon.net/c4d/s22/us/html/OEPLAIN-ID_MG_BASEEFFECTOR_GROUPEFFECTOR.html)):

| Param | Range | Descrição (verbatim) |
|---|---|---|
| Strength | [-∞..+∞%] | "adjust the overall strength of the effect. Values of less than 0% and greater than 100% can be entered" — default 100% `[conhecimento consolidado]` |
| Selection | tag link | MoGraph Selection tag: "will only affect the clones belonging to the selection"; Weightmap tag: "the clone weighting (0–100%) … will be multiplied with the effector strength" |
| Minimum / Maximum | [-∞..+∞%] | clamp/expande a faixa interna do valor do effector (defaults −100%/100% p/ Random, 0/100% p/ maioria `[conhecimento consolidado]`) |

**Aba Parameter** ([OEPLAIN-ID_MG_BASEEFFECTOR_GROUPPARAMETER.html, s22](https://help.maxon.net/c4d/s22/us/html/OEPLAIN-ID_MG_BASEEFFECTOR_GROUPPARAMETER.html)) — o CORAÇÃO do modelo: o effector produz UM escalar `s` por clone; esta aba diz o que `s` compra:

- **Transform group:** checkbox Position + P.XYZ [m] · checkbox Scale + S.XYZ ou **Uniform Scale** (1 número) + **Absolute Scale** ("prevents incorrect orientation" com peso negativo) · checkbox Rotation + R.HPB [°]. **Transform Mode**: Relative | Absolute | **Remap** ("scaling … via addition to the initial size (Relative)" ou "starting from 0 (Remap)"). **Transform Space**: **Node** (espaço do próprio clone) | Effector (espaço do effector) | Object.
- **Color group:** Color Mode (Off · Effector Color · **Fields Color** · Custom Color) · Blending Mode (Mix · Add · Subtract · Multiply · Divide) · Color · Use Alpha/Strength.
- **Other group:** **Modify Clone** [-∞..+∞%] (empurra o clone pela sequência de filhos — troca QUAL objeto o clone mostra; é assim que Sound faz "medidor de LED") · **Time Offset** [s] (desloca o RELÓGIO da animação do filho por clone — o effector vira controle de retiming!) · **Visibility** (esconde clones onde s passa do limiar) · **Weight Transform** [-∞..+∞%] (ESCREVE a weightmap — effectors a jusante leem) · **U/V Transform** (move o clone nas coordenadas UV internas do arranjo).

**Aba Deformer** ([OEDELAY-ID_MG_EFFECTORDEFORMER_GROUP.html, s22](https://help.maxon.net/c4d/s22/us/html/OEDELAY-ID_MG_EFFECTORDEFORMER_GROUP.html)): Deformer = **Off · Object · Point · Polygon** — o mesmo effector, como filho de um mesh, desloca pontos/polígonos usando o mesmo campo `[estrutura confirmada; detalhe consolidado]`.

**Aba Falloff (legado) → aba Fields (R20+):** todo effector carrega uma Fields list (ver C).

### B3. Específicos por effector

- **Plain** ([OEPLAIN.html, s22](https://help.maxon.net/c4d/s22/us/html/OEPLAIN.html)): "performs no special task … this Effector's influence is restricted solely to its Falloff value (i.e., only via Fields)" — se falloff infinito, "constantly outputs the maximum of 100%" ([overview 7443](https://help.maxon.net/c4d/en-us/Content/html/7443.html)). É a prova de que **o campo É o valor**.
- **Random** ([7443](https://help.maxon.net/c4d/en-us/Content/html/7443.html); detalhes `[conhecimento consolidado]`): Random Mode = **Random** (estático por id) · **Gaussian** (distribuição normal) · **Noise** (3D noise animável — Animation Speed, Scale) · **Turbulence** (noise fractal) · **Sorted**; **Synchronized** (mesmo valor pros 3 eixos de P/S/R); **Indexed** (random por índice, não por posição — estável sob re-arranjo); **Seed** (default 12345).
- **Delay** ([OEDELAY.html, en-us](https://help.maxon.net/c4d/en-us/Content/html/OEDELAY.html)): "ensures that the effects of other Effectors with regard to position, scale and rotation do not begin abruptly, but with a specified temporal delay"; **deve ficar DEPOIS (abaixo) dos effectors que suaviza na lista** — a pilha é ordenada e um modificador vê o acumulado até ali. Mode = **Average** (média móvel) · **Blend** (lerp exponencial ao alvo) · **Spring** ("very useful option for letting clone movements look more realistic" — overshoot/oscilação); Strength [%] = quanto do passado segura. A doc aponta os sucessores: [Delay Field (FLDELAY.html)](https://help.maxon.net/c4d/en-us/Content/html/FLDELAY.html) e [Decay Field (FLDECAY.html)](https://help.maxon.net/c4d/en-us/Content/html/FLDECAY.html).
- **Sound** ([OESOUND-ID_MG_BASEEFFECTOR_GROUPEFFECTOR.html, 2026](https://help.maxon.net/c4d/2026/en-us/Content/html/OESOUND-ID_MG_BASEEFFECTOR_GROUPEFFECTOR.html)) — o mais rico: Sound Track (arquivo) → gráfico espectral vivo; o usuário desenha **Probes** (retângulos no plano freq×amplitude, Ctrl+RMB drag; N probes) — cada Probe: Low/High Frequency, Low/High Loudness, Sampling = **Peak** ("highest value within the Probe") | **Average** | **Step** ("distributes curve profile within the Probe range" pelos clones — o medidor de espectro clássico), Strength por probe (negativo inverte), cor própria (Custom Color/Gradient). Mapeamento probes→clones: **Iterate** (alterna) · **Distribute** (cada probe pega um bloco contíguo de clones) · **Blend** (mistura contínua). Globais: Decay (queda pós-pico), Channels Mono/Stereo, Logarithmic [0-100%], Clamp, **Freeze Value**, Min/Max.
- **Formula**: campo de fórmula com variáveis (id, index, count, falloff, t, posição…) `[conhecimento consolidado]` — nosso `motion.expression` é isomorfo.
- **Inheritance** `[conhecimento consolidado]`: Object link; Mode **Direct** (copia pose) | **Animation** (copia a TIMELINE do objeto, re-executada por clone com **Fall-off temporal por índice** — a onda de "cada clone repete a animação do líder defasado"); Step Gap; Morph entre dois arranjos MoGraph (origem→destino guiado pelo campo).
- **Push Apart** `[conhecimento consolidado]`: Mode **Push Apart** | **Scale** (encolhe até não sobrepor) | **Hide** (esconde o perdedor); Radius; Iterations — resolvedor iterativo de sobreposição, determinístico, não-físico.
- **ReEffector** `[conhecimento consolidado]`: lista de effectors-alvo + Strength/campo próprio → multiplica a influência DELES (meta-máscara).
- **Spline** `[conhecimento consolidado]`: Spline link + Rail; Strength; Mode de arranjo ao longo; Segment Mode; Offset por clone (Start/End).
- **Step** `[conhecimento consolidado]`: rampa por índice com **Spline** de forma + Cutoff; Time Offset animável (a "escada que anda").
- **Target** ([slug OETARGET confirmado](https://help.maxon.net/c4d/en-us/Content/html/58091.html) via link `OETARGET-MGTARGETEFFECTOR_GROUPEFFECTOR.html#direction`): "lacks Parameter tab" — só orienta: Target Mode (Object/Camera), Up Vector, Pitch on/off `[consolidado]`.
- **Time** `[consolidado]`: P/S/R **por segundo**, acumulando — rotação infinita sem keyframe.
- **Volume** ([7443](https://help.maxon.net/c4d/en-us/Content/html/7443.html)): "limits an Effector's effect to a given object's volume" — mesh arbitrário como máscara dura.
- **Group** `[consolidado]`: lista de effectors filhos, um Strength/Fields comum, per-child weight.

### B4. Field objects — parâmetros

Tabs padrão de TODO field object ([FRANDOM.html, s22](https://help.maxon.net/c4d/s22/us/html/FRANDOM.html)): **Basic, Coordinates, Field, Remapping, Color Remap, Direction, View Settings** — ou seja, cada field é um OBJETO na cena com transform próprio (posicionável, animável, parentável) + remap embutido + saída de cor + saída de direção.

| Field | Params principais `[consolidado, exceto citado]` |
|---|---|
| Linear | Length, Direction (X±/Y±/Z±), Clip to Shape; o gradiente ao longo de um eixo |
| Spherical | Size (raio 100%→0 na borda); o "esfera de influência" clássico |
| Box | Size XYZ + falloff nas 6 faces |
| Cylinder / Cone / Torus / Capsule | dimensões da primitiva + Direction |
| Radial | setor ANGULAR (Start Angle, End Angle, Axis) — rampa por ângulo em volta de um eixo |
| **Random** | "generates random values for each clone or object point"; Noise types editáveis ("grayscale values will then be used to ascertain a value") ([FRANDOM s22](https://help.maxon.net/c4d/s22/us/html/FRANDOM.html)); modos Random/Noise/Turbulence, Seed, Relative Scale, Animation Speed |
| Shader | qualquer shader/textura 2D/3D amostrado no espaço escolhido |
| Sound | o motor do Sound Effector como field reutilizável |
| Formula | expressão f(x,y,z,t) |
| Python | código por amostra |
| Group | contêiner com pilha filha (compose e reuse) |
| (Objeto arrastado) | mesh vira campo de DISTÂNCIA (proximal); emitter vira campo de partículas ([FLPROXIMITY.html, s22](https://help.maxon.net/c4d/s22/us/html/FLPROXIMITY.html)) |

### B5. Field LAYERS (modificadores na pilha)

Confirmados por fetch/busca: **Solid** (valor constante — o "Plain" da pilha), **Curve**, **Step**, **Quantize** ("values will be restricted to certain discrete values … the value interval remaining constant"), **Clamp**, **Colorizer/Color Remap**, **Decay** ([FLDECAY](https://help.maxon.net/c4d/en-us/Content/html/FLDECAY.html)), **Delay** ([FLDELAY](https://help.maxon.net/c4d/en-us/Content/html/FLDELAY.html)), **Freeze** ("saves the current list state and can subsequently be used like any other Layer"), **Invert**, **Rangemap**, **Time**, **Formula**, **Proximal** ([FLPROXIMITY](https://help.maxon.net/c4d/s22/us/html/FLPROXIMITY.html)).

O par temporal é a joia: **Delay layer** "influences the temporal course of changes (due to fields/modifier layers arranged in the field list in front of it; which also includes color changes)" com modos **Spring** e **Smooth** ([FLDELAY en-us](https://help.maxon.net/c4d/en-us/Content/html/FLDELAY.html)); **Decay** é o irmão (o valor sobe instantâneo e DECAI com meia-vida — rastro). Isso significa que **a pilha de campo tem ESTADO entre frames**: um modificador de pilha pode ser uma equação diferencial, não só uma função.

### B6. Remapping (comum a todo field) — [FCYLINDER-ID_FIELDCONTROLS.html, 2026](https://help.maxon.net/c4d/2026/en-us/Content/html/FCYLINDER-ID_FIELDCONTROLS.html)

| Param | Range | Verbatim/nota |
|---|---|---|
| Enable Remapping | bool | "Enables or disables the Remapping function (incl. Color Remap)" |
| Strength | [-∞..+∞%] | intensidade entre Min/Max |
| Multiplier | [-∞..+∞%] | "applies to the entire Remapping function" (Strength só entre Min/Max) |
| Invert | bool | "A value range of 0.4 to 0.6 will … be translated to 1 to 0 instead of 0 to 1" |
| **Inner Offset** | [0..100%] | "range of strongest effect, then diminishes to zero" — o platô interno (núcleo 100%) |
| Mirror Graph | bool | display; "disabled for Linear Fields, used for symmetric fields like Spherical" |
| Min / Max | [-∞..+∞%] | limites de saída + **Clamp Min** / **Clamp Max** separados |
| **Contour Mode** | None · Quadratic · Step · Quantize · **Curve** | "None (linear transfer) … Curve (freely definable)" |
| Curve | [-100..100%] | curvatura do Quadratic |
| Steps | [1..2³¹] | contagem do Step/Quantize |
| Spline | curva livre | + **Spline Animation Speed** ("animates curve along X-axis"!), Spline Offset, Spline Range |

---

## C) O DESENHO DO SISTEMA FIELDS — spec para reimplementação

Fonte-mestra: [Fields, 58091.html](https://help.maxon.net/c4d/en-us/Content/html/58091.html) + [FLBASE-ID_FIELDLAYER_BASICGROUP.html](https://help.maxon.net/c4d/en-us/Content/html/FLBASE-ID_FIELDLAYER_BASICGROUP.html).

### C1. O modelo mental

> "Fields represent a spatial distribution of strength values that control an effect."

Um Field é uma função `sample(p: worldpos, ctx) -> {value: f32, color: rgba, direction: vec}` — **três canais, não um**. A pergunta que o sistema responde: *"neste ponto do espaço, quanto/de que cor/em que direção?"* — e QUALQUER consumidor pode fazê-la: effectors, deformers, Volume Builder, forças, vertex maps, seleção, weight tags. **Um sistema de falloff, N consumidores** — é isso que matou o falloff legado por-objeto.

### C2. A pilha (Field list)

1. Uma **Field list** é uma pilha ordenada de camadas, avaliada **de baixo para cima** (como layers do Photoshop). Cada consumidor (effector, deformer, tag) possui UMA lista.
2. Cada camada é (a) um **Field object** (fonte espacial: esfera, linear, random, shader, som…) — objeto REAL na cena, com transform próprio, animável, **reutilizável em várias listas** (a mesma esfera dirige 3 effectors); ou (b) um **Modifier layer** (pós-processo sobre o acumulado: curve, quantize, clamp, invert, delay, decay, freeze…); ou (c) um **objeto qualquer arrastado** (mesh→distância, emitter→presença); ou (d) uma **pasta/Group** (sub-pilha).
3. **Por camada:** Enable por CANAL (checkbox Value / Color / Direction independentes — "Not all channels display for every Field type"), **Blending mode**, **Opacity/Strength 0–100%**, e **Mask** (dropdown que aceita OUTRA sub-pilha de fields — máscara da camada).
4. **Blending modes por camada** (verbatim de FLBASE): **Normal** ("overwrites all previous Fields with its own value" a 100%), **Add**, **Subtract**, **Multiply** ("effective for masking via null values"), **Screen** ("prevents maximum values exceeding 100%"), **Overlay** ("combination of Multiply and Negative Multiply"), **Min**, **Max**, **Clip** ("functions as a mask across all channels using layer volume").
5. **Modifier layers operam sobre o RESULTADO acumulado até ali** — daí "o Delay deve ficar depois": ordem é semântica.
6. **Sub-fields:** parâmetros de uma camada podem ter a própria mini-pilha ("Shift + clicking on a sub-folder will make newly created Fields and Layers subordinate to this folder") — recursão controlada.

### C3. Pipeline de amostragem (pseudocódigo)

```
fn sample_list(p, t, id, state) -> Sample {
  acc = Sample::zero();               // value=0, color=none, dir=0
  for layer in list.bottom_up() {
    if !layer.enabled { continue }
    s = match layer {
      FieldObject(f) => {
        lp = f.world_inverse * p;      // campo tem transform próprio
        raw = f.shape_distance(lp);    // 0..1 espacial (ou noise/shader/som)
        remap(raw, f.remapping)        // Inner Offset -> Contour -> Min/Max/Clamp -> Invert
        // + f.color_remap(raw) e f.direction(lp) para os outros canais
      }
      Modifier(m) => m.apply(acc, state[layer.id], dt),  // pode ter ESTADO (delay/decay/freeze)
      Folder(sub) => sample_list_sub(p, ...),
    };
    if let Some(mask) = layer.mask { s.value *= sample_list(mask, p, ...) }
    for ch in [Value, Color, Direction] where layer.ch_enabled(ch) {
      acc[ch] = blend(layer.mode, acc[ch], s[ch], layer.opacity);
    }
  }
  return acc;
}
```

Pontos críticos para o PH2D:

- **Remap é POR FIELD, não global** — cada fonte já entrega 0..1 moldado (Inner Offset = platô; Contour = perfil da rampa; Clamp opcional dos dois lados; Invert). A pilha então COMPÕE fontes já moldadas. Nosso `motion.falloff` de nó único colapsa as duas fases.
- **Min/Max SEM clamp por default** em vários pontos — valores <0 e >100% fluem de propósito (um Add de dois campos passa de 1; o consumidor decide truncar). O sistema é linear-space até o consumidor.
- **Modificadores com ESTADO** (Delay/Decay/Freeze) tornam a pilha uma função de `(p, t, história)` — no nosso vocabulário, um trecho da pilha é Pure e outro é sim-like; o precedente interno é o par `Cook::cook_scoped`/rings. Freeze = `pulse.sample_hold` espacial.
- **Canal Direction**: campos podem responder um VETOR (ex.: para o Field Force e para orientar clones) — no PH2D isso conversa direto com a coluna `accel`/forças: um "field" nosso poderia alimentar `force.*` como máscara E como direção.
- **Camada = referência, não cópia**: o mesmo Field object em N listas; animar a esfera move todos os efeitos juntos. No PH2D: o falloff virar NÓ REFERENCIÁVEL (um sub-grafo/porta) em vez de parâmetro interno.

### C4. Tradução recomendada para PH2D (síntese da pesquisa)

O desenho campeão NÃO é "mais formas no `motion.falloff`" — é: (1) `falloff` vira **pilha de camadas** (fontes espaciais + modificadores escalares com blending), (2) cada fonte com remap próprio (Inner Offset/Contour/Clamp/Invert), (3) saída multi-canal (peso + cor + direção), (4) camadas com máscara recursiva, (5) modificadores temporais com estado (delay/decay/freeze) usando a infra de checkpoint existente. Nossos `value.map_range`, `motion.noise`, `motion.expression`, `pulse.sample_hold` já são camadas dessa pilha com outros nomes — a reforma é de COMPOSIÇÃO, não de primitivas.

---

## D) UX — o fluxo do artista

- **Fluxo clássico (2 gestos):** criar Cloner → arrastar o mesh pra dentro → menu MoGraph > Effector > Random — o effector recém-criado **se auto-conecta ao Cloner selecionado** (zero fiação manual). Depois: selecionar o effector e mexer no Parameter tab. É a razão do culto ao MoGraph: *resultado visível em <10 segundos*, profundidade opcional depois.
- **Attribute Manager** ([How to use the Attribute Manager, r21 5822.html](https://help.maxon.net/c4d/r21/us/html/5822.html)): tudo são tabs horizontais por objeto (Basic/Coordinates/Object/Transform/Effectors/Fields…); a Fields list é um mini-gerenciador de camadas EMBUTIDO no painel (linhas com checkbox por canal, dropdown de blending, slider de opacity) — o "Photoshop layers" dentro de um atributo. Multi-seleção de objetos edita em massa.
- **O que os artistas amam** `[consenso da comunidade, consolidado]`: auto-wire cloner↔effector; Plain+Field linear animado = a "onda de revelação" em 4 cliques; Fields reutilizáveis entre effectors/deformers/tags; Multi-Instance para milhões; Sound Effector R19+ (probes desenhadas no espectro); Delay/Spring de um clique.
- **O que odeiam:** a explosão de combinatória Effector×Falloff pré-R20 (por isso Fields NASCEU); performance de pilhas de fields profundas com Delay (estado por ponto); Push Apart limitado; a ordem-importa da lista de effectors ser invisível para iniciantes; Inheritance ser críptico; **o falloff legado era por-effector e morria com ele** — a Maxon precisou de uma quebra (R20) para desacoplar. Lição direta para nós: desenhar o falloff como recurso compartilhável DESDE JÁ evita a migração deles.
- **Defaults que ensinam:** todo effector nasce com Position Y = 50cm ligado `[consolidado]` (mexer no Strength já MOSTRA algo); Fields novos nascem com remap linear e clamp ligado; o Cloner Linear nasce 3 clones empilhados em Y — nada nasce inerte.

## E) XPresso (breve) — lições de UI de grafo

Fontes: [XPresso Editor, 5829.html](https://help.maxon.net/c4d/en-us/Content/html/5829.html) · [Iterate, GVITERATE.html](https://help.maxon.net/us/html/GVITERATE.html) · [Falloff node, GVMOGRAPH_FALLOFF (s22)](https://help.maxon.net/c4d/s22/us/html/GVMOGRAPH_FALLOFF-ID_GVPORTS.html).

- **Portas coloridas por papel**: entradas sob quadrados azuis, saídas sob vermelhos; portas são ADICIONÁVEIS por menu no nó (o nó Object expõe qualquer parâmetro do objeto como porta on-demand) — o contrato do nó é aberto, não fixo.
- **Iterators**: "Iterator nodes can each output many values per frame … correspond to the for-next loop" — iteração é um NÓ-FONTE que multiplica o fluxo a jusante, não um contêiner; casa com nosso modelo de streams (a stream JÁ é o iterator implícito).
- **Prioridades explícitas**: valores −499..499 em categorias (Initial/Animation/Expressions/Dynamics/Generators) — "the same expression can produce different results depending on whether it is executed before or after other expressions". A lição NEGATIVA: prioridade numérica manual é a fonte nº 1 de bugs de XPresso; nosso cook determinístico por topologia é superior — não importar.
- **Existe um nó Falloff no XPresso** que amostra uma Fields list por posição — confirma que Fields é API amostrável de fora, não feature de effector.

## Fontes

- https://help.maxon.net/c4d/r21/us/html/OMOGRAPH_CLONER-ID_OBJECTPROPERTIES.html (Cloner completo)
- https://help.maxon.net/c4d/en-us/Content/html/7443.html (Effectors overview)
- https://help.maxon.net/c4d/s22/us/html/OEPLAIN-ID_MG_BASEEFFECTOR_GROUPEFFECTOR.html · …GROUPPARAMETER.html (abas comuns)
- https://help.maxon.net/c4d/en-us/Content/html/OEDELAY.html · https://help.maxon.net/c4d/2026/en-us/Content/html/OESOUND-ID_MG_BASEEFFECTOR_GROUPEFFECTOR.html
- https://help.maxon.net/c4d/en-us/Content/html/58091.html (Fields) · https://help.maxon.net/c4d/en-us/Content/html/FLBASE-ID_FIELDLAYER_BASICGROUP.html (blending por camada)
- https://help.maxon.net/c4d/2026/en-us/Content/html/FCYLINDER-ID_FIELDCONTROLS.html (Remapping) · https://help.maxon.net/c4d/s22/us/html/FRANDOM.html · https://help.maxon.net/c4d/en-us/Content/html/FLDELAY.html · FLDECAY.html · https://help.maxon.net/c4d/s22/us/html/FLPROXIMITY.html
- https://help.maxon.net/c4d/en-us/Content/html/5829.html (XPresso) · https://help.maxon.net/us/html/GVITERATE.html · https://help.maxon.net/c4d/r21/us/html/5822.html (Attribute Manager)

Limitações honestas: as páginas específicas do Random/Delay effector e de vários field objects individuais não renderizaram por fetch (404 nos slugs tentados ou conteúdo truncado) — os parâmetros desses itens vêm de conhecimento consolidado do produto e estão marcados; modos e semântica têm alta confiança, defaults numéricos exatos desses itens merecem conferência no app.