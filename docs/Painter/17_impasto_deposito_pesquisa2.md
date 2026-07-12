# #16 — Impasto, pesquisa 2: o MODELO DE DEPÓSITO e a superfície de knobs

> **Por que existe:** após as Fases 1–3 landarem, o Enio olhou o resultado e disse *"Não sei se melhorou
> ou piorou. Ficou mais difícil de ajustar."* O handoff
> ([HANDOFF_line_Painter_impasto_2026-07-12](../HANDOFF_line_Painter_impasto_2026-07-12.md)) deixou a
> hipótese: **o modelo está errado na raiz — a altura herda o perfil MACIO da cor, então o relevo é um
> domo, não um corpo com borda.** Esta pesquisa (2026-07-12, 5 varreduras de fontes primárias com
> WebFetch — regra [[feedback_no_industrial_claims_without_verification]]) responde às 6 perguntas do
> handoff §4. O doc [15](15_impasto_pesquisa_e_design.md) segue válido para o que cobriu (h+luz global,
> normal 4-tap, f32 com sinal); este cobre o que ele NÃO perguntou: **como o traço vira altura, e
> quantos knobs o estado-da-arte realmente expõe.**

---

## 1. A medição que confirma a hipótese (ANTES da pesquisa — handoff §2.3: "mede isso primeiro")

Sonda no harness real (`probe_edge_melt_cross_section`, cross-section de traço arrastado, r=40,
Depth 0.7, Amount 0.5, elev 45°, diffuse-only, `--release`):

| fixture | perfil de h (spine → 90% da meia-largura) | pico de \|Δ\| (níveis) | onde fica o pico |
|---|---|---|---|
| **soft default** (hardness 0, Smooth — o pincel que o Enio usa) | 0.70 → 0.39 → 0.16 → 0.07 (**domo puro**) | **7.3** | **31%** da meia-largura (dentro do traço) |
| **disco duro** (hardness 1, Constant) | 0.70 → 0.70 → 0.70 → 0.70 (platô) | **10.3** | **97%** (na borda) |

E na borda visível do soft (15% de cobertura): **1 nível** — nada. Conclusões:

- O default lê como **borrão**: resposta fraca, espalhada, longe da borda. É o "não sei se melhorou".
- O disco duro lê como **corpo** (luz na borda) — mas a parede tem 1–2 px: fina demais para tinta.
- **Nenhum knob conserta isso** — o problema é o *perfil* (§2.1 do handoff), não o *ganho*. Confirmado.
- O duplo peso por cobertura (§2.3 do handoff) é real: o passe escala a INCLINAÇÃO pelo body **e** o
  efeito pelo body — a borda do soft morre quadraticamente. Mas desfazê-lo sozinho não cria a borda
  que não existe no h; o fix é no perfil, com o peso da luz recalibrado junto (§6.3).

## 2. O que cada app expõe (fontes primárias; contagens verificadas)

### 2.1 Corel Painter (Impasto clássico — o padrão-ouro do modelo h+luz)

Docs oficiais product.corel.com (gerações ~2021 e 2023) — páginas "Adjust and create Impasto brush",
"Impasto lighting and depth", "Blend Impasto with layers", "Apply, display and clear Impasto":

