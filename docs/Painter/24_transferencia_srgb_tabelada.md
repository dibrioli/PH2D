# 24 — A transferência sRGB vira TABELA (os dois knobs EXPERIMENTAL)

> **Estado:** landou na `line/Painter` (2026-07-23). O 1º smoke REPROVOU
> (*"ainda muito lento"*) e a 2ª rodada respondeu — ver **§6.1**, que é onde
> está o número do produto.
> **Escopo:** `ph2d-wet-paint` + o composite de `ph2d-tool-painter`
> (**emenda 2 do [ADR-0109](../architecture/decisions/0109-rayon-exception-watercolor-composite.md)**:
> o fan-out por linhas). Nenhum schema, nenhum contrato congelado, **nenhuma dep
> nova** (o rayon já era do tool). O caminho default (os dois knobs OFF) é
> **byte-idêntico** — o fingerprint da sessão não se moveu.

## 1. O reporte

> *"No modo Wet Paint: Tuning, temos problemas graves de otimização com Pigment
> mixing e Glaze Layering."* (Enio)

Estava certo, e por uma margem que não é de afinação. Medido no **flood** (a cena
que o [ADR-0134](../architecture/decisions/0134-wet-paint-fluid-sim-returns-cpu-first-parity-tested.md)
declara como *upper bound*: ~110k células molhadas) e no composite de folha cheia:

| | OFF | ON (antes) | razão |
|---|---|---|---|
| tick do sim (pior classe de cadência) | 9,86 ms | **122,77 ms** | 12,5× |
| `drying_pass` | 3,63 ms | 77,44 ms | 21,3× |
| `advect` | 1,68 ms | 57,34 ms | 34,2× |
| composite do produto (900×450) | 6,45 ms | **133,33 ms** | 20,7× |
| `render_region` (referência) | 7,33 ms | 178,15 ms | 24,3× |

O *kill criterion* do ADR-0134 para o tick é **12 ms**. Com Pigment mixing ligado
o tick era **122,77** — dez vezes o orçamento, ou ~8 fps. Inusável, não lento.

## 2. A causa é UMA

`libm::pow`, medido em **~24 ns**, chamado:

- **9×** por mistura de cor (`ColorMix::Km::mix`, 3 canais × 2 entradas + 1 saída),
- **15×** por célula advectada (`km_weighted_mean_color`, 4 cantos × 3 canais + 3),
- **9×** por pixel por camada no glaze.

E ele é caro porque é um `x^y` **geral e correctly-rounded**. Nunca precisamos de
um geral: os expoentes são as duas constantes do sRGB (2,4 e 1/2,4) e o domínio é
[0,1]. Uma tabela de nós + interpolação linear responde a mesma pergunta em
**~2,2 ns** — 11×.

## 3. O desenho: `colorops/transfer.rs`

Três tabelas (`N = 16384`, 384 KB, construídas **lazy** — uma sessão que nunca
liga um knob EXPERIMENTAL nunca as aloca):

- `to_linear` — uniforme sobre [0,1];
- `to_srgb_fine` / `to_srgb_coarse` — o inverso, **partido em 1/32**.

**Por que partir o inverso:** a curvatura dele se concentra no lado escuro
(`f'' ~ r^-1.583`). Com os mesmos 4096 nós, uniforme mede **1,6e-5** e partido
mede **4,3e-7** — 38× melhor pelo mesmo custo de lookup.

### O determinismo é preservado, e por um argumento mais forte

Os nós são computados **uma vez por `libm`** ⇒ todo nó é o valor de referência
bit a bit em qualquer OS. Entre nós correm apenas `+ − * /`, comparação e um
truncamento float→int — todos **exatamente especificados pelo IEEE-754**, e o
Rust nunca os contrai em FMA. Logo a saída é bit-idêntica cross-OS, que é a
propriedade que a *port law* (`lib.rs`) exige de um transcendental.

### Precisão MEDIDA (`tests/transfer_accuracy.rs`, 8 gates)

Um nível de byte = **3,92e-3**. Contra isso:

