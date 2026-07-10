# 07 — `motion.noise`: campo de ruído (decisão + evidências)

**Data:** 2026-07-10 · **Linha:** `line/MotionNodes` · **Escopo:** um nó `motion.noise` — um
campo de ruído **gradiente** de Perlin (2002), espacialmente coerente, com fBm + variantes.
**Método:** pesquisa-antes-de-implementar (DIRETRIZ). Fontes primárias (Ken Perlin, Inigo Quilez,
Gustavson, docs de Cavalry/AE/Houdini/Blender) + o MiniCavalryV2 clean-room. URLs no rodapé.

---

## §1 Por que um nó novo — a distinção de `motion.wiggle`

O repo já tem ruído, mas do tipo errado para isto:

- **`motion.wiggle`** amostra `noise(tempo, índice_da_instância)` — cada elemento faz jitter na
  **própria linha** do campo, então eles se mexem **independentes** (nervosismo). É o `wiggle()`
  do After Effects.
- **`motion.noise`** amostra `noise(posição·scale, tempo)` — elementos **vizinhos no espaço** leem
  pontos próximos de UM campo contínuo, então fluem **juntos** (turbulência coerente: fumaça,
  correnteza, deriva). É o "Noise field" do Cavalry/Houdini/Blender.

E o wiggle usa **value noise**; este usa **gradient noise** — a diferença de qualidade do §2.
Falsificado no teste `the_field_displaces_y_coherently_across_neighbours`: vizinhos se movem por
frações da amplitude (mesma onda), não sorteios independentes.

## §2 Gradient noise, não value noise — a decisão de qualidade

| | Value noise (o do wiggle) | Gradient noise (Perlin, este) |
|---|---|---|
| Nos vértices do lattice | interpola um VALOR aleatório → geralmente ≠ 0 | dot(gradiente, 0) = **exatamente 0** |
| Aparência | artefatos de grade axis-aligned ("griddy") | isotrópico, extremos ENTRE os vértices, orgânico |
| Custo | mais barato | dot-product por canto + lookups |

O zero-nos-vértices é a propriedade **definidora** e a razão do gradient noise não ter padrão de
grade (Quilez, "gradient noise"). **Provado por teste** (`gradient_noise_is_zero_at_lattice_points`
+ `gradient_noise_differs_from_value_noise_at_the_lattice`). É por isso que gradient noise é o
default da indústria.

## §3 Perlin 2002, não Simplex — e a correção da patente

O algoritmo é o **Improved Perlin 2002** ("Improving Noise", Ken Perlin), duas melhorias sobre 1985:

1. **Fade quíntico** `6t⁵−15t⁴+10t³` no lugar do cúbico `3t²−2t³`. Perlin: o cúbico tem 2ª
   derivada `6−12t` ≠ 0 nas bordas → *"artifacts show up when Noise is used in bump mapping"*.
   O quíntico tem 1ª **e** 2ª derivadas zero nas bordas → sem dobras visíveis. (Já usávamos o
   quíntico no value noise do wiggle; aqui ele volta.)
2. **Gradientes fixos** no lugar de aleatórios normalizados (que precisam de normalização e podem
   se aglomerar). Perlin: *"none of the gradient directions is too near any others, so they will
   never bunch up"*, e o dot vira soma pura.

**Escolha de gradientes 2D:** usei `{(±1,±2), (±2,±1)}` (herdados do MiniCavalry), não o
`{(±1,0),(0,±1),(±1,±1)}` da restrição 2D "óbvia". Razão: os meus têm **magnitude uniforme √5**
(isotrópicos), enquanto o outro mistura 1 e √2 (anisotropia leve). A pesquisa confirma que a
variante de tabela de Gustavson existe justamente para evitar essa anisotropia — *"Either is
fine"*, e a uniforme é a melhor. O dot é `±u ± 2v` (só uma multiplicação por 2). Normalização
`1/1.5` (o pico empírico é ~1.49; pinado pelo teste `bounded`).

**Simplex — rejeitado como líder, guardado como 2º tipo futuro.** A pesquisa desfez uma premissa
minha errada: **o Perlin clássico NUNCA foi patenteado; foi o *simplex* que foi** (US 6.867.776,
2005, **expirado jan/2022** — daí o OpenSimplex existir). Então "sem patente" sempre foi do
gradient noise clássico, não do simplex. E em **2D** a vantagem O(n) do simplex é negligível (3 vs
4 cantos); ele custa muito mais código (skew/unskew, seleção de simplex), e sob fBm (que motion
sempre usa) a vantagem de isotropia dele é mascarada pelo empilhamento de oitavas. Gradient noise
2002 é mais simples, determinístico, transcendental-free, e é o "look que artistas reconhecem".
Simplex fica como **2º `type` selecionável** num follow-up (drop-in no mesmo wrapper fBm), como
Cavalry/Houdini fazem — mas não lidera.

## §4 fBm + variantes — o conjunto canônico

fBm (fractional Brownian motion) soma oitavas de frequência dobrando (**lacunarity** 2.0) e
amplitude decaindo (**gain/roughness**), normalizado por `Σaᵢ` para ficar em faixa fixa
independente do nº de oitavas (Quilez, "fbm"; = o toggle Normalize do Blender). Duas variantes de
uma linha (rectificação **por oitava**, não no resultado):

