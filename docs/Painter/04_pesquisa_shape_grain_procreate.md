# 04 — Pesquisa: Shape + Grain do Procreate → mapeamento para a nossa engine

> **Fase 1 do** [`HANDOFF_shape_grain_dual_texture.md`](HANDOFF_shape_grain_dual_texture.md).
> **Entregável:** entendimento fiel dos painéis **Shape** e **Grain** do Procreate + tabela de
> mapeamento Procreate→PH2D + decisão **Shape substitui vs multiplica o falloff**.
> **Status:** pesquisa concluída (2 agentes web, ≥2 fontes por claim). **Não implementa nada.**
> Checkpoint da rodada: tag `painter-pre-shape-grain-2026-06-24` @ `928bd303` · baseline verde 132/127/21.

---

## §0 — TL;DR (a conclusão que decide a arquitetura)

1. **Procreate compõe o dab assim:** `cobertura = SHAPE_alpha(footprint) × GRAIN_value(coord) × dinâmica`.
   O **Shape** é a *silhueta/ponta* (o alpha do carimbo); o **Grain** é a *textura dentro* da silhueta
   (multiplica a cobertura, preto→0, branco→1). **Confirmado** por múltiplas fontes; a aritmética exata
   (premult/gamma/blend-mode) o Procreate **não publica** — tratar como afinável.
2. **A nossa engine já faz exatamente metade disso:** hoje `w = falloff_weight(t); w *= texture_sample`
   ([`dab.rs:396-429`](../../crates/ph2d-painter-brush/src/dab.rs)). Ou seja **o nosso `falloff` é o
   Shape** (silhueta procedural redonda) e **o nosso slot único de textura é o Grain**. A hipótese do
   Enio se confirma: falta só o **Shape vindo de imagem** (hoje a silhueta é sempre o falloff redondo).
3. **Shape e Grain são ortogonais** no Procreate (qualquer shape × qualquer grain) → **dois slots
   independentes** é a modelagem correta.
4. **Decisão (detalhe em §4):** o **Shape SUBSTITUI** o falloff como fonte de silhueta **quando há
   imagem atribuída**; **sem imagem, a silhueta continua sendo o falloff** (default = byte-idêntico a
   hoje). O `hardness`/falloff segue sendo o edge do brush redondo e, opcionalmente, um feather do
   shape.
5. **Movement do Grain = o nosso `TextureMapping`** que já existe: **Moving = `ViewPlane`**,
   **Texturized = `Tiled`**. Confirmado contra [`texture.rs:205-219`](../../crates/ph2d-painter-brush/src/texture.rs).

---

## §1 — Modelo mental compartilhado (as duas fontes)

Toda fonte confirma a metáfora: **um traço é a Shape carimbada repetidamente ao longo do Stroke Path**;
o **Grain** é "rolado" dentro de cada carimbo como um rolo de tinta texturizado. Shape = *container*;
Grain = *textura dentro do container*. Os dois são imagens **grayscale** importadas, com painéis de
controle **independentes**.

Fontes primárias: Procreate Handbook — Brush Studio Settings
(`help.procreate.com/procreate/handbook/brushes/brush-studio-settings`) + Pocket Handbook.
Secundárias confiáveis: RetroSupply (Ultimate Procreate Brush Guide), Ebb & Flow Creative
(All About the Procreate 5 Brush Studio), Paperlike, Shutterstock, Brush Galaxy.

---

## §2 — Painel **SHAPE** (a silhueta / ponta do dab)

### 2.1 Shape Source
- **O que é:** uma imagem **grayscale** que define a **silhueta (alpha) de cada dab**. Cada pixel de
  brilho → cobertura daquele texel. Editável no Shape Editor (importar foto/arquivo/colar/Source
  Library). Um *tap* (não arraste) no canvas = um único carimbo não-rotacionado da fonte.
