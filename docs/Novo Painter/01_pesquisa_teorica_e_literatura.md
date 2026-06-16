# 01 — Base teórica & literatura do Brush Engine

> Fundamentação para um brush engine raster **stamp-based** estilo Procreate (caminho → carimbos de
> Shape a um espaçamento, modulados por Grain, com Wet Mix e modos de acumulação de alpha). Cada
> afirmação tem citação primária. Pesquisa de 2026-06-15 (3 agentes paralelos: manual, teoria, formato).

---

## 1. Fundamentos do modelo Dab / Stamp

**O loop central.** Uma pincelada é um caminho; avança-se por ele em passos de comprimento-de-arco
fixos e carimba-se a máscara de **Shape** em cada passo.

- **Spacing = fração do diâmetro.** Regra canônica do Photoshop: spacing default = **25% do diâmetro**
  ("brush de 40px com spacing 25% → carimba um spot de 40px a cada 10px"), logo `step = spacing_frac × diâmetro`.
  Com spacing desligado, a velocidade do cursor define o espaçamento. — Adobe, *Create and modify brushes*.
- Spacing baixo → dabs sobrepõem → linha lisa/dura; spacing alto → carimbos discretos ("tire tracks").
  O Procreate descreve idêntico ("quantas vezes o shape se carimba ao longo do path"). — Procreate Handbook.
- **MyPaint** expressa a densidade inversa: `res2 = dist / base_radius × DABS_PER_BASIC_RADIUS`, logo
  `step = base_radius / dabs_per_radius`; + `dabs_per_second` p/ airbrush parado. — libmypaint `mypaint-brush.c`.
- **Krita**: spacing default 0.1; "Auto" ~0.8 bom p/ inking; "isotropic" força só-diâmetro. — Krita Manual.

**Os dois modos de acumulação — o coração de build-up vs wash:**

- **Build-up** = cada dab compõe direto no canvas com Porter-Duff source-over: `out.a = src.a + (1−src.a)·dst.a`.
  Auto-interseção e passadas repetidas escurecem sem limite até 1. — Krita ("Build-up trata opacity como flow");
  Photoshop **Flow** + Airbrush; GIMP **Incremental**.
- **Wash / uniform-alpha** = dabs acumulam **cobertura num buffer separado por-pincelada**, capada no max
  por-dab, e compõem no canvas **uma vez** no fim. "Se a pincelada se cruza, não dobra o blend." — fhtr/ShaderPaint.
  Krita: "Wash (default) trata opacity como transparência da pincelada"; **Flow só atua em Wash**.
- **Receita prática do wash-buffer:** o ingênuo `out.a = max(src.a, dst.a)` causa "artefatos em cruz" com
  brushes macios; o fix é source-over da cobertura mas **saturando** no alpha do próprio dab
  (`if color.a > srcA: color.a = max(dst.a, srcA)`) → área de pincelada chapada com borda suave. — fhtr.
- **Nuance MyPaint:** como faz source-over de dabs, expõe `opaque_linearize` (corrige a não-linearidade de
  empilhar dabs). — libmypaint `brushsettings.json`.

**Ancestral na literatura:** Hsu & Lee, "Skeletal Strokes," SIGGRAPH 1994 — figura arbitrária como "tinta"
deformada ao longo do path (linhagem do shape-along-path).

> **Para o PH2D:** o `RenderingMode=6` já modela a família Glaze/Blending (ver §6). O caminho de
> avaliação precisa do **buffer de cobertura por-pincelada** (wash) E do source-over acumulativo (build-up),
> selecionáveis pelo modo. Isto já existe parcialmente em `apply_stamps_wash`/`apply_stamps_buildup`.

---

## 2. Wet Mixing / Mixer-Brush (modelo reservatório pickup-and-deposit)

**Modelo unificador:** o brush carrega um **reservatório** de tinta; cada dab **pega** alguma cor do canvas
(taxa ∝ carga do canvas) e **deposita** alguma cor do reservatório (taxa ∝ carga do reservatório); o
reservatório mistura em direção à cor pega; dilution = água/transparência; charge = capacidade; attack =
velocidade de carga/depósito; pull/length = distância/histórico do esfregaço. É exatamente o que os engines
comerciais implementam sob nomes diferentes.

**Base acadêmica — Baxter et al.:**