- **fBm**: `Σaᵢ·noise` — bipolar `[-1,1]`, o campo à deriva default.
- **Turbulence**: `Σaᵢ·|noise|` — unipolar `[0,1]`, dobras/billows afiados (a "turbulence" original
  do Perlin: fumaça, mármore).
- **Ridged**: `Σaᵢ·(1−|noise|)²` — turbulence invertida, cristas afiadas (montanhas, veios).

Provadas por teste (`turbulence_and_ridged_are_unipolar`, `roughness_zero_collapses_to_the_first_octave`).

## §5 Param surface — a interseção das 4 ferramentas

A pesquisa tabelou os nomes de Cavalry/AE/Houdini/Blender; escolhi a **interseção**, o que qualquer
artista de motion já conhece:

| Param (nosso) | default | Cavalry | AE | Houdini | Blender |
|---|---|---|---|---|---|
| `amplitude` | 1.0 | Amplitude | Contrast | Amplitude | (escala downstream) |
| `scale` | 0.4 | Frequency/Noise Scale | Scale | Frequency/Element Size | Scale (5.0) |
| `octaves` | 3 | Octaves | Complexity (6) | Turbulence | Detail (2.0) |
| `roughness` | 0.5 | Gain | Sub Influence | **Roughness** | **Roughness** (0.5) |
| `type` | fBm | Cubic/Simplex/Value | Fractal Type | Noise Type | (fBm) |
| `speed` | 0.4 | Time Scale | Evolution | Offset | (4º eixo W) |
| `seed` | 0 | Seed | Random Seed | — | — |

`scale` default 0.4 (não 5.0 do Blender) porque o mundo PH2D é em **metros** (~unidades), não
pixels — features de ~2 m. **Lacunarity fixa em 2.0** (não exposta): é o default universal
raramente tocado; mantém o nó enxuto. **Domain warp** (o `Distortion` do Blender, `f(p+fbm(p))`
do Quilez — *"the cheap trick that makes noise look alive"*) fica como **follow-up** (+1 param,
+1 noise eval); as variantes turbulence/ridged já dão bastante variedade nesta wave.

## §6 Determinismo (HR-5)

Todo o caminho é hash inteiro + polinômio (fade quíntico) + dot-products + `floor`. **Zero
transcendental por chamada** (nada de `sin`/`exp`/`pow`; sem `PERM` table — um hash puro é
stateless e seedável). O tempo entra no domínio (scroll em Y), não via RNG. Um campo de noise
replaya bit-idêntico cross-platform, como todo nó Motion. O próprio teste do core evita `sin` (um
`sin_like` triangular) para o crate inteiro passar num grep de transcendentais.

## §7 Resumo da decisão

| # | Decisão | Ref dominante |
|---|---|---|
| 1 | **gradient** noise, não value (zero-nos-vértices, sem grade) | Perlin / Quilez |
| 2 | **Perlin 2002**, não Simplex (2D: O(n) negligível; mais simples; o "look") | pesquisa §2 |
| 3 | gradientes **isotrópicos** `(±1,±2)/(±2,±1)`, não o set anisotrópico | Gustavson |
| 4 | fBm + **turbulence/ridged** por oitava; lacunarity 2 fixa | Quilez |
| 5 | param surface = **interseção** das 4 ferramentas | Cavalry/AE/Houdini/Blender |
| 6 | **campo espacial coerente** (≠ jitter por-índice do wiggle) | Cavalry/Houdini |
| 7 | patente: era do **simplex** (expirada 2022), não do Perlin clássico | (correção) |

Follow-ups nomeados: **Simplex como 2º `type`** · **Domain warp** (`Distortion`) · Lacunarity/Min-Max
expostos se pedirem.

---

## Fontes primárias (2026-07-10)

- **Ken Perlin:** [reference impl (fade/grad/perm)](https://cs.nyu.edu/~perlin/noise/) ·
  [*Improving Noise*, SIGGRAPH 2002](https://dl.acm.org/doi/10.1145/566654.566636) ·
  [GPU Gems Ch. 5 (rationale do quíntico e dos gradientes)](https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-5-implementing-improved-perlin-noise)
- **Inigo Quilez:** [gradient noise](https://iquilezles.org/articles/gradientnoise/) ·
  [value noise & derivadas](https://iquilezles.org/articles/morenoise/) ·
  [fbm](https://iquilezles.org/articles/fbm/) ·
  [domain warping](https://iquilezles.org/articles/warp/)
- **Simplex/patente:** [Gustavson, "Simplex Noise Demystified"](https://cgvr.cs.uni-bremen.de/teaching/cg_literatur/simplexnoise.pdf) ·
  patente US 6.867.776 (simplex, expirada jan/2022); OpenSimplex (Kurt Spencer, 2014)
- **Param surfaces:** [Cavalry Noise](https://docs.cavalry.scenegroup.co/nodes/behaviours/noise/) ·
  [AE Noise/Grain + wiggle](https://helpx.adobe.com/after-effects/using/noise-grain-effects.html) ·
  [Houdini Turbulent Noise VOP](https://www.sidefx.com/docs/houdini/nodes/vop/turbnoise.html) ·
  [Blender Noise Texture](https://docs.blender.org/manual/en/latest/render/shader_nodes/textures/noise.html)
- **MiniCavalryV2 (clean-room):** `src/nodes/noise.js` (Perlin 2D single-octave, domínio
  posição·scale + tempo), `src/core/helpers.js` `perlin2`/`pFade`/`pGrad` (fade quíntico 2002,
  gradientes `±u±2v`)