| grandeza | medido |
|---|---|
| forward, máx \|tabela − libm\| | 1,5e-8 |
| inverso, idem | 7,1e-8 |
| porta 0..255 | 1,3e-8 |
| porta K/S (erro relativo) | 5,4e-5 |
| round-trip de cor | 0,0000 de 255 |
| **lavagem parada, 5000 re-misturas** | **0,016 nível de byte** |
| célula re-misturada por frentes, 5000× | 2,0e-5 de 255 |

## 4. ⛔ MEDIDO E REJEITADO: tabular também a razão K/S

`c → K/S` é uma função 1-D, então *parece* um lookup esperando acontecer — e
fundir de fato mediu **2,2 ms mais rápido** no flood. **Não refaça.**

K/S tem **zero quadrático no branco** e `dR/dKS → −∞` ali, então a composição é
mal-condicionada exatamente onde a tinta é mais clara. A consequência não é
sutil: **uma lavagem PARADA deixa de ser parada.** O `libm` é ponto fixo exato
sob re-mistura (60,00000 continua 60,00000 após 5000 passes); a tabela fundida
caminhava para 59,73 em c=60 e **253,51 em c=254** — meio nível de byte de
deriva de cor em tinta que ninguém tocou.

O porquê, para não se re-derivar: no caminho não-fundido a etapa `KS→R` é a
inversa **exata** de `R→KS`, então o erro da tabela em `r` **cancela** no
round-trip. A tabela fundida não tem essa propriedade — seu erro em KS não
cancela contra nada.

Ir por refletância é bem-condicionado no mesmo ponto (`dKS/dR → 0` quando
`R → 1`), ao preço de **uma divisão**. É a correção mais barata do módulo.
Pinado pelo gate `the_ks_door_tracks_its_exact_chain` (a fusão sangra nele com
erro relativo **2,096** contra barra 1e-4 — quatro ordens de separação).

## 5. Estrutural: a entrada preguiçosa em refletância

`render_region` entrava no espaço linear em **todo** pixel, inclusive onde
nenhuma camada pinta — um round-trip identidade. Folha nua: **1,264 ms → 0,323 ms**.

⚠️ **Este renderizador não tem chamador de produto.** O composite vivo passa por
`render_pigment_region_visual`, cujo braço de glaze já fica atrás de
`la <= 0.0 { continue }` e portanto nunca foi ansioso. O glaze do artista
acelerou **pela tabela**, não por aqui. Os dois ficam consistentes de propósito:
este é o *reference look* da SPEC §13 e os dois não podem divergir sobre o que
um pixel não-pintado custa ou mostra.

⚠️ E é um fix **só de perf**: o round-trip é preciso o bastante para os bytes já
serem idênticos, então nenhuma comparação de saída distingue as duas versões. O
gate correspondente **cronometra** em vez de comparar.

## 6. O resultado

| | OFF | ON (depois) | razão | ganho |
|---|---|---|---|---|
| tick do sim (flood, pior classe) | 10,38 ms | **18,88 ms** | 1,8× | **6,5×** |
| `drying_pass` | 3,48 ms | 9,86 ms | 2,8× | 7,9× |
| `advect` | 1,68 ms | 7,93 ms | 4,7× | 7,2× |
| **sessão representativa** (traço roteirizado) | 0,83 ms | **0,89 ms** | 1,07× | **2,9×** |
| composite do produto (900×450) | 6,49 ms | **18,17 ms** | 2,8× | **7,3×** |
| `render_region` (referência) | 6,88 ms | 22,38 ms | 3,3× | 8,0× |

**O número que decide a usabilidade é a sessão representativa**: num traço real
o checkbox custa **0,06 ms/tick**. O flood continua sendo o pior caso declarado,
e lá o custo de ligar K–M é +8,5 ms sobre um orçamento de 12 — ver §8.

## 6.1 ⚠️ O smoke reprovou: "ainda muito lento, mais lento que o HTML/JS"

O reporte estava certo e a minha medição estava **incompleta** — eu media a 900×450
(o tamanho do reference) enquanto o **smoke abre 1024²**, 2,6× as células. Medindo
o FRAME do produto (um passo de 40 Hz + um composite):