- **DAB** (Baxter, Scheib, Lin, Manocha, SIGGRAPH 2001, DOI 10.1145/383259.383313) separa cada superfície
  de tinta em **surface** (fina, molhada) sobre **deep/reservoir** (grossa). Três ops: blend bidirecional
  (brush↔canvas no contato), **replenish** (surface reabastecida do reservoir; reservoir reabastecido ao
  molhar a palette) e secagem. Cor nova = **média ponderada por volume**:
  `C_new = (V_r·C_i + V_l′·C_i′) / (V_r + V_l′)`, aplicada a **brush E canvas** (a cor do brush muda no meio
  da pincelada). — Baxter PhD, UNC 2004, §4.5.
- **IMPaSTo** (Baxter, Wendt, Lin, NPAR 2004, DOI 10.1145/987657.987665) — 5 princípios: tinta move na
  direção empurrada; **tinta é conservada**; transferência exige contato e é **maior em movimento**; **mais
  carga do brush → mais depositado** (princípio 4 = `r_deposit ∝ reservoir`); **mais tinta no canvas → mais
  pego** (princípio 5 = `r_pickup ∝ canvas`). Algoritmo 1 (por-célula, unidirecional, direção = `sign(a_b − a_c)`):
  ```
  velocityCutoff = smoothstep(0.2, 0.3, ‖v‖)
  amt = (xferDir>0 ? a_b : a_c) · XFER_FRACTION · equalPaintCutoff · velocityCutoff
  amt = clamp(amt, 0, MAX_XFER_QUANTITY)
  ```
  Cor guardada como **concentrações de pigmento**, renderizada via **Kubelka–Munk** sobre base espectral de 8 pigmentos.

**Mapeamento comercial (o que cada slider do Procreate É):**

| Conceito-alvo | IMPaSTo / DAB | Photoshop Mixer | Krita Color Smudge | Procreate Wet Mix |
|---|---|---|---|---|
| Reservatório / carga | camada "reservoir" `a_b` | **Load** | quantidade de cor do brush | **Charge** |
| Taxa de depósito `r_deposit` | `XFER_FRACTION` (b→c), princ. 4 | **Flow** (+**Mix**) | **Color Rate** | **Attack** |
| Taxa de pickup `r_pickup` | `XFER_FRACTION` (c→b), princ. 5 | **Wet** | **Smudge Radius** | **Pull** |
| Reservatório mistura → cor pega | `C_new` ponderado por volume | poço de pickup | média Dulling | mix no meio da pincelada |
| Esfregaço / histórico | advecção da footprint | (via spacing) | **Smudge Length** + Smearing | **Pull** |
| Dilution / água | glazes K-M finas | Load baixo → seco | Opacity | **Dilution** / **Wetness Jitter** |
| Contato + gating por velocidade | princ. 3 + `velocityCutoff` | baseado na pincelada | spacing/opacity | charge ao longo da pincelada |
| Cor subtrativa | **Kubelka–Munk** | RGB | RGB | RGB + (Mixbox em versões recentes) |

> **Para o PH2D:** o `WetMixParams` já tem `dilution/load/attack/pull/grade/blur/blur_jitter/wetness_jitter`.
> Falta a **avaliação**: o reservatório por-pincelada + pickup/deposit por-dab, com a cor passando por Mixbox.

---

## 3. Matemática da mistura de cor

**Kubelka–Munk (mistura de pigmento).** Dois coeficientes dependentes de comprimento-de-onda: K (absorção),
S (espalhamento). Reflectância de camada opaca: `K/S = (1−R∞)²/(2R∞)`, inversa
`R∞ = 1 + K/S − √((K/S)² + 2(K/S))`. Mistura = **somas de K e de S ponderadas por concentração, separadamente**:
`(1−R)²/(2R) = Σ(Cₙ·Kₙ) / Σ(Cₙ·Sₙ)` — só a **razão** mapeia (não-linearmente) p/ reflectância; por isso
mistura de pigmento ≠ lerp RGB. **Correção de Saunderson** trata a fronteira ar/filme:
`Rₘ = k₁ + (1−k₁)(1−k₂)R / (1−k₂R)`. — Kubelka & Munk 1931; Saunderson JOSA 1942.

