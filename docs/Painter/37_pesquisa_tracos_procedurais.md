# 37 — Traços procedurais: o que o Alchemy ensina, e o que o mundo já ama

> Pedido do Enio, 2026-08-12: *"Vamos criar novas features para enriquecer o modo digital do nosso
> módulo painter. Investigue 3 tools do app de desenho Alchemy: (1) Estilo: Linha ou forma sólida …
> (2) Speed Shapes (3) Pressure Shapes. Além desses, vai investigar outros algoritmos e recursos
> gerados de maneira procedural (principalmente traços) e que são úteis e bem quistos pelos artistas
> ao redor do mundo. Faça uma pesquisa e traga um relatório."*
>
> Este é o relatório. Ele **não constrói nada** — traz o que as três tools do Alchemy de fato são
> (citadas do manual, não de memória), o levantamento do que **já existe neste repo** para nenhuma
> das propostas reconstruir o que shipa, o estado da arte por família, e uma recomendação ordenada
> por valor ÷ custo com o que **precisa ser MEDIDO antes** de virar wave.

---

## 0. Política e método

⚠️ **O Alchemy é GPL-3.** Vale a mesma regra do Blender Texture Paint e do reference JS da aquarela:
**comportamento, nunca código.** Tudo aqui saiu do **manual** (documentação de comportamento, FLOSS
Manuals) e das páginas do projeto — nenhuma linha de fonte foi lida, e nenhuma será.

