# Doc 75 — `value.unary`: o operador de UM argumento do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68 (`value.curve`), 69
> (`value.noise`), 70 (`value.mix`), 71 (`value.quantize`), 72 (`value.gain`),
> 73 (`value.step`), 74 (`value.normalize`).

## O que é

O operador de **um argumento** — aplica UMA função a cada elemento de um campo.
É o **contraponto do `value.math`**, que funde DOIS campos com uma operação: a
mesma convergência *"um nó, um seletor de operação, não uma explosão de crates
por-op"* que os editores maduros alcançam (o Math CHOP do TouchDesigner, a
Expression do Nuke, o `abs`/`sqrt`/`frac` do VEX), mas para uma entrada só. Não
havia como fazer `abs`/`floor`/`square`/`reciprocal` de um campo — o `value.math`
é só binário (Add/Sub/Mul/Div/Min/Max).

- **input** `in` : VALUE (`Instances, Scalar, Frame`, coluna `v`)
- **output** `out` : VALUE — **unário**, comprimento preservado
- **params** `op` (Abs · Negate · Sign · Floor · Fract · Square · Sqrt · Reciprocal)
- **Effect** `Pure` (sem clock, sem estado)

## As operações (todas transcendental-free — HR-5)

Algébricas ou aritméticas, então a porta GPU é bit-comparável à CPU e o nó é
**device-resident** — **paridade de dispositivo medida: `max|d| = 2.38e-7`**
(rampa `[1,5]` por Reciprocal → drive Y, RTX):

- **Abs** — `|x|` (dobra um sinal bipolar para unipolar).
- **Negate** — `−x` (inverte).
- **Sign** — `−1 / 0 / +1` (extrai a direção). ⚠️ **explícito**, porque o
  `f32::signum` do Rust dá `±1` para `±0`, discordando do `sign(0) = 0` do WGSL.
- **Floor** — `⌊x⌋` (trunca para o inteiro abaixo, em QUALQUER escala — distinto do
  `value.quantize`, que posteriza `[0,1]` em `N` níveis).
- **Fract** — `x − ⌊x⌋ ∈ [0,1)` (o dente-de-serra repetido; o resto que o Floor
  descarta). Escrito `x - floor(x)` nos dois lados (não o builtin `fract`).
- **Square** — `x²` (uma forma ease-in, ou uma magnitude).
- **Sqrt** — `√x`, negativos **CLAMPADOS a 0** (ease-out; `sqrt` é algébrico e
  correctly-rounded nos dois caminhos, então exato — não é transcendental).
- **Reciprocal** — `1/x`, `x = 0` **GUARDADO a 0** (nunca um `inf`/`NaN`).

## Decisões

1. **A entrada NÃO é clampada** (aritmética vale em qualquer escala); os DOIS
   guards (Sqrt negativo, Reciprocal por zero) são os únicos limites, e ambos
   produzem `0`, bit-exato nos dois caminhos. No WGSL o guard é um `if`, então
   **nenhum `inf`/`NaN` é sequer computado**.
2. **Só operações algébricas / aritméticas.** `sin`/`cos`/`exp`/`log`/`pow` são
   transcendentais (HR-5 as barra; o `value.lfo` usa a aproximação parabólica); o
   `sqrt` **não** é (é algébrico, correctly-rounded), então entra.
3. **Sign é explícito nos dois lados**, e Fract é `x − floor(x)` (não `fract`), para
   a paridade não depender da igualdade byte-a-byte de dois builtins.

## Rejeitados

- **Ceil / Round** — Ceil é `−floor(−x)`; Round é quantize-com-passo-1. A família de
  arredondamento converge para Floor+Negate e para o `value.quantize`; incluir os
  dois seria redundância. Floor **fica** por não ter outra casa (o quantize é
  `[0,1]`-domain).
- **sqrt tabelado / reciprocal por Newton** — o hardware `sqrt` e `1/x` são
  correctly-rounded e mais rápidos que uma tabela (a `line/Painter` MEDIU e reprovou
  um LUT de sqrt no impasto). Usar o builtin.
- **Um nó por op (`value.abs`, `value.floor`, …)** — a explosão de crates que o
  `value.math` já recusou. Um nó, um seletor.

## Preço / cobertura

Kernel WGSL = a porta verbatim de `unary_one` (um `switch` sobre `op`, o `Sign`
explícito, os dois guards por `if`), binding `ReadWrite` na coluna `v`,
`count_law: None` (unário). Sem `applicable` ⇒ **sem fallback de CPU**. Paridade RTX
pelo caminho **Reciprocal** (a divisão real, o guarded — não um `abs` trivial); naga
valida o `switch`.

**Gates:** cada op computa sua função (a família de sinal, o par de arredondamento,
os algébricos; `sign(0)=0` NÃO é `signum`) · os dois guards mantêm finito (`sqrt`
negativo→0, `1/0`→0, e finito para qualquer entrada) · Floor + Fract reconstroem a
entrada (o par complementar) · cook end-to-end (`abs` de `[-2,0,3]` = `[2,0,3]`) ·
registro · **paridade de dispositivo** (`#[ignore]`, RTX, Reciprocal sobre `[1,5]`).

## Demo — `PH2D_VALUE_UNARY_SMOKE=1`

Duas fileiras de 24 instâncias, o MESMO driver bipolar `[-2, 2]`: de cima
`value.unary(Abs)` o dobra num **V** (alto nas pontas, `0` no meio onde o driver
cruza zero); de baixo o driver **cru** direto em Y — uma rampa reta que atravessa o
zero (a referência). O nó marcado `>> EVALUATE <<` é o unary — selecione, troque
**Op** para Square (o V vira parábola), Sign (patamares `±1`), Floor (degraus) ou
Fract (dentes de serra).