- **Polaridade:** as fontes **divergem** sobre branco-vs-preto = sólido. O consenso operacional:
  **brilho → alpha**, com um **invert** (gesto de dois dedos) que deixa o usuário escolher.
  *Implicação p/ nós:* tratar como "luminância → cobertura" + um toggle de **Invert** (já temos
  Contrast/Brightness no slot de textura; Invert é trivial). O default absoluto fica **configurável**.
- **Suavidade de borda:** **não** vem só do alpha da fonte — é o **Shape Filtering** (§2.9) que decide
  serrilhado vs. suave. A fonte pode ser uma silhueta dura.

### 2.2 Rotation (o controle "segue o traço" — o coração)
Slider **assinado −100% … 0% (neutro) … +100%** que **acopla o ângulo do carimbo à direção do traço**:
- **0% (centro):** ângulo fixo em screen-space — **não** segue o traço.
- **+100% (Follow Stroke):** o carimbo gira para **alinhar com a direção do movimento** (pétalas/setas
  acompanhando a curva). **= exatamente o nosso `Rake`** (`advance_rake`).
- **−100%:** segue o inverso da direção.
- Intermediários = acoplamento parcial.
- **Input Style** (versões novas) troca a *fonte* da rotação: padrão (direção do traço) vs **Azimuth**
  (tilt do Apple Pencil) vs Azimuth+Barrel-roll. **Azimuth sobrescreve o slider** só para Apple Pencil.
- ⚠️ **Não existe** dropdown None/Distance/Tilt/Follow (isso é terminologia do **Photoshop**); o
  Procreate expõe **um slider contínuo** + o toggle Azimuth. (Paperlike erra dizendo "neutro = 50%";
  o Handbook diz 0%.)

### 2.3 Scatter
- **Ambiguidade real entre as fontes** (registrada honestamente):
  - Handbook desktop: *"randomize its rotation each time it stamps"* → **scatter rotacional**.
  - Pocket Handbook + comunidade de brush-makers: *"offsets it by a random amount … scatter your
    shapes around your central path"* → **scatter posicional** (espalha as cópias para fora do path).
- **Resolução p/ design:** tratar Scatter primariamente como **offset posicional aleatório por dab**
  (o look clássico de folhas/spray). A aleatoriedade **rotacional** fica melhor atribuída a
  **Randomized** (§2.4). *No nosso mundo:* offset posicional ≈ o `jitter`/`jitter_absolute_px` que
  **já temos** ([`spec.rs:48-83`](../../crates/ph2d-painter-brush/src/spec.rs)).

### 2.4 Randomized
- Rotação aleatória do shape **por traço** (Handbook: *"when your stroke begins"*) — cada traço difere
  do anterior. (Fontes divergem se é por-traço ou por-dab; o oficial é por-traço.) *No nosso mundo:* o
  **`jitter_rotate`** (Jitter Rotate, per-dab) é o primo direto; por-traço seria um seed por stroke.

### 2.5 Count & Count Jitter
- **Count 1–16:** N carimbos por passo de spacing (sobrepostos). Só é visível **combinado com Scatter
  e/ou rotação** (senão os N empilham idênticos). **Count Jitter 0–100%:** randomiza o N por ponto.
  *No nosso mundo:* **não temos** — é feature nova (W2), gated + HR-5.

### 2.6 Azimuth
- Mapeia a **direção do tilt do Apple Pencil** → rotação do carimbo (caligrafia/bico-de-pena).
  Sobrescreve o slider de Rotation só p/ Pencil. *No nosso mundo:* depende de **tilt/azimuth de
  entrada** — **fora de escopo** desta rodada (o input de caneta o Enio testa; sem azimuth no pipeline
  ainda). Anotar como follow-up.

### 2.7 Flip X / Flip Y
- Toggles de **espelhamento** horizontal/vertical (determinísticos no Handbook). A variante
  "**jitter** de flip" (espelho aleatório por-dab) aparece em fontes terceiras mas **não foi
  confirmada** no texto oficial — tratar Flip como **toggle determinístico** salvo evidência no build
  de referência. *No nosso mundo:* trivial (negar `u`/`v` do frame do shape); baixa prioridade.