| canvas | knobs | step | composite | frame | fps |
|---|---|---|---|---|---|
| 900×450 (= o JS) | OFF | 3,10 | 4,29 | 7,40 | **135** |
| 900×450 | ON | 11,23 | 8,45 | 19,68 | 51 |
| **1024² (nosso smoke)** | **OFF** | 7,81 | 11,42 | **19,23** | **52** |
| **1024² (nosso smoke)** | **ON** | 28,75 | 21,68 | **50,43** | **20** |

Dois fatos que a tabela expõe e que a minha primeira leitura escondia:

1. **No tamanho do reference nós somos ~3× mais rápidos que a barra dele** (o JS
   pede o flood sob 15 ms/step; nós fazemos 11,3 mesmo com K–M). O "mais lento que
   o JS" é, em boa parte, **canvas 2,6× maior**.
2. **Com os knobs DESLIGADOS o app já estava a 52 fps** a 1024². Parte do que o
   artista sentia **não eram os knobs** — era o custo-base naquele tamanho.

### O composite virou ROW-PARALLEL (ADR-0109 emenda 2)

Ele é um **map puro por-pixel sobre linhas disjuntas** — o caso que o ADR-0109 já
sanciona para o composite óptico da aquarela, no mesmo crate. O engine expõe **uma
linha** (`render_pigment_row_visual`) e **não spawna nada**; o leque abre no tool.

| | serial | 32 vias | |
|---|---|---|---|
| composite 1024², glaze OFF | 15,82 ms | **1,62 ms** | 9,8× |
| composite 1024², glaze ON | 44,00 ms | **3,49 ms** | 12,6× |

**Byte-idêntico e não por argumento:** a medição compara os dois buffers e falha se
um byte diferir.

**Frame resultante a 1024²:** OFF 19,23 → **9,4 ms (52 → 106 fps)** · ON 50,43 →
**32,2 ms (20 → 31 fps)**.

⚠️ **O teto agora é o SOLVER**, que é **serial por semântica** (ADR-0134: o brake lê
wetness viva escrita na mesma passada; o drying lê o vizinho pós-update; o advect
subtrai dos cantos-fonte). Com Pigment mixing ligado ele é **89% do frame**.

## 7. Gates

- **`tests/transfer_accuracy.rs`** (8) — precisão, condicionamento, deriva.
  ⚠️ O oráculo da lavagem parada é a **cadeia exata iterada igual**, nunca "a cor
  fica onde estava": abaixo do piso de refletância toda cor divide um K/S, então
  o próprio `libm` move c=12 → 12,7 na primeira mistura. A 1ª versão deste gate
  falhou por isso — estava medindo a física da referência e chamando de erro da
  tabela.
- **`tests/perf_experimental.rs`** (5) — razões OFF/ON no mesmo run (wall-clock
  mediria o perfil, não o produto), papel nu **com controle positivo**, e o
  glaze grátis onde não há tinta.
- **7 mutações, 7 sangram.** ⚠️ Duas nasceram "sobreviventes" por **erro de
  mutação, não por buraco de gate**: a primeira mutava `srgb_to_linear`, que as
  portas quentes não usam, e a da entrada ansiosa mutava um ponto **depois do
  `continue`** do laço, onde nenhuma das versões entra.

## 8. Aberto

- **O flood com K–M ligado fica em 18,9 ms contra o kill de 12 ms** do caminho
  default. Não é regressão (era 122,8) e a cena é o *upper bound* declarado, não
  o caso típico — mas o número está acima da barra e fica **nomeado**, não
  escondido. O que resta ali é o piso do algoritmo: 15 lookups + 3 `sqrt` por
  célula no `advect`, 18 + 6 no `drying_pass`.
- **A cadeia settle→rewet do `drying_pass` reconverte `susp_rgb`** — os dois
  valores de K/S da segunda mistura já foram computados pela primeira. Vale ~33%
  do custo de K–M ali, e exige passar K/S através de `lift_settled`, que é porta
  **compartilhada** com as tools ativas do doc 23. Seria uma segunda porta com
  numérica diferente (pula o arredondamento f32 intermediário) — **não feito**.
- Os `KNOB_DEFS` e a UI do Tuning não foram tocados.
