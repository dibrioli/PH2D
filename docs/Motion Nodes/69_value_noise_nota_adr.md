# Doc 69 — `value.noise`: o driver COERENTE do domínio de VALOR (nota-ADR)

**Data:** 2026-07-25 · **Linha:** `line/motion-value` (reaberta pós-integração) · **Modo:** L

## O que é

`value.noise` — um **campo de ruído coerente** por instância: um valor suave que
varia ao longo da fileira (espacial) E evolui no tempo (temporal). É o *produtor*
puro de `motion.wiggle` (que escreve um canal de transform), exatamente como
`value.lfo` é o produtor de `motion.oscillator`. Um nó de VALOR fonte
(`(Instances, Scalar, Frame)` no `v`), `Effect::Temporal` (lê o playhead), HR-5.

## Pesquisa (regra-ouro — porto por SEMÂNTICA, não por código)

O "dá vida" de todo editor de motion converge:

| App | Nó / recurso |
|---|---|
| **After Effects** | `wiggle(freq, amp)` — ruído coerente sobre o tempo |
| **Cinema 4D** | MoGraph Random effector, modo **Noise** (por-clone, coerente, animável) |
| **TouchDesigner** | **Noise CHOP** (type, period, harmonics, translate, spread) |
| **Blender** | **Noise Texture** (Scale, Detail, Roughness) |
| **Houdini** | `noise`/`turb` VOP + fBm |

A semântica comum: um campo contínuo amostrado por instância, com detalhe fractal
(octaves) e evolução temporal.

## A decisão que define o nó: COERENTE, não BRANCO

O domínio de valor já tinha aleatoriedade — `value.instance_field` modo **Random**
é um hash por instância: vizinhos **descorrelacionados** (ruído BRANCO, um valor que
salta). O que faltava é o oposto: um campo **contínuo**, em que vizinhos leem pontos
próximos do lattice e **fluem juntos** — uma onda suave que escorre. Essa é a
distinção inteira, e é o que a demo mostra lado a lado.

## As decisões

1. **Reusa o lattice do `motion.wiggle`, não reinventa.** O `noise.rs` é um espelho
   *leaf-local* do `motion.wiggle`: o MESMO `hash2` (mix inteiro u32), `fade`
   (smootherstep `6t⁵−15t⁴+10t³`) e `value_noise_2d` (bilinear), copiados por
   drop-crate — o vocabulário compartilhado é o **comportamento**, não um símbolo.
   Assim `value.noise` e `motion.wiggle` amostram o MESMO campo (gate
   `one_octave_fbm_is_the_bare_value_noise`).

2. **fBm por cima (o superset PRO).** `octaves` (Detail do Blender) + `roughness`
   somam camadas em dobro de frequência, **normalizadas pela soma dos pesos** ⇒ a
   faixa NÃO cresce com o detalhe (a convenção CHOP/wiggle). Em `octaves = 1` é uma
   camada só = exatamente o campo do wiggle.

3. **Os dois eixos, nomeados.** `frequency` escala o eixo das INSTÂNCIAS (detalhe
   espacial — baixo é uma ondulação suave, alto descorrelaciona vizinhos); `speed`
   escala o eixo do TEMPO (0 congela o campo); `seed` desloca o lattice. `amplitude`
   escala, `offset` desloca. `value_i = fbm(t·speed, i·frequency + seed) · amplitude
   + offset`.

4. **100% GPU-resident, sem fallback.** O kernel WGSL é o porto byte-espelho do
   lattice + fade + fBm (as libs `vn_*` ↔ `wg_*`), lendo `params.playhead` (o
   uniforme mágico, como o `value.lfo`). Sem gate `applicable` — o sequenciador
   nunca cai pra CPU (o norte "maximize GPU"). `NodeManifest` intacto (§6 conferido:
   `NodeOp=2`/`OpResolver=1`/`NodeManifest=8`).

## Alternativas rejeitadas

- **Ruído de GRADIENTE (Perlin), como o `motion.noise`:** **descartado para o v1** —
  o `motion.noise` é 2D sobre POSIÇÕES (precisa de `P`), e o produtor de valor lê o
  `in` só para a contagem (não tem posição). O par certo é o `motion.wiggle` (ruído
  de valor 1D sobre `(índice, tempo)`), e reusá-lo mantém os dois byte-consistentes.
- **Expor a `lacunarity`:** **fixa em 2.0** (o padrão quase-universal — Blender/
  Houdini/Cavalry defaultam aí). Expor é um tweak raro que alargaria a superfície de
  param por pouco ganho. É um limite deliberado, não esquecimento.
- **Distortion / domain warp** (o `Distortion` do Blender): **deferido** — é uma
  segunda avaliação de ruído deslocando a coordenada; um nó/knob futuro, não v1.
- **Um fallback pra CPU:** desnecessário — o kernel nasce device-resident.

## O preço (medido)

- Paridade CPU↔GPU no **dispositivo (RTX)** dentro de ε: `max |Δ|` = **4,89e-6** no
  canal P de `grid → value.noise → drive(Y)` com `octaves = 3` (o gate
  `value_noise_kernel_matches_the_cpu_on_the_device`, `#[ignore]`, roda na lane de
  GPU). O hash inteiro é bit-exato; o fade/fBm são polinômios com ε de f32.
- O `generated_wgsl_validates` (naga sobre TODO kernel, presença exaustiva) valida o
  laço de octaves de bound dinâmico (`clamp(round(octaves), 1, 8)`).

## Demo

`PH2D_VALUE_NOISE_SMOKE=1` — duas fileiras de 24, a MESMA geometria, só a FONTE do
valor difere. De cima **NOISE** (`value.noise → drive(Y)`): uma onda suave que
**escorre com o tempo**. De baixo **WHITE** (`instance_field(Random) → map_range →
drive(Y)`): uma fileira **serrilhada e estática**. Selecione o `value.noise` → os
knobs no painel (Frequency = detalhe espacial, Speed = evolução, Octaves/Roughness =
o fBm). O nó cozinha 100% na GPU.
