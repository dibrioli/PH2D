# 38 — Plano: o card **Line**, o Style Line/Solid e os traços procedurais

> Ordem do Enio, 2026-08-12: *"crie um plano de implementação e salve o plano. Quero essas novas
> features dentro de um card acima de Composite Brush. Dentro do card um checkbox para Style —
> Line/Solid e um Dropdown com cada uma das opções de traço/linha. Ao escolher um tipo de traço,
> sliders de ajustes específicos para aquele tipo aparecem no card. Por padrão o dropdown é none."*
>
> A pesquisa que decide **quais** tipos entram é o doc [37](37_pesquisa_tracos_procedurais.md).
> Este doc é o **como**: a forma exata do card, onde cada coisa mora, as waves em ordem de custo, os
> gates de cada uma, e o que é decisão do Enio em vez de engenharia.
>
> ⚠️ **Clean-room:** o Alchemy é GPL-3 e o Krita também. Tudo aqui é **comportamento**, lido de
> manual — nenhuma linha de fonte de nenhum dos dois foi lida, e nenhuma será.

---

## 1. A forma do card (o que o Enio pediu, desenhado)

O card fica **imediatamente acima do Composite Brush** — em `paint_brush.rs`, entre o `4b′` (Inpaint)
e o `4c` (Composite).

```
┌─ Line ───────────────────────────────────┐
│  ☐ Solid                                 │   ← Style: desmarcado = Line, marcado = forma sólida
│  Type   [ None            ▾ ]            │   ← default None
│                                          │
│  … as rows DO TIPO escolhido …           │   ← nada aqui enquanto Type = None
└──────────────────────────────────────────┘
```

Com **Type = Sketchy**, por exemplo:

```
┌─ Line ───────────────────────────────────┐
│  Type   [ Sketchy         ▾ ]            │
│  Reach        ▬▬▬▬▬▬●▬▬▬   0.35          │
│  Density      ▬▬▬●▬▬▬▬▬▬   0.20          │
│  Line Width   ▬▬●▬▬▬▬▬▬▬   0.12          │
│  Opacity      ▬▬▬▬●▬▬▬▬▬   0.30          │
│  ☑ Magnetify                             │
└──────────────────────────────────────────┘
```

**Três leis de forma, e cada uma tem precedente executável neste repo:**

1. **O card pinta SÓ as rows do tipo escolhido.** É o `each_kind_paints_only_the_rows_it_uses` da
   §12 da física e o `knob_family()` do card do Sculpt. Row de outro tipo pintada aqui é knob morto.
2. **O checkbox `Solid` só é pintado onde o tipo tem caminho fechado para preencher**
   (`LineKind::honours_style()`). Sketchy/Wire/Spray produzem *muitos fios curtos*, não uma
   silhueta — oferecer Solid neles seria um checkbox que não faz nada. Ver §5.1: isto é uma decisão
   e não um detalhe.
3. **O dropdown não é pintado enquanto tiver uma opção só.** Na W1 o card tem apenas o checkbox; o
   dropdown nasce na W2, junto com o primeiro tipo. Um dropdown de uma linha é o mesmo controle
   morto com outra roupa.

---

## 2. Onde cada coisa mora (levantado por `grep`, não de memória)

| O quê | Onde | Nota |
|---|---|---|
| O card | **arquivo novo** `crates/ph2d-panel-painter-layers/src/paint_line.rs` | molde: `paint_composite.rs` (178 LOC, card com borda própria) — ⚠️ `paint_brush.rs` está em **611**, o card não cabe lá |
| A chamada | `paint_brush.rs`, entre o `4b′` e o `4c` | uma linha |
| Os ids | `ph2d-editor-core/src/ids/chrome/painter.rs` | **`hash_node_id("painter_line.…")`** ⇒ **nenhum gate de contagem**, nenhum id numérico a alocar |
| O registro | `populate.rs` (499) | sem ele o widget pinta, registra hit e fica **morto sob o mouse** |
| A rota do Click | `event.rs` (567) | checkbox + dropdown |
| A rota do slider | `event_brush_forward.rs` | a whitelist `is_forwardable_brush_slider` |
| O estado | `BrushSpec` (`ph2d-painter-brush/src/spec.rs`, 445) | ⚠️ **`BrushSpec` não é serde** ⇒ **zero `PROJECT_SCHEMA`**, zero save antigo afetado |
| O espelho do painel | `BrushSettings` (`brush_settings.rs`, 581) | o snapshot que o card lê |
| O emissor de dabs | `ph2d-painter-brush/src/stroke.rs` — **691/700** | ⚠️ todo produtor novo nasce em **módulo irmão**, obrigatoriamente |

**Estado novo, num bloco só:**

```rust
pub enum LineKind { None, Speed, Sketchy, Wire, Spray }   // discriminantes de wire 0..4

// em BrushSpec:
pub style_solid: bool,      // false = Line (o mundo de hoje), true = forma sólida
pub line_kind: LineKind,    // None por default
pub line: LineParams,       // TODOS os params de TODOS os tipos, num struct só
```

⚠️ **`LineParams` é um struct ÚNICO com neutro, nunca um struct por tipo.** É o molde do
`KernelResolver`/`SkinParam` e dos canais de side-metadata do registry: um tipo que não consome um
campo não sabe que ele existe, e `LineParams::default()` é o mundo de hoje **byte a byte**. Um enum
com payload por variante espalharia `match` por todo sítio de construção e faria o tipo N+1 tocar
todos eles.

---

## 3. As três leis que toda wave obedece

1. **O neutro é BYTE-IDÊNTICO.** `line_kind == None && !style_solid` ⇒ o depósito produz os mesmos
   bytes de hoje. Não é promessa: é **gate de fingerprint** rodado em toda wave, e é a rede que
   torna cada uma reversível.
2. **Uma porta.** Os tipos que **moldam** um dab entram no `walk_dab` (onde Symmetry, Tiling, shape
   editors, pressão, Jitter, Shape e Grain já se penduram — foi assim que o smear field herdou tudo
   de graça). Os que **emitem dabs a mais** entram no emissor, **antes** do `push_symmetric`, para
   que as cópias de simetria espelhem os fios e não só o eixo. Isto é gate, não comentário.
3. **Medir antes de limitar (§0 do `CLAUDE.md`).** Nenhum slider ganha faixa sem a sonda ao lado.
   Um teto que só diz "por segurança" é um palpite esperando um smoke.

---

## 4. As waves