**K–M em graphics — Curtis et al., "Computer-Generated Watercolor," SIGGRAPH 1997** (DOI 10.1145/258734.258896).
Glaze de espessura finita: `R = sinh(b·S·x)/c`, `T = b/c`, `c = a·sinh(b·S·x) + b·cosh(b·S·x)`,
`a = (S+K)/S`, `b = √(a²−1)`. Composição de camadas com inter-reflexão:
`R = R₁ + T₁²·R₂/(1−R₁R₂)`, `T = T₁T₂/(1−R₁R₂)`. *(É o motor que removemos — fica como referência de cor,
não de simulação.)*

**Mixbox — Sochorová & Jamriška, "Practical Pigment Mixing for Digital Painting," ACM TOG 40(6):234,
SIGGRAPH Asia 2021.** Wrapper RGB-in/RGB-out sobre K–M: cada cor sRGB → **espaço latente** (base pequena de
pigmentos-primários + residual RGB aditivo); mistura concentrações no latente + K–M; recombina → sRGB. O
residual garante round-trip exato (qualquer RGB entra/sai). Ganha do lerp RGB porque RGB é aditivo
(amarelo+azul → cinza-lama) enquanto K–M latente é subtrativo (**amarelo+azul → verde**). Tem **impl de
referência aberta + LUT** com bindings p/ **Rust, GLSL, HLSL**.

> ⚠️ **ALERTA DE LICENÇA (crítico p/ o PH2D):** o Mixbox é **CC BY-NC 4.0 (non-commercial)**. Se o PH2D
> for comercial, **não** dá pra dropar a LUT/impl do Mixbox como está; implementa-se a mistura K–M latente
> de forma independente (o *método* é publicado; a *LUT/código* é o artefato restrito). Já temos trabalho
> K–M espectral próprio (ADR-0080/0091) e `ph2d-color::pigment_space` — checar a proveniência do que está lá.

**RYB vs RGB vs espectral — Gossett & Chen, IEEE InfoVis 2004:** cubo RYB com RGB nos 8 cantos + interpolação
trilinear + easing de viés-de-canto; bate a roda subtrativa do artista (amarelo+azul=verde).

**sRGB vs luz-linear na composição.** Energia de luz soma linearmente; sRGB é gamma (~2.2), então a média de
valores codificados subpondera o brilho → midpoints lamacentos + franjas escuras em bordas antialiased.
**Ortogonalidade:** linearize p/ *cobertura/composição*; use K–M/Mixbox p/ *mistura de matiz* — dois eixos
independentes. Photoshop expõe "Blend RGB Colors Using Gamma"; Krita documenta o "black border" do blur em gamma.

---

## 4. Aplicação de Grain / Textura

**Modelo de duas texturas:** uma marca = **Shape** (silhueta/máscara do dab) × **Grain** (textura rolada
dentro do shape). — Procreate Handbook.

**Distinção de espaço de coordenadas (carrega o peso):**

- **Texturized** = amostra o grain em **coords de canvas/mundo** → textura fixa no canvas; dabs sobrepostos
  leem o mesmo valor → idempotente, nítido ("imprimido; não muda por mais que sobreponha"). Como esfregar
  giz de cera sobre papel embaixo.
- **Moving** = amostra o grain em **coords locais da pincelada** → textura viaja com a marca → riscado/borrado,
  acumula. — Procreate Handbook. Sub-params só-Moving: Movement, Rotation, Depth Jitter, Zoom→Follow Size.

**Depth / Min Depth (contraste + piso):** Depth escala a contribuição do grain (min = invisível, max = vívido);
**Depth Minimum** é um piso que a contribuição nunca cruza, mesmo na pressão mínima. Photoshop espelha:
Depth 0–100%, Minimum Depth (piso), Texture Each Tip (re-amostra por dab).

**Krita = spec de modulação de alpha mais explícita:** **Multiply** (`a = a_dab × g`, soft), **Subtract**
(buracos de borda dura), **Height/Linear Height** (range expandido, cobertura total numa passada),
**Lightness/Gradient Map**; + Scale, Offset, Brightness/Contrast, **Neutral Point**, Invert, **Strength**,
**Cutoff/Cutoff Policy** (threshold/remap antes de modular o alpha).

**Pipeline canônico (síntese):**
```
a_dab   = Shape(x,y) × pressure × falloff
g       = grain(uv)                      # uv = canvas (Texturized) | local-da-pincelada (Moving)
g'      = remap(g; brightness, contrast, invert, neutral, cutoff/threshold)
g''     = lerp(min_depth, depth, g')     # escala de depth + piso
a_final = modulate(a_dab, g''; mode)     # Multiply (soft) | Subtract/Height (hard)
```

