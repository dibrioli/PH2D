# ARQUIVO — 38_plano_linha_procedural.md (história, 834 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`38_plano_linha_procedural.md`](../../../Painter/38_plano_linha_procedural.md) em 2026-08-18, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-106,340-370,409-417,521-521,712-732,777-780,819-823,904-916,956-960,1030-1139` do original.
>
> ⚠️ **A única alteração ao corpo:** 1 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

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
2. **O custo do Solid contra o Line de hoje.** O doc [35](../../../Painter/35_boolean_o_que_o_vector_ensina.md) §1.1
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

### W5 — **Spray** (a contagem) — FECHADA (2026-08-14)

⚠️ **Temos a metade errada disto hoje:** `jitter`, `jitter_scale`, `jitter_rotate` deslocam **um**
dab. Spray é **contagem** — `N` cópias no mesmo instante. É a diferença entre *tremer* e *espalhar*.

⚠️ **E é essa frase que corrige as outras duas linhas que este plano escrevia aqui.** A wave foi
construída como o plano manda e as duas notas dele caíram na leitura:

**(1) As rows eram quatro; ela traz UMA.** `Spread`, `Size Jitter` e `Angle Jitter` **já shipam** —
são o `jitter` (posição, uma fração do diâmetro), o `jitter_scale` e o `jitter_rotate` do card
**Jitter**, e são a MESMA operação: o próprio §3.4 do doc 37 diz *"deslocam um dab"*. Construir
gêmeos deles aqui seria a **segunda porta** para a mesma pergunta — e esta casa já removeu um caso
idêntico, o *Random Angle* por-slot de Shape+Grain, em 2026-07-19, **porque o Jitter Rotate do traço
era o superset**. Logo o spray traz **`Count`**, e mais nada.

**(2) Ele NÃO é um `LineKind`.** `Speed`, `Sketchy` e `Wire` são leis **da linha** e mutuamente
exclusivas — cada uma responde *que forma o traço tem*. Uma contagem de marcas é um **multiplicador
da emissão** e **compõe com todas elas**: um chicote sprayado e um rabisco sprayado são pedidos
legítimos, e pôr o spray naquele dropdown os tornaria **inexprimíveis**. É também o modelo do
Photoshop (*Scattering* é um painel que empilha com o resto do pincel, não um modo) — o Krita só o
faz motor próprio porque lá ele é uma ENGINE inteira, com o resto do pincel dentro.
⚠️ E a §5.1 deste plano já tropeçava nisso: ela lista *"Sketchy/Wire/Spray produzem muitos fios
curtos, não uma silhueta"*, o que é falso do Spray — ele produz **dabs**, sobre o mesmo caminho-base.

**O que shipou:** `BrushSpec.spray_count` (default **1** = o mundo de sempre, byte a byte) e a row
**Count** como a **PRIMEIRA** do card Jitter — a posição é a feature, porque é ela que transforma as
três rows abaixo de *tremor* em *nuvem*.

**Onde:** `crates/ph2d-painter-brush/src/stroke/spray.rs`. O fan-out mora em **`Stroke::stamp_at`**,
a porta única por onde um ponto do CAMINHO vira marca — ela funde `dab_at` + `emit`, que eram
adjacentes nos cinco sítios que as chamavam. ⚠️ **A fusão é o que dá ao spray um lugar onde
*"quantas marcas este ponto deixa"* seja respondido uma vez:** o arremesso, os fios e o vão do
arremesso são fatos do **caminho** (uma vez); o jitter e o carimbo são fatos da **marca** (`n` vezes).
A alternativa era um canal de estado (`dab_at` guarda a posição pré-jitter, `emit` a lê) — o *um
fato, duas cópias* que este módulo já pagou caro.

⚠️ **A nuvem é centrada no ponto do CAMINHO, nunca na primeira marca dele.** O desenho barato
(*"derive cada cópia do dab base por mais um deslocamento"*) espalha por **duas** vezes o raio que o
slider promete e desloca a nuvem do caminho; ele tem gate próprio (`the_cloud_is_centred_on_the_path_
point_not_on_its_first_mark`) e a mutação que o instala sangra só nele.

⚠️ **As cópias não passam pelo `Stroke::emit`, e é correção e não economia** — aquela porta alimenta
a memória dos fios e preenche o vão do arremesso, e as duas são do CAMINHO: uma nuvem de oito marcas
empurraria oito pontos para a memória do Sketchy (o traço costurando-se ao próprio spray) e mandaria
o preenchedor percorrer oito vezes o mesmo vão. O gate é `the_spray_does_not_feed_the_thread_memory`.