- **Brush:** `Draw To` (Color and Depth / [Color ≤2021] / Depth) · `Depth Method` (**Uniform** / Erase /
  Paper / Texture Luminance / Original Luminance / Weaving Luminance) · `Depth` (%, máx **300**) ·
  `Invert` · `Negative Depth` · `Expression` (9 fontes, com o receituário oficial "Pressure + Invert =
  pressão leve deposita mais tinta") · `Min Depth` · `Depth Jitter` + `Smoothness` · **`Smoothing`**
  (suaviza a ALTURA renderizada do traço — em setting baixo *"individual brush dabs can appear"*;
  tutorial WetCanvas recomenda **160%+**) · `Plow` (desloca relevo existente).
- **Semântica do Uniform:** *"applies brushstrokes with **even depth** and little texture"* — depósito
  PARELHO, não proporcional à opacidade. Métodos de luminância: *"Light areas of the medium receive
  more depth; dark areas receive less. Black areas appear flat."*
- **Acumulação tem TETO:** *"the accumulated artwork will begin to top out and appear as if the strokes
  are **pressed against glass**"* (tutorial WetCanvas, Painter IX-era; secundária mas explícita).
- **Canvas (Surface Lighting):** Enable Impasto · esfera de luz (8 presets ou N luzes custom arrastáveis,
  SEM campos numéricos de ângulo) · por-luz Brightness/Concentration + cor · Exposure global ·
  **Amount** (*"visibility of brush striations"* — ganho global de relevo aparente) · Picture · Shine
  (sem expoente) · Reflection (env-map). O rig é compartilhado com Apply Surface Texture.
- **Layer:** Composite Depth = Add (default) / Subtract / Replace / Ignore.
- **NÃO expõe:** luz por-traço · ambient · expoente specular · slope-gain por-brush · ângulos numéricos ·
  smoothing canvas-level · edição do buffer além de Clear Impasto.
- **Restrição estrutural:** depth só em dab types carimbados (Circular/Static Bristle/Computed Circular)
  — **a altura do Painter TAMBÉM cavalga o dab**; a resposta deles ao carimbo aparente é o `Smoothing`
  agressivo, não um perfil separado.
- **Nota estratégica:** a Corel julgou esse modelo insuficiente para *feel* e lançou **Thick Paint**
  (2019) como segundo sistema (Amount/Pickup/Bleed/Strength/Radius de deslocamento) — o teto do modelo
  h+luz é conhecido pela própria dona dele.

### 2.2 ArtRage (o polo físico)

Manuais artrage.com (oil-paint-settings, the-canvas, palette-knife, bump-blend-modes) + AR6 Feature List:

- **NÃO existe knob de Depth em nenhuma tool natural** (word-search na página do Oil Brush: "Depth: No").
  Espessura é **emergente**: `Loading` (reservatório que ESGOTA — *"how long your brush strokes can be
  before running out of paint"*) × `Thinners` (50% *"mostly 'flatten' the paint"*) × pressão.
- Traço começa grosso e **afina até secar** (depletion); smear = "pasta de dente" que se espalha e
  esgota, **sem** conservação em cristas.
- Por-pixel armazena cor + **volume + umidade + reflectividade** (entrevista do dev, NZ Herald).
- Relevo por camada = **bump channel** com blend próprio: **Maximum (default!)** / Add / Replace.
- Luz global: on/off + **Angle + Intensity** (AR6) + Metallic. *"Turning off the Canvas Lighting…
  the canvas will appear perfectly flat."*
- O caráter do relevo é **sulco de cerda numa pasta** + cauda dry-brush — não um rim duro na borda.

### 2.3 Rebelle 8 (o mais moderno)

Manual escapemotions.com (panel-properties/oil, panel-visual-settings, panel-layers, brush-creator):

- **Superfície DIÁRIA do pincel: 1 knob** — `Loading` (*"amount of color (i.e. oil paint) applied"*).
  Os caps de relevo (`Max. Impasto Height` 0–200, `Max. Impasto Smudge`, `Thick Impasto` toggle) moram
  no **Brush Creator** — autoria de preset, não pintura.
- **Global (Visual Settings): 2 knobs base** — `Impasto Depth` (0–10) + `Gloss` (0–10). PRO adiciona 3
  set-and-forget (SoftShadows, Shadow Altitude, Environment 1/2/Legacy).
- **NÃO existe knob de ângulo de luz** — direção assada nos environment maps; o dev (Peter Blaskovic)
  confirmou em comentário no blog: *"This would be not possible at the moment."*
- Per-layer: override de Impasto Depth + blend com as de baixo.
- **Out-of-box: 0 knobs para um traço bom** — preset + pintar.

### 2.4 Krita / Procreate (a escola pré-assada)

- Krita 5.2/5.3 **não tem** canal vivo de impasto (varredura docs.krita.org; feature request de 2020
  aberto). O que tem: (a) **Lightness-map brush tips** — luz PRÉ-ASSADA na ponta, recolorida pelo FG
  (*"can even create an effect of thick paint… (sometimes called an 'impasto' effect)"*), com 1 knob
  diário (`Lightness Strength`, sensor-drivable); (b) filtro **Phong Bumpmap** estático com **24
  controles** (4 luzes × azimuth/inclination/cor + ka/kd/ks/shininess) — o conto-moral do excesso.
- Procreate: já coberto no doc 15 (sem canal de altura; 3D pintado no bitmap).

## 3. A pergunta central — como o estado-da-arte dá BORDA à altura (6 mecanismos verificados)

**Nenhum sistema examinado deriva a altura da opacidade macia da cor.** O nosso
`h = Depth × coverage × silhueta` é o caminho *"Smooth"* do Photoshop — o que a doc da Adobe descreve
como o de baixa fidelidade (*"does not preserve detailed features"*).

1. **Bevel por distance transform da BORDA da cobertura** (PS `Chisel Hard/Soft`; confirmado também
   pelo reimplementador do Photopea: *"Most of this time PP is computing distance transform"*).
   `h = Depth × contour(min(d_borda, Size)/Size)` ⇒ platô interno + ombro de largura controlada
   (`Size`), **independente da maciez da cor**.
2. **Perfil PRÓPRIO da altura, editável** (PS `Contour` — o usuário *"sculpt[s] the ridges, valleys,
   and bumps"* da seção transversal; Blender falloff por-brush com presets Sharp/Constant; Hertzmann
   NPAR 2002 dá à altura **uma textura separada** da opacidade — Fig. 3: height map estriado + opacity
   map macio, imagens DIFERENTES).
3. **Composite-não-add com offset por traço** (Hertzmann, explícito: *"the height map is not
   cumulative. We experimented with adding stroke heights instead of compositing, but found it
   difficult to prevent hidden strokes from appearing"* — fronteiras entre traços viram
   descontinuidades que a luz lê como borda).
4. **Teto fixo / platô por traço** (Blender **Layer brush**: *"the height [is] capped. This creates the
   appearance of a **flat layer**"*, com `Height` em unidades de cena e **Hardness alta por default**
   *"to ensure the profile of layers is more noticeable"*; ZBrush Layer: *"raises… by a fixed amount…
   overlapping parts of the stroke do not undergo additional displacement"*; Painter: "pressed against
   glass"; Rebelle: `Max. Impasto Height`).
5. **Acumular rumo a um PLANO** (Blender Clay/Clay Strips + Plane Offset/Trim — achata enquanto
   constrói; ponta QUADRADA por default = ombro duro).
6. **Conservação de volume / deslocamento** (IMPaSTo NPAR 2004: advecção conservativa + velocidade de
   pressão `v_p = −c∇p` ⇒ a tinta empurrada acumula na fronteira do traço; WetBrush SIGGRAPH Asia
   2015: incompressibilidade a nível de cerda; Corel Thick Paint = a versão degenerada barata,
   `Strength`/`Radius` de deslocamento por dab). É o `Plow` — **deferido e nomeado** (plano §6).

E (7) **desacoplamento no shading** (PS Gloss Contour, normais aleatorizadas por traço) — complementa,
mas *"cannot by itself fix a dome profile"*.

## 4. A escala principiada (mata o `SLOPE_GAIN=40` como número mágico)

A decomposição da indústria para o ganho altura→inclinação é **`s = altura_física / tamanho_do_texel`**:

- **Blender Bump node:** `Distance` = *"Multiplier for the height value to control the overall
  distance"* (a altura FÍSICA que 1.0 do mapa representa) separado de `Strength` 0–1 (blend artístico).
  A prática da comunidade: Strength fica em 1, ajusta-se Distance.
- **Substance "Height to Normal World Units":** `Height Depth (cm)` ÷ `Surface Size (cm)` — resolução
  cancela por construção. O Sampler expõe o MESMO filtro com um toggle "Use World Units" (ON = físico,
  OFF = `Intensity` unitless) — as duas apresentações lado a lado.
- **Unity Shader Graph `Normal From Height`:** Strength *"Considered in **real-world units**,
  recommended range 0–0.1"*, com derivadas em espaço de mundo. **HDRP:** amplitude em **centímetros**
  no shader source (`_HeightAmplitude… // In world units`).
- **Substance 3D Painter:** **NENHUM knob de gain** — canal [-1,1], conversão fixa, o usuário escolhe
  só o método (Sharp/Sobel). O knob mora no VALOR depositado.
- **Blender sculpt (fonte, `draw.cc`):** o depósito é literalmente
  `offset = normal × radius × strength` — **altura = fração do RAIO do pincel**, invariante de escala.

**Tradução para o nosso `n = normalize([-s·∂h/∂x, -s·∂h/∂y, 1])`:** o `SLOPE_GAIN=40` de hoje afirma,
sem saber, que *"um ridge de h=1 tem 40 texels de altura"* — e ainda é multiplicado por `Amount` e por
`body`. O modelo principiado: **declarar quantos pixels de tinta h=1.0 representa** (uma constante com
significado físico, medida no probe) e computar a inclinação como geometria pura, sem gain livre.

## 5. Quantos knobs (a pergunta de UX do Enio)

| app | brush diário | global | ângulo de luz? | traço bom out-of-box? |
|---|---|---|---|---|
| Corel Painter | 1 essencial (`Depth`) + painel power-user | Amount/Shine/… num diálogo raramente aberto | só esfera arrastável | sim (variants prontos) |
| ArtRage | **0** (Loading é volume, não depth) | on/off + angle + intensity | sim | sim |
| Rebelle 8 | **1** (`Loading`) | **2** (`Impasto Depth` + `Gloss`) | **não existe** | **sim, 0 knobs** |
| Krita (lightness) | 1 (`Lightness Strength`) | 0 | não existe | sim (preset) |
| **PH2D hoje** | 4 (+Enable) | 5 | Angle + Elevation | **não — é a queixa** |

A lição não é um número mágico de knobs — é que (a) **o traço default já nasce parecendo tinta grossa**
em todos eles, (b) o global é set-and-forget, e (c) **nenhum app expõe dois ganhos que escalam a mesma
percepção** (o nosso `Depth`×`Amount`). O Painter tem Depth+Amount, mas o Amount mora num diálogo de
canvas raramente aberto — o nosso mora no MESMO painel, três linhas abaixo. É a definição do "ficou
difícil de ajustar".

## 6. Decisões para o PH2D (o que o plano §10 implementa)

### 6.1 O perfil do corpo (mata o domo — mecanismos 2+4, custo zero)

A altura deixa de copiar a silhueta e passa por uma **curva de corpo**:
`body(w) = smoothstep(W_TAIL, W_SOLID, w)` com `W_TAIL = 0.10`, `W_SOLID = 0.35`:

- `w ≥ 0.35` ⇒ **platô cheio** (o "even depth" do Uniform do Painter; o "flat layer" do Blender Layer).
- `0.10 < w < 0.35` ⇒ o **ombro** — a parede fica onde a tinta é fina-mas-visível (10–35% de
  cobertura), como tinta real: o filme para um pouco DENTRO da mancha.
- `w ≤ 0.10` ⇒ **zero relevo** na cauda quase invisível (o halo do §2.3 fica impossível por construção:
  não há mais micro-inclinação sobre papel).

Por que remap de `w` e não distance transform: para falloffs procedurais `w` é monótono na distância,
então `body(w)` É um perfil-na-distância (o bevel do PS), sem EDT por move; comuta com o envelope por
`max` (monótono); e preserva a regra 1 (a altura continua consumindo exatamente a silhueta que a cor
consome — Shape image inclusa). A largura do ombro deriva da dureza do pincel (pincel macio ⇒ ombro
mais largo), que é o acoplamento CERTO — o errado era o domo inteiro.

O grão (`Depth Source: Grain`) agora entalha sulcos num **platô** — as estrias de cerda que são a
assinatura do Painter/ArtRage — em vez de modular um domo mole.

### 6.2 Teto de acumulação (mecanismo 4 — "pressed against glass")

O commit entre traços continua **Add** (Substance/Painter default), mas satura em `±H_CEIL = 2.0`
(duas cargas cheias). Empilhar para sempre era o outro jeito de o relevo virar borrão.

### 6.3 Inclinação física (mata `SLOPE_GAIN`, `Amount` e o body² — §4)

- `DEPTH_UNIT_PX` — quantos pixels de tinta `h = 1.0` representa (medido no probe; a constante agora
  TEM dimensão e um jeito de estar errada). Inclinação = `∇h × DEPTH_UNIT_PX`, por texel, sem gain.
- O `body` **sai da inclinação** (a geometria é a que é; o h já carrega a própria borda) e permanece
  só no EFEITO, com saturação rápida: `body_eff = min(1, cover/COVER_SOLID)` — tinta ≥35% de cobertura
  sombreia cheia; papel nu segue **byte-idêntico** (o contrato do passe não muda).
- **`Amount` MORRE** (o gêmeo acoplado do Depth — a queixa literal). `Depth` é a única percepção de
  espessura. `Angle`/`Elevation`/`Shine` ficam (direção e brilho, percepções ortogonais).

### 6.4 O que NÃO muda (provado pelas 24 gates existentes)

Choke point único · capsule sweep (o fix do festão) · regra do RNG copiado · envelope por magnitude
dentro do traço · eraser scrub · mapas por camada + cover · luz RELATIVA (plano = 1.0 exato,
byte-identidade) · perf por amostragem in-place · Watercolor intocado (short-circuit + gate).

### 6.5 Superfície final de knobs

| onde | fica | morre |
|---|---|---|
| Brush | Enable · **Depth** (−1..1) · Smoothing · Depth Source (Uniform/Grain) · Draw To | — |
| Canvas | Show Impasto · Light Angle · Elevation · Shine | **Amount** |
| constantes | `DEPTH_UNIT_PX` (medida, com dimensão) · `W_TAIL/W_SOLID` (a definição de "borda") · `H_CEIL` · `AMBIENT`/`SHININESS` (modelo de luz; Painter também não expõe) · `GRAIN_GROOVE` · `SETTLE_MAX_PX` | `SLOPE_GAIN` (vira `DEPTH_UNIT_PX`) · o `body` dentro da normal |

`Smoothing` fica: com o perfil novo ele vira o knob de *assentamento* de um ombro real (viscosidade),
não mais um segundo borrador do mesmo domo que o falloff já borrava.

## 7. Fontes principais

Corel: product.corel.com help 540215550/540111155 (Impasto brush / lighting / layers / apply-clear /
Expression / Apply Surface Texture) · WetCanvas "Adding Impasto depth to a brush" (Painter IX-era) ·
Thick Paint pages. ArtRage: artrage.com/manuals (oil-paint-settings, the-canvas, palette-knife,
paint-roller, blending, bump-blend-modes) · ArtRage 6 Feature List (PDF) · manual AR2.5 (PDF) ·
NZ Herald (dev). Rebelle: escapemotions.com manual 8 (properties/oil, visual-settings, layers,
brush-creator/paint) · blog "Rebelle 8 realistic oil shader" (+ comentário do dev). Krita:
docs.krita.org (brush_tips, options, filters/map) · fonte do Phong (wdgphongbumpmap.ui) ·
release notes 5.0. Photoshop: helpx "Layer effects and styles" (via espelho PDF) · Photopea blog 1.1
(distance transform) · tutsplus B&E guide. Sculpt: docs.blender.org (layer, clay, clay_strips, draw,
falloff, brush_settings) · blender `draw.cc`/`clay_strips.cc` (fonte) · help.maxon.net ZBrush
(sculpting-brushes, depth). Escala: Adobe experienceleague (Painter height-map-painting, texture-set,
Designer Normal + Height-to-Normal-World-Units, Sampler height-to-normal) · docs.unity3d
(NormalMapImport, Shader Graph Normal-From-Height, HDRP Displacement) · HDRP Lit.shader (fonte) ·
Unreal procedurals · Marmoset height/displacement · polycount wiki. Academia: Baxter/Wendt/Lin
**IMPaSTo** NPAR 2004 (PDF via archive.org) · Chen et al. **WetBrush** SIGGRAPH Asia 2015 (PDF) ·
**Hertzmann, "Fast Paint Texture"** NPAR 2002 (PDF, dgp.toronto.edu). Claims secundários estão
marcados no texto; o restante foi lido de fonte primária.