---

## 5. Dinâmica de entrada — Pressão / Tilt

**Pressão → curva de resposta → alvos independentes.** Pressão crua normalizada [0..1] (`UITouch.force` /
`maximumPossibleForce`), remapeada por uma **curva de resposta** por-parâmetro (editor 2D: x = pressão de
entrada, y = saída), e dirige **Size, Opacity, Flow, Bleed** independentemente, cada um com seu min/max.
Photoshop separa: size em Shape Dynamics→Size Jitter→Pen Pressure; opacity/flow em Transfer. Krita: pressão
é um **sensor** com curva editável, anexável a qualquer opção.

**Flow vs Opacity (a distinção per-dab vs per-stroke-cap):** **Flow = força por-dab** — dabs sobrepostos
*dentro de uma pincelada* acumulam e escurecem além do valor de flow. **Opacity = teto por-pincelada** — a
tinta nunca excede esse teto numa pen-down, por mais passadas; o teto "reseta" só ao levantar. (É o split
build-up/wash da §1 exposto como dois parâmetros dirigíveis por pressão.)

**Tilt — altitude & azimuth (ranges exatos):**
- **Altitude** `UITouch.altitudeAngle`: **radianos, 0 a π/2**; **0 = Pencil deitado** (paralelo à tela),
  **π/2 (90°) = perpendicular**.
- **Azimuth** `azimuthAngle(in:)`: **radianos, 0 a 2π**, relativo à view, **0 = eixo +x (direita)**.
- **Wacom / W3C Pointer Events** reportam **tiltX/tiltY** (±60°, spec até ±90°); derive
  altitude = magnitude, azimuth = `atan2(tiltY, tiltX)`.

**O que os brushes fazem com tilt:** **altitude → opacity/size/bleed/gradation** (sombrear com a lateral do
lápis); **azimuth → rotação do dab** (traços caligráficos direcionais). Sliders de Tilt do Procreate: Angle
(threshold), Opacity, Gradation, Bleed, Size, Size Compression.

**Velocidade:** Krita tem sensor **Speed** + **Drawing Angle** de primeira classe; Procreate/Photoshop usam
**Taper/Fade** (rampa por distância/passos).

---

## 6. A família de modos "Glaze" (estratégias de acumulação de alpha)

**Rendering do Brush Studio = 6 modos** em duas famílias, do mais leve ao mais pesado de deposição:
**Glaze** (Light, Uniform, Intense, Heavy) + **Blending** (Uniform, Intense). Light Glaze = "padrão e mais
leve"; Uniform Glaze = "similar ao Adobe Photoshop"; Heavy Glaze = "forte; mantém a opacidade ao misturar";
Intense Blending = "mais pesado; brushes molhados que esmagam e misturam".

**A distinção comportamental (só visível em opacidade baixa):**
- **Glaze = um único cap de cobertura por pincelada.** Um brush glazed deposita um tom para a pincelada
  inteira até levantar a caneta, **não importa quantas vezes vá e volte**. Mas entre pinceladas *separadas*
  acumula (cada camada translúcida empilha como glazing a óleo).
- **Blending = build-up contínuo** mesmo dentro de uma pincelada (sobreposição escurece como marcador) +
  esfregaço de cor do Wet Mix. "Não dá pra ver diferença em 100% de opacidade — precisa de brushes de
  opacidade baixa."

**Mapeamento à taxonomia da §1:** Glaze = **família wash dentro da pincelada** (cap chapado uniforme) mas
**família build-up entre pinceladas**. Blending = **build-up puro**. Light→Uniform→Intense→Heavy = quão
agressivo o alpha acumula por dab/passada.

**A matemática da acumulação (a definição formal do split):** Porter & Duff, "Compositing Digital Images,"
SIGGRAPH 1984. Operador over: `α_o = α_a + α_b(1−α_a)`. Compor opacity `a` sobre si `n` vezes:
```
opacity_n = 1 − (1 − a)^n      (build-up: sobreposição escurece — a=0.1 → 1,2,5,10 dabs = .10,.19,.41,.65)
opacity   = a                  (wash/glaze: aplicação única capada, chapada independente de n)
```
A desigualdade `1 − (1−a)^n ≠ a` **é** a distinção build-up-vs-wash; em `a=1` ambos = 1, por isso os modos
parecem idênticos em 100%. O over é associativo → render de pincelada parcial compõe idêntico ao render do todo.