### 2.8 Roundness (squash)
- Achata o shape de círculo→elipse via um widget de círculo (nó verde = **rotação base**; nós azuis =
  **squash**). Modificadores dinâmicos: **Pressure/Tilt Roundness** e **Roundness Jitter (V/H)**.
  *No nosso mundo:* o **Size X/Y** do texture-frame já dá squash anisotrópico de textura, mas **não do
  footprint do dab**. Roundness real precisa de um **footprint elíptico** (W2, opcional).

### 2.9 Shape Filtering (None / Classic / Improved)
- Antialiasing da borda quando o shape é escalado: **None** = nearest (cru/serrilhado, preserva grão),
  **Improved** = amostragem suave atual. *No nosso mundo:* já amostramos a `ImageMask` **bilinear**
  ([`patterns.rs` `sample_image`](../../crates/ph2d-painter-brush/src/texture/patterns.rs)); "None"
  seria um modo nearest. Baixa prioridade; default = bilinear (≈ Improved).

### 2.10 Interação com dinâmica e spacing
- **Spacing é o relógio-mestre do Shape:** todo comportamento per-dab (Scatter/Count/Rotation/Roundness
  jitter) é avaliado **uma vez por passo de spacing**. Spacing→0 ⇒ carimbos se fundem (Shape some como
  unidade); spacing alto ⇒ você vê cada Shape. **= o nosso walk de Space** ([`stroke.rs`](../../crates/ph2d-painter-brush/src/stroke.rs)).
- **Pressão → size/opacity** escala o Shape e multiplica a opacidade per-dab. **= as nossas dynamics.**

---

## §3 — Painel **GRAIN** (a textura dentro da forma)

### 3.1 Grain Source
- Imagem **grayscale** tileável; **modula a cobertura dentro da silhueta** como multiplicador (preto→0,
  branco→1; cinza→parcial). Metáfora do "rolo". Tem **invert** de dois dedos. *No nosso mundo:* **é
  exatamente o nosso slot de textura atual** — `sample()` retorna `[0,1]` e `dab.rs` faz `w *= s`.

### 3.2 Movement: **Moving vs Texturized** (o que MAIS importa)
- **Moving:** o grão **viaja com o traço** (relativo ao dab) → streaky/blur; passes sobrepostos
  **acumulam**. **= o nosso `TextureMapping::ViewPlane`** (default; coords relativas ao footprint).
- **Texturized:** o grão fica **preso ao canvas** (revela um papel estático registrado); passes
  sobrepostos **NÃO** acumulam além da textura. **= o nosso `TextureMapping::Tiled`** (coords do canvas).
- ✅ **Hipótese do Enio confirmada** contra [`texture.rs:205-219`](../../crates/ph2d-painter-brush/src/texture.rs)
  e a doc viva ([`HANDOFF` §3](HANDOFF_shape_grain_dual_texture.md)).

### 3.3 Controles do Grain
| Controle | Efeito | Disponível em | Já temos? |
|---|---|---|---|
| **Scale** | tamanho absoluto da textura dentro da forma | ambos | ✅ `Size X/Y` |
| **Zoom** (Cropped ↔ Follow Size) | textura acompanha (ou não) o tamanho do brush | **Moving only** | ⚠️ parcial — hoje View escala com o raio (≈ Follow Size); "Cropped" (fixo) seria um modo novo |
| **Rotation** (0% locked ↔ Follow Stroke) | gira o grão com a direção do traço | **Moving only** | ✅ `Rake` / `Angle` |
| **Depth** | força do grão sobre a cor base | ambos | ⚠️ parcial — hoje o grão multiplica a cobertura "cheio" (Depth=1 implícito); **Depth** = lerp(1, grão, depth) é knob novo |
| **Min Depth** | piso da visibilidade do grão sob pressão baixa | **Moving only** | ❌ (precisa de Depth dinâmico por pressão) |
| **Depth Jitter** | oscila grão↔cor base aleatoriamente | **Moving only** | ❌ (W2, gated, HR-5) |
| **Offset Jitter** | offset de registro do grão por **novo traço** | **Moving only** | ⚠️ temos `Random` (offset por-dab); por-traço = seed por stroke |
| **Blend Mode** do grão | operador grão↔cor base | ambos | ⚠️ temos `BrushBlend` no dab (não um blend separado grão↔cor) |
| **Brightness / Contrast** | nível/contraste do grão antes de modular | ambos | ✅ `params[0]/[1]` (Contrast/Brightness) |
| **Filtering** (None/Classic/Improved) | AA do grão | ambos | ⚠️ bilinear fixo |