### W0 — as três medições (nenhuma mudança de produto)

O doc 37 §5 nomeia três números que **decidem o desenho** e que ninguém tem. Sonda em
`crates/ph2d-tool-painter/src/tool/paint/line_probe.rs`, `#[ignore]`, `--test-threads=1`:

1. **Qual fórmula de velocidade sobrevive ao polling.** O doc 28 §5.48 já mediu que *o custo é por
   DAB, não por evento* — o mesmo caminho de 640 px custa o mesmo entregue em 16 ou em 640 eventos.
   ⚠️ Uma velocidade derivada **de evento a evento** herda a taxa de polling do mouse e faz o mesmo
   gesto desenhar diferente em duas máquinas. A candidata que sobrevive é **arc-length por relógio
   de parede** — e o `Dab` já carrega `arc_len` (o Shape Flow o pôs lá). **Medir as duas** com o
   mesmo caminho a 1 px e a 8 px de espaçamento; a que ficar plana é a certa.
2. **O custo do Solid contra o Line de hoje.** O doc [35](35_boolean_o_que_o_vector_ensina.md) §1.1
   mede que o **traçado é 4,777 dos 6,448 ms** de uma elipse de 200 px. Se `Solid` para antes do
   traçado, ele pode sair **mais barato que o Line**. Instrumento: o
   `measure_what_the_boolean_is_made_of`, que já existe.
3. **Quantos dabs um Sketchy emite por evento.** É a família que estoura orçamento sem avisar — o
   Krita ship um *Simple Mode* justamente para pincel grande. O número sai da sonda **antes** de o
   slider ter faixa.

**Entrega:** três tabelas, com o número ao lado. Zero produto. **FEITA em 2026-08-12** — as sondas
são `ph2d-painter-brush::line_probe` (as duas de motor) e
`ph2d-tool-painter::tool::paint::line_probe` (a da figura), todas `#[ignore]`.

#### W0.1 — a fórmula de velocidade: **arco por TICK**, e a medição é categórica

O MESMO quarto de círculo (raio 200, 314 px de arco), entregue em 8/32/128/512 eventos:

| eventos | desloc/ev | razão | dabs | arco/dab | arco/quadro | razão |
|---|---:|---:|---:|---:|---:|---:|
| 8 | 44,786 | 1,00 | 64 | 4,800 | 37,800 | 1,00 |
| 32 | 10,133 | 0,23 | 66 | 4,800 | 39,000 | 1,03 |
| 128 | 2,474 | **0,06** | 66 | 4,800 | 39,000 | 1,03 |
| 512 | 0,615 | **0,01** | 66 | 4,800 | 39,000 | 1,03 |

⚠️ **O deslocamento por evento varia 73× para o MESMO gesto** — ele descreve o dispositivo, não a
mão. O arco por quadro fica plano. E o **arco por dab é 4,800 constante**: no método `Space` os dabs
saem a intervalos fixos de arco, então essa distância **não carrega informação de velocidade
nenhuma** — está medido para ninguém a propor de novo.

⚠️ **Os 3% residuais da primeira linha são o ESTABILIZADOR, atribuídos por ablação pelo knob:** com
`stabilizer = 0` a coluna fica em **1,00 nas quatro densidades**. É um resíduo do *caminho*, que
nenhuma fórmula de velocidade remove — e são 3% contra 73%, três ordens de grandeza de distância, então
a escolha não muda.

⚠️ **E o relógio já existe sem tocar contrato congelado:** o `Tool` recebe `on_tick(dt_ms: f32)` e o
`Stroke` tem `tick(dt, out)`. O `CanvasPointer` **não** carrega timestamp e é da superfície
`CanvasPaintTool`, congelada no ADR-0040 emenda 3 (§6) — acrescentar um campo ali seria Coord-only +
ADR, e a medição diz que não é preciso.

#### W0.2 — o Solid é MAIS BARATO que o Line, e o número é 53%

Ellipse de 200 px, Digital, canvas 4096, um move (o composite roda a cada move):

| formas | converte | rasteriza | **PAGA** | traça | **PULA** | células | pts |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0,000 | 1,555 | **1,555** | 1,778 | **53,3%** | 1 440 000 | 3 393 |
| 2 | 0,000 | 3,083 | **3,083** | 3,141 | 50,5% | 2 592 000 | 5 345 |
| 4 | 0,000 | 4,821 | **4,821** | 3,997 | 45,3% | 3 319 200 | 6 725 |

E o que o Solid **acrescenta** (reduzir a máscara `SS=3` a cobertura + compor) sobre a MESMA janela
de 1,44 M células: **0,423 ms** (0,260 reduzir + 0,162 compor). ⚠️ *Esta é a única coluna estimada da
W0 e está marcada como tal* — a peça não existe, o que se mediu foi um laço equivalente sobre uma
janela do tamanho que o composite de fato produziu.

⇒ **Solid ≈ 1,98 ms contra Line ≈ 3,33 ms + os 3 393 pontos de contorno que ainda viram dabs.** A
previsão da §5 do doc 37 está **confirmada**: parar antes do traçado sai mais barato que traçar.

#### W0.3 — o orçamento do Sketchy, e a grandeza que orça não é a contagem

Quarto de círculo, 128 eventos, **todo par** dentro do alcance (densidade 1,0):

| raio | arco | dabs | reach | fios | fios/dab | compr(px) | compr/arco | dens@2× |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 12 | 312 | 131 | 1 diâm | 1 125 | 8,59 | 16 051 | **51,4** | 0,039 |
| 24 | 312 | 66 | 1 diâm | 540 | 8,18 | 15 244 | 48,9 | 0,041 |
| 48 | 307 | 33 | 1 diâm | 243 | 7,36 | 13 352 | 43,5 | 0,046 |
| 96 | 307 | 17 | 1 diâm | 99 | 5,82 | 10 062 | 32,8 | 0,061 |
| 24 | 941 | 197 | 4 diâm | 6 864 | 34,84 | 666 762 | **708,7** | 0,003 |

⚠️ **`fios/dab` é ~8 no alcance de um diâmetro, seja qual for o pincel** — o alcance escala com o
pincel, então a lei é **livre de escala**. Bom: o slider não precisa de calibração por tamanho.

⚠️ **Mas quem orça é o COMPRIMENTO, não a contagem:** a um diâmetro os fios somam **~50× o arco do
traço**; a quatro diâmetros, **~700×**. Um Sketchy de densidade 1,0 pinta cinquenta vezes o que a
mão desenhou. ⇒ **a `Density` não é enfeite, é o orçamento**, e o teto sai derivado: para manter o
gasto em 2× o traço, a densidade fica em **~4% no alcance de um diâmetro** e **~0,3% em quatro**.

