# 89 · Família 15 — VALUE (26 nós)

> História verbatim: [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| `value.math` — `Power` · `Logarithm` · `Exponent` — Blender Math os ship | recusado com motivo, e a cerca está no código: o comentário do MANIFEST diz "Power/log are omitted by HR-5". E o uso artístico já está coberto: `value.gain` (Schlick 199… | vivo, tabela |
| `value.math` — Boolean Math (AND·OR·NOT·XOR) — Blender GN tem nó dedicado; Cavalry tem Logic |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.47 |
| `value.unary` — `Ceil` · `Round` · `Truncate` — Blender Float to Integer (Round/Floor/Ceiling/T… | O ITEM É DO value.quantize, E ELE FECHOU EM 2026-08-15 (a linha dele, abaixo: Truncate no índice 3). Reconferido no código em 2026-08-19 — Mode::{Round, Floor, Ceil… | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.49 |
| `value.unary` — `One Minus` / `Invert` — presente em toda referência (Blender Subtract-de-1; Ca… |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.50 |
| `value.unary` — `Sin`/`Cos`/`Exp`/`Log` | HR-5 (mesma cerca do `value.math`); o seno periódico existe em `value.wave`/`value.lfo` (aproximação parabólica, transcendental-free) | vivo, tabela |
| `value.unary` — `Clamp` — Blender ship um nó Clamp dedicado (Min/Max, modos Range/MinMax) |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.52 |
| `value.gain` — a faixa de operação — o nó CLAMPA a entrada em `[0,1]` e o doc-comment manda pô… | REFUTADO 2026-08-26 — a cerca declarada FOI MEDIDA e está de pé (~~P2~~) | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.56 |
| `value.map_range` — "· curva (D2)" — doc 63 §3.2 |  | vivo, tabela |
| `value.step` — histerese / Schmitt trigger (`Off to On Delay` / `On to Off Delay`) — TD Logic… | REFUTADO 2026-08-26 — a cerca existe e é MAIS FORTE do que a célula sabia (~~P2~~) | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.74 |
| `value.lfo` — Minimum / Maximum em vez de `amplitude`+`offset` — Cavalry Oscillator e Noise u… |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.83 |
| `value.lfo` — Custom wave por CURVA — Cavalry "Wave Style: … Custom (Graph)" |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.84 |
| `value.lfo` — Pulse Width / duty cycle do Square — TD LFO CHOP |  | vivo, tabela |
| `value.lfo` — Time Offset / Time Scale por nó — Cavalry: "Time (auto-conectado; desconectável… |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.86 |
| `value.wave` — mesmas linhas do `value.lfo` (custom wave · duty · min/max) |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.88 |
| `value.noise` — Min/Max (cluster RANGE) |  | vivo, tabela |
| `value.time` — loop / ping-pong do relógio — TD Timer CHOP (`length`, `cycles`, `cycle limit`) |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.95 |
| `value.time` — saída em FRAMES além de segundos — Blender Scene Time ship as duas saídas; TD T… |  | vivo, tabela |
| `value.instance_field` — Min/Max do Random — Blender Random Value; Cavalry Random |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.107 |
| `value.instance_field` — Boolean / `Probability` — Blender Random Value (Data Type Boolean + Probability… |  | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.108 |
| `value.instance_field` — um 4º modo "Random Integer / N baldes" |  | vivo, tabela |
| `value.attribute` — tipos de dado (int/bool/color) — Blender Named Attribute tem 7 | N/A por domínio: o VALUE deste grafo é escalar por contrato (doc 12); cor tem a própria família (`motion.color_ramp`/`color_array`) | vivo, tabela |
| value.smooth · median · percentile… — `Effect` / dry-wet 0..1 — TD Filter CHOP o ship; e todo deformer deste repo tem… |  | vivo, tabela |
| `value.number` — uma constante de TEXTO (o `Value`/`String` das entradas do Blender GN) | recusada | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md), tabela l.233 |
| `value.table` — vários fluxos de dados de UM ficheiro: o `.mgjson` da Adobe carrega N streams c… | recusada | vivo, tabela |
| _Linhas ✅ (fechadas) da tabela_ — arquivadas quando o doc passou dos 40 KB | O placar e o gate `conferencia()` leem as linhas VIVAS (≥ 1 por nó); as fechadas estão verbatim no arquivo | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/15_value.md) |

**Data:** 2026-08-09 · **Linha:** `line/motion-value` · **Briefing:** [doc 89 §3](../89_plano_conferencia_dos_nos.md)
**Referências:** [`referencia_pesquisa_blender_gn.md`](../referencia_pesquisa_blender_gn.md) (Math · Map Range · Mix · Compare · Float Curve · Random Value · Attribute Statistic · Switch/Index Switch/Menu Switch · Blur Attribute) · [`referencia_pesquisa_niagara_stardust.md`](../referencia_pesquisa_niagara_stardust.md) (**Dynamic Inputs** · Float from Curve) · [`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md) (Utilities · Oscillator · Noise · Random · Number Range · Arrays · Contexts) · TouchDesigner CHOPs (Math · Filter · LFO · Pattern · Logic · Analyze · Switch · Slope · Limit) · Houdini VEX/CHOP.

⚠️ **Esta família foi RECUSADA EM ATACADO pelo doc 88 §9** (*"um `value.unary` é um verbo sobre um número; um param a mais ali é cerimônia"*) — **sem uma única comparação com o mercado**. A recusa está revogada (doc 89 §0). O que segue é a comparação.

---

## §0 — O FATO QUE DECIDE METADE DAS LINHAS: o `motion.expression` NÃO alcança o domínio de valor

O briefing pede que a expressibilidade seja **tentada**, e a primeira tentativa é a mais importante: *o `motion.expression` (parser VEX-lite, fórmula como text param) pode tornar exprimível grande parte do domínio de valor?*

**Medido no MANIFEST** (`crates/ph2d-node-motion-expression/src/lib.rs`):

```
inputs:  &[PortSpec { name: "in",  ty: INST_VEC2 }]     ← STREAM, não VALUE
outputs: &[PortSpec { name: "out", ty: VALUE }]
```

E as ligações (`fn attr`): `i` · `n` · `t` · `f` (índice normalizado) · **qualquer coluna `Column::Scalar` por nome** · params `a`/`b`/`c`/`d`.
O parser oferece `+ − * /`, `< > ==`, `&& ||`, `sin cos abs sqrt floor fract min max mix noise smoothnoise select wiggle`.

⇒ **O `motion.expression` é uma FONTE de valor, nunca um TRANSFORMADOR de valor.** Ele lê o *stream* e emite VALUE; ele **não tem porta VALUE de entrada**. Consequência exata:

| Sub-família | O que a expressão pode fazer |
|---|---|
| **fontes** (`instance_field` · `noise` · `lfo` · `wave` · `pattern` · `time` · `attribute`) | **muita coisa** — uma fórmula sobre `i`/`n`/`t`/`f`/colunas cobre boa parte dos gaps |
| **transformadores** (os outros 16, que recebem VALUE) | **NADA** — o valor não tem como entrar na fórmula |

⚠️ **Existe UM caminho de volta, e ele é destrutivo:** `value.* → motion.drive(channel, mode=Set) → stream → motion.expression("<canal>")`. Mas o `channel` do `motion.drive` é um enum de **cinco canais de ARTE** (`X · Y · Rotation · Size · Opacity`) — o round-trip **queima um canal visível**. É o caso-tipo do "exprimível a um custo que ninguém paga" (§7 do plano): entra como **P1**, nunca como refutação.

⚠️ **E o `field.combine` NÃO socorre o domínio de valor:** os portos dele são `INST_VEC2` (medido no MANIFEST), não VALUE — os oito blend modes dele (`Normal · Add · Subtract · Multiply · Screen · Min · Max · Overlay`) vivem no domínio de CAMPO. A ausência deles no `value.mix` é gap real, não fatoração.

**Refutações que SOBREVIVERAM à tentativa** (as boas notícias, contadas na §5): `1−x` · `ceil`/`round` · `clamp` · `AND`/`OR` · `a>b` · min/max-vs-amplitude/offset · custom wave · duty cycle · máscara por probabilidade · random com min/max · Factor da curva · dry/wet dos filtros · clock em loop/pingpong · `range`/`stddev`/`Selection` do reduce · time offset/scale.

---

## §1 — ARITMÉTICA E OPERADORES (4 nós)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `value.math` | idem | **`Power` · `Logarithm` · `Exponent`** — Blender Math os ship | **NÃO** | ⛔ **recusado com motivo, e a cerca está no código:** o comentário do MANIFEST diz *"Power/log are omitted by HR-5"*. E o **uso artístico** já está coberto: `value.gain` (Schlick 1994) dá gamma/contraste **sem `pow`**, e `value.unary` tem `Square`/`Sqrt` | ⛔ | — |
| `value.unary` | idem | `Sin`/`Cos`/`Exp`/`Log` | **NÃO** | ⛔ **HR-5** (mesma cerca do `value.math`); o seno periódico existe em `value.wave`/`value.lfo` (aproximação parabólica, transcendental-free) | ⛔ | — |
| `value.mix` | idem | **BLEND MODES** (Multiply·Screen·Overlay·Difference·Darken·Lighten·Dodge·Burn…) — Blender Mix os ship (18 modos para Color); **e o nosso `field.combine` já tem 8** (`Normal·Add·Subtract·Multiply·Screen·Min·Max·Overlay`) | **NÃO** — ⚠️ **conferido no MANIFEST: `field.combine` opera em `INST_VEC2`, não em VALUE.** Não há caminho | **omissão**, e ela é *assimétrica*: a mesma pergunta tem resposta rica num domínio e nenhuma no outro | ~~**P1**~~ ✅ **FECHADO 2026-08-15** | `blend` default **`Mix`** (lerp) ⇒ hoje **bit a bit**; nove modos, e a lei e' `lerp(a, f(a,b), t)` -- ⚠️ **nao e' uma 2ª porta do `value.math`**: com `t = 1` os dois coincidem, e e' essa coincidencia que prova a lei |
| `value.gain` | `strength` · `mode` (Gain·Bias) | — Houdini VEX `bias()`/`gain()`; Unity Shader Graph **Contrast**; AE Curves S-shape; Cavalry **Falloff Graph** | — | ✅ **NATUREZA, com mecanismo:** a família Schlick tem **um** grau de liberdade (`a ∈ (0,1)`) e o `strength` **é** esse grau re-assinado; um terceiro knob não teria o que controlar. E `strength = 0 ⇒ a = 0.5 ⇒ 1/a − 2 = 0` **exato** ⇒ o neutro é bit-exato por construção (cerca no doc-comment) | — | — |

---

## §2 — REMAPEAMENTO E FAIXA (5 nós)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `value.map_range` | idem | *"· curva (D2)"* — doc 63 §3.2 | **SIM — e é um nó IRMÃO:** `value.curve` existe (crate `ph2d-node-value-curve`, `ParamWidget::Curve`, LUT no device). ⚠️ **Item do doc 63 ENVELHECIDO** | — | ⛔ | — |
| `value.curve` | `in_lo` · `in_hi` · `out_lo` · `out_hi` · **`curve`** (text param, `ParamWidget::Curve`) | **`Factor`** (influência 0–1, lerp entre entrada e curva) — Blender **Float Curve** o ship como a PRIMEIRA entrada (§B, URL `.../math/float_curve.html`) | **SIM** — `value.mix(a = entrada, b = saída da curva, factor)`, 1 nó + uma bifurcação do fio | omissão de ergonomia | ~~**P2**~~ ✅ **FECHADO 2026-08-22** | `factor = 1` ⇒ hoje, **verbatim**: as duas âncoras saem por RAMO, porque `v + 1,0·(moldado − v)` não é `moldado` em `f32` — a álgebra sozinha mudaria o DEFAULT do nó por um ulp em todo valor. Acima de `1` **extrapola** de propósito (a caricatura da curva), que um clamp proibiria em silêncio |
| `value.wrap` | `lo` · `hi` · `mode` (Clamp·Repeat·Mirror) | — Blender: **três nós** para isto (`Clamp` · Math `Wrap` · Math `Ping-Pong`); TD **Limit CHOP** (Clamp/Loop/Zone/Quantize) | — | ✅ **NATUREZA, e melhor que a referência:** o dropdown colapsa três nós do Blender em um; os três modos **são** as três maneiras de tratar o que sai da faixa (parar · repetir · refletir) | — | — |
| `value.quantize` | `step` · `mode` (Round·Floor·Ceil) | **`Truncate`** (para zero) — Blender **Float to Integer** ship os 4; TD Math CHOP **Integer: None/Round/Floor/Ceiling** | **NÃO em 1 nó** — `sign · floor(abs)` = 3 nós | **omissão** (um item de enum) | ~~**P1**~~ ✅ **FECHADO 2026-08-15** | `Truncate` apendado no indice 3 ⇒ 0..2 intactos. A assinatura visual e' o **degrau de largura DUPLA sobre a origem** (os dois lados colapsam para zero) -- e' isso que o gate da cena mede |
| `value.step` | idem | **`Smoother Step`** como 3º modo — Blender Map Range | **NÃO** (mesmo item do `map_range`) | omissão | ~~**P1**~~ ✅ **FECHADO 2026-08-15** *(compartilhado com `map_range` — UMA wave)* | modo apendado no indice 2. ⚠️ A lei DEGENERADA (`width = 0` colapsa no degrau duro) mora numa funcao so' (`band_t`) e o modo novo a HERDA -- uma 2ª copia concordaria hoje e divergiria ao primeiro ajuste da guarda |

---

## §3 — GERADORES TEMPORAIS E ESTOCÁSTICOS (4 nós)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `value.lfo` | idem | **Pulse Width / duty cycle** do Square — TD **LFO CHOP** | **SIM** — `lfo(Saw) → value.step(threshold = duty, Hard)` = quadrada de duty arbitrário, 2 nós | natureza | ⛔ | — |
| `value.wave` | idem | ⚠️ **observação estrutural, não gap:** `value.wave` e `value.lfo` compartilham as 5 formas e 4 dos 5 params — a diferença é **qual eixo dirige a fase** (`i/n` vs `t`). Nenhuma referência faz esse corte (a Cavalry usa UM Oscillator com **Use Index Context**; o Wave Texture do Blender recebe uma coordenada) | um enum `source = Time \| Index` colapsaria os dois num nó | **decisão de produto** (mata um id de nó) | — | — |
| `value.noise` | idem | Min/Max (cluster RANGE) | **SIM** — mesma bijeção do `value.lfo` | natureza | ⛔ | — |
| `value.time` | idem | **saída em FRAMES além de segundos** — Blender **Scene Time** ship as duas saídas; TD Timer tem `units` | **SIM** — `rate = fps` | natureza | ⛔ | — |

---

## §4 — FONTES POR-INSTÂNCIA (2 nós)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `value.instance_field` | idem | um 4º modo "Random Integer / N baldes" | **SIM** — `Random → map_range(0..N) → quantize(1, Floor)` | natureza | ⛔ | — |
| `value.pattern` | idem | **offset/rotação do padrão** (`(i+k) % steps`) | **NÃO** (não há entrada de índice) | omissão | ~~**P2**~~ ✅ **FECHADO 2026-08-22** | `offset = 0` ⇒ hoje, bit-a-bit **nos dois modos** (`round(i)` e `floor(i)` são `i`, e em `Linear` a fração `0` devolve o slot verbatim). A volta é periódica nos DOIS sentidos por `rem_euclid` — o `%` de Rust devolve negativo, e um offset negativo é o caso normal do knob |

---

## §5 — LEITURA DE ATRIBUTO (1 nó)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `value.attribute` | idem | tipos de dado (int/bool/color) — Blender Named Attribute tem 7 | — | ⛔ **N/A por domínio:** o VALUE deste grafo é escalar por contrato (doc 12); cor tem a própria família (`motion.color_ramp`/`color_array`) | ⛔ | — |

---

## §6 — FILTROS DE VIZINHANÇA (4 nós)

⚠️ **A observação que governa a sub-família inteira, e que nenhuma referência tem:** os quatro filtram sobre a **vizinhança de ÍNDICE** (`i−r .. i+r` na ordem do stream). Em toda referência a vizinhança é **tempo** (TD Filter CHOP: amostras de um canal) ou **espaço/topologia** (Blender **Blur Attribute**: vizinhos da malha; Houdini Attribute Blur: raio no ponto-nuvem). A nossa não é nenhuma das duas — **ela é a ordem em que as instâncias por acaso estão**. O que dá sentido a ela é um `motion.sort` a montante, e **esse é o mesmo mecanismo que o doc 88 §9.2 já escreveu para o `slit_scan`** (*"o eixo é um `motion.sort` a montante"*). ⇒ os quatro são **magros por NATUREZA, com mecanismo nomeado**: o parâmetro que falta é a ORDEM, e ela mora noutro nó.

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `value.smooth` · `median` · `percentile` · `slope` | — | **`Effect` / dry-wet 0..1** — TD Filter CHOP o ship; e **todo deformer deste repo tem mistura** | **SIM** — `value.mix(a = entrada, b = filtrado, factor)`, 1 nó + bifurcação | natureza | ⛔ | — |
| `value.median` | `radius` (0..16; kernel GPU com `MAX_RADIUS` 33) | **`De-Spike` + `Tolerance`** — TD Filter CHOP ship o par | **PARCIAL** — o `median` **É** o de-spike (o próprio teste do arquivo diz *"a lone spike is deleted, not spread"*); falta só a tolerância | omissão | ~~**P2**~~ ✅ **FECHADO 2026-08-25** | `tolerance = 0` ⇒ hoje **ao bit, e não por convenção**: a condição é `|amostra − mediana| > 0`, falsa só quando os dois são o MESMO número. ⭐ **A cerca é honrada:** a janela não cresce — a mediana é calculada como sempre e a tolerância decide **depois** se ela substitui a amostra. ⚠️ A fixtura tem pico (`9`) **e** ondulação (`±0,3`) no mesmo campo, senão a régua não distingue «apaga tudo» de «apaga só o que passa da barra» |
| `value.percentile` | `radius` · `percentile` | — TD e Houdini ship **Median** (p = 0.5) e mais nada | — | ✅ **NATUREZA — e aqui NÓS somos o superset:** um filtro de rank com percentil livre não existe nas referências consultadas. Ver §8 | — | — |
| `value.slope` | `scale` | `Units` (por amostra / por segundo) e `Interpolate` — TD **Slope CHOP** | **SIM** — o `scale` **é** o divisor da unidade | ✅ **NATUREZA:** a derivada tem um grau de liberdade (a unidade por que se divide) e `scale` é ele | ⛔ | — |
| *(família)* | — | **blur ESPACIAL** (por proximidade 2D, não por índice) — Blender Blur Attribute e Houdini Attribute Blur são espaciais | **NÃO** — o `motion.sort` dá **um eixo**, não uma vizinhança 2D | **omissão estrutural** | **NÓ NOVO**, fora desta conferência de params — nomeado na §8 | — |

---

## §7 — AGREGAÇÃO, NORMALIZAÇÃO E SELEÇÃO (3 nós)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `value.reduce` | idem | **`Median` global** — Blender Attribute Statistic; TD Analyze; Houdini | **NÃO** — o `value.median` é **janelado** e o `radius` capa em 16; um rank global precisa do conjunto inteiro | **omissão** | ~~**P1**~~ ✅ **FECHADO 2026-08-15** | modo **7**. ⚠️ **Não é um monóide** — um rank global não tem combinador associativo — então ele **RECUSA o device** pela mesma porta dos dois acima, e o censo diz `[no-kernel]`. Contagem par = a média dos dois centrais (a alternativa, o central-de-baixo, faz a mediana SALTAR quando um elemento entra no conjunto) |
| `value.normalize` | `mode` (Range·MaxAbs) | banda de saída (0..1 vs −1..1 vs livre) — TD Math CHOP `Range from/to` | **SIM** — `normalize → value.map_range` | ✅ **NATUREZA, com mecanismo:** os dois modos **são** as duas normalizações que existem (afim-para-unidade · escala-por-magnitude); uma terceira precisaria de uma banda-alvo, e a banda é trabalho do `map_range` — a **mesma fatoração** que o doc-comment do `field.remap` declara. ⚠️ E o nó já é *melhor* que a referência: no Blender isto são 2-3 nós (`Attribute Statistic → Map Range`) | ⛔ | — |
| `value.switch` | idem | — | — | ✅ **A recusa do doc 88 (*"zero params é o contrato deles"*) SOBREVIVE aqui — e agora com referência em vez de raciocínio:** o seletor é uma **porta** e não um param **de propósito**, porque no Blender o Index Switch também recebe um *field* — é o que permite escolher por instância. Zero params é a forma certa | — | — |

**Contagem (DERIVADA por `ferramentas/placar_conferencia.py`, reconciliada em 2026-09-14):** 78 linhas — **P0 = 0** · **P1 = 0** · **P2 = 0** · ✅ fechadas **52** · ⛔ recusadas/refutadas **24** · natureza **1**. ⭐⭐⭐ **A FOLHA FECHOU, e com ela a CONFERÊNCIA INTEIRA: 455 linhas em 18 folhas, ZERO P0, ZERO P1, ZERO P2.** ⭐⭐ **A wave de 2026-08-26 tirou QUATRO, e só UMA foi código:** o `value.attribute` ganhou quatro canais (**Index · Count · Id · Life Fraction**) e a `Life Fraction` fechou DUAS células de uma vez — a dela e a da curva-sobre-vida, que a referência ranqueia como o 3.º maior roubo de UX e que passou de *dois nós com a vida digitada à mão* para **um nó e zero números**. ⛔ **E as outras duas caíram por CERCA MEDIDA:** a fatoração declarada do `value.gain` funciona (o gain cru chapa `−2..6` em `0 0 1 1 1`; a cadeia devolve `−2,00 −1,60 2,00 5,60 6,00`), e a histerese que o `value.step` pedia **já existe** no `pulse.threshold` — com as DUAS metades (Schmitt de amplitude **e** debounce de tempo), que é mais do que a célula sabia. ⭐⭐ **A última — a avaliação preguiçosa do `value.switch` — fechou em 2026-08-27, e o que a destravou não foi alcance novo: foi reconferir a PREMISSA da recusa.** Ela dizia *«merece a própria linha e o próprio portão de fecho»*, e isso é uma afirmação sobre o RELÓGIO, não sobre o mecanismo — assim que houve linha e portão, a nota tinha de ser relida (§0.0: *quem move o número que tornava algo inalcançável tem de reconferir a nota*). ⚠️ **E o §6 nunca esteve em causa**: o plano é construído pela shell e entregue ao `Cook` como estado, então nem `NodeOp` nem `OpResolver` se mexem e nenhum sítio de chamada muda — o que a recusa tratava como *mudar o escalonador* era, medido, *acrescentar-lhe um mapa*. ⏸️ Fica **18 células em pausa na folha 16 (rig)**, que é decisão do Enio e não trabalho pendente. ✅ **A folha não tem mais P1** — o último era um PONTEIRO para um item já fechado (a linha do `value.unary`), e o `switch: N entradas` desceu a P2 por medição. ⚠️ **A wave de 2026-08-22 fechou DEZ P2 de uma vez** (`MultiplyAdd` · `SmoothMin/Max` · `Clamp Result` · `Factor` da curva · `offset` do `quantize` · `invert` do `step` · `fade_in` do `lfo` · `interp`+`offset` do `pattern` · `blend` do `switch`), e o que as juntava não era a data: **cada uma é um default que REDUZ ao nó que shipava**, e as dez têm o mesmo gate de neutralidade (bit-a-bit na CPU, e um PAR com controle no `gpu_cpu_parity_arith`, medido no device). Cena de smoke: **`=78`**.

Re-medir: `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"` — ⚠️ **esta linha é DERIVADA da coluna `P` da tabela acima; não a edite à mão** (a contagem desta conferência envelheceu SEIS vezes, e a folha 13 chegou a contradizer a própria prosa três parágrafos abaixo).

---

## §8 — `SUPERAR:` — o que só o nosso substrato torna barato

1. **O filtro cujo EIXO é escolhido a montante — e ele já existe, sem ninguém saber.** Nenhuma referência deixa você dizer *"borre este valor ao longo do eixo X do layout"*: o TD borra ao longo do TEMPO, o Blender ao longo da TOPOLOGIA. Os nossos quatro filtros leem a **ordem do stream**, e o `motion.sort` **é** o seletor de eixo ⇒ `sort(por X) → smooth(r=8) → sort(por Index)` é um blur direcional de um campo arbitrário ao longo de um eixo arbitrário, **sem kernel novo**. Isto não é código: é um doc-comment e uma cena de smoke. (E é o mesmo mecanismo que o doc 88 §9.2 escreveu para o `slit_scan`.)
2. **Ruído que faz LOOP EXATO — não "loop-ish".** A Cavalry aproxima o Looping sobre frames; o nosso `value.noise` já é **função pura de `(i, t, params)`** e o scrub é **bit-exato** (GGPO, doc 11) ⇒ um `loop_period` aqui garante *o mesmo valor em `t` e em `t+P`, ao bit*, e nenhuma referência pode prometer isso porque todas re-simulam. O item P1 vira um diferencial ao ser construído com essa garantia gateada.
3. **Aleatoriedade por `id`, determinística cross-OS.** O Blender teve de **construir** ids estáveis dentro do *Distribute Points on Faces* para que o Random Value sobrevivesse a mudanças de densidade. O nosso `motion.emitter` já cunha `hash(seed, id, lane)` e o determinismo cross-OS é **gate de CI**. Chavear o `value.instance_field(Random)` no `id` dá um random que sobrevive a `sort`/`cull` **e** é idêntico nas três máquinas — o do Blender é estável por sessão, não por máquina.
4. **O `value.percentile` já É o superset.** TD e Houdini oferecem `Median` (p = 0.5) e nada mais; o percentil livre sobre a janela não aparece em nenhuma referência consultada. É capacidade que temos e não nomeamos — merece a cena de smoke que a torna visível (p=0.1 e p=0.9 sobre o mesmo campo é um "envelope" que ninguém mais desenha em um nó).
5. **A tabela de valores como TEXT PARAM, não como widget de lista.** A cura do teto de 8 do `value.pattern` pelo canal de text param (precedente `motion.expression` / `ramp` do `color_ramp`) dá algo que a lista da Cavalry não dá: uma tabela **ilimitada, colável de uma planilha, diffável no git e versionada com o grafo** — porque o text param já é canal serializado de primeira classe.
6. **O nome que não resolve vira BADGE, não zero.** O gap do `Exists` do Blender é, aqui, um consumidor a mais do `ph2d-motion-diagnose` (ADR-0155) — a classe *"erro que não produz erro"* que aquele ADR já sabe diagnosticar e curar no gesto. A referência devolve um booleano que o artista tem de ligar em algum lugar; nós podemos **apontar para o nó**.

---

## §9 — `CERCAS:` — decisões já registradas (grepadas antes de propor)

1. **`value.math`** — o MANIFEST diz, no comentário do `op`: *"Power/log are omitted by HR-5"*. Recusa deliberada, não esquecimento.
2. **`value.gain`** — o módulo inteiro argumenta **Schlick 1994** *porque* evita `pow` (HR-5) e torna o port GPU bit-comparável; e `strength = 0 ⇒ a = 0.5 ⇒ 1/a − 2 = 0` **exato** ⇒ *o neutro é bit-exato por construção*. Qualquer reparametrização tem de preservar isso.
3. **`value.gain`** — *"the input is CLAMPED to `[0,1]` … put a `value.map_range` before it"*: a fatoração está **declarada**, não esquecida.
4. **`value.median` (kernel GPU)** — *"the selection is a counting rank, no sort"*, `MAX_RADIUS` limita a 33, e `vmd_round` casa com o `f32::round` do Rust porque *"a half-even disagreement would size a different window"*. Mexer no `radius` mexe no array de registradores.
5. **`value.attribute`** — nota do **ADR-0136**: o NOME da coluna é text param (dinâmico, inexprimível como binding estático) ⇒ a projeção roda no `StreamOp::Project` e o kernel GPU é `PASSTHROUGH`. Modos novos entram **naquele ladder**.
6. **`value.pattern`** é `NodeUiCategory::Source`, não Utility — é um GERADOR (o mesmo reparo de classificação que o doc 88 fez no `motion.lattice`).
7. **`value.instance_field`** — *"Pure: a fresh field per cook, a pure function of N + params. No clock/state."* Um `use_node_as_seed` é estrutura de grafo, não estado — mas tem de nascer **desligado** (lei 6).
8. **`value.math`** — divisão por zero *"collapses to zero, not infinity"*, com gate próprio. Não "consertar" para `inf`.
9. **`value.curve` · `value.map_range` · `value.noise`** carregam **`ParamHardMax`** (`in_hi`/`out_hi` 100 · `amplitude` 100 · `frequency` 4) — o slider dual do doc 88 §11. Todo param novo decide o próprio teto duro.
10. **`value.lfo`** é o único da família com **`ParamUnitDecl`** (`period` em segundos). Todo param temporal novo declara unidade.
11. **`value.mix`** — o doc-comment diz que o `clamp` é *"Blender's Clamp Factor"*: a ausência do *Clamp Result* é conhecida, não omissa por descuido.
12. **`value.switch`** tem **zero `register_param_ui`** — é o único da família sem hints, e isso é correto (não há param).

---

## §10 — `O DOC 63 ERROU EM:` (envelheceu nos DOIS sentidos)

1. **`value.curve` marcado `FALTA` / P0** (§2.4: *"Float Curve — mapeia valor por CURVA (widget D2) + Factor"*) — **JÁ EXISTE**: crate `ph2d-node-value-curve`, `ParamWidget::Curve`, LUT no device, `ParamHardMax`. Só o **`Factor`** falta, e ele é **refutável** por `value.mix`. ⚠️ Item que manda construir o que está construído.
2. **`value.index_switch` marcado `FALTA` / P1** (*"N entradas por inteiro"*) — **JÁ EXISTE como `value.switch`** (porta `select` + `in0..in3`). O que resta é N > 4, e isso esbarra em `NodeManifest.inputs: &'static [PortSpec]` (**contrato congelado**), não numa wave de params.
3. **`value.stat` P1** — a frase *"o nosso `Sum/Min/Max` GPU ganha os irmãos"* está desatualizada: **`Mean` já entrou**. A lista honesta de hoje é `median · stddev · variance · range · Selection · GroupID`, e **três deles são refutáveis por composição** (range 3 nós · stddev 5 · Selection 4) — o que muda a prioridade de metade do item.
4. **`value.random` P0** (*"nó dedicado: Float/Int/Bool(probability), min/max, ID+Seed separados"*) — **metade já existe** (`value.instance_field(Random)` + `seed`), e **min/max e bool-probability são REFUTÁVEIS** (`map_range` · `step`, 1-2 nós). ⚠️ **O que sobra do item é justamente o que ele NÃO diz:** a chave é o **ÍNDICE**, não o `id` — e é essa metade que é P0.
5. **§3.2 para `value.map_range`** pede *"interpolation … · curva (D2)"* — a metade "curva" foi satisfeita por um **nó irmão** (`value.curve`), não por um param. Só o enum de 4 modos sobrevive do item.
6. **Doc 88 §9 conta "VALUE (24 nós)"** — são **23**. Medido por duas rotas independentes: `ls crates | grep '^ph2d-node-value-'` = 23 e `grep -rh 'NodeTypeId::of("value\.' | sort -u` = 23. Um número errado numa tabela de recusa é como uma família ganha ou perde um nó sem ninguém ver.

---

## §11 — Placar

| P | n | itens |
|---|---|---|
| **P0** | **0** | ~~`value.noise` **`type`** (Perlin/Simplex/Value/**Cellular** — kernel novo)~~ ✅ **FECHADO em 2026-08-13** (`kernel` = Value/Perlin/Cellular; o **Simplex recusado por medição**, ver a linha §1) — ⚠️ o `value.instance_field` **chave por `id`** FECHOU em 2026-08-13 (`KeyBy::{Index,Id}`, o `key` gateado fora do modo Ramp, que é posicional por definição), e o ⚠️ **quem move o número reconfere a nota:** esta contagem sobreviveu ao próprio fechamento por um dia — ⚠️ o terceiro (`value.attribute`, componentes X/Y de `Vec2`) foi **reconferido em 2026-08-12 e já estava fechado**: o `MODE_COMPONENT_BASE` shipou antes de alguém voltar a esta folha |
| **P1** | **0** | ~~`math`: Modulo~~ ✅ · ~~compare-com-epsilon~~ ✅ (grupo E) · ~~blend modes do `mix`~~ ✅ · ~~`map_range`+`step`: interpolação~~ ✅ · ~~`quantize`: Truncate~~ ✅ · ~~`lfo`: BPM~~ ✅ · ~~`noise`: lacunarity~~ ✅ · ~~loop_period~~ ✅ · ~~pan do domínio~~ ✅ · ~~`time`: idade normalizada~~ **⇒ P2** (grupo E — a célula ERROU: `life` sempre foi coluna) · ~~`instance_field`: `use_node_as_seed`~~ ✅ (grupo D — o param chama-se **`unique_per_node`**) · ~~`pattern`: tabela sem teto (text param)~~ ✅ (grupo D) · ~~`attribute`: `Exists` (via diagnose)~~ ✅ (grupo E) · ~~`smooth`: iterations+weight~~ ✅ · ~~`reduce`: median · stddev/variance · Selection · GroupID~~ ✅ · ~~`switch`: N entradas (⚠️ contrato)~~ **⇒ P2 em 2026-08-19**, e a nota de contrato estava a conflacionar duas coisas: `&'static [PortSpec]` barra a arity **DINÂMICA**, não uma lista estática maior (o §6 congela a contagem de CAMPOS do `NodeManifest`, não o comprimento de um slice). A composição foi **MEDIDA** — `2·⌈(N−4)/3⌉ + 1` nós, todos os índices certos, per-elemento preservado — e a nota antiga (*"3 nós para 8 entradas"*) errava duas vezes. ⚠️ **A folha não tem mais P1.** · ⚠️ Histórico da contagem, que envelheceu três vezes: o grupo C deixou **6** sobre uma lista de seis, o grupo D fechou dois ⇒ **4**; a linha do `value.unary` acima carrega um `P1` que é do **`quantize`**, e aquele já está riscado — quem conta lendo `**P1**` cru acha cinco. |
| **P2** | 5 | ergonomia — ~~MultiplyAdd~~ ✅ · ~~SmoothMin/Max~~ ✅ · ~~Clamp Result~~ ✅ · ~~Factor da curva~~ ✅ · curva-sobre-vida · ~~portas do `wrap`~~ ✅ · ~~offset do `quantize`~~ ✅ · ~~invert do `step`~~ ✅ · histerese · ~~fade-in do `lfo`~~ ✅ · ~~interp/offset do `pattern`~~ ✅ · canais do picker · dry-wet (refutado, fica como conveniência) · ~~blend do `switch`~~ ✅ · faixa do `gain` (natureza declarada) · ~~filtros causais do `smooth`~~ ✅ · ~~De-Spike do `median`~~ ✅ (a cerca do rank sem sort foi honrada: a tolerância decide DEPOIS da mediana) · ~~lanes de `vel`/`size` no picker~~ ✅ · avaliação preguiçosa do `switch` (escalonamento do cook, não param). ⚠️ **Os CINCO que restam são os que não são params.** As três capacidades novas que a nota anterior nomeava fecharam em 2026-08-25 (portas do `wrap` = faixa por-instância · lanes = alcance de uma coluna `Vec2` · filtros causais), e o que sobra é de outra natureza: **histerese** pede ESTADO · **curva-sobre-vida** e **canais do picker** pedem modos novos na escada do `Project` (o `Index`/`Count`/fração-de-vida não são colunas que alguém escreva) · **faixa do `gain`** é natureza declarada · **avaliação preguiçosa do `switch`** é escalonamento do cook. *Nenhum deles é «mais um param».* |
| **⛔** | **17** | recusados **com referência**: ~~range do `reduce` (3 nós)~~ — ⚠️ **saiu daqui e foi CONSTRUÍDO em 2026-08-15**: as duas reduções que ele pede já estavam dobradas, então ele custou um braço de `switch` e não uma capacidade · **iterations do `smooth`** (o `weight` alcança o mesmo com UM passe, medido a 1-2% da amplitude, e no device, onde `N` passes não cabem) · power/log e sin/cos (HR-5, cerca no código) · `1−x` (é `map_range`) · ceil/round (é `quantize`) · clamp (é `wrap`) · AND/OR (são `Min`/`Max`) · min-max vs amplitude-offset (mesma parametrização) · custom wave (é `lfo(Saw)→curve`) · duty cycle (é `lfo(Saw)→step`) · time offset/scale (é `motion.time_remap`) · loop/pingpong do clock (é `wrap`) · min/max do random (é `map_range`) · probability (é `Random→step`) · random-integer (é `map_range→quantize`) · frames do `time` (é `rate`) · dry-wet dos filtros (é `mix`) · edge-detect (é `slope`) · tipos de dado do `attribute` (domínio escalar por contrato) |
| **✅ natureza, com mecanismo** | 6 | `value.gain` (um grau de liberdade em Schlick) · `value.wrap` (as três saídas da faixa) · `value.time` (rápido/quando/por-instância) · `value.slope` (a unidade) · `value.normalize` (as duas normalizações que existem) · `value.switch` (o seletor é PORTA de propósito) · **e os quatro filtros**, cujo param faltante é a ORDEM e ela mora no `motion.sort` |

**Wave sugerida por sub-família** (a §8 do plano manda uma wave por família; dentro desta, a partição natural é):
**W-A** os três P0 (kernels/ladders distintos, mas UMA cena de smoke: um campo aleatório estável sob `sort`, um ruído celular e uma cor pela velocidade-X) · ~~**W-B** o cluster NOISE completo (lacunarity·loop·pan, com o `type`)~~ ✅ **FECHADO** (o `type`/kernel em 13/08, os três restantes no grupo B de 15/08) · ~~**W-C** o cluster REMAP~~ ✅ (grupo A, 15/08) · ~~**W-D** o cluster REDUCE (median·stddev·variance·Selection·GroupID)~~ ✅ **FECHADO** (o grupo C de 15/08, com o `Range` de carona e os pesos do `smooth` na mesma cena) · ~~**W-E** os dois de tabela/semente (`pattern` por text param, `use_node_as_seed`)~~ ✅ **FECHADO** (o grupo D de 15/08, cena `=44`). ⚠️ **Nenhuma das duas metades custou um canal novo:** a tabela viaja pelo `LutSpec` que a curva construiu em 25/07, e a identidade do nó chega ao device por um uniforme **derivado** do kernel — o molde de `declares_window`, não um 7º `KernelResolver`.

---

## §12 — OS TRÊS NÓS QUE ESTA FOLHA NUNCA CONFERIU (2026-09-14, ciclo 6 W4)

⛔⛔ **O cabeçalho diz *«23 nós»* e a família tem VINTE E SEIS.** Os três nasceram depois desta folha
(dois deles desta própria linha), e por isso o placar lia `0 aberto` para eles **por ausência**.
Censo: `every_node_has_a_conference_row_or_is_named_in_the_debt`.

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `value.cursor` | **0** — `in` lido só pela CONTAGEM; saídas `x` e `y` | o `followMouse` do **Cavalry** mora no PRÓPRIO *Falloff*, com gizmo arrastável ([cavalry](../referencia_pesquisa_cavalry.md); folha 08, célula do `followMouse`) | ⭐ **`SUPERAR:`** — um toggle por nó seriam três cópias da mesma leitura (`motion.falloff`, `field.box`, `field.radial_sweep`) e **só saberia mover um CENTRO**; como produtor ele alcança **todo param de todo nó** pelo fio dirigido (doc 58). O gizmo arrastável é do painel, não do nó | **natureza** | ✅ **fecha a célula da folha 08** | — |
| `value.input` | 8 — `kind` (Number/Toggle/Trigger/Color) · `value` · `state` · `fire` · `r g b a`, mais o text `name`; saídas `out` · `r g b a` · `pulse` | as entradas tipadas da *State Machine* do **Rive** (Number · Boolean · Trigger) e o *Data Binding* (cor); os *parameters* do **Animator** da Unity; os *User Parameters* do **Niagara** (override por componente) — [doc 128 §2](../128_ciclo_19_o_jogo_manda.md) | **NÃO** — nenhuma cadeia lia um valor do JOGO por nome (só o `$cursor` e os objectos) | **natureza** (ciclo 19) | ✅ | sem publicação vale o número do próprio cartão; o Trigger é um pulso de UM tique decidido por quem escreve (o do Rive; ⛔ o da Unity, que fica armado) |
| `value.follow` | 3 — `mode` (Linear/Smooth/Spring) · `duration` · `bounce` | o *Lag CHOP* do **TouchDesigner**; o `dampTime` do `Animator.SetFloat` da **Unity**; o `xfade_time` do `AnimationNodeTransition` do **Godot**, CORRIDO como oráculo (fixture `godot_xfade.txt`, pior `≤ 2e-5`) — [doc 128 §5.2](../128_ciclo_19_o_jogo_manda.md) | **NÃO** — o `value.smooth` é ESPACIAL e o `motion.spring` só age em canais de transformação | **natureza** (ciclo 19) | ✅ | Linear `0,3 s`; a mudança conta a partir do tique em que CHEGA (o Godot) |
| `value.number` | idem | — | — | ⚠️ **O `debug.const` foi RETIRADO com a nota *«a capacidade já existe por duas cadeias de UM nó»*, e o report do dono refutou-a**: três atalhos existirem e o artista não achar nenhum não é uma capacidade. *Exprimível não é alcançável* — a lei desta folha, aplicada a ela própria | **cerca REABERTA** | ✅ **fechada 2026-08-27** | — |
| `value.table` | 2 + 3 textos — `interp` (Step/Linear) · `outside` (Hold/Loop) · `file` · `time` · `value` | **vários fluxos de dados de UM ficheiro**: o `.mgjson` da Adobe carrega N streams com tipo, e o `dataValue()` amostra o instante ([manual AE](https://helpx.adobe.com/after-effects/using/data-driven-animations.html), citado no header do nó) | **SIM, um nó por fluxo** — dois `value.table` sobre o MESMO ficheiro com colunas `value` diferentes entregam `x` e `y`. ⚠️ A cadeia é linear no número de fluxos, e o ficheiro é lido duas vezes | **cerca** (composição de N−1 nós) | ⛔ **recusada** | — |

**Contagem desta secção:** 4 linhas — **P0 = 0** · **P1 = 0** · **P2 = 0** · ✅ **2** · ⛔ **2**.

### ⛔ O que NÃO virou linha, e porquê

O `outside` do `value.table` tem `Hold` e `Loop`, e a memória diz que um leitor de tabela costuma
oferecer mais extensões (`Slope`, `Mirror`). **Não há referência CITADA neste repo** para isso — os
docs cobrem `Extend/Clamp/Clip` de *path deform* ([houdini_mops §260](../referencia_pesquisa_houdini_mops.md)),
que é outro assunto. ⇒ **fica de fora da tabela**, e não como célula em pausa: o placar recusou a
linha sem veredito e tem razão. *Uma linha de conferência sem referência citada é um palpite com
cara de medição*, e uma tabela que a aceitasse passava a contar palpites como dívida medida. Quem
tiver a referência abre a célula.