**Wet Edges** = suaviza/borra as bordas da pincelada p/ mimetizar pigmento sangrando no papel (escurecimento
de borda da aquarela); separado do **Blend Mode** do brush (aplicado à pincelada inteira).

---

## Reading list (rankeada para implementador)

1. **IMPaSTo — Baxter, Wendt, Lin, NPAR 2004** — o algoritmo canônico de pickup/deposit interativo
   (Algoritmo 1: transferência bidirecional com gating de contato+velocidade, conservação) + K–M.
   Melhor fonte única p/ as equações de wet-mixing. http://gamma.cs.unc.edu/IMPASTO/publications/Baxter-IMPaSTo_Web-NPAR04.pdf
2. **fhtr / ShaderPaint — "Brush stroke blending" (2016)** — única fonte com equações GPU-ready de build-up
   vs wash, o buffer por-pincelada, o artefato de cruz do max-alpha, e o fix do saturated-over.
   http://fhtr.blogspot.com/2016/05/brush-stroke-blending.html
3. **Mixbox — Sochorová & Jamriška, TOG 40(6):234, SIGGRAPH Asia 2021** — mistura prática de pigmento, impl
   ref aberta + LUT (bindings Rust/GLSL). ⚠️ CC BY-NC. https://scrtwpns.com/mixbox/ · https://github.com/scrtwpns/mixbox
4. **Krita Manual — Opacity/Flow, Color Smudge, Texture, Sensors** — melhor referência *aberta* de
   implementação nos 6 tópicos, as definições mais precisas de alpha/transfer.
   https://docs.krita.org/en/reference_manual/brushes/brush_settings/opacity_and_flow.html
5. **Procreate Handbook — Brush Studio (Settings)** — o vocabulário canônico do engine-alvo.
   https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings
6. **Curtis et al., "Computer-Generated Watercolor," SIGGRAPH 1997** — K–M de espessura finita R/T + composição
   de glaze. https://dl.acm.org/doi/10.1145/258734.258896 · equações: https://davis.wpi.edu/~matt/courses/watercolor/rendering.html
7. **Baxter PhD dissertation, UNC 2004** — write-up mais completo do reservoir/replenish + `C_new` ponderado.
   http://gamma-web.iacs.umd.edu/papers/documents/dissertations/baxter04.pdf
8. **libmypaint source** (`mypaint-brush.c`, `brushsettings.json`) — math real de spacing + opacity por-dab + opaque_linearize.
   https://github.com/mypaint/libmypaint
9. **Adobe Photoshop Help** — *Create/modify brushes* (spacing) + *Add dynamic elements* (pressão/tilt/textura)
   + *Paint with the Mixer Brush* (Wet/Load/Mix/Flow). https://helpx.adobe.com/photoshop/using/creating-modifying-brushes.html
10. **Apple Developer — `altitudeAngle` / `azimuthAngle(in:)`** — ranges definitivos do stylus (0–π/2; 0–2π).
    https://developer.apple.com/documentation/uikit/uitouch/1618118-altitudeangle
11. **Porter & Duff, SIGGRAPH 1984** + Ciechanowski — o over + acumulação `1−(1−a)^n`.
    https://ciechanow.ski/alpha-compositing/
12. **Kubelka & Munk 1931** + Gossett & Chen InfoVis 2004 (RYB) — fundamentos de mistura de cor.
13. **Hsu & Lee, "Skeletal Strokes," SIGGRAPH 1994** — ancestral do shape-along-path.

## Dois insights de design para o PH2D

- **Mixbox é CC BY-NC 4.0** (não-comercial). Se o PH2D for comercial, implementar K–M latente de forma
  independente (método publicado; LUT/código restrito). Cruzar com `ph2d-color::pigment_space` (ADR-0080/0091).
- **Ortogonalidade de dois eixos é a chave:** *cobertura/composição* (build-up vs wash, linear vs sRGB) é
  independente de *mistura de matiz* (lerp RGB vs K–M/Mixbox). São estágios separados no dab pipeline e devem
  ser caminhos de código separados e alternáveis independentemente.