⚠️ **E isto decide a ARQUITETURA do W3:** com 16 mil px de fio por traço curto, um fio **não pode
ser uma corrente de dabs** (a spacing de sub-pixel de um fio fino põe a conta em 10⁴–10⁶ dabs). Ele
tem de ser um **segmento rasterizado**, que é como o Harmony e o Krita o desenham.

---

### W1 — o CARD e o **Style: Solid**

**Entrega.** O card **Line** acima do Composite Brush, com o checkbox `Solid` **vivo**: marcado, o
traço deixa de ser a área varrida pelo pincel e passa a ser a **área cercada pelo gesto**, preenchida
— e a espessura deixa de entrar (o pedido do Enio, e o `Style` do Alchemy: *"toggle between drawing
with a line or solid fill"*).

**Onde o motor se pendura.** O caminho fechado só existe onde há caminho autorado: os métodos
`Line`/`Arc`/`Ellipse`/`Polygon`/`FreeHand` (que já re-carimbam a figura inteira por frame via
`restamp_shapes_preview`) e o `Space`, cujo caminho amostrado se fecha no pen-up. ⇒ **duas rotas, a
mesma porta de escrita.**

⚠️ **A rota barata é parar antes do traçado.** O `stroke_boolean_contours` hoje rasteriza → flood por
componente → **traça (Moore)** → polilinha densa → dabs. A região preenchida **já está em mãos** no
meio disso e é jogada fora. `Solid` = consumir o bitmap e escrever a região.

**As duas decisões da wave estão RESPONDIDAS** (§5.2 e §5.3): a borda é **da forma, com cobertura
exata por área na banda de borda** (`SS = 3` erra > 8/255 em **51% da borda** — medido), e um gesto
aberto **fecha sozinho**, sem limiar. E a W0.2 mediu que **o Solid sai mais barato que o Line**
(1,98 ms contra 3,33 + os 3 393 pontos de contorno que ainda viram dabs).

**Toca:** `paint_line.rs` (novo) · `paint_brush.rs` (+1 linha) · `populate.rs` · `event.rs` ·
`ids/chrome/painter.rs` (`PAINTER_LINE_SOLID`) · `BrushSpec.style_solid` · `BrushSettings` · a porta
de escrita de canvas (`fork_par` + **`declare_wrote`** — sem declarar a janela o commit de undo
varre o canvas inteiro, doc 28 §5.17).

**Gates.** (a) o card é pintado **acima** do Composite (arch-gate de ordem sobre o fonte — a
posição é o pedido) · (b) o checkbox está **registrado e vivo sob o mouse** (seam com `click_at`
REAL, nunca `WidgetEvent` sintético — a lição das 36 células do W2c da física) · (c) `!style_solid`
é **byte-idêntico** (fingerprint) · (d) um laço fino e solto pinta uma região **maior que a área
varrida** (o oráculo é a APARÊNCIA: contar texels entintados dentro do laço, não a regra) · (e) a
espessura **não muda nada** em Solid (dois raios, mesmos bytes) — é literalmente a frase do pedido,
executável. **Mutações:** trocar a região pelo rastro ⇒ (d) sangra · ignorar `declare_wrote` ⇒ o
gate de janela do undo sangra.

**Smoke `PH2D_LINE_SMOKE=1`:** um laço aberto e um laço fechado, Line e Solid lado a lado; e o
**CONTROLE** — com o checkbox desmarcado o traço tem de ficar exatamente como hoje.

#### W1c — a família dos SHAPE EDITORS (fechada 2026-08-13)

⚠️ **A "rota barata" que esta wave prescrevia — *consumir o bitmap do `stroke_boolean_contours`* —
foi DESCARTADA na construção, e o motivo é o W1a:** o `stroke_boolean_contours` rasteriza a `SS = 3`,
faz flood por componente e **traça** o contorno de volta, e a W0.3 mediu que `SS = 3` erra > 8/255 em
**51% da borda** e custa **14× a cobertura exata**. Consumir o bitmap dele seria pagar caro por uma
borda pior do que a que o `fill_coverage` já dá.

**O que shipou é mais curto:** um shape editor **já É um caminho autorado**, então `shape_loop` (em
`solid_shapes.rs`) devolve o laço que cada forma desenha — pelos **MESMOS produtores de geometria que
o carimbo de dabs usa** (`offset_curve_spine` · `ellipse_perimeter` · `polygon_perimeter` ·
`line_corner::expand`+`offset_polyline`+trim) — e os laços vão direto ao `fill_coverage`. Nenhum
raster intermediário, nenhum traçado, nenhuma segunda ideia de *onde a forma está*.

⚠️ **E a OPERAÇÃO booleana saiu de graça, pelo SENTIDO do laço:** o preenchimento é `nonzero`, então
orientar `Add`/`Overlay` num sentido e `Remove` no oposto **é** a união-menos-subtração — um `Remove`
vira buraco porque a soma corrida volta a zero ali. Sem motor booleano, sem `SS`, sem flood.

⚠️ **Uma chamada de `fill_coverage` para N laços, e é CORREÇÃO e não economia:** preenchê-los um a um
comporia a borda anti-aliased **duas vezes** na sobreposição e deixaria uma linha escura exatamente
onde duas formas se tocam.

**Gates** (`solid_deposit_tests.rs`): a elipse em Solid é um **disco** e não um anel (área medida
contra π·r², e a razão contra o contorno) · a espessura **não entra**, com o CONTROLE em Line onde
ela tem de entrar · um `Remove` **abre buraco**. **Mutações: 2, as 2 sangram** — neutralizar o desvio
no funil mata os dois primeiros (e deixa os do gesto à mão livre VERDES, porque são outra porta) ·
fixar o sentido em CCW mata só o do buraco.

⚠️ **E a 1ª fixture do buraco media UMA forma achando que media duas:** um Down dentro de uma figura
que já existe **REATIVA** aquela figura em vez de começar outra (`stroke_router`) — medido,
`parked = 1 → 0` e `loops = 1`. A fixture passou a montar as duas formas como estado parqueado, o
padrão que os gates booleanos desta crate já usam.

---

### W2 — o DROPDOWN e o primeiro tipo: **Speed**

**Entrega.** O dropdown `Type` nasce (`None` default + `Speed`), e com ele a **entrada de
velocidade**, que é o desbloqueio de metade do doc 37.

**O que o tipo faz.** *"Accentuates the pen speed to create shapes that throw the line beyond the
actual pen position"* — o dab pousa em `p + v·k`, então o gesto rápido arremessa a linha muito além
do dedo. É **inércia**: o oposto exato do estabilizador, que já mora ao lado (`stroke/stabilize.rs`)
e que **já mede a velocidade e a descarta** (ele guarda a posição filtrada e a crua lado a lado; a
diferença entre elas é o número).

**Rows do tipo:** `Amount` (quanto da velocidade é aplicada) e `Curved` (o *Line Type* do Alchemy:
retas × curvas).

⚠️ **A fórmula é `Δarco / dt_ms` no TICK, e a W0.1 a escolheu por medição:** o deslocamento por
evento varia **73×** para o mesmo gesto (é o dispositivo), o arco entre dabs é **constante por
construção** (zero informação), e o arco por quadro fica plano. O relógio já existe
(`Tool::on_tick(dt_ms)` / `Stroke::tick`) e **não toca contrato congelado** — o `CanvasPointer` não
carrega timestamp e é superfície congelada. E ⚠️ **a velocidade nasce como CAMPO do frame do
dab, não como número solto**: o próximo tipo (Sketchy) a lê para o *distance-opacity*, e o Splatter
futuro a lê para a direção do respingo. Duas leituras da mesma grandeza por duas fórmulas é a falha
de duas-portas que este módulo já pagou quatro vezes.

**Gates.** O tipo NEUTRO ignora o relógio (byte-idêntico) · o mesmo caminho a 1 px e a 8 px de
espaçamento produz o **mesmo** arremesso (a lei do caminho, não da amostragem — a doença que este
módulo curou quatro vezes no relevo) · a antecipação é um TEMPO (quadruplicar a velocidade
quadruplica o arremesso). **Mutação:** velocidade por evento ⇒ o gate de espaçamento sangra.

---

#### W2 — FECHADA (2026-08-13), e as DUAS reprovações de smoke que a reescreveram

**A primeira** (*"speed não é igual o Alchemy"*): a magnitude estava certa desde o começo (1 598 px
de arremesso num chicote — o *"off the screen"* do manual), e o defeito era a CONTINUIDADE. A
velocidade é medida por TIQUE ⇒ ela é uma ESCADA, e o arremesso saía como uma fileira de arcos
deslocados, um por quadro (vão de **25× o passo nominal**). Curada por uma rampa sobre o arco do
próprio quadro — hoje uma **EMA ponderada por comprimento**, a mesma lei do `heading`.

**A segunda** (*"não é idêntico na FORMA e ainda fica pontilhado"*, com a foto) tinha **duas causas
independentes**, e nenhuma era a que a primeira consertou:

1. **PONTILHADO — o arremesso ESTICA o caminho da tinta e ninguém re-espaçava.** O motor emite um dab
   a cada `spacing × diâmetro` do caminho da **MÃO**; entre dois dabs a tinta anda `passo × (1 + Δv/v)`,
   e um gesto de artista (que acelera na reta e freia na curva) chega facil a 5×. Medido num pincel de
   raio 4: vão de **1,61 diâmetro** com a **contagem de dabs IDÊNTICA** à do traço sem arremesso — os
   mesmos dabs espalhados por um caminho mais longo. **O Alchemy não tem este problema porque o traço
   dele é um `GeneralPath` PERCORRIDO**: uma curva traçada é contínua por construção. O equivalente
   num motor de dabs é honrar o espaçamento **onde a tinta cai**, e é o que o `fill_thrown_gap` faz
   (vão **1,61 → 0,12 diâmetro**, idêntico ao controle em todo raio).
2. **FORMA — o arremesso é uma ALAVANCA.** ⚠️ A primeira hipótese (a escada da velocidade) foi
   **REFUTADA por medição**: suavizá-la levou a razão de virada de 25,1 para 24,4× — *quase nada*.
   Olhando ONDE a virada estava, não era quina de quadro: era **ondulação de ±5 px numa reta**. O
   arremesso desloca a tinta por `k` px ao longo do heading, e o heading do **Rake** é suavizado sobre
   `0,1 × diâmetro` — **1,6 px**, o certo para girar um carimbo que tem de ACOMPANHAR o traço. Com
   `k = 470`, o tremor residual de **0,6°** vira ±5 px. A cura é a **MIRA com janela própria, longa
   como o próprio arremesso** (`L = k`): o erro lateral é `k·Δθ`, então para ele ficar abaixo de uma
   fração do diâmetro o `Δθ` tem de encolher como `1/k`. Razão **24,4× → 0,9×**, contra um controle
   de 6,8° — a tinta passa a virar *menos* que a mão, que é o que uma curva maior tem de fazer.

⚠️ **E o `Amount` MORREU** (Enio: *"em alchemy o slider não é necessário"*): a antecipação é UMA
constante de tempo (`SPEED_LOOKAHEAD_S = 1/10 s`), e um tempo **auto-escala** — a mesma constante
arremessa 20 px num traço lento e 400 num rápido, que é precisamente a lei que o `Speed Shapes`
afirma. Sumiram o campo, o setter, o id, a row, o `populate` e a rota de `SetValue`. ⚠️ **É a única
calibração que sobrou, e ela é decisão de LOOK: o SMOKE é quem a julga**, e movê-la é uma linha.

⛔ **`Curved` não foi construído, e não é adiamento:** num motor de dabs não existe *segmento*, então
"retas × curvas" não tem referente — qualquer significado seria inventado. O `Line Type` do Alchemy
governa o `GeneralPath` dele, que tem segmentos de verdade.

**9 gates · 8 mutações · 7 sangram** (a sobrevivente é a janela da EMA de velocidade, **medida** e
documentada: depois que a mira passou a fazer o trabalho pesado, o valor dela deixou de ser
observável nos regimes medidos).

**Smoke `=2`:** o mesmo gesto devagar e depressa; e um traço lento tem de ser indistinguível do
`None`.

#### O que a construção MUDOU no plano (fechada 2026-08-13)

**A `Curved` NÃO foi construída, e é decisão, não esquecimento.** O manual do Alchemy oferece *"Line
Type — toggle between drawing straight lines and curved lines"*, e ali a Speed Shapes produz uma
FORMA (um caminho fechado) cujos segmentos são retos ou curvos. **Num motor de dabs não há
segmento** — o traço já é um contínuo de carimbos —, então qualquer significado que eu desse a
`Curved` seria INVENTADO. Um knob cujo comportamento a referência não determina é um knob que mente;
ele volta no dia em que houver uma pergunta de produto por trás dele.

⚠️ **A velocidade NÃO virou campo do `Dab`, e o plano dizia que sim.** A justificativa era o consumo
futuro (Sketchy, Splatter), mas um campo com **um consumidor interno** e nenhum leitor é um campo
morto que atravessa o `Dab` inteiro. O que a lei exige é *um lugar computa, todos leem*, e isso é
satisfeito por **`Stroke::speed_px_s()`** — a porta que o Sketchy vai ler. O `Dab` fica intacto.

⚠️ **O `SPEED_LOOKAHEAD_S` é MEDIDO, e é um TEMPO.** A W0.1 mede ~39 px de arco por quadro num gesto
ligeiro de quarto de círculo ⇒ **~2 340 px/s**. A janela é **um quadro de 60 fps**, então `Amount = 1`
arremessa exatamente o arco de um quadro (~39 px naquele gesto) e o teto — `MAX_SPEED_AMOUNT = 8` —
diz **quantos quadros à frente** a tinta pode ir: 312 px ali, que é o *"and possibly off the screen
itself"* do manual. Acima disso a tinta está mais de um oitavo de segundo à frente da mão.

⚠️ **E o arremesso é ESTRUTURALMENTE inerte na família dos shape editors**, sem um `if` para isso:
cada preenchimento de forma constrói uma `Stroke` **fresca**, e o `speed_px_s` só sai de zero num
TIQUE — a mão nunca correu por ela. Um gesto sólido também o ignora, porque o `solid_path` é o
caminho do PONTEIRO: arremessar a fronteira de uma mancha seria outra feature, e não foi construída.

**Gates** (`line_speed_tests.rs` + `seam_line_card.rs`): a tinta cai adiante do dedo · o arremesso é
invariante ao ESPAÇAMENTO e à TAXA DE EVENTOS · `Amount = 0` e o tipo `None` são byte-idênticos ·
a velocidade é medida ANTES do desvio do Airbrush · o `Amount` conta quadros · o ponteiro escolhe
cada tipo · a row `Amount` existe só com `Speed` · o slider é vivo sob o mouse. **7 mutações, 6
sangram.**

⚠️ **A sobrevivente ACUSOU a minha afirmação, não um buraco** (a 3ª vez nesta linha): eu escrevera
que a ordem *arremesso antes do jitter* era load-bearing. O `apply_jitter` soma um deslocamento que
**não depende da posição**, e o arremesso é outra translação — **duas translações comutam**, então as
duas ordens dão o mesmo ponto ao bit. O comentário foi corrigido em vez de um gate ser escrito.

⚠️ **E TRÊS gates de invariância nasceram VERDES SOBRE A MUTAÇÃO que mata a medição de velocidade** —
com o arremesso em zero, *"dois arremessos iguais"* é verdade de graça. Cada um ganhou uma asserção de
**não-vacuidade** antes da comparação; sem ela eles afirmavam que duas ausências são iguais.

⚠️ **`stroke.rs` estava a NOVE linhas do teto de LOC** (691/700), então esta wave o cruzou por
existir. Dois cortes por responsabilidade: `stroke/speed.rs` (a inércia do gesto — o assunto desta
wave) e `stroke/dab_build.rs` (*como UM dab é construído*, um estágio nomeado do pipeline do
cabeçalho). O pai fica em **652**.

#### E o smoke REPROVOU: *"speed não é igual o Alchemy"* (Enio, 2026-08-13)

⚠️ **A magnitude estava certa e a FORMA não.** A sonda mediu primeiro, antes de qualquer hipótese: o
arremesso alcança **1 598 px** num chicote em `Amount = 8` — *"off the screen"* como o manual promete
—, então *"não vai longe"* estava descartado. O defeito era outro e é medível: num arco rápido (quarto
de círculo r=150 em 6 quadros) o maior **vão entre dabs vizinhos** valia **25× o passo nominal em
`Amount = 1` e 99× em `Amount = 4`**. A tinta não saía como uma linha — saía como uma **fileira de
arcos deslocados e desconectados, um por quadro**.

**A causa:** a velocidade é medida por TIQUE, logo ela é uma **ESCADA** — um valor por quadro,
constante dentro dele, saltando na fronteira. O Alchemy arremessa **por ponto gravado**, e é por isso
que a curva dele é contínua.

⚠️ **E a medida por tique NÃO pode ser abandonada** — é ela que a W0.1 provou ser a única
device-independente (o per-evento varia 73×, o per-dab é constante por construção). O que muda é
COMO ela é aplicada: a velocidade que cada dab cavalga **caminha** da anterior para a nova ao longo do
**arco daquele quadro**.

⚠️ **A rampa não tem constante mágica, e isso está GATEADO:** o comprimento dela é o arco do próprio
quadro, então ela dura *um quadro de percurso* a 300 px/s e a 3 000 px/s igualmente. A mutação que a
troca por uma constante em pixels **sangra** (vão 22,7 contra um diâmetro de 20 no topo da faixa) —
a auto-escala é load-bearing, não decoração.

**Medido depois:** vão/passo **25,5 → 2,0** (`a=1`) · **50,1 → 3,0** (`a=2`) · **99,2 → 5,0** (`a=4`),
e a tinta **continua caindo para FORA do giro** (raio médio 153,6 / 162,7 / 192,8 contra os 150 do
gesto), que é a metade do manual que já estava certa.

⚠️ **O oráculo do gate novo é o DIÂMETRO, não o passo** — o que faz uma linha ser sólida é os dabs se
sobreporem, e um arremesso que estica o caminho **aumenta o passo de propósito** (é a feature). O gate
pergunta o que o olho pergunta: *dá para ver buraco?*

⚠️ **E o acessor público mudou de significado, de propósito:** `Stroke::speed_px_s()` publica a
**MEDIDA** do tique (a grandeza que descreve o gesto, e a que o Sketchy vai ler); a rampeada é privada
e existe só para o arremesso. Duas grandezas, e a que sai pela porta é a que tem nome.

---

### W3 — **Sketchy** (o *neighbour points*)

**Entrega.** O traço procedural mais amado do mundo: guarda-se todos os pontos do traço e, a cada
ponto novo, ligam-se os vizinhos dentro de um raio com fios de opacidade baixa. Sai o hachurado de
lápis / teia que ninguém imita à mão. (Ze Frank → Mr.doob/Harmony → Krita *Sketch*.)

⚠️ **É o único item do doc 37 que não precisa de entrada nova nem de geometria nova** — só da
memória do traço, que já existe (é o que o FreeHand simplifica no pen-up e o que o `taper`
re-percorre no resolve de cauda).

**Rows:** `Reach` (o raio de vizinhança) · `Density` · `Line Width` · `Opacity` · ☑ `Magnetify` (os
fios se atraem para trechos próximos — ligado por default no Krita).

⚠️ **A W0.3 decidiu duas coisas desta wave.** (1) **A `Density` é o ORÇAMENTO, não um enfeite:** com
densidade 1,0 os fios somam **~50× o arco do traço** no alcance de um diâmetro e **~700×** em
quatro; o teto sai derivado — **~4%** e **~0,3%** para manter o gasto em 2× o traço. (2) **Um fio
NÃO pode ser uma corrente de dabs** — 16 mil px de fio num traço curto põem a conta em 10⁴–10⁶ dabs
—, ele é um **segmento rasterizado**, que é como o Harmony e o Krita o desenham. ⚠️ E o `fios/dab`
é **~8 em qualquer tamanho de pincel** (o alcance escala com ele), então a lei é livre de escala e o
slider não precisa de calibração por tamanho.

**Onde se pendura:** o **emissor**, antes do `push_symmetric` (lei 2 da §3) — e ⚠️ em **módulo
irmão**, porque `stroke.rs` está em 691/700.

**Gates.** `Density = 0` byte-idêntico · o fio nasce **entre pontos do MESMO traço** (nunca do
anterior) · sob Symmetry os fios espelham (não só o eixo) · **o orçamento**: dabs por evento sob o
teto que a W0 mediu, com o `Simple Mode` nomeado se o número pedir. **Mutação:** vizinhança global
em vez de por-traço ⇒ o gate do traço anterior sangra.

**Smoke `=3`.**

---

#### W3 — FECHADA (2026-08-13)

**O que shipou:** o motor (a memória, a vizinhança, o orçamento, o canal de saída) + a
**rasterização** dos fios + as **cinco rows** do card + o depósito pela porta de proteção.

**⚠️ O teto da `Density` era `0,04` e a MEDIÇÃO o subiu para `0,40` — dez vezes.** O número antigo
saía de um proxy (`fio/arco`) e o próprio doc dele mandava medir *o tempo de rasterização por evento*
assim que o rasterizador existisse. Medido pela porta `on_canvas_pointer` (espiral de 8 voltas,
r=24, 2048²):

| density | reach | width | ms/evento | pior ms |
|---:|---:|---:|---:|---:|
| 1,00 | 1 | 1 | 0,226 | **0,433** |
| 0,40 | 4 | 4 | 2,508 | **6,384** |
| 0,50 | 4 | 4 | 3,156 | **8,040** ⇠ estoura |
| 1,00 | 4 | 4 | 6,029 | **16,181** |

`0,40` é a maior densidade cujo PIOR evento cabe no kill de 8 ms **com o alcance e a espessura nos
tetos deles**. ⚠️ O preço de um escalar governar um produto de dois (`density × reach²`) está
**nomeado** no doc da constante: no alcance de 1 diâmetro sobra 20× de margem.

**⚠️ Uma razão que eu ia escrever no motor era FALSA e foi conferida:** *"dois fios de sentidos
opostos cancelariam o winding e abririam um buraco no cruzamento"*. Não abrem — inverter um segmento
inverte **a ordem dos vértices E a normal**, então a orientação do quad é invariante ao sentido
(medido: área com sinal `−60` nos dois). O preenchimento único de todos os quads não é *unsound*, é
**chapado** (a regra não-zero satura em 1 e o cruzamento sai igual ao fio solto) — e chapado já basta
para compor `over` por fio, que é o que o acúmulo do hachurado exige. Gate: `reversing_a_thread_paints_the_same_ink`.

**⚠️ E a premissa que o plano deixou escrita envelheceu:** *"só da memória do traço, que já existe (é
o que o `taper` re-percorre no resolve de cauda)"* — o replay do taper foi **REMOVIDO em 2026-08-10**.
A memória nasceu nesta wave.