> "Texturize whole shape" **não existe** como controle nomeado (confusão com o modo Texturized).

### 3.4 Composição (hipótese confirmada)
```
shape_a   = SHAPE_alpha(footprint_local)               // silhueta (Shape ou, default, falloff)
grain_v   = bright/contrast( GRAIN_value(coord) )       // coord = dab-local (Moving) OU canvas (Texturized)
grain_eff = lerp(1, grain_v, Depth·pressão), piso DepthMin
cobertura = shape_a · grain_eff · flow/opacity_dynamics
```
- **Multiplicativo por default** — bem corroborado. **Qualificação A:** o frame de coord do grão muda
  com o Movement (Moving=dab-local / Texturized=canvas) — exatamente o nosso `mapping`. **Qualificação
  B:** o **Blend Mode do grão** pode trocar o `×` por outro operador (não-hardwired). Depth/Min/Jitter
  escalam o termo do grão antes do blend.

---

## §4 — DECISÃO: o Shape **substitui** ou **multiplica** o falloff?

> A pergunta que o [HANDOFF §5](HANDOFF_shape_grain_dual_texture.md) manda decidir com evidência.

**Fatos:**
- No Procreate **não existe** um "falloff radial" separado por baixo do Shape: a silhueta é o **alpha da
  própria imagem de Shape**, com a suavidade vindo do alpha + Shape Filtering. Um shape de **estrela**
  pinta uma estrela com as pontas **crocantes até a borda do footprint**.
- Na nossa engine **hoje**, a silhueta é o `falloff_weight(t)` radial. Se o Shape **multiplicasse** o
  falloff, as pontas da estrela seriam **erodidas** pelo falloff redondo na borda do footprint —
  **infiel** ao Procreate.

**Decisão: o Shape SUBSTITUI o falloff como fonte de silhueta — mas só quando há Shape atribuído.**

```
silhueta(pixel) =  falloff_weight(t)            se Shape.kind == None   (DEFAULT — byte-idêntico a hoje)
                =  shape_alpha(footprint_coord)  se Shape atribuído       (ponta de imagem, crocante)
```

**Por quê (e por que não multiplicar):**
1. **Fidelidade:** estrela pinta estrela; o footprint redondo não morde a silhueta importada.
2. **Back-compat trivial:** Shape vazio = falloff de hoje = baseline **byte-idêntico** (o teste de
   regressão W0 é inegociável).
3. **`hardness`/falloff continua útil:** segue sendo o edge do **brush redondo procedural** (sem Shape)
   e podemos, **opcionalmente** (não no MVP), expor o falloff como um **feather/erode multiplicativo do
   shape** para quem quiser suavizar uma silhueta dura — um toggle "Shape softness", não o default.
4. **Composição limpa:** generaliza o pixel para `silhueta × grain`, onde **cada slot tem um default
   neutro** (silhueta→falloff, grain→1.0). All-default ⇒ exatamente o `dab.rs` de hoje.

**Alternativa rejeitada (multiplicar sempre):** mais simples de codar, mas erode formas duras e torna o
falloff um efeito colateral indesejado sobre toda silhueta importada — infiel e surpreendente.

---

## §5 — Tabela de mapeamento Procreate → PH2D → gap

