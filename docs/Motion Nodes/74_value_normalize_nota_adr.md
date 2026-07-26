# Doc 74 — `value.normalize`: o FIT-TO-RANGE (o 1º reducer do domínio de valor) (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68 (`value.curve`), 69
> (`value.noise`), 70 (`value.mix`), 71 (`value.quantize`), 72 (`value.gain`),
> 73 (`value.step`).

## O que é

O primeiro **reducer** do domínio de valor. Todos os outros nós de valor mapeiam a
instância `i` a partir da instância `i`; este responde `i` com um número que **não
existe até olhar TODAS as instâncias** — o min e o max do campo. É a forma
`reduce → broadcast → map` que os deformers (bend/twist/spherize) usam, aplicada a
uma coluna `v`.

- **input** `in` : VALUE (`Instances, Scalar, Frame`, coluna `v`)
- **output** `out` : VALUE — **unário**, comprimento preservado
- **params** `mode` (Range | MaxAbs)
- **Effect** `Pure` (sem clock, sem estado)

## Por que importa

A família inteira de shapers — `value.gain`, `value.curve`, `value.step` — opera em
`[0,1]`, e o conselho vigente era *"faça um `value.map_range` do driver cru
primeiro"*. Mas o `map_range` exige que você **saiba** o range de entrada, e um
`value.noise`, um `instance_field` Random ou um `value.attribute` tem range
**desconhecido**. O `value.normalize` o **descobre**: sem min/max digitado, sem
adivinhar.

## A referência: fit-to-range por extensão

O padrão-ouro é encaixar pela extensão do próprio campo — o `fit()` do Houdini com
um min/max promovido a detail, o Math CHOP "Range" (from auto) do TouchDesigner, o
"Fit" de um grade. Roda `min` e `max` sobre o stream inteiro, que são **reduções
bit-exatas** (associativas e exatas em qualquer ordem — sem ε, ao contrário de um
`Sum`), então a porta GPU casa com a CPU termo a termo e o nó é **device-resident**
— **paridade de dispositivo medida: `max|d| = 4.77e-7`** (rampa esticada a `[-3,5]`
→ normalize Range → drive Y, RTX; essencialmente exata).

**`mode`** escolhe o range-alvo:

- **Range** — `(v − min) / (max − min)` → `[0,1]`. O fit automático; onde cada
  elemento se situa entre o baixo e o alto do campo.
- **MaxAbs** — `v / max(|min|, |max|)` → `[−1,1]`, **sinal e zero preservados**. O
  normalize certo para um sinal BIPOLAR (um LFO ou noise em torno de `0`): o Range
  deslocaria o zero, o MaxAbs o mantém, escalando só para preencher `[−1,1]`.

## Decisões

1. **Duas reduções `Min`/`Max`, ambas bit-exatas — e os DOIS modos derivam delas.**
   `Max` e `Min` são exatos em qualquer ordem de avaliação (a árvore da GPU dá o
   MESMO bit-pattern do fold sequencial da CPU), então não há ε. O MaxAbs não pede
   uma 3ª redução: `max(|min|, |max|) = max(max, −min)` (o `min` é o mais negativo,
   logo `−min` é a sua magnitude). Um kernel, duas reduções, os dois modos.

2. **Campo degenerado → `0`, nunca uma divisão por zero.** `max == min` (Range) ou
   campo todo-zero (MaxAbs) não têm extensão. Range colapsa em `0` (o min mapeia no
   baixo); MaxAbs preserva o sinal, então um campo constante NÃO-zero mapeia no seu
   **sinal** (`±1`, magnitude cheia) e só o campo todo-zero — já centrado — vai a
   `0`. O guard é um `if` no WGSL, então **nenhum NaN é sequer computado**.

3. **O valor a dobrar é o elemento VERBATIM (`value: "v"`).** Sem produto para o
   device contrair num FMA (a armadilha de paridade do `motion.twist`); com só
   `Min`/`Max` a paridade é exata por matemática, não por ε.

## Rejeitados

- **Um modo Center (subtrai a média)** — precisa de um `Sum` (que é **ε**, não
  bit-exato — a adição de float não é associativa) e da contagem `N`, e é outro
  intento (remover offset, não encaixar num range). Fora de escopo; mantido
  bit-exato. Uma normalização Z-score (standardize) pediria variância → `sqrt`
  (transcendental, HR-5): recusada.
- **Params de range de saída (out_lo/out_hi)** — misturaria o *fit* com o *place*;
  Range dá `[0,1]`, MaxAbs dá `[−1,1]`, e um `value.map_range` a jusante posiciona.
  Mantido puro (é exatamente o par com o `map_range`: um descobre o range de
  ENTRADA, o outro define o de SAÍDA).
- **Um `value.reduce` genérico (broadcast do min/max/sum)** — seria mais
  Houdini-like (promover, depois usar), mas menos ergonômico: `(v−min)/(max−min)`
  como nós soltos são vários nós e duas reduções, enquanto o caso comum (encaixar)
  é UM drop. O reducer geral pode vir depois; este é o composto que se alcança.

## Preço / cobertura

Kernel WGSL = a porta verbatim de `normalize_one` lendo `reduce_min()` /
`reduce_max()`, binding `ReadWrite` na coluna `v`, `count_law: None` (unário). A
sequência roda as duas reduções ANTES do kernel (passe próprio) e entrega os dois
escalares ao corpo. Sem `applicable` ⇒ **sem fallback de CPU**. Paridade RTX pelo
caminho **Range** (min no numerador E no denominador, o exercício completo das duas
reduções); naga valida. ⚠️ **Prova que o canal de reduce dos deformers generaliza
ao domínio de valor** — o `reduce → broadcast → map` rodou 100% no device sobre uma
coluna `v` Scalar, não só sobre `P` Vec2.

**Gates:** Range encaixa em `[0,1]` (min→0, max→1, meio→0.5, identidade sobre campo
já normalizado) · MaxAbs preserva sinal e zero (mais-negativo→−1, 0→0) · campo
degenerado é finito e bem-definido (constante Range→0, constante MaxAbs→sinal,
todo-zero→0) · saída finita para qualquer campo · cook end-to-end (campo `[10,40]`
de range desconhecido → `[0,1]`, a redução descobriu min=10/max=40) · registro ·
**paridade de dispositivo** (`#[ignore]`, RTX, Range sobre range `[-3,5]` real).

## Demo — `PH2D_VALUE_NORMALIZE_SMOKE=1`

Duas fileiras de 24 instâncias, o MESMO driver num range arbitrário `[2, 10]`: de
cima `value.normalize(Range)` acha `min=2`/`max=10`, encaixa em `[0,1]` e escala à
altura da fileira — a rampa **ancora no chão e sobe até `ARCH`**; de baixo o driver
**cru** direto em Y fica em `[2, 10]` — deslocado para cima e duas vezes mais alto
(a referência). O nó marcado `>> EVALUATE <<` é o normalize — selecione, troque
**Mode** para MaxAbs e veja a fileira re-centrar em torno do zero.
