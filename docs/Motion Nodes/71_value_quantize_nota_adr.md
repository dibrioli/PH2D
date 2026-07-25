# Doc 71 — `value.quantize`: a escada do domínio de VALOR (nota-ADR)

**Data:** 2026-07-25 · **Linha:** `line/motion-value` (reaberta pós-integração) · **Modo:** L

## O que é

`value.quantize` — snapa um valor numa **grade** de espaçamento `step`: o visual de
**degraus / posterizado**. É o terceiro SHAPER de valor, ao lado do `value.map_range`
(linear) e do `value.curve` (forma livre): onde aqueles movem um valor suavemente,
este colapsa um campo contínuo em níveis discretos — a assinatura de stop-motion,
drives grosseiros, quantização retrô. Um nó de VALOR unário (`(Instances, Scalar,
Frame)` no `v`), `Pure`, HR-5.

## Pesquisa (regra-ouro — porto por SEMÂNTICA, não por código)

Todos convergem em snap-a-grade com um modo de arredondamento:

| App | Nó / recurso |
|---|---|
| **TouchDesigner** | **Limit CHOP** (Quantize) — Step + modo |
| **Blender** | **Snap** / increment (Math → Snap) |
| **After Effects** | **Posterize** (níveis) |
| **Cavalry** | **Quantize** |
| **Houdini** | `rint`/`floor` sobre a grade |

## As decisões

1. **Grade por TAMANHO, não por contagem.** `q = round(v / step) · step` snapa `v`
   ao múltiplo mais próximo de `step` — **range-agnóstico**, então compõe (um
   `value.map_range`/`value.curve` antes ou depois define a faixa). `step = 0.25`
   vira uma rampa `[0,1]` em `{0, 0.25, 0.5, 0.75, 1}`. Contagem-de-níveis exigiria
   uma faixa embutida (a segunda porta que o domínio evita — o `map_range` já é o
   dono da faixa).

2. **`step = 0` é PASSTHROUGH** (a identidade). Um nó recém-largado não faz nada até
   você escolher a grade — seguro largar antes de decidir. Guarda contra divisão por
   (quase-)zero.

3. **`mode` (Round/Floor/Ceil).** Round = mais próximo (escada simétrica); **Floor**
   = snapa PARA BAIXO (a escada sample-and-hold, o valor nunca excede a entrada);
   Ceil = para cima. Round é meio-longe-do-zero (o `f32::round` do Rust), casado ao
   `vq_round` do WGSL (o `round` do WGSL é meio-par) — CPU e GPU concordam.

4. **100% GPU-resident, sem fallback.** O kernel WGSL é o porto do mesmo snap; sem
   gate `applicable` (o norte "maximize GPU"). `NodeManifest=8` intacto (o kernel é
   side-metadata).

## Alternativas rejeitadas

- **Contagem de níveis (`steps = 5`) em vez de tamanho:** exigiria uma faixa `[lo,hi]`
  embutida — o `value.map_range` já é o dono da faixa, e duas portas para "a faixa"
  divergem. Compõe-se: `map_range → quantize → map_range`.
- **Um `offset` para fasear a grade:** marginal — o `mode` já cobre a maior parte da
  escolha de fase (round centra, floor alinha às bordas), e um `value.math` de soma
  antes/depois faz o resto. Fora por enquanto; fácil somar se um uso pedir.
- **Uma versão suave (smoothstep entre níveis):** derrotaria o propósito (degraus
  DUROS). Quem quer suave usa o `value.curve`.

## O preço (medido)

- Paridade CPU↔GPU no **dispositivo (RTX)**: `max |Δ|` = **0** (BYTE-IDÊNTICO) no
  canal P de `grid → lfo → quantize(Floor, step 0,37) → drive(Y)` (o gate
  `value_quantize_kernel_matches_the_cpu_on_the_device`, `#[ignore]`). O snap é
  `/`·`floor`·`×` — sem transcendental, sem aproximação, então os dois lados casam
  ao bit (melhor que os nós de onda, que carregam ~1e-6 do seno paramétrico). Floor
  + um `step` não-redondo é o pior caso para um deslize `round` ou `/`-depois-`×`.
- O `generated_wgsl_validates` (naga, presença exaustiva) valida o kernel.

## Demo

`PH2D_VALUE_QUANTIZE_SMOKE=1` — duas fileiras de 24, o MESMO LFO viajante, só o
quantize difere. De cima **SMOOTH** (`lfo → drive(Y)`): uma senoide contínua que
ondula. De baixo **STEPPED** (`lfo → quantize(step=1) → drive(Y)`): a MESMA onda
snapada numa grade de 1 — os pontos pousam em Y discretos, uma senoide em degraus.
Selecione o quantize → arraste o **Step** (grade mais grossa) e troque o **Mode**
(Round/Floor/Ceil). Cozinha na GPU.