E a regra que este doc segue por dentro: **antes de propor, medir o repo.** A lição está registrada
na memória do projeto (*"uma lista de pendências velha faz a próxima LLM propor construir o que
existe"*) e ela já mordeu no módulo de áudio. Cada item abaixo carrega uma linha **"o que já
temos"**, verificada por `grep` no código que shipa — não por lembrança.

---

## 1. As três do Alchemy

O modelo mental do Alchemy, que decide as três: **um gesto não pinta pixels, ele produz uma FORMA**
(um caminho vetorial). Todo módulo "Create" é uma lei diferente de *como o gesto vira caminho*, e o
"Style" decide se aquele caminho é **contornado** ou **preenchido**. É o oposto exato do nosso
motor, que é uma **lista de dabs** carimbada ao longo do caminho — e é por isso que as três
importam: elas são a pergunta que o nosso motor nunca fez.

### 1.1 Style — Line ou Solid

> **Manual, verbatim:** *"Style — Toggle between drawing with a line or solid fill."*
> E no módulo Shapes: *"The **Outline** button allows you to temporarily draw with just the outline
> while in solid shape mode."*

**O que é.** Em `Line` o caminho é traçado com a espessura corrente. Em `Solid` o caminho é
**fechado e preenchido** — e aí a espessura deixa de significar coisa alguma (o pedido do Enio diz
exatamente isto). Um gesto que se cruza produz uma região com buracos ou não, conforme a regra de
preenchimento (nonzero × even-odd).

⚠️ **A distinção que decide a implementação, e que é fácil errar:** `Solid` **não é a área varrida
pelo pincel** — essa nós já temos (o depósito é um envelope `max` sobre os dabs, que é literalmente
a união do rastro). `Solid` é a **área CERCADA pelo gesto**. Desenhar um laço fino e solto pinta uma
mancha gorda; é essa a ferramenta.

**O que já temos — e aqui há uma surpresa medida.** O `stroke_boolean_contours` do Painter **já
computa a região preenchida e depois a joga fora**: ele rasteriza as formas autoradas a SS=3, faz
*flood* por componente, e então **traça o contorno (Moore)** para devolver uma polilinha que vira
dabs. O doc [35](35_boolean_o_que_o_vector_ensina.md) §1.1 mede isso: numa elipse de 200 px o
traçado sozinho custa **4,777 dos 6,448 ms**, e devolve **3 400 pontos** para descrever quatro
cúbicas.

⇒ **`Solid` é *parar antes do traçado*.** O bitmap da região já está em mãos no meio do caminho que
hoje shipa. Isso não torna a wave trivial (falta a porta de escrita, o anti-aliasing da borda —
hoje a borda é feita pelos dabs — e a decisão de produto abaixo), mas muda a ordem de grandeza:
não é um motor de rasterização novo, é um **consumidor novo de um subproduto existente**.

**A decisão de produto que ela carrega.** No Alchemy `Style` é **global** (uma tecla, `S`), porque
lá *tudo* é forma. Para nós as opções honestas são duas, e são diferentes:

- **(a) um `StrokeMethod`** — "Solid" ao lado de FreeHand/Ellipse/Polygon. Barato, encaixa no menu
  que já existe, e herda os shape editors (a forma fica **editável** antes do Apply, que é
  exatamente o gesto do Alchemy melhorado).
- **(b) um switch do pincel** (Line/Solid) que atravessa todos os métodos. Mais fiel ao Alchemy e
  mais caro: cada método teria de responder *"qual é o meu caminho fechado?"*, e `Dots`/`Airbrush`
  não têm resposta boa.

**Recomendo (a).** É onde o gesto já mora, e é o único lugar onde a forma continua editável.

### 1.2 Speed Shapes

> **Manual, verbatim:** *"Speed Shapes — Accentuates the pen speed to create shapes that throw the
> line beyond the actual pen position."*
> Controles: **Speed** (*"controls how much speed is applied"*) e **Line Type** (*"toggle between
> drawing straight lines and curved lines"*).
>
> E o irmão dele, o módulo *affect* **Displace**: *"Changes shape geometry by contracting or
> expanding the shape based on the movement of the pen. **Faster movement causes the shape to
> contract closer towards the current pen location. Likewise, slower movements push points
> away.**"*

**O que é.** A velocidade do gesto vira **extrapolação**: o ponto depositado não é onde o cursor
está, é `p + v·k`. Movimento rápido arremessa a linha muito além do dedo — *"and possibly off the
screen itself"*, diz o manual. É **inércia**, e o efeito é o oposto exato do estabilizador: o
estabilizador **atrasa** a linha, o Speed Shapes a **adianta**.

**O que já temos: nada — e a velocidade já está medida e descartada.** `git grep speed|velocity` no
`spec*.rs` do `ph2d-painter-brush` volta **vazio**: velocidade não é entrada de nada. Mas o
`stroke/stabilize.rs` mantém, lado a lado, a posição **filtrada** (`stab_pos`) e a **crua**
(`last_raw_pos`) — a diferença entre elas, por evento, **é** a velocidade. O número existe; ninguém
o lê.

⚠️ **E é a entrada mais barata que existe:** o mouse a dá. Nada de tablet, nada de winit, nada de
plataforma. Ver §2 — é isto que faz do Speed a alavanca central deste relatório.

### 1.3 Pressure Shapes

> **Manual, verbatim:** *"Pressure Shapes — Uses the pressure from a pen tablet to control line
> thickness."*

**O que é.** O mais simples dos três: pressão → espessura. No Alchemy ele é um *módulo* separado
porque lá a espessura é geometria (o contorno da fita de largura variável **é** a forma).

⚠️ **Nós já temos a FEATURE e não temos o DISPOSITIVO — e isso está pinado num teste que roda.**
`crates/ph2d-painter-brush/src/dynamics.rs` mapeia pressão → raio e pressão → cobertura, com mínimo
por canal (o default é `size_pressure: true`, `size_min: 0.1`). E
`shells/desktop/tests/the_desktop_shell_has_no_pen_pressure.rs` demonstra, varrendo a grade inteira
de sliders, que **nenhuma combinação move um pixel**, porque a shell entrega `1.0` literal.

O levantamento daquele gate (2026-07-31) é categórico, e a nota que circulava — *"custa uma
função"* — **é falsa** no winit 0.30: `Touch{force}` é touchscreen, `TouchpadPressure` é trackpad da
Apple, `CursorMoved` não carrega pressão, `AxisMotion` é valuator cru do XInput2 com `AxisId` opaco
(não há como perguntar *qual* eixo é pressão nem a faixa dele), e o backend **Wayland** — que é o
que a máquina do Enio roda — não implementa `zwp_tablet_v2`.

⇒ **O item não é "construir Pressure Shapes". É ligar a caneta**, e são duas rotas, nenhuma barata:
subir o winit (fundação de janela do app inteiro ⇒ cross-line, classe ADR) ou um caminho de tablet
por plataforma (dep nova, `unsafe`, segunda fonte de eventos ao lado do winit).

**O que dá para entregar HOJE, sem o dispositivo:** a **velocidade** no lugar da pressão. E não é
gambiarra — é literalmente por isso que o Alchemy tem os dois módulos separados: um para quem tem
tablet, outro para quem tem mouse. Mais o **Taper**, que já shipa e já dá a ponta afinada
(início/fim, com `tip` e `opacity` próprios).

---

## 2. O achado do levantamento: a nossa matriz de dinâmica é 1×2, e a única entrada dela está morta

Este é o parágrafo mais importante do relatório.

O Krita — que é a referência mais completa de *o que um traço procedural pode ler* — expõe **16
sensores**. Marquei os que **um mouse já dá** (sem tablet, sem plataforma nova):

| Sensor (Krita) | mouse? | temos? |
|---|:---:|---|
| Pressure / PressureIn | ✗ | mapeado (`dynamics.rs`), **inerte** — a shell entrega 1.0 |
| **Speed** — *"how much the brush is affected by the speed at which you draw"* | ✅ | **✗** |
| **Drawing Angle** — *"by which direction you are drawing in"* | ✅ | **quase** — `Dab::dir` existe (Rake/Flow/Chisel o leem); não é entrada genérica |
| **Distance** — *"over length in pixels"* | ✅ | **quase** — `Taper` é a ponta, não a régua |
| **Time** — *"over drawing time in seconds"* | ✅ | **quase** — `airbrush_rate_s` é um timer, não uma entrada |
| **Fade** — *"over length, proportional to the brush size"* | ✅ | **✗** |
| **Fuzzy (Dab)** — *"basically the random option"* | ✅ | ✅ — a família `jitter_*` |
| **Fuzzy Stroke** — *"randomness per stroke"* | ✅ | ✗ |
| **Perspective** — *"by the perspective assistant"* | ✅ | ✗ (não há assistentes) |
| X-tilt / Y-tilt / Tilt-direction / Tilt-elevation | ✗ | ✗ |
| Rotation (barrel) / Tangential Pressure | ✗ | ✗ |

E os **alvos**. Hoje o `Dynamics` tem exatamente **dois** (`radius_scale`, `coverage_scale`), ambos
rampa linear sem curva. O `BrushSpec` tem dezenas de números que ninguém pode modular: `dab_angle`,
`dab_flatten`, `spacing`, `hardness`, `grain_depth`, a cor, o falloff.

⇒ **A matriz é 1 × 2, com o 1 morto.**

⚠️ **A alavanca de maior valor por custo deste relatório não é uma feature — é a MATRIZ**, porque
toda feature abaixo entra nela em vez de virar um caso especial. E dois primitivos caros dela **já
shipam neste painel**:

- a **curva de resposta**: `BrushSpec.custom_falloff: FalloffCurve` já é editável arrastando, pelo
  `InteractiveState::CurvePoint` (o editor de falloff do Painter — e é o mesmo primitivo que a
  `line/motion-value` reusou para o `ParamWidget::Curve`);
- a crate **`ph2d-curve`** (leaf, dep-free) existe desde 2026-07-25, com LUT para o device.

Nada disso precisa nascer. O que falta é o **canal**: `(entrada, alvo, curva, faixa)`.

---

## 3. As famílias que o mundo ama

Cada item: o que é · quem shipa · por que os artistas gostam · **o que já temos** · o tamanho da wave.

### 3.1 Sketchy / *neighbour points* — o traço que se costura sozinho

A ideia mais copiada da história do desenho na web: guarda-se **todos** os pontos do traço e, a cada
ponto novo, ligam-se os vizinhos dentro de um raio com linhas de opacidade baixa. Sai o hachurado de
lápis / teia de aranha que ninguém consegue imitar à mão.

Origem: o *Scribbler* de **Ze Frank**, implementado em JS pelo **Mr.doob** no **Harmony**; virou o
*Webink* do DeviantArt Muro e o **Sketch brush engine** do Krita (*"a line based brush engine, based
on the Harmony brushes. Very messy and fun."*). Parâmetros do Krita que valem copiar de
comportamento: *Line Width* · *Offset Scale* · *Density* · **Magnetify** (*"causes curve lines to
attract to nearby line sections"*, ligado por default) · *Random RGB* · *Random Opacity* ·
*Distance Opacity* (*"opacity based on pen speed and distance"*) · *Use Distance Density* · *Paint
Connection Line* · *Simple Mode*.

**O que já temos:** nada disto. Mas a estrutura de que ele precisa — *"não guarde só o ponto
anterior, guarde o array de todos os pontos do traço"* — **existe**: o `Stroke` já carrega o caminho
(é o que o FreeHand simplifica no pen-up, e o que o `taper` re-percorre no resolve de cauda).

**Wave:** média. É um produtor de dabs novo (segmentos finos), não um motor de rasterização novo —
e o Painter já sabe carimbar uma linha de dabs.

⚠️ **E ele é o candidato mais forte do relatório por um motivo que não é estético:** é o único da
lista que **não precisa de entrada nova nem de geometria nova** — só da memória do traço, que já
está lá.

### 3.2 Curve / *history* engine — o arame

Krita: *"a fun tool that you can use for jazzing up your linework"*. Guarda os últimos **N** pontos
e liga o ponto atual a eles com curvas; parâmetros: *line width*, *history size*, *curves opacity*,
*paint connection line*, *smoothing*. É o primo do 3.1 com memória **limitada** — dá o traço de
arame/laço, ótimo para efeito e caligrafia solta.

**Wave:** pequena, **se** o 3.1 for feito primeiro (é o mesmo produtor com uma janela deslizante).

### 3.3 Hatching — a hachura que se REVELA

Krita: *"the hatching itself is mostly fixed in location, so drawing with a hatching brush usually
acts more like **revealing** the hatching underneath than drawing with brushes of parallel lines."*
Ângulo, separação, espessura, e as variantes cruzadas.

**O que já temos:** o **Grain** com mapeamento de tela (`View`) é exatamente *"o padrão está no
mundo, o pincel revela"* — a metade conceitual já shipa. O que falta é a família de padrões
(paralelas, cruzadas, com dinâmica de separação).

**Wave:** pequena-média. É um gerador de textura procedural a mais no slot que já existe.

### 3.4 Spray / scatter — N partículas por dab

Krita Spray, Photoshop *Scattering + Dual Brush*, Painter *Image Hose*. Um dab vira **N** cópias
espalhadas num raio, com distribuição, tamanho e rotação randomizados.

**O que já temos: a metade errada.** O `jitter` (posição), `jitter_scale`, `jitter_rotate`,
`jitter_spacing` e o `color_jitter_*` deslocam **um** dab. Spray é **contagem** — `N` cópias no
mesmo instante. É a diferença entre *tremer* e *espalhar*.

**Wave:** pequena. O fan-out é no `walk_dab` (a mesma porta única onde Symmetry e Tiling já se
penduram) ⇒ herda tudo de graça, exatamente como o smear field herdou.

### 3.5 Partícula / fio — Krita Particle

*"A brush that draws wires using parameters. These wires always get more random and crazy over
drawing distance."* Partículas com massa e gravidade deixando rastro.

⚠️ **Nós temos o motor inteiro** — e não no Painter: o **Motion Nodes** shipa boids, verlet,
soft-body, `force.curl` divergence-free (Bridson), grade espacial na GPU, tudo com paridade CPU×GPU
gateada. O que falta é a **ponte**: um traço do Painter que semeia partículas num campo.

### 3.6 Fita com física — Alchemy **Ribbon Shapes**

> *"Leaves a trail of ribbon like shapes."* Controles: **Size · Spacing · Friction · Gravity**.

Uma fita presa ao cursor por uma mola com atrito e gravidade. É o Krita **Dyna** ("massa e arrasto")
levado à geometria. Artistas adoram porque o traço **pesa** — a linha atrasa nas curvas e chicoteia
na saída.

**O que já temos:** o estabilizador (`stroke/stabilize.rs`) é a metade *arrasto* deste modelo, sem
massa e sem gravidade.

### 3.7 Splatter — Alchemy **Splatter Shapes**

> *"Creates paint like splatters on the canvas."* Controles: **Size · Drips**.

Respingo + escorrido. ⚠️ **O escorrido nós já temos**, e é físico: a gravidade do Wet Paint. O que
falta é o **respingo** (o *flick* que arremessa gotas na direção do movimento) — que é a §3.4 (N
cópias) com a §1.2 (velocidade) como direção. **Duas peças já propostas se combinam nesta terceira**
sem código novo de motor.

### 3.8 Reconhecimento de forma — *QuickShape*

Procreate: *"draw a line or shape and **keep your finger held** on the canvas. After a second,
QuickShape will invoke automatically and your stroke will 'snap' into a perfect line, arc,
poly-line, ellipse, triangle or quadrilateral."* E o segundo dedo torna a elipse um círculo. O CSP
resolve por modificador (Shift), o que é a versão pobre — o gesto do *segurar* é o que os artistas
citam.

**O que já temos:** os shape editors (Line/Arc/Ellipse/Polygon/FreeHand) e o **refit Schneider com
corner-split** (`curve_refit.rs`), que é a metade difícil de *reconhecer*. O que falta é o
**classificador** (*este traço é um círculo? um retângulo?*) e o **gatilho de segurar**.

**Wave:** média, e é a que mais muda a sensação do app no dia a dia.

### 3.9 Assistentes e guias

Krita *Assistants* (ponto de fuga 1/2/3, elipse, régua paralela, concêntrica, perspectiva de peixe),
Procreate *Drawing Guides* + modo **Assisted**, CSP *réguas de perspectiva*. E o Krita ainda expõe
um **sensor `Perspective`**, isto é: o assistente não só restringe — ele **modula o pincel**.

**O que já temos:** o snap do módulo Vector (alinhamento 1-D, reivindicação 2-D sobre curvas, guias,
réguas — integrado 2026-08-02) e a simetria do Painter. ⚠️ **O snap do Vector NÃO alcança o
Painter** hoje.

**Wave:** média-grande, e é decisão de produto onde ela mora (o Vector já tem o motor; o Painter tem
o gesto).

### 3.10 Modificadores de PROCESSO — a metade do Alchemy que ninguém copia

Os módulos *affect* do Alchemy não mudam o traço, mudam **o ato de desenhar**:

- **Blindness** — *"stops the canvas display being updated"* (+ Redraw / Autoredraw). Desenhar às
  cegas, o exercício clássico de ateliê.
- **Limit** — *"limits the total number of shapes on the canvas at any one time … first in, first
  out, removing older shapes first"*. O canvas que **esquece**.
- **Repeat** — *"duplicates shapes on rollover"*, com intervalo.
- **Random** — *"when rolling over a shape, the area around the mouse will have its points
  randomised"*.
- **Smooth** — suaviza no rollover (com um botão *Repeat* que **acumula os passos como formas
  novas**).
- **Colour Switcher** — cor/transparência aleatórias por traço, com cor-base e faixa.
- **Mirror** e **Gradient** — nós já temos os dois (Symmetry; e o gradiente por-traço é o
  `ramp_alpha`/stamp ramped).

⚠️ **É a categoria mais barata do relatório inteiro e a mais estranha ao nosso app.** Ela existe
porque o Alchemy tem uma tese — *"Alchemy is not about making an image, it's about the process"*,
e o app **nem tem undo nem salva raster**. Adotar Blindness/Limit num editor de produção é uma
decisão de PRODUTO, não de engenharia: são ferramentas de **ideação**. Trago porque o pedido diz
*"bem quistos pelos artistas"* e estes são citados em toda entrevista sobre Alchemy — mas não os
recomendo antes das outras.

### 3.11 Os módulos do Alchemy que valem como ideia solta

- **X Shapes** — *"hard and sharp edged shapes that move erratically according to the **line
  duration and pen speed**"*. Outra vez: velocidade.
- **Median Shapes** — *"draws a 'median' line between the last line and current line"* — o traço que
  responde ao anterior. Barato e distinto.
- **Detach Shapes** — a forma nasce **longe** do cursor (slider de distância). É o Speed Shapes com
  offset constante em vez de velocidade.
- **Inverse Shapes** — *"draws like a regular pen except inverted. When the pen is up it draws, and
  when down it does not."* Uma piada que artistas usam a sério.
- **Scrawl Shapes** — rabisco procedural (*Flow · Detail · Noise*).
- **Mic Shapes** / **Mic Expand** — a voz controla espessura (*Fatten*) ou desloca a linha
  (*Shake*). ⚠️ Nós temos um **módulo de áudio inteiro** (42 efeitos, FFT, streaming) — a ponte
  microfone→pincel custaria quase nada e é o tipo de coisa que rende vídeo.
- **Trace Shapes** — baixa uma imagem aleatória do Flickr, esconde atrás do canvas, e a caneta
  **gruda nas bordas dela** (*Snap Distance · Tolerance*). A ideia reaproveitável não é o Flickr, é
  a **caneta que gruda nas bordas de uma imagem de referência** — que, para um app de pintura, é
  desenho assistido sobre foto.

### 3.12 Generativo — e por que este item é diferente

Flow fields (campos de ruído que dirigem partículas), *differential growth*, *space colonization*
(o algoritmo de venação/árvore por atratores), *stippling* por Voronoi ponderado / Lloyd,
reaction-diffusion, *circle packing*. É o vocabulário inteiro da arte generativa contemporânea.

⚠️ **Nós já temos quase todos, e não no Painter:** o Motion Nodes shipa curl noise divergence-free,
boids, verlet, soft-body, **voronoi por JFA**, distribuições, deformers na GPU e a grade espacial —
com censo de cobertura e paridade CPU×GPU gateadas. O `ph2d-sdf` do 3D tem EDT exata e Surface Nets.

⇒ **O item generativo não é "escrever os algoritmos". É a PONTE**: um traço do Painter que semeia um
campo do Motion, e um campo do Motion que deposita tinta no canvas do Painter. É uma decisão de
arquitetura cross-módulo (e provavelmente ADR), não uma wave de pincel.

### 3.13 Rough / *hand-drawn* — o traço que finge ser à mão

`rough.js` (o traço do Excalidraw): toda primitiva é redesenhada 2× com deslocamento pseudo-aleatório
e curvas ligeiramente tortas. É o efeito mais reconhecível da década em diagramas.

**O que já temos:** os shape editors + o `jitter`. Um modo "Rough" nos métodos Line/Ellipse/Polygon
seria **muito** barato e daria uma família visual inteira.

---

## 4. Recomendação

Ordenada por **valor ÷ custo**, com o motivo de cada posição.

| # | Item | Por que aqui |
|---|---|---|
| **1** | **A entrada `Speed`** (§1.2) + o canal de dinâmica que a acompanha (§2) | A entrada mais barata que existe (o mouse a dá), já medida e descartada pelo estabilizador, e **desbloqueia** Speed Shapes, Displace, X Shapes, Distance-Opacity do Sketchy e o respingo do Splatter. É uma entrada, não uma feature. |
| **2** | **Style: Solid** (§1.1) | Pedido explícito, e o levantamento mostra que a região preenchida **já é computada e jogada fora** pelo boolean. Entregue como `StrokeMethod`, herda os shape editors. |
| **3** | **Sketchy / neighbour points** (§3.1) | O traço procedural mais amado do mundo, e o único que **não precisa de entrada nem de geometria nova** — só da memória do traço, que já existe. Traz o Curve/history (§3.2) quase de graça atrás dele. |
| **4** | **Spray / contagem** (§3.4) | Wave pequena no `walk_dab` (herda Symmetry/Tiling/shape editors de graça) e é o multiplicador do Splatter (§3.7) e do Hatching (§3.3). |
| **5** | **QuickShape** (§3.8) | O que mais muda a sensação diária; metade difícil (refit) já shipa. |
| **6** | A **matriz** completa (§2) — mais alvos, curva por canal | Depois de 1 e 3, quando houver duas entradas e três alvos, o caso especial começa a doer. Fazer antes é infra sem consumidor. |

**O que NÃO recomendo agora, com o motivo:**

- **Pressure Shapes como wave do Painter** — a feature existe e o **dispositivo** não. O trabalho
  real é ligar a caneta, e ele é **cross-line, classe ADR** (subir o winit) ou plataforma nova. Vale
  abrir, mas como decisão do Enio sobre a fundação, não como feature de pincel. E quando abrir, a
  primeira coisa a fazer é **recalibrar** os sliders que hoje nunca foram exercidos (o gate diz
  isso).
- **Assistentes de perspectiva no Painter** (§3.9) — o motor de snap já existe no Vector; construir
  um segundo no Painter é a segunda porta para a mesma pergunta. É wave de **ponte**, e é decisão de
  produto.
- **Blindness / Limit / Auto-Clear** (§3.10) — ferramentas de ideação num app que tem undo, save e
  camadas. Baratas, mas mudam a tese do produto; só com pedido.
- **Generativo** (§3.12) — não é falta de algoritmo, é falta de ponte entre módulos. ADR antes de
  código.

---

## 5. O que MEDIR antes de qualquer uma virar wave

> ✅ **MEDIDO em 2026-08-12** — as três tabelas, com os números, estão no
> [plano 38 §4 W0](38_plano_linha_procedural.md). Em resumo: a velocidade é **arco por tick** (o
> deslocamento por evento varia **73×** para o mesmo gesto) · o **Solid sai mais barato que o Line**
> (ele pula **53%** do composite e os 3 393 pontos de contorno) · e o Sketchy é orçado pelo
> **comprimento** de fio, que a densidade 1,0 põe em **~50× o arco do traço**.

A §0 do `CLAUDE.md` manda medir antes de limitar, e há três números que decidem o desenho:

1. **A velocidade que o nosso `on_canvas_pointer` de fato entrega.** O `stabilize.rs` filtra
   *tremor* — a taxa de eventos e a distância entre eles variam com o polling do mouse, e §5.48 do
   doc 28 já mediu que **o custo é por DAB, não por evento** (o mesmo caminho de 640 px custa o
   mesmo em 16 ou em 640 eventos). ⚠️ Uma entrada `Speed` derivada de *evento a evento* herda a taxa
   de polling; a que sobrevive é derivada do **arc-length por tempo de relógio**, como o
   `arc_len` que o Shape Flow já carrega no `Dab`. **Medir antes de escolher a fórmula.**
2. **O custo do Solid contra o boolean de hoje** — o doc 35 diz que o traçado é 74% dos 6,4 ms numa
   elipse. Se `Solid` pula o traçado, ele pode ser **mais barato** que o Line. Ninguém sabe: medir
   pela porta do produto (`measure_what_the_boolean_is_made_of` é o instrumento que já existe).
3. **Quantos dabs um Sketchy emite por evento.** É a família que pode estourar o orçamento sem
   avisar (o Krita ship um *Simple Mode* justamente para pincel grande). O número tem de sair da
   sonda antes do slider ter faixa — senão o teto vira palpite, que é o que a §0 proíbe.

---

## Fontes

- [Alchemy — al.chemy.org](https://al.chemy.org/) · [Features](https://al.chemy.org/features/) ·
  [karldd/Alchemy (GPL)](https://github.com/karldd/Alchemy)
- Manual do Alchemy (FLOSS Manuals) —
  [Create Modules](https://archive.flossmanuals.net/alchemy/ch014_create-modules.html) ·
  [cópia integral](https://raw.githubusercontent.com/jamalmazrui/FLOSSMan/master/Alchemy.htm)
- Krita Manual — [Brush Engines](https://docs.krita.org/en/reference_manual/brushes/brush_engines.html) ·
  [Sketch](https://docs.krita.org/en/reference_manual/brushes/brush_engines/sketch_brush_engine.html) ·
  [Hatching](https://docs.krita.org/en/reference_manual/brushes/brush_engines/hatching_brush_engine.html) ·
  [Spray](https://docs.krita.org/en/reference_manual/brushes/brush_engines/spray_brush_engine.html) ·
  [Particle](https://docs.krita.org/en/reference_manual/brushes/brush_engines/particle_brush_engine.html) ·
  [Sensors](https://docs.krita.org/en/reference_manual/brushes/brush_settings/tablet_sensors.html)
- [mrdoob/harmony](https://github.com/mrdoob/harmony) — o Sketchy, creditado ao *Scribbler* do Ze Frank
- [Procreate Handbook — QuickShape](https://help.procreate.com/procreate/handbook/guides/quickshape)
- [morphogenesis-resources](https://github.com/jasonwebb/morphogenesis-resources) — o catálogo de
  referência de *differential growth*, *space colonization*, reaction-diffusion e afins