**O `Magnetify` foi construído ERRADO e corrigido pelo MANUAL** (2026-08-14). A v1 era *"o fio perto
puxa mais"* — uma rampa de opacidade por distância dentro do `ThreadInk`. O manual do Krita a
desmente, verbatim: *"It's what causes curve lines to form **between two close line sections** …
With Magnetify off, the curve line just forms on either side of the **current active portion** of
your connection line."* Ele decide **QUE PARES viram fio**, não com que força um fio desenha ⇒ a lei
mora no MOTOR (`stroke::sketchy`), onde os pares nascem, e a régua da *porção ativa* é o **ARCO**
percorrido — a distinção que o próprio `note_sketchy_point` já tinha escrita como o motivo de a
varredura não poder cortar por arco (*voltar sobre si mesmo é o único jeito de um ponto velho estar
perto*). O que a rampa fazia é o que o Krita chama de *Distance Opacity* / *Use Distance Density*,
dois controles SEPARADOS que este card não oferece. O LOOK segue sendo do smoke.

**⚠️ Escopo NOMEADO:** o Sketchy costura só nos métodos **incrementais** (`is_incremental`). Os de
re-carimbo re-emitem a figura inteira por quadro, então a memória cresceria por quadro e a teia
adensaria enquanto o artista olha. Gate: `a_re_stamp_method_does_not_sew` — e ⚠️ **a fixture dele foi
escolhida por medição**: com `Anchored`/`FreeHand`/`Line` a mutação SOBREVIVE (o roteador de figuras
intercepta antes do `park_stroke`, e a tinta sai igual pelo motivo errado); só o `DragDot` morde
(44 → 147 texels).