| Procreate | Conceito PH2D (HEAD) | Status / gap |
|---|---|---|
| **Shape source** (alpha do carimbo) | **slot NOVO** `shape` (imagem via `ImageMask`, igual ao grain hoje) + fallback falloff | ❌ **falta o slot** — é o coração da rodada |
| Shape **Rotation = Follow Stroke** | `Rake` aplicado ao **frame do shape** (reusa `advance_rake`) | ⚠️ temos Rake p/ textura; estender ao shape |
| Shape **Rotation fixa / Angle** | `angle_deg` do frame do shape | ⚠️ análogo ao texture `angle_deg` |
| Shape **Scatter** (posicional) | `jitter` / `jitter_absolute_px` (offset de posição do dab) | ✅ já existe (reusar) |
| Shape **Randomized** (rot. aleatória) | `jitter_rotate` (per-dab) aplicado ao frame do shape | ✅ já existe (estender ao shape) |
| Shape **Count / Count Jitter** | — | ❌ feature nova (W2), gated + HR-5 |
| Shape **Roundness** (squash) | footprint elíptico (Size X/Y do dab) | ❌ nova (W2, opcional) |
| Shape **Flip X/Y** | negar `u`/`v` do frame do shape | ❌ trivial (baixa prioridade) |
| Shape **Filtering** | bilinear (✅) / nearest (None) | ⚠️ bilinear default; nearest opcional |
| Shape **Azimuth** | tilt/azimuth de entrada | ❌ **fora de escopo** (sem pipeline de azimuth) |
| **Grain source** | **slot ATUAL** `texture` (= grain, sem renomear o conceito) | ✅ já é o grain |
| Grain **Moving** | `TextureMapping::ViewPlane` | ✅ idêntico |
| Grain **Texturized** | `TextureMapping::Tiled` | ✅ idêntico |
| Grain **Scale / Brightness / Contrast** | `Size X/Y`, `params[0]/[1]` | ✅ já existe |
| Grain **Rotation (Follow)** | `Rake` / `Angle` do texture-frame | ✅ já existe |
| Grain **Depth** | lerp(1, grão, depth) | ⚠️ knob novo (hoje Depth=1 implícito) |
| Grain **Min Depth / Depth Jitter / Offset Jitter** | — | ❌ novos (W2, gated; Min Depth precisa de pressão→Depth) |
| Grain **Zoom (Cropped/Follow Size)** | View escala com raio (≈Follow) | ⚠️ "Cropped" (fixo) = modo novo |
| Grain **Blend Mode** (grão↔cor) | `BrushBlend` no dab | ⚠️ semântica diferente; defer |

---

## §6 — Escopo: o que entra (paridade que vale) e o que fica fora

### Dentro (paridade que agrega — alvo desta linha de trabalho)
- ✅ **Dois slots ortogonais Shape + Grain** com fallback do falloff (o destravamento principal).
- ✅ **Shape de imagem** (silhueta/ponta importada) — a capacidade que hoje é impossível.
- ✅ **Shape: Rotation Follow-Stroke (Rake) + Angle fixo + Random rotation** (reusa o que já existe).
- ✅ **Shape: Scatter posicional** (reusa `jitter`).
- ✅ **Grain: Moving/Texturized + Scale + Bright/Contrast + Rake/Angle** (já temos — só re-rotular como Grain).
- ✅ **Grain: Depth** (lerp 1↔grão) — knob barato e muito usado.
- 🟡 **W2 opcionais (gated, HR-5):** Shape Count/Count-Jitter, Roundness (footprint elíptico),
  Flip X/Y, Grain Depth-Jitter, "Cropped" zoom. Cada um *gated* (só sorteia/altera quando ativo).

### Fora (não agrega o suficiente / sem pipeline)
- ❌ **Azimuth** (Shape e Grain) — exige tilt/azimuth do Apple Pencil no pipeline de entrada; sem isso
  hoje. Follow-up separado.
