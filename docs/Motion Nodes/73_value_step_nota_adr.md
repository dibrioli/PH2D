# Doc 73 — `value.step`: o GATE / COMPARADOR do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68 (`value.curve`), 69
> (`value.noise`), 70 (`value.mix`), 71 (`value.quantize`), 72 (`value.gain`).

## O que é

O **gate** do domínio de valor. Os shapers (map_range, curve, quantize, gain)
levam um campo a OUTRO campo da mesma forma; os combinadores (math, mix, switch)
fundem dois campos. Faltava a peça no meio: o **comparador** que transforma um
campo contínuo numa **máscara `[0,1]`** — o peso que um `value.mix` cruza ou o
seletor que um `value.switch` escolhe. É o que os shapers alimentam e os
combinadores consomem.

- **input** `in` : VALUE (`Instances, Scalar, Frame`, coluna `v`)
- **output** `out` : VALUE — **unário**, comprimento preservado, saída em `[0,1]`
- **params** `threshold` (default **0.5**) · `width` (default **0**) · `mode` (Hard | Smooth)
- **Effect** `Pure` (sem clock, sem estado)

## A referência: Step / Smoothstep

O padrão-ouro é o par **Step / Smoothstep** que todo motor procedural traz —
Houdini VEX `step()`/`smoothstep()`, o Step e o Smoothstep dos Shader Graphs do
Unity/Unreal, o Logic/Limit CHOP do TouchDesigner, o Compare do Cavalry. Os dois
são **transcendental-free (HR-5)**: Hard é uma comparação, Smooth é o cúbico de
Hermite `3t² − 2t³`. Então a porta WGSL é bit-comparável à CPU e o nó é
**device-resident** — **paridade de dispositivo medida: `max|d| = 8.34e-7`**
(rampa `[0,1]` por step(Smooth, 0.5, 0.4) → drive Y, RTX; essencialmente exata).

**`mode`** escolhe a borda:

- **Hard** — gate binário: `v ≥ threshold → 1`, senão `0`. `width` é ignorado.
- **Smooth** — banda smoothstep de largura **`width`** centrada no `threshold`:
  `0` abaixo de `threshold − width/2`, `1` acima de `threshold + width/2`, `0.5`
  no limiar, rampa C¹ entre eles. `width = 0` colapsa no Hard, **bit-exato**.

## Decisões

1. **A entrada NÃO é clampada; a saída é uma máscara `[0,1]`.** Uma comparação
   vale em qualquer escala — um `value.step` em `threshold = 2` sobre um
   `instance_field` Index seleciona *"as instâncias além do índice 2"* —, então o
   driver entra com o range que tiver. Só o resultado é normalizado. **Difere do
   `value.gain`**, que clampa a entrada porque bias/gain só existem em `[0,1]`; um
   gate não tem esse limite natural.

2. **`width` é uma banda COMPLETA centrada no limiar.** O artista pensa *"corte no
   0.5, amoleça 0.2"*, não em dois edges soltos (`e0`/`e1` como no Smoothstep de
   shader). `threshold` diz onde, `width` diz quão macio — a parametrização de um
   *"Compare com tolerância"*. `width < 0` é tratado como `0` (a mesma banda
   degenerada do Hard).

3. **`width = 0` faz Smooth ≡ Hard, bit-exato.** A banda degenerada (`hi ≤ lo`) cai
   na mesma comparação (`x ≥ threshold`). As duas rotas concordam na costura, então
   subir o `width` de zero é contínuo — não há salto ao trocar de modo.

4. **O smoothstep é escrito à mão nos DOIS lados** (não o builtin do WGSL): o mesmo
   `(x−lo)/(hi−lo)` clampado + `t·t·(3−2t)`, para a paridade CPU×GPU ser exata.

## Rejeitados

- **Um param `invert` ("abaixo do limiar")** — é `1 − x`, um `value.math` a
  jusante. Mantido mínimo: `threshold`, `width`, `mode`.
- **Uma identidade / passthrough neutro** — um gate é inerentemente uma
  transformação (thresholdar não tem "não fazer nada"); ao contrário do
  `value.map_range` (cujo `[0,1]→[0,1]` é identidade), não há default neutro. O
  default `Hard, threshold 0.5` é um gate limpo no meio.
- **A potência / `pow` para uma banda com curvatura ajustável** — quebraria HR-5 e
  a paridade; o Hermite `3t²−2t³` é o padrão-ouro e é polinomial. Para uma curva de
  borda diferente, empilhe um `value.gain` antes ou depois.
- **Confundir com `pulse.threshold` (domínio PULSE) e `motion.step` (domínio
  MOTION)** — o primeiro detecta uma BORDA e emite um EVENTO; o segundo opera no
  domínio de movimento. Este é o gate do domínio de VALOR, produzindo uma máscara
  contínua por instância.

## Preço / cobertura

Kernel WGSL = a porta verbatim de `step_one` (o `select` do Hard, a banda e o
Hermite do Smooth), binding `ReadWrite` na coluna `v`, `count_law: None` (unário).
Sem `applicable` ⇒ **sem fallback de CPU**. Paridade RTX pelo caminho **Smooth**
(a divisão + o polinômio reais, não o `select` trivial do Hard, que esconderia uma
banda errada); naga valida.

**Gates:** Hard é gate binário no limiar (exato, `width` ignorado) · Smooth ramp
sobe monotônico e pina `0`/`1` nas pontas com `0.5` no limiar · `width = 0` faz
Smooth ≡ Hard bit-exato · a saída é máscara finita em `[0,1]` para toda
entrada/limiar/largura/modo (entrada não-clampada) · cook end-to-end (rampa por
gate Hard 0.5 = `[0,0,1,1,1]`) · registro · **paridade de dispositivo**
(`#[ignore]`, RTX, Smooth `width 0.4` — o caminho de divisão real).

## Demo — `PH2D_VALUE_STEP_SMOKE=1`

Duas fileiras de 24 instâncias, a MESMA rampa: de cima `value.step(Hard, 0.5)` →
**penhasco** (as instâncias colapsam em dois patamares, chão e topo, onde a rampa
cruza `0.5`); de baixo a rampa reta (referência). O nó marcado `>> EVALUATE <<` é o
step — selecione, arraste **Threshold** (o degrau desliza), troque **Mode** para
Smooth e suba **Width** (o penhasco vira uma rampa-S macia).
