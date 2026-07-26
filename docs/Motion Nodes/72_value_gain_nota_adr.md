# Doc 72 — `value.gain`: o shaper de CONTRASTE / GAMMA do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68 (`value.curve`), 69
> (`value.noise`), 70 (`value.mix`), 71 (`value.quantize`).

## O que é

O quarto **shaper** do domínio de valor. `value.map_range` move um campo
linearmente; `value.curve` o passa por uma LUT desenhada à mão; `value.quantize`
o colapsa numa grade. Faltava o shaper **paramétrico** — a curva-S de **contraste**
e o **gamma** — cujo único knob (`strength`) é **animável e dirigível por
instância**, o que uma curva assada não é.

- **input** `in` : VALUE (`Instances, Scalar, Frame`, coluna `v`)
- **output** `out` : VALUE — **unário**, comprimento preservado
- **params** `strength ∈ [-1,1]` (default **0 = neutro**) · `mode` (Gain | Bias)
- **Effect** `Pure` (sem clock, sem estado)

## A referência: Schlick 1994

O padrão-ouro é **Schlick, "Fast Alternatives to Perlin's Bias and Gain
Functions"** (Graphics Gems IV, 1994) — as duas funções racionais que dão as
formas de bias/gain **sem `pow`**:

- `bias(t,a) = t / ((1/a − 2)(1 − t) + 1)` — gamma; `a>0.5` empurra para cima.
- `gain(t,a)` = duas meias-bias em torno de `0.5` — curva-S; `a<0.5` dá contraste.

Perlin as definia com potência (`t^(ln a / ln 0.5)`), **transcendental**. Schlick
é só `+ − × ÷`, então o nó é **transcendental-free (HR-5)** e a porta WGSL é
bit-comparável à CPU — **paridade de dispositivo medida: `max|d| = 4.77e-7`**
(rampa `[0,1]` por gain(0.6) → drive Y, RTX). Todo motor procedural traz o par:
Houdini VEX `bias()`/`gain()`, o Contrast/Gamma de um grade, o S-curve das Curvas
do After Effects, o Contrast do Shader Graph.

## Decisões

1. **`strength ∈ [-1,1]`, 0 = neutro, com SINAL intuitivo.** O `a ∈ (0,1)` nativo
   do Schlick tem o meio em `0.5` e direções contra-intuitivas (para *gain*,
   `a<0.5` = mais contraste). Mapeamos `strength` para `a` com sinais OPOSTOS por
   modo — Gain `a = 0.5 − 0.5·strength`, Bias `a = 0.5 + 0.5·strength` — para que
   **positivo seja sempre "mais"** na direção natural do modo. Em `strength = 0`,
   `a = 0.5`, onde `1/a − 2 = 0` **exato** (potência de dois) ⇒ as duas formas
   reduzem à identidade **BIT-EXATA**: o nó é neutro por construção.

2. **Opera na banda `[0,1]`; o input é CLAMPADO.** Bias/gain são definidos em
   `[0,1]`; fora dela o denominador pode cruzar zero e explodir. Clampamos `t` a
   `[0,1]` (a mesma escolha do Float Curve do Blender) ⇒ nunca um NaN. Consequência
   honesta: em `strength = 0` a saída é `clamp(v,0,1)`, não um passthrough total —
   ponha um `value.map_range` antes para normalizar um driver arbitrário e outro
   depois para posicionar o resultado (a separação shape × range: quantize snapa,
   curve range-mapeia, este SÓ molda).

3. **`a` clampado a `[1e-4, 1−1e-4]`** (guarda far-field): `strength = ±1` cai
   exatamente em `a = 0`/`1` (o `1/a` divide por zero / o degenera). O clamp é o
   que torna os extremos finitos; `strength = 0` (`a = 0.5`) mora bem no meio dele.

## Rejeitados

- **Potência de Perlin (`t^γ`)** — transcendental, quebra HR-5 e a paridade de
  dispositivo. Schlick existe exatamente para evitá-la.
- **Range de saída no nó (out_lo/out_hi como no `value.curve`)** — misturaria
  shape com range; `value.map_range` já é o ranger. Mantido PURO `[0,1]→[0,1]`.
- **Dois nós separados (`value.bias` + `value.gain`)** — um nó com `mode` é o
  padrão-pro (como `value.quantize` hospeda Round/Floor/Ceil): uma casa, dois
  shapes, default neutro em ambos.

## Preço / cobertura

Kernel WGSL = a porta verbatim de `gain_one` (clamp + mapa de sinal + as duas
racionais), binding `ReadWrite` na coluna `v`, `count_law: None` (unário). Sem
`applicable` ⇒ **sem fallback de CPU**. Paridade RTX `4.77e-7`; naga valida.

**Gates:** neutro é identidade bit-exata (os dois modos) · Gain espalha os
extremos e pina `0.5` · Bias empurra para uma ponta com endpoints fixos · clamp a
`[0,1]` + finitude em toda força/modo · cook end-to-end · registro · **paridade de
dispositivo** (`#[ignore]`, RTX, `strength = 0.6` — o caminho de divisão real, não
a identidade neutra que esconderia uma porta errada).

## Demo — `PH2D_VALUE_GAIN_SMOKE=1`

Duas fileiras de 24 instâncias, a MESMA rampa: de cima `value.gain(Gain, 0.7)` →
degrau-S de contraste; de baixo a rampa reta (referência). O nó marcado
`>> EVALUATE <<` é o gain — selecione, arraste **Strength** (o S fica mais forte /
inverte) e troque **Mode** para Bias (a rampa curva para uma ponta).