⚠️ **O gate do fan-out é `allows_jitter()`, o MESMO do resto do sorteio por-dab** — Drag Dot,
Anchored e Grid Stamp põem **uma** marca deliberada (o Anchored é um carimbo ancorado no clique; o
Grid Stamp promete que todo carimbo pousa no centro de uma célula), e é por isso que eles já não
sorteiam nada.

**Symmetry e Tiling saem de graça:** toda cópia sai por `push_symmetric`, exatamente como o dab base
⇒ o fan-out fica no mesmo estrangulamento onde a Symmetry já se pendura, e o Tiling é a jusante.

⚠️ **A PRIMEIRA nuvem tem de PARECER uma nuvem, e sem isso o `Count` era um controle MORTO** — medido,
não suposto: o pincel de fábrica tem `strength`/`flow` em `1,0`, então `n` marcas empilhadas pintam
**exatamente** o que uma pinta, e o artista sobe a contagem para ver a tela não mudar. A cura é o
molde do `toggle_brush_impasto`: ao pedir a **primeira** marca extra (`1 → n`), o tool arma
`SPRAY_DEFAULT_SPREAD` no `Jitter → Position` **só se ele ainda estiver no zero de fábrica** — *arma
um default, nunca impõe política*, e a metade que a torna segura é a segunda (um espalhamento
escolhido pelo artista, inclusive um zero deliberado, nunca é reescrito). Quatro gates, quatro
mutações.

**Teto: `SPRAY_COUNT_MAX = 16`, MEDIDO** contra o kill de 8 ms/evento — a tabela de 27 células (três
raios × nove contagens) mora no doc-comment da const, levantada por
`measure_the_spray_budget_per_event`.

⚠️ **A escolha do CANTO é a decisão da constante, e a tabela mostra por quê:** o custo é um PRODUTO
(`contagem × área`), então um teto escalar tem de dizer onde mediu. No canto que os dois sliders
alcançam **juntos** (`r = BRUSH_SIZE_MAX_PX`) o teto seria **4** — e a razão é que **um único dab ali
já custa 2,582 ms**, um terço do orçamento *antes de existir spray nenhum*: derivar dali seria deixar
um slider que **nunca foi capado por tempo** ditar o teto de um que é, a forma invertida do §0. No
outro extremo, a referência r=24 das outras tabelas deste plano comporta **mais de 256**, o que daria
uma pista cuja faixa útil vive nos primeiros 8% — legítimo pelo relógio, ilegítimo pelo controle. O
`16` é o pincel GRANDE de verdade (r=200), e o preço de o número ser um está nomeado: 16× de margem
no canto barato, e oito marcas estouram o quadro num pincel de 512 px.

⚠️ **E a PRIMEIRA tabela desta wave era CEGA, pelo motivo mais fácil de não ver:** a sonda clampa em
`SPRAY_COUNT_MAX`, então as linhas `24 / 32 / 48 / 64` mediram todas **a mesma nuvem de 16** — 0,309
/ 0,310 / 0,318 / 0,312 ms, uma tabela **plana** que se lê como *"o custo satura"* e significa *"a
sonda parou de medir"*. **A sonda que escolhe o teto não pode ser capada pelo teto.** A 2ª corrida,
com a const levantada, também trouxe um artefato: a **primeira** espiral de uma corrida paga o
*first-touch* e mediu o dobro das seguintes (`count 1` mais caro que `count 2`, não-monotônico onde a
lei é linear) — hoje ela é descartada.

**Gates:** 8 no motor (`spray_tests.rs`) + 3 de costura por PONTEIRO (`seam_spray.rs`) + 1 de
round-trip no painel. ⚠️ **A projeção `contagem → pista` só sai da função como a posição DESENHADA do
polegar** — não entra no store, não move retângulo de hit, não publica evento —, então mutá-la deixa
a workspace inteira verde; é o mesmo mecanismo que obrigou o anel do pincel a ter gate próprio, e a
cura é a mesma: afirmar a propriedade onde ela mora (`the_track_and_the_count_are_inverses_over_the_
whole_span`, no crate do painel). **10 mutações, 10 sangram.**

---

### W6 — opcionais, só com pedido