**A porta única `park_stroke`** substitui os quatro `self.paint.stroke = Some(stroke)` do ciclo de
traço. ⚠️ Esquecê-la falha **ALTO** (o traço não volta e o pincel para de pintar), não em silêncio.

**Gates:** 7 no rasterizador · 6 no motor · 4 no depósito · 3 no seam. **8 mutações, 8 sangram.**

**Aberto:** os shape editors (o escopo acima) · o smoke `=3`.

---

### W4 — **Wire** (o *history*)

O primo do W3 com memória **limitada**: liga o ponto atual aos últimos `N`. Dá o traço de
arame/laço. Rows: `History` · `Width` · `Opacity` · `Smoothing`.

**Wave pequena — é o mesmo produtor com uma janela deslizante.** Se ela não for pequena, o W3 foi
construído errado, e isso é um sinal e não um contratempo.

---

#### W4 — FECHADA (2026-08-14)

**O que shipou:** o `LineKind::Wire` (o *Curve brush engine* do Krita), as rows `History` · `Line
Width` · `Opacity` · ☑ `Connection Line`, e a cena de smoke do card.

**A wave FOI pequena, e é o sinal que o plano previu:** zero geometria nova, zero canal novo, zero
rasterizador novo. O Wire é o mesmo `note_thread_point`, o mesmo `ThreadInk`, o mesmo depósito pela
porta de proteção — o que muda é **a pergunta que escolhe os pares**: o Sketchy pergunta *quem está
perto no CANVAS*, o Wire *quem está perto no PERCURSO*.