- ❌ **Min Depth dinâmico por pressão** — depende de Depth dinâmico por pressão (temos dynamics, mas é
  refinamento; defer até Depth básico existir).
- ❌ **Grain Blend Mode separado (grão↔cor base)** — semântica distinta do nosso `BrushBlend` (dab↔layer);
  o multiply default cobre o caso comum. Defer com justificativa.
- ❌ Paridade 1:1 de todo knob exótico (Offset-Jitter por-traço, Shape Filtering modes) — incluídos só
  se baratos.

---

## §7 — Restrições reais já confirmadas no código (entram no design, doc 05)

1. **`BrushSpec` é `Copy` e sem pixels** ([`spec.rs:30`](../../crates/ph2d-painter-brush/src/spec.rs)).
   Slot novo de Shape = bloco `Copy` de knobs + buffer de imagem **fora** do spec (`PaintState`).
2. **Pixels de imagem vivem em `PaintState`** (`texture_image` + `texture_image_version`,
   [`paint.rs:142-146`](../../crates/ph2d-tool-painter/src/tool/paint.rs)). Shape precisa de
   `shape_image` + `shape_image_version` análogos.
3. **Composição hoje:** `w = falloff_weight(t); if active { w *= sample(texture) }`
   ([`dab.rs:396-429`](../../crates/ph2d-painter-brush/src/dab.rs)). Alvo: `w = silhueta; w *= grain`.
4. **StampMask** já assa `falloff × texture` em `u8` ([`stamp.rs:38-60`](../../crates/ph2d-painter-brush/src/stamp.rs))
   — generaliza para `silhueta × grain`.
5. **4 rotas de cache** ([`paint.rs:396-421`](../../crates/ph2d-tool-painter/src/tool/paint.rs)):
   ramped > cached(StampMask, View) > canvas_cached(Tiled/Stencil) > per_pixel. As regras
   `is_cacheable`/`is_canvas_cacheable`/`has_per_dab_rotation` ([`texture.rs:333-352`](../../crates/ph2d-painter-brush/src/texture.rs))
   viram um **produto de 2 slots**.
6. **Brush NÃO é serializado** (grep: `BrushSpec` deriva só `Clone,Copy,Debug,PartialEq`; não há
   `Serialize` nas crates do painter) — **zero impacto de SCHEMA_VERSION/save**. Brush é estado de
   ferramenta.
7. **`paint_texture_section` é reusado** pelo editor de **Texture-LAYER** em modo `compact`
   ([`paint_texture.rs:227-247`](../../crates/ph2d-panel-painter-layers/src/paint_texture.rs)) — o
   redesenho do painel **não pode quebrar** esse caminho. A Texture-LAYER mapeia ao conceito **Grain**.
8. **HR-5:** qualquer sorteio novo do Shape (Scatter/Count/Random rot.) **gated** + **ordem fixa de
   draw**, espelhando [`jitter.rs`](../../crates/ph2d-painter-brush/src/jitter.rs).

→ **Continua em** [`05_design_dois_slots_textura.md`](05_design_dois_slots_textura.md).

---

## §8 — Incertezas registradas (honestidade de pesquisa)

- **Scatter = posição ou rotação?** Handbook desktop vs Pocket divergem. Adotamos **posição** (consenso
  da comunidade) e deixamos rotação aleatória para **Randomized**. Reavaliar contra o app real.
- **Polaridade branco/preto do Shape source:** fontes divergem → expor **Invert**, default configurável.
- **Aritmética de composição** (premult/gamma/Depth-curve/blend-mode do grão): **não publicada** pelo
  Procreate → afinável; o nosso default (multiply em straight-space, como `dab.rs` já faz) é a base.
- **Randomized por-traço vs por-dab:** oficial = por-traço; o nosso `jitter_rotate` é por-dab (próximo
  o suficiente; um seed-por-stroke é refinamento).
- **Grain Blend Mode (lista/ default):** fonte oficial inacessível (403). Defer.