- ✅ **A PREMISSA DESTA LINHA ESTAVA ERRADA, a referência a desmentiu, e a FAIXA foi construída**
  (Enio, 2026-08-15, com a captura do Alchemy ao lado da nossa; construída na mesma sessão). **O
  `Ribbon Shapes` do Alchemy NÃO é uma curva atrasada** — é uma **FAIXA com travessas**: a marca
  segue o gesto e dentro dela correm riscos atravessados que se abrem e fecham conforme a mão
  acelera. O que a wave tinha construído era o **seguidor massa-mola** (uma linha só, atrasada), que
  é o **Dynamic Brush do Krita** (`Mass` + `Drag`). O bullet abaixo citava a referência de uma e
  descrevia a outra; hoje o massa-mola é **metade** da fita — o trilho de tinta.
  - **O desenho, derivado da captura e não do código** (Alchemy e Krita são GPL-3 e a regra deste
    repo é *comportamento sim, linha de código nunca*): dois **trilhos** — o do DEDO e o
    **ATRASADO** — ligados por **travessas**, com a **largura da faixa sendo o próprio atraso**.
    É por isso que ela **estreita nos picos**, onde a mão desacelera, que é o que a captura mostra —
    e o gate que a pina é uma **RAZÃO** entre duas velocidades (`the_band_is_the_lag_so_it_opens…`),
    nunca um comprimento absoluto.
  - ⚠️ **A FITA É O TERCEIRO CONSUMIDOR DA TINTA DE FIO, e shipou sem as duas rows dela** (fechado
    2026-08-15). O trilho de fora e TODA travessa saem pelo `thread_ink` — a porta única do depósito
    de fios —, logo `thread_width_px` e `thread_opacity` decidem **como a faixa aparece**; e o
    `RIBBON_SLIDERS` não as trazia, então os dois números eram alcançáveis apenas trocando para
    Sketchy, mexendo, e voltando. *Um controle que governa o que se vê e vive noutro modo é um
    controle que o artista não tem.* O doc das `THREAD_INK_ROWS` já dizia *"as MESMAS duas rows nos
    dois tipos, na MESMA posição, porque são o mesmo fato"* — a fita é o terceiro tipo e ficou de
    fora da frase.
  - ⚠️ **E o card Line tinha seam, mas a fita nasceu FORA dele** — a cicatriz que este plano registou
    uma wave antes (*"os três sliders da W6 shiparam com id, row, `populate`, encaminhamento e
    setter, e nenhum gate os exercitava"*) reincidiu no lugar exato: o `seam_line_card.rs` nasceu
    cobrindo `Type` / `Solid` / Sketchy / Wire, a fita acrescentou quatro sliders, e a lista do gate
    **não seguiu a família**. *Um gate por FAMÍLIA que não é estendido com a família apodrece no
    mesmo lugar em que nasceu.* Fechado com os dois irmãos de sempre (presença-E-ausência ·
    arrasto por ponteiro REAL), **3 mutações, 3 sangram** — tirar as rows de tinta do array · tirar
    um id do `populate` · tirar um id da whitelist do `event_brush_forward`.
  - ⚠️ **E o `ribbon_diag` contava METADE do que a fita desenha:** ele soma dabs por fonte, e o
    trilho de fora mais as travessas são **fios**, por um canal próprio — um traço de faixa correcto
    e uma faixa que não é costurada davam a MESMA linha de log. O par novo é
    `fios: cosidos=N carimbados=M`, e são **dois** números de propósito: um contador só mostra `0`
    tanto quando *o motor não costurou* quanto quando *o depósito recusou*, que são curas opostas —
    e é literalmente a confusão que custou a hora de diagnóstico desta wave (**343 travessas por
    traço com o depósito mudo**). Com o par, `0/0` acusa o motor e `343/0` acusa o depósito.
  - ⚠️ **E o defeito que custou a hora de diagnóstico foi uma PORTA DUPLICADA:** havia dois
    `sews_threads` — um no `LineKind` (que respondia *o tipo costura?*) e um no `BrushSpec` (que o
    depósito pergunta) — e o **doc do segundo dizia consultar o primeiro sem o consultar**: ele
    enumerava `sketchy_active() || wire_active()`. Ligar a fita no portão do enum deixou o motor a
    costurar **343 travessas por traço** com o depósito **mudo**, e a imagem saía idêntica ao
    controle. **O portão do enum não existe mais** e o do spec é um `match` **EXAUSTIVO sem braço
    `_`**, então um quarto costurador é um erro de compilação em vez de uma faixa que nunca pinta.
  - ⚠️ **E o SMOKE DA FAIXA reprovou as ESPÍCULAS, que a correção da cauda tinha tratado pela
    metade** (Enio, 2026-08-15: *"parece melhor mas com as espículas do algoritmo antigo"*). A
    ablação pela porta do produto — a mesma imagem renderizada com e sem cada fase — nomeou a fase
    em uma corrida: a **PAUSA** acrescentava **18 013 px de tinta, 5 715 deles ESCUROS** (tinta
    cheia ⇒ dabs, não fios), numa caixa de 725 × 396 px; a **CAUDA** acrescentava **zero**.
    - **A lei:** uma mola que converge para um alvo **PARADO** anda em **linha reta**. Enquanto o
      dedo se move o alvo foge e a ponta descreve uma curva; assim que ele pára, a convergência é um
      segmento de largura cheia atravessando o desenho. **Não é do pen-up** — é de *qualquer*
      instante em que a mão pára com o botão preso, e cortar a coleira no pen-up tratou o sintoma no
      único instante em que ele tinha nome.
    - **A cura é o RELÓGIO DO GESTO:** um tique em que o dedo não andou não entrega tempo à fita.
      Ela **congela** — posição *e* velocidade —, então retomar continua de onde parou (medido: **13
      dabs** no tique retomado, contra **239** se ela descartasse o estado). É o que a referência
      faz: o Alchemy não tem relógio nenhum, a fita dele é função dos PONTOS de entrada.
    - ⚠️ **E TRÊS gates repousavam na premissa que isto dissolveu, encarados em vez de deixados
      verdes:** a GRAVIDADE media o pingar com a mão parada (a fixture morreu, o número `g·τ²` não —
      ele é o mesmo no equilíbrio sob arrasto) · o CHICOTE media a ultrapassagem depois da mão parar
      (mudou para a QUINA, onde a massa leva a ponta para fora da curva, com os dois controles
      intactos) · e *"a mão parada assenta"* passou a ser **trivialmente verdadeiro no tique 0**,
      então cedeu o lugar à lei nova. O piso do `RIBBON_DAMPING_MIN` **fica**, mas a tabela dele
      media um recurso que deixou de existir: hoje ele bounda **LOOK**, não tinta.
  - ⚠️ **E O SMOKE SEGUINTE REPROVOU DE NOVO — a lei do gesto estava CERTA e era METADE.** Enio,
    2ª rodada com foto, o mesmo veredito (*"parece melhor mas com as espículas do algoritmo
    antigo"*). A auditoria correu em **três lentes independentes** (rota de entrega · geometria dos
    fios · máquina de estados por tique) e **as três convergiram na mesma causa**, que não é a mola:
    - **Num traço de fita há DOIS percorredores de caminho.** O `tick_ribbon` é um; o outro é
      **`Stroke::settle`**, que o tool chama em todo quadro **PARADO** e que percorre de `last_pos`
      — a cabeça do trilho da FITA — até `stab_pos`. E com a fita armada o `extend` cai no braço
      vazio (`StrokeMethod::Space if ribbon_active() => {}`), logo `Stroke::stabilize` **nunca
      corre** e o `stab_pos` fica **no ponto do PEN-DOWN o gesto inteiro**.
    - **A geometria cai da aritmética, e é a da foto:** `blend = 0,54` com o `stabilizer = 0,5` que
      SHIPA ⇒ o `settle` percorre até `pen_down + 0,54·(cursor − pen_down)` e o tique seguinte volta
      de lá — **duas retas partilhando um ápice que o dedo nunca visitou**, o triângulo agudíssimo.
      Medido: **295,0 px em 123 dabs** (gate) e o ápice previsto em `x = 748` contra **749,5**
      medido (sonda). Escuras porque são **DABS de largura e cobertura cheias** — e a 3ª lente mediu
      a tinta de um fio (**largura 1,0 px, opacidade 0,25**), que é o cinza-claro das travessas.
    - **A cura é a PORTA ÚNICA:** `settle` é **inerte com a fita armada**, e a guarda mora no motor
      (`stabilize.rs`), nunca num `if` do tool — o tool pergunta *"a mão parou?"*, que continua
      legítimo; quem sabe que a fita já É o passa-baixa do caminho é o motor. Um `if` do lado de lá
      deixaria o próximo `LineKind` que move o CAMINHO nascer com o defeito de volta. E **não há
      atraso a drenar**: a fita persegue o `last_raw_pos` CRU de propósito, então o estabilizador
      está fora do caminho dela por desenho.
    - ⚠️ **DUAS cegueiras de fixture, cada uma bastando sozinha** — e é isto que fez a ablação
      anterior reportar *"PAUSA: nenhuma tinta nova"* sobre uma foto que a mostrava: **(a)** as
      **cinco** fixtures de fita desta linha cravam `stabilizer: 0.0`, que é exactamente o valor que
      faz o `settle` sair no segundo `if`, sobre um produto cujo default é **0,5** — a irmã exacta
      do `hardness = 1.0` do smear; **(b)** as sondas simulavam a pausa entregando um `Move` na
      MESMA posição, e isso põe `moved_this_frame = true` ⇒ `parked == false` ⇒ **o `settle` nunca
      corria**. *A fixture modelava a pausa exactamente da única maneira que mantém adormecido o
      mecanismo que ela existe para medir.* As duas foram corrigidas, e a prova de que deixaram de
      ser cegas é o A/B: com a guarda a coluna PAUSA mede **0 dabs**; sem ela, **462 dabs / 244 px
      de reta**.
    - ⚠️ **E o pen-up herdava o cursor envenenado:** se o último quadro antes do `Up` fosse parado,
      a cauda partia do `stab_pos` — uma **terceira** reta do mesmo ápice, pelo mecanismo que a nota
      do `finish_ribbon` já dava por curado. A guarda fecha isto de carona.
  - ⚠️ **E o LEQUE DAS TRAVESSAS fechou logo a seguir (ordem *"construa"*), com uma lei de uma
    linha:** a cadência era medida **só no arco da FITA**, e o ponto de fora é colhido no trilho do
    DEDO na mesma fração — então quando a mão desacelera numa crista **enquanto a fita descarrega
    atraso**, cada quadro emite uma ou duas travessas com a ponta de fora no mesmo sítio e o leque
    cresce quadro a quadro.
    - ⚠️ **A interpolação de `f` não o alcançava, e não é falha dela:** ela resolve o caso ESPELHO —
      dedo rápido, fita lenta, várias travessas num quadro —, que vive **dentro** de um tique. Este
      vive **ATRAVÉS** deles, e nenhuma fração de um tique vê o tique seguinte. Foi por isso que a
      régua por-tique reportava **27 px** sobre um leque de centenas.
    - **A lei: a faixa avança o que os DOIS trilhos avançaram** (`min`), nunca o arco da fita —
      *duas travessas só são distinguíveis se as DUAS pontas andaram; se uma ficou parada, o que se
      desenhou não é uma escada, é um leque.*
    - ⚠️ **No regime que o smoke aprovou isto é NEUTRO**, e é o que o torna seguro: em regime a fita
      anda à velocidade do dedo (o atraso é um deslocamento constante), logo `min ≈ dist(a, b)` e a
      faixa é a mesma — o render mediu **62 px de faixa e preenchimento 0,43..1,00 antes e depois**.
      A mudança é estritamente para MENOS travessas, e só onde os dois trilhos discordam.
    - **Medido:** leque **148,5 → 0,0 px** na sonda do gesto ondulado (27,0 · 94,5 · 148,5 nos pesos
      0,08 · 0,20 · 0,45 — ele cresce com o atraso), e a contagem de travessas cai só onde elas eram
      duplicatas (215 → 181 · 202 → 129 · 179 → 100).
    - ⚠️ **E o gate nasceu ASSIMÉTRICO — quem o completou foi uma mutação que sobreviveu.** A 1ª
      versão media só o ápice no DEDO, e a cadência pelo arco do dedo (a cura ingénua da metade que
      eu via) **passava nas duas réguas** enquanto abria o leque espelho na ponta da FITA. Hoje o
      gesto tem as duas assimetrias — corre · **freia** · **arranca** — e a mesma régua mede as duas
      pontas: `dist(a,b)` sangra **297,0 px em 12 travessas** no ápice do dedo, `dist(fa,fb)` sangra
      **54,0 px em 3** no ápice da fita, o `max` repete os 297,0, e o `min` é o único dos quatro que
      fecha as duas metades.
    - ⚠️ **E a cerca do gate irmão foi ENCARADA, não deixada verde:** o doc do
      `the_rungs_of_one_tick_do_not_fan_to_a_single_point` afirmava cobrir *"as cristas, onde a mão
      desacelera"* sobre uma fixture que é um arrasto **RETO a velocidade constante**. A frase estava
      certa sobre o fenômeno e errada sobre aquela régua; ele continua a guardar a interpolação, que
      é a outra metade e continua load-bearing (ele **passa** sob a mutação que sangra o irmão).
  - ⚠️ **E A CAUDA DO PEN-UP FOI INIBIDA — decisão de PRODUTO do Enio** (2026-08-15: *"no mouse up o
    fim do traço cresce um segmento indesejado; iniba o traço residual no mouse up"*). A fita acaba
    exactamente onde o último tique a deixou.
    - ⚠️ **Isto REVOGA uma cerca de Chesterton, e ela fica ESCRITA em vez de apagada:** a cauda
      soltava a coleira (o termo de mola saía e ficavam a inércia, o atrito e a gravidade) e
      percorria até a ponta assentar — o *follow-through* da animação clássica, e o que o Alchemy (a
      forma termina no release) e o Dyna do Krita (a massa é solta) fazem. Custava **71 dabs / 84 px**
      no peso que shipa (30/34,8 no 0,20 · 77/91,2 no 1,00), e o artista lê esse crescimento *depois*
      de soltar como um segmento a mais, não como peso. **É isso que decide, não o precedente.**
    - ⚠️ **O argumento contra o SALTO ATÉ O CURSOR continua de pé** — a fita nunca é encerrada
      percorrendo em linha reta até `last_raw_pos` (o que o estabilizador faz *"para o traço acabar
      exactamente onde a caneta levantou"*); a tinta **deve** ficar atrás. O que morreu foi a outra
      metade: que ela continua a **chegar** depois da soltura. E é a **terceira** vez que este
      pen-up desenha uma reta indesejada, cada vez por outro mecanismo — a mola presa ao cursor (369
      px, curado cortando a coleira) · o cursor envenenado pelo `settle` · e o crescimento em si.
    - **Higiene que a remoção arrastou, e nenhuma é opcional:** o parâmetro `leashed` do
      `step_ribbon` ficou com um só valor de chamada ⇒ **morreu** (parâmetro morto é código que
      mente) · os consts `RIBBON_TAIL_MAX_S`/`RIBBON_TAIL_REST_PX_S` ficaram `pub` **sem chamador**,
      que é a *segunda resposta à espera* que este repo já nomeou ⇒ removidos, com a **tabela de
      orçamento** que o primeiro hospedava mudando-se para o doc do `RIBBON_LAG_MAX_S` (ela mede o
      ORÇAMENTO, não a cauda, e dois outros docs a citam) · e o doc do `RIBBON_DAMPING_MIN`, que
      justificava o piso com *"a cauda ficaria a balançar"*, passou a dizer o que de facto sobra.
    - ⚠️ **DOIS gates tinham a premissa dissolvida e foram ENCARADOS:** o `the_tail_does_not_run_to_
      the_finger` exigia que a cauda **andasse** ≥ 100 px — afirmando o que o produto não faz mais —
      e virou `the_pen_up_adds_nothing_because_the_ribbon_ends_where_it_is` (as duas metades que
      sobrevivem: o pen-up não move um pixel **e** a tinta acaba atrás do dedo; mutação: **146 dabs**);
      o `the_tail_sews_nothing…` passou a valer trivialmente e virou `the_pen_up_sews_nothing…`, mais
      forte e no canal de FIOS, que é outra pergunta.
    - ⚠️ **E é a TERCEIRA substituição desta linhagem de gate:** o primeiro
      (`the_tail_arrives_when_the_pen_lifts`) afirmava o DEFEITO como lei (exigia pousar a 40 px do
      dedo, que era a espícula); o segundo curou o mecanismo e exigiu que ela andasse; o terceiro
      afirma que ela não anda. *Um gate escrito sobre uma feature herda a vida dela.*
  - ⚠️ **E o card Line não tinha seam nenhum** — os três sliders da W6 shiparam com id, row,
    `populate`, encaminhamento e setter, e **nenhum gate os exercitava** (um `grep` pelo id nos
    testes do repo devolvia nada). O `line_seam_tests.rs` fecha as duas condições que o
    `wiring_parity` é cego a ver: *o clique chega ao tool?* e *a sequência leva a algum lugar?*
- **Ribbon (o massa-mola, a construir-se `Dyna`)** — ✅ **CONSTRUÍDO** (2026-08-14/15, ordem *"Siga"*). A fita presa ao cursor por uma mola
  com atrito e peso; o traço **pesa**. Sliders **Weight · Friction · Gravity** (`Size` e `Spacing`
  do Alchemy já são do pincel — dois donos para o mesmo número seria a segunda porta).
  - ⚠️ **A premissa do plano estava certa e é o que decide a arquitetura:** *"o estabilizador já é a
    metade arrasto deste modelo"*. A fita move o **CAMINHO**, não a tinta — é um passa-baixa, como o
    estabilizador, e realimentar um passa-baixa é estável por construção. O `Speed` faz o oposto
    (move a TINTA e deixa o caminho intacto) porque a velocidade dele é medida *do* caminho, e
    realimentá-la a somaria a si mesma. **Por isso ela custa tão pouco:** para o espaçamento, os
    fios, o preenchedor de vão, a Symmetry, o Tiling e o Spray, **a fita É o traço** — nenhum deles
    sabe que ela existe.
- **Rough** — ✅ **CONSTRUÍDO** (2026-08-15, ordem *"siga implementando"*). O traço que finge ser à
  mão (`rough.js`/Excalidraw): a linha **vagueia** e é desenhada **duas vezes**. Sliders
  **Roughness · Bowing · Passes**.
  - ⚠️ **A PERGUNTA ABERTA DESTE ITEM JÁ ESTAVA RESPONDIDA, no doc do próprio `LineKind`** — o
    plano perguntava *"é um `LineKind` ou um efeito do editor de forma?"* e o cabeçalho daquele
    módulo separa as duas coisas em uma frase: *"o `StrokeMethod` responde **como este caminho é
    AUTORADO** (mão livre, linha, elipse…) e o `LineKind` responde **que lei procedural decora o
    caminho autorado** — as duas perguntas são ortogonais"*. Roughen não é uma maneira de autorar:
    é uma lei que decora um caminho já autorado. **É um `LineKind`.**
  - ⚠️ **E o medo que a pergunta carregava (*"o que o Apply assa?"*) dissolve num PRECEDENTE desta
    casa, não numa invenção:** o **Offset** dos shape editors já é *drawing-only* — os pontos e a
    guia ficam pristinos e só o desenho desloca (`bake_curve_offset` = no-op). O Rough é a mesma
    forma de coisa: o artista autora uma elipse limpa e a TINTA cai torta. Nada reescreve
    `verts`.
  - ⚠️ **ELE NÃO É O JITTER, e essa é a linha que decide a matemática.** O motor já espalha cada dab
    independentemente (`jitter`), e um deslocamento pseudo-aleatório POR DAB **seria** aquilo — a
    segunda porta para a mesma pergunta, que esta casa já removeu duas vezes (o *Random Angle*
    por-slot, o `sews_threads` do enum). O que o `rough.js` faz e o jitter não é **vaguear**: o
    desvio é COERENTE ao longo do traço, e é por isso que o resultado lê como mão e não como poeira.
    Logo o campo é **ruído de valor 1-D no ARCO** — baixa frequência —, avaliado na PERPENDICULAR
    do heading.
  - ⚠️ **DUAS oitavas, porque o `rough.js` tem DOIS knobs e eles são amplitudes de escalas
    diferentes:** o `roughness` dele desloca os pontos de controle (o tremor curto) e o `bowing`
    desloca o MEIO do segmento (o arqueamento longo). Uma oitava só colapsaria os dois num slider e
    tornaria o arco inexprimível. Os comprimentos de onda são em **DIÂMETROS de pincel** — a mesma
    unidade do `sketchy_reach_px` e do `wire_history_px`, porque a escala natural desta casa é o
    pincel, não o pixel.
  - ⚠️ **A SEGUNDA PASSADA é o que ninguém consegue exprimir hoje** (é o `disableMultiStroke` do
    `rough.js`, ligado por default): duas caminhadas ao longo do MESMO caminho, com sementes
    diferentes, que divergem e se cruzam — o contorno duplo do Excalidraw. Ela sai pela porta que já
    existe para *um ponto do caminho deixa `n` marcas* (`stamp_at` → o molde do `spray_copies`), e
    **não** pelo `emit`: aquela porta também alimenta a memória dos fios e preenche o vão do
    arremesso, que são fatos do CAMINHO e acontecem uma vez.
  - ⚠️ **IDEMPOTÊNCIA SOB RE-CARIMBO é a propriedade load-bearing, e ela cai de graça da escolha do
    ARCO como semente.** Os shape editors re-carimbam a figura INTEIRA a cada quadro, então um
    desvio semeado num contador por-dab faria a figura **FERVER enquanto o artista só olha** — é a
    doença que este módulo nomeia desde o sculpt (*"o re-stamp por-frame dos shape editors
    empilharia enquanto o artista só OLHA"*). Sendo `d` função PURA de `(arco, passada, spec)`, o
    mesmo desenho dá os mesmos dabs **ao bit**, e é isso que o gate afirma.
---

### W7 — o `Solid` passa a USAR o pincel, e com ele todo tipo, a Symmetry e o Tiling

**Ordem do Enio, 2026-08-15**, em três frases que são **uma**:

> *"Solid deve usar o pincel com o falloff e espessura do traço como no modo flip."*
> *"Todos os types devem ser compatíveis com solid."*
> *"Simetry e Tiling devem ser compatíveis com os traços e solid."*

#### A frase que decide tudo

**`Style: Solid` deixa de ser *a região EM VEZ do traço* e passa a ser *a região SOB o traço*** — o
modelo do **Flip**, onde um `FlipStroke` tem `fill: Option<Fill>` e **continua a ter a largura e a
dureza dele**. O preenchimento é a região cercada; o contorno é o pincel.

⚠️ **Isto REVOGA a §1.1 deste plano**, e a revogação é do mesmo autor: até aqui o Solid **suprimia**
o carimbo, sob o pedido de 2026-08-12 (*"para forma sólida a espessura da linha passa a não ser
considerada"*). Os gates que pinavam aquela lei (`in_solid_the_brush_width_does_not_enter…`,
`in_solid_a_shape_ignores_the_brush_width…`) foram **substituídos**, não apagados: o que eles
afirmam agora é o oposto medível, e a metade *"a região continua cheia"* ficou ao lado para nenhuma
das duas poder passar sozinha.

#### As três metades eram a mesma, e é por isso que a wave é pequena

| o pedido | o que o entrega |
|---|---|
| falloff + espessura | os dabs voltam a ser carimbados — **o pincel é o pincel** |
| todo tipo compatível | Speed/Sketchy/Wire/Ribbon/Rough **decoram o traço**, e não havia traço |
| Symmetry + Tiling nos traços | eles moram na porta do dab: chegam **de graça** |
| Symmetry + Tiling no Solid | a única metade que a wave teve de construir |

#### As duas transações, e por que não podem ser uma

O preenchimento é sempre um **re-carimbo** (a cada ponto novo o polígono INTEIRO muda de forma), mas
o traço não: metade dos métodos deposita **cumulativamente**. Isso dá duas transações, e o
`drag_preview` é **um slot só** — duas encadeadas nele deixariam só a última de pé, restaurando por
cima dos dabs que a outra acabou de carimbar.

- **métodos de RE-CARIMBO** (Drag Dot / Anchored / Line + os cinco shape editors): a mancha viaja
  **dentro** do `stamp_drag_preview`, e a região salva é a **união** das duas caixas;
- **métodos CUMULATIVOS** (mão livre, Airbrush, Dots, Grid Stamp): a mancha é uma transação própria,
  **descascada antes** do lote e **re-escrita depois** — e o bracket mora na **porta única do
  carimbo** (`stamp_dabs`), nunca nos seis sítios do ciclo de traço. ⚠️ **O modo de falha de
  esquecer um sítio aqui não é *deixar de desenhar*, é APAGAR:** o restore de um snapshot velho
  desfaz a tinta que o artista acabou de pôr.

⚠️ **E a ORDEM entre a mancha e o traço não é observável** — as duas tintas são a mesma cor, e o
`over` de duas fontes da mesma cor é **comutativo** (`a₁ ⊕ a₂ = a₁ + a₂ − a₁a₂` é simétrico, e a cor
sai `c` nos dois casos). É esse fato que **permite** as duas famílias usarem transações diferentes
sem a imagem discordar.

#### O que a Symmetry teve de aprender sobre um LAÇO

`symmetric_loops` nasce ao lado de `push_symmetric` (dabs) e `push_symmetric_segment` (fios), sobre os
mesmos primitivos. ⚠️ **O WINDING é REPOSTO, e é correção e não higiene:** o preenchimento é
`nonzero`, então dois contornos de sentidos opostos **se furam** — é assim que o `Remove` de um shape
editor abre um buraco. Uma **reflexão inverte o sinal da área**; sem repor a ordem dos pontos, um
espelho cujo eixo cruza a figura devolveria uma cópia que **CANCELA** a original e abriria um buraco
onde tem de haver tinta. Uma **rotação preserva** o sinal, então só o braço do espelho reverte.
Gates: a propriedade (o sinal) e o oráculo de aparência (o miolo cheio, medido no rasterizador).

#### O que o Tiling teve de aprender

`tiled_loops` e `tiled_threads` perguntam à **MESMA `axis_offsets`** que os dabs já usam — a régua de
um laço é a **caixa** dele, e a de um fio é a caixa **mais meia espessura**. ⚠️ Sem o meio-lado, um
fio que corre RENTE à costura (caixa de altura zero, tinta de `width_px` de largura) não seria
replicado e a tile ficaria com **meia linha**; é o mesmo motivo pelo qual a régua de um dab é
`centro ± raio`. Uma segunda regra de *"o que cruza a costura"* divergiria da dos dabs no dia em que