**⚠️ A janela é de ARCO, não de contagem de pontos, e a decisão não é gosto.** O Krita mede a
história em PONTOS; num motor de dabs isso seria uma janela cujo tamanho depende do **Spacing** —
apertar o espaçamento encurtaria o arame sem ninguém tocar no slider. É a doença que esta linha já
curou quatro vezes no relevo (*a lei é função do CAMINHO, nunca de quão fino o motor amostrou o
caminho*). Pelo mesmo motivo a **contagem de cordas é FIXA** (`WIRE_CURVES_PER_DAB = 4`, amostradas
igualmente espaçadas em arco): *"ligue aos últimos N"* ao pé da letra são `N` fios por dab com `N` =
quantos dabs couberam na janela, e o custo voltaria a ser função do Spacing. **É por isso que o Wire
não tem slider de densidade — nem aqui nem no Krita.**

**⚠️ O `Smoothing` do Krita foi CORTADO, e o motivo é o manual dele:** *"Smoothing: which… I have no
idea actually. I don't see any differences with or without it. Maybe it's for tablets?"* A própria
referência não sabe o que o controle faz. Construí-lo seria shipar um knob que provavelmente não faz
nada — o controle morto que este codebase nomeia em toda wave. No lugar dele entrou o **`Paint
connection line`**, que o manual DEFINE (*"which toggles the visibility of the connection line"*) e
que o plano tinha deixado de fora: desligado, o traço em si não é pintado e sobra só o arame.
⚠️ A supressão mora na **MESMA porta do `Style: Solid`** (`stamp_dabs`) — uma segunda casa para *"não
deposite dabs"* é como as duas passam a discordar num modo que só uma delas conhece.

**⚠️ E o teto do `History` era uma OPINIÃO minha, que a medição derrubou.** A primeira versão escreveu
`6.0` por raciocínio (*"o meio da pista do Krita"*); medido pela porta do produto, a 6 sobram **17× de
margem** contra o kill de 8 ms. O teto é **24**, com a tabela no doc da constante. E o achado de
FIXTURE que o torna honesto: **a espiral que mede o Sketchy SUB-MEDE o Wire** — nela o traço enrola
sobre si mesmo, então uma corda de 1 152 px *de arco* liga dois pontos a ~200 px um do outro no
canvas, e o que custa a rasterizar é o COMPRIMENTO da corda. Na espiral `history 24` custava **1,038**
no pior evento; num traço **RETO**, **3,914**. Um teto tirado da espiral é um teto que o produto dobra
assim que o artista desenha uma linha.

**⚠️ Um gate meu nasceu VERMELHO sobre um produto CORRETO**, e o fato ficou escrito para ninguém
"consertar" o motor: com a janela em 6 diâmetros (144 px de arco) contra um grampo de 240 px no total,
**a dobra está DENTRO da janela** e o arame alcança a outra perna — pelo percurso, que é a lei dele.
*Um arame CRUZA uma dobra que caiba na janela dele*; o que ele nunca faz é cruzar por proximidade no
canvas. O gate passou a comparar os dois tipos **com o mesmo número** (um diâmetro para cada), que é o
que o torna uma prova sobre a PERGUNTA e não sobre o tamanho da janela.

**⚠️ O rename que a wave obrigou:** `sketchy_width_px`/`sketchy_opacity` viraram
`thread_width_px`/`thread_opacity` (e `sketchy_raster` → `thread_raster`, `stroke::sketchy` →
`stroke::threads`, `sketchy_deposit` → `thread_deposit`). Um fio de Sketchy e um de Wire são a **mesma
substância pelo mesmo rasterizador**; dois campos seriam duas respostas a *"quão grosso é um fio deste
pincel?"*, e trocar de tipo faria o card mostrar um número que a tinta não usa. As duas rows ficam na
MESMA posição nos dois tipos — trocar de tipo muda a LEI, não onde os controles foram parar.

**⚠️ E a wave achou um VERMELHO LATENTE que eu mesmo shipei no commit do Magnetify:** `stroke.rs` foi
de 697 para **704 LOC**, e o gate de LOC mora na `ph2d-editor-core` — `cargo test -p
ph2d-painter-brush` nunca o alcança (a causa estrutural que physics, motion-value e Vector já
documentaram). Curado por **responsabilidade**: a memória dos fios (`pts` · `out` · `rng`) virou
`threads::ThreadMemory`, no módulo que a PRODUZ e que sabe o que cada campo significa (686 LOC).

**Gates:** 5 no motor · 1 no depósito · 3 no seam. **6 mutações, 6 sangram** (o canvas em vez do arco ·
a janela contando pontos · ligar a todos os pontos · prender na origem em vez de pular · a supressão
ignorar o flag · a porta única esquecer o Wire).

**Smoke: `PH2D_LINE_SMOKE=1`** — uma cena para o **CARD**, não uma por tipo (três cenas seriam três
canvas idênticos, e a costura que importa — *escolher o tipo* — ficaria pulada em todas). Ela dá o
canvas e o tamanho do pincel e **não arma tipo nenhum**: o dropdown `Type` é exatamente o que o smoke
existe para exercitar.

---

### W5 — **Spray** (a contagem)

⚠️ **Temos a metade errada disto hoje:** `jitter`, `jitter_scale`, `jitter_rotate` deslocam **um**
dab. Spray é **contagem** — `N` cópias no mesmo instante. É a diferença entre *tremer* e *espalhar*.

Rows: `Count` · `Spread` · `Size Jitter` · `Angle Jitter`.

**Onde:** o fan-out no `walk_dab` — a mesma porta única de Symmetry/Tiling ⇒ herda tudo de graça.
Wave pequena, **e é o multiplicador**: com o Speed da W2 ela vira o **respingo** do Splatter (o
escorrido nós já temos, e é físico — a gravidade do Wet Paint).

---

### W6 — opcionais, só com pedido

- **Ribbon** (o *Ribbon Shapes* do Alchemy: `Size · Spacing · Friction · Gravity`) — a fita presa ao
  cursor por uma mola com atrito e peso; o traço **pesa**. O estabilizador já é a metade *arrasto*
  deste modelo, sem massa e sem gravidade.
- **Rough** — o traço que finge ser à mão (`rough.js`/Excalidraw): a primitiva redesenhada 2× com
  deslocamento pseudo-aleatório. Muito barato sobre os shape editors, e dá uma família visual
  inteira.

---

## 5. As decisões — as três primeiras RESPONDIDAS pelo Enio em 2026-08-12

### 5.1 O `Solid` é oferecido em quais tipos? — ✅ **"para todos que forem possíveis"**

⚠️ **E a resposta honesta é: são TODOS**, porque todo tipo mantém o **caminho-base** do gesto — o
Sketchy, o Wire e o Spray são decoração ADITIVA por cima dele (é o que o *Paint Connection Line* do
Krita liga e desliga). Logo há sempre uma silhueta para preencher.

⚠️ **A porta única `LineKind::honours_style()` FICA mesmo devolvendo `true` em todos os casos de
hoje**, e não é cerimônia: ela é onde a resposta mora no dia em que nascer um tipo **sem** caminho-base
(um que só emita partículas). Sem a porta, esse tipo nasce com um checkbox morto e ninguém percebe.
Gate: `every_kind_with_a_base_path_offers_solid`.

### 5.2 A borda de uma forma sólida — ✅ **"pode ser sólida e específica para essa tool; faça o melhor possível"**

Então é **borda da FORMA**, e *"o melhor possível"* virou uma medição em vez de uma opinião (W0.3 do
`line_probe` do tool): erro de cobertura contra a referência `SS = 32`, num disco de raio 300 —

| lei | níveis | erro médio | erro máx | **px > 8/255** | custo |
|---|---:|---:|---:|---:|---:|
| **`SS = 3`** (o de hoje) | 10 | 9,47 | 38,43 | **51,1%** | 11,3 ms |
| `SS = 4` | 17 | 5,96 | 26,15 | 22,7% | 19,9 |
| `SS = 8` | 65 | 1,92 | 13,45 | 1,4% | 72,5 |
| `SS = 16` | 257 | 0,57 | 7,47 | 0,0% | 284,0 |
| **ÁREA EXATA** (o que shipou) | 256 | **0,31** | **2,47** | **0,0%** | **0,8** |

⚠️ **O `SS = 3` que o composite usa hoje erra mais de um degrau visível em METADE da borda** — ele
foi calibrado para ser *traçado*, e um traçado joga a cobertura fora.

⚠️ **E a última linha decide sozinha: a acumulação de área com sinal é MAIS precisa que `SS = 16` e
catorze vezes mais BARATA que o `SS = 3`.** Ela não amostra — integra a área exata do pixel coberto
numa passada `O(arestas + área)`, sem buffer supersampleado. Não há trade a ponderar: *"o melhor
possível"* e *"o mais barato"* são a mesma escolha, e é ela que shipou
(`ph2d-painter-brush::solid`). O resíduo de 0,31 nível é o polígono de 2048 lados que aproxima o
círculo da fixture mais o erro da própria referência, não o rasterizador.

### 5.3 O que `Solid` faz com um gesto que não fecha? — ✅ **"fechar sozinho"**

Fecha do último ponto ao primeiro, como o Alchemy. **Sem limiar de área e sem caso especial**: uma
pincelada reta em Solid vira uma *sliver* de área quase zero, e essa consequência fica **nomeada** em
vez de remendada — um limiar seria um caso especial que a §0 manda medir antes de escrever, e não há
o que medir enquanto ninguém reclamar.

### 5.4 O Style é do PINCEL ou do documento?

Ele mora no `BrushSpec` (por-modo, como todo o resto), o que significa que **cada meio tem o seu**.
Se o Enio quiser um único Style atravessando Digital/Aquarela/Impasto, ele vira estado do tool com
fan-out para os slots — o molde do `toggle_brush_impasto`, que escreve nos três slots de relevo
porque *os três são o mesmo assunto*.

---

## 6. O que fica FORA, com o motivo

- **Pressure Shapes.** Temos a feature (`dynamics.rs`) e **não temos o dispositivo**:
  `shells/desktop/tests/the_desktop_shell_has_no_pen_pressure.rs` demonstra, varrendo a grade inteira
  de sliders, que nenhuma combinação move um pixel. A cura é **subir o winit** (fundação de janela do
  app inteiro ⇒ cross-line, classe ADR) ou um caminho de tablet por plataforma. Não é wave de pincel.
- **A matriz de dinâmica completa** (doc 37 §2). Depois da W2 e da W3 haverá duas entradas e três
  alvos; antes disso é infra sem consumidor.
- **Assistentes de perspectiva.** O motor de snap já é do Vector; um segundo no Painter é a segunda
  porta. É wave de ponte.
- **Generativo** (flow field, growth, voronoi). Os algoritmos já são nossos, no Motion Nodes, com
  paridade CPU×GPU gateada. Falta a **ponte**, e ela é ADR.
- **Blindness / Limit / Auto-Clear.** São ferramentas de ideação, num app que tem undo, save e
  camadas. Mudam a tese do produto; só com pedido explícito.

---

## 7. Ordem de execução, resumida

| Wave | Entrega | Tamanho | Depende de |
|---|---|---|---|
| **W0** | as três medições | pequena | — |
| **W1** | o card + **Solid** | média | W0 (2) |
| **W2** | o dropdown + **Speed** | média | W0 (1) |
| **W3** | **Sketchy** | média | W0 (3) |
| **W4** | **Wire** | pequena | W3 |
| **W5** | **Spray** | pequena | — |
| **W6** | Ribbon · Rough | — | só com pedido |

⚠️ **A W1 e a W2 são independentes** — se o Enio quiser ver o card mais cedo, a W1 sozinha já entrega
um card com um controle vivo. O que não pode acontecer é o card nascer com o dropdown de uma opção
só.
