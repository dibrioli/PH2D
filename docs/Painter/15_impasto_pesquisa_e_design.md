# #16 — Traço de aspecto 3D: pesquisa + design (Impasto)

> **Status: PROPOSTA — aguarda decisão do Enio. Zero código escrito.**
> Item #16 do [tracker](13_fila_integracao_watercolor_secoes.md#L125). Decisão do Enio (2026-07-11):
> é **FEATURE NOVA, não substituição** — o Per-Layer Color **fica**, e o #11/#15 seguem valendo.

---

## 1. A pergunta e a resposta direta

**Pergunta do Enio:** *"quero pintar brushes com aspecto 3D como os artistas do Procreate fazem"* —
como Procreate/outros fazem, e dá pra fazer mais barato que o Per-Layer Color?

**Resposta curta, e ela reenquadra o pedido:** existem **duas escolas**, e a do Procreate **não é a que
produz 3D de verdade**.

- **Procreate NÃO tem canal de altura nem iluminação.** O Brush Studio inteiro (Stroke Path,
  Stabilization, Taper, Shape, Grain, Rendering, Wet Mix, Color Dynamics, Dynamics, Apple Pencil,
  Properties, Materials) **não tem um único knob de depth/height/light/specular**
  ([Procreate Handbook](https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings)).
  O `Materials` (metallic/roughness) é só para pintar **em modelo 3D** — outra coisa. O 3D dos brushes
  "impasto" comerciais do Procreate é **sombra e luz PINTADAS no bitmap do shape/grain** (ou o truque de
  duplicar a camada, escurecer e deslocar). É **arte**, não **shading**. A luz não gira, o relevo não
  reage a nada, e não existe impasto-sobre-impasto.
  *(Confiança média-alta: a Procreate não publica arquitetura; é evidência negativa da doc oficial +
  o comportamento do ecossistema.)*

- **Quem faz relevo de verdade** — Corel Painter (Impasto), ArtRage, Rebelle, Substance 3D Painter, e a
  literatura — usa **exatamente** o modelo que suspeitávamos: **um canal de ALTURA (`h`) acumulado pelo
  traço + um passe de iluminação que deriva a normal do gradiente de `h`**. *(Confiança alta.)*

**A prova mais limpa** de que o relevo é dinâmico e não pintado vem do manual do ArtRage: *"Turning off
the Canvas Lighting removes all texture effects so paint texture will vanish and **the canvas will appear
perfectly flat**"* ([ArtRage — The Canvas](https://www.artrage.com/manuals/the-canvas/)). Se a textura
estivesse assada no pixel, desligar a luz não a apagaria.

**Consequência para nós:** o caminho "Procreate" (3D pré-assado no bitmap) **nós já temos** — é
literalmente o slot **Shape/Grain** do [ADR-0100](../architecture/decisions/0100-dual-texture-slots-shape-grain.md)
com uma imagem sombreada, e o Per-Layer Color é a nossa versão do dual-brush. Não há feature nova a
construir ali. O que **não** temos é a outra escola: **relevo real, iluminado**. É isso que o #16 deve
entregar, e é o que separa "parece um filtro" de "parece tinta".

---

## 2. O que a indústria faz (só o verificado)

| Pergunta | Convergência | Confiança |
|---|---|---|
| Relevo = `h` acumulado + luz, ou pré-assado? | **Ambas as escolas existem.** Media-natural (Painter · ArtRage · Rebelle · Substance · academia) = `h` + luz. Tablet-first (Procreate) = pré-assado, sem luz. Krita e Clip Studio: **não têm** relevo nativo. Photoshop: meio-termo (luz global sobre o **alpha**, não sobre `h` acumulado). | Alta |
| A luz é global ou por-traço? | **Global, sempre. Zero apps com luz por-traço.** Painter: *"These controls are global — they affect all the Impasto brushstrokes on all layers"*. Photoshop: `Use Global Light` do documento. ArtRage: ângulo+intensidade do canvas. Rebelle: painel Visual Settings (F12). | Alta |
| `h` é por-camada? | **Sim, e com blend-mode PRÓPRIO, separado do blend de cor.** Painter: **Composite Depth = Add / Subtract / Replace / Ignore** por camada. Substance: canal Height com blend por camada (default **Add**, *"useful to accumulate height information"*). Rebelle: impasto depth por camada. | Alta |
| Normal do gradiente de `h`? | Sim — mas **por diferença central de 4 vizinhos, NÃO Sobel**. Krita (`posup/posdown/posleft/posright`) e a literatura (`n = normalize([-s·∇H, 1])`) fazem finite differences. O fator **`s` (height-to-slope)** é o que vira o slider "Depth/Amount" do usuário. | Alta |
| Modelo de shading? | **Blinn-Phong/Phong-Lambert é o piso** (Krita: ka=0.2, kd=0.5, ks=0.3, shininess=2, **4 luzes**), IBL/GGX é o teto (Rebelle usa environment maps reais; a academia já foi pra GGX microfacet). | Alta |
| Como acumular sem escadinha? | Três mecanismos atestados: (a) compositar `h` com **o MESMO alpha-compositing da cor** (front-to-back — a única formulação publicada explicitamente); (b) **`h` em float com sinal** (Substance: HDR, remap [0,255]→[-1,1], negativo = cavar) — sem quantização 8-bit não há banding; (c) suavização explícita (`Smoothing` do Painter). | Média |
| Custo | É um passe de tela sobre um buffer, e é **caro o bastante pra ter botão de desligar**: a Corel documenta `Canvas > Hide Impasto` como **dica de performance**. | Média-alta |

**Fontes principais:** [Painter — Impasto lighting/depth](http://product.corel.com/help/Painter/540215550/Main/EN/Win-Documentation/Corel-Painter-Impasto-lighting-and-depth.html) ·
[Painter — Composite Depth por camada](https://product.corel.com/help/Painter/540111155/Corel-Painter-en/Corel-Painter-Blend-Impasto-with-layers.html) ·
[Painter — brush Impasto (Draw To / Depth Method / Plow)](http://product.corel.com/help/Painter/540215550/Main/EN/Win-Documentation/Corel-Painter-Adjust-and-create-Impasto-brush.html) ·
[ArtRage — The Canvas](https://www.artrage.com/manuals/the-canvas/) ·
[Rebelle 8 — Visual Settings](https://www.escapemotions.com/products/rebelle/manual/8.2/interface/panel-visual-settings/) ·
[Krita — Phong Bumpmap](https://docs.krita.org/en/reference_manual/filters/map.html) ·
[Substance — Height map painting](https://experienceleague.adobe.com/en/docs/substance-3d-painter/using/painting/advanced-channel-painting/height-map-painting) ·
[Procreate Handbook — Brush Studio](https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings) ·
[heightfield shading (normal + Blinn-Phong)](https://nils-olovsson.se/articles/heightfield_shading/).

**Lacunas honestas:** nenhum fabricante publica o operador de acumulação por-dab (só a academia);
a doc oficial da Adobe sobre Bevel & Emboss não abriu (usei fonte secundária); o paper IMPaSTo
(Baxter/Wendt/Lin, NPAR 2004) está inacessível (cert quebrado + paywall).

---

## 3. Por que isto encaixa no PH2D quase sem atrito

O mapa do nosso pipeline diz que **três peças já existem**:

1. **Já temos um canal de altura canvas-sized em `f32`.** `wet_substrate: Vec<f32>`
   ([paint.rs:553](../../crates/ph2d-tool-painter/src/tool/paint.rs#L553)) guarda a **altura do dente do
   papel** (`paper_h`), memoizada, `NaN` = não computado. A aquarela já **lê** essa altura por-pixel
   ([watercolor_render.rs:435](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_render.rs#L435))
   e a usa como gate de deposição (Curtis §4.5: pigmento assenta nos vales). É o precedente exato do
   formato, do dirty-rect e do lifecycle que o `h` do traço precisa.

2. **Os pixels das camadas vivem num `BTreeMap` que o undo já clona inteiro.**
   `images: BTreeMap<RtLayerId, LayerImage>` ([tool/mod.rs:80](../../crates/ph2d-tool-painter/src/tool/mod.rs#L80));
   o snapshot de undo faz `images: self.images.clone()`
   ([layers/undo.rs:21](../../crates/ph2d-tool-painter/src/tool/layers/undo.rs#L21)). Um **mapa irmão
   `heights: BTreeMap<RtLayerId, Vec<f32>>`** herda undo, documentos e persistência **pelo mesmo caminho**,
   sem tocar o `LayerStack` (que é só metadado — os pixels nunca estiveram lá).

3. **O Grain já É uma altura.** O sample do grain (`s`, escalar 0..1 por texel) é exatamente o dado que a
   aquarela trata como altura de papel via `granulation_gate`
   ([texture.rs:518](../../crates/ph2d-painter-brush/src/texture.rs#L518)). Ou seja: **`Depth Source = Grain`
   sai de graça** — as estrias de cerda do impasto reusam o slot Grain do ADR-0100 sem um sampler novo.

E **o que NÃO existe**: grep negativo confirmado — zero `sobel`, `normal_map`, `bump`, `specular`,
`light_dir`, `lighting` em `ph2d-painter-brush`, `ph2d-tool-painter`, `ph2d-painter-effects` e no bridge.
**Não há nenhum passe de iluminação em lugar nenhum do Painter.** Campo aberto, sem cerca de Chesterton.

---

## 4. Design proposto

### 4.1 Semântica (o modelo)

```
        dab  ─────────────►  stroke_height (envelope do traço, f32, max-blend)
                                      │  no release
                                      ▼
   camada i ──►  heights[i]  (f32 com sinal, lazy: None = sem impasto)
                                      │  composite (soma em z-order · Fase 1)
                                      ▼
                     h_total  ──►  normal (∇h, diferença central 4-tap)
                                      │
   paper_h (já existe) ───────────────┤   ── Blinn-Phong (ambient/diffuse/specular) ──►  módula o RGB composto
                                      │
                       luz GLOBAL do documento (azimute · elevação · shine)
```

**Decisões, cada uma ancorada numa fonte:**

- **`h` é `f32` COM SINAL, por camada, alocado lazily.** `None` = camada sem impasto ⇒ **custo zero e
  byte-idêntico**. Sinal negativo = **cavar** (o `Negative Depth`/`Erase` do Painter, o HDR [-1,1] do
  Substance). Float mata o banding na origem — é por isso que ninguém sério guarda height em u8.
- **Acumulação dentro do traço = envelope por `max`**, exatamente como o `stroke_coverage` da aquarela
  já faz ([paint.rs:419](../../crates/ph2d-tool-painter/src/tool/paint.rs#L419)). Uma passada = espessura
  uniforme (não empilha dab-sobre-dab dentro do mesmo traço → zero escadinha, de graça).
- **Acumulação ENTRE traços = `Add` no nível da camada.** Dois traços cruzados ficam mais grossos — é o
  default do Substance (*"useful to accumulate height information"*) e do Painter (`Composite Depth: Add`).
  A distinção envelope-no-traço × soma-entre-traços é a mesma que `Opacity` × `Flow`, que o brush já tem.
- **Normal por diferença central 4-tap** (não Sobel — as duas implementações públicas que li usam finite
  differences). `n = normalize([-s·∂h/∂x, -s·∂h/∂y, 1])`, com **`s` = height-to-slope** exposto como o
  slider **Amount** (é literalmente o "ganho do relevo" do Painter e o "Impasto Depth" do Rebelle).
- **Uma luz GLOBAL do documento** (nenhum app tem luz por-traço). Blinn-Phong: ambient + difuso +
  specular. **HR-5:** `sqrt` é aceito no repo, mas o expoente de shininess vai para **LUT construída uma
  vez** — o precedente é `watercolor_lut.rs`, cujo header já diz *"the `ln`/`exp`/`pow` run only here,
  never per pixel — HR-5"*.
- **O papel entra no MESMO campo de altura.** `h_total = paper_h·(peso) + Σ h_camadas`. É o que Rebelle
  faz (analisa o height map do papel) e o que a literatura faz (fBm+trama somados ao H, encobertos pelo
  impasto grosso). **Ganho grátis:** com a luz ligada, o dente do papel passa a ter relevo de verdade —
  mas é **mudança visível**, então fica **atrás do toggle** (ver §4.4).

### 4.2 Convivência (requisito, não detalhe)

| Com | Como convive |
|---|---|
| **Per-Layer Color** | **Ortogonal, e FICA** (ordem do Enio). Um produz **RGB**, o outro produz **`h`**. Podem inclusive combinar: cada camada de shape contribui altura. Nada a depreciar. |
| **Shape / Grain (ADR-0100)** | O Grain **vira fonte de altura** (`Depth Source = Grain`) reusando o sampler existente. O Shape segue mandando na silhueta. Zero mudança de contrato. |
| **Aquarela / wash** | **Semanticamente exclusivos por brush** — aquarela é tinta fina, impasto é óleo/acrílico; a própria Rebelle separa "Oils & Acrylics" de watercolor. Mas os **dois compartilham o campo de altura do papel** (`paper_h`), que já existe. Fase 1: brush com Watercolor ON ignora Depth (e a UI desabilita o card). |
| **Compositor GPU** | Fase 1 é **CPU**: com impasto ligado, `gpu_eligible` devolve `None` e o composite cai no caminho CPU (que já é a referência canônica, e o fallback já existe e é usado). Fase 2 promove o passe a um `LayerOp` GPU novo. **Isso não é dívida escondida — é o `Hide Impasto` da Corel virado do avesso**, e o kill-criterion (§5) mede se dói. |

### 4.3 Onde o código mora (isolamento — regra B' do Modo L)

Tudo em **módulos IRMÃOS novos**, nada engordando arquivo compartilhado (os candidatos naturais estão
todos no teto de LOC: `watercolor_render.rs` 699/700, `paint.rs` 653, `dab.rs` 605):

| Novo | Onde | Papel |
|---|---|---|
| `tool/paint/impasto.rs` | `ph2d-tool-painter` | o canal `h`: acumulação por-dab, dirty-rect, lifecycle |
| `tool/paint/impasto_light.rs` | `ph2d-tool-painter` | normal + Blinn-Phong + LUT do expoente |
| `tool/paint/impasto_settings.rs` | `ph2d-tool-painter` | rota de `PanelEvent` + setters + reset (espelho de `watercolor_settings.rs`) |
| `ids/chrome/painter_impasto.rs` | `ph2d-editor-core` | ids próprios + arrays `PAINTER_IMPASTO_{CLICKS,FIELDS}` |
| `paint_impasto.rs` + `populate_impasto.rs` | `ph2d-panel-painter-layers` | o card (espelho de `paint_watercolor_paper.rs`, 354 LOC) |

**Atrito já mapeado (e resolvido no papel):** `event.rs` do painel está com **dispensa de LOC congelada em
601/600** — uma linha nova quebra o gate. A saída é o sibling `event_brush_forward.rs`, que **já tem
precedente** (`is_deform_click` é chamado de lá) → substituição LOC-neutra, sem allowlist nova.
E `populate.rs` está em 591/600 ⇒ o Impasto **já nasce** como `populate_impasto.rs`.

### 4.4 UI (os knobs — interseção do que os 4 apps sérios expõem)

**Card `Impasto` (por brush)** — seção nova no painel, colapsável, atrás de `Enable`:

| Knob | Range | Fonte |
|---|---|---|
| **Depth** | 0..1 (com pressure) | `Depth` do Painter / `Impasto Depth` do Rebelle |
| **Depth Source** | `Uniform` · `Grain` · `Shape` | `Depth Method` do Painter (uniform / paper / luminance) — reusa ADR-0100 |
| **Draw To** | `Color` · `Depth` · `Color + Depth` | `Draw To` do Painter — permite pincel que **só cava** ou **só levanta** |
| **Smoothing** | 0..1 | `Smoothing` do Painter (suaviza o jitter de depth) |

**Card `Lighting` (canvas-level, um por documento)** — precedente: os params canvas-level da aquarela já
moram no painel do brush (`reset_brush_watercolor` reseta canvas-level junto):

| Knob | Range | Fonte |
|---|---|---|
| **Show Impasto** (toggle) | on/off | `Canvas > Hide Impasto` da Corel — **é botão de performance, não capricho** |
| **Light Angle** (azimute) | 0..360° | universal (PS `Angle`, ArtRage, Painter) |
| **Light Elevation** | 0..90° | PS `Altitude`, Rebelle `Shadow Altitude` |
| **Amount** (height-to-slope `s`) | 0..1 | Painter `Amount` — o ganho do relevo |
| **Shine** (specular) | 0..1 | Painter `Shine`, Rebelle `Gloss` |

**Default: `Show Impasto` OFF ⇒ pipeline byte-idêntico.** Nenhum quadro existente muda de aparência.

### 4.5 Fora do 1º corte (nomeados, não escondidos)

- **Composite Depth por camada** (`Add`/`Subtract`/`Replace`/`Ignore`) + escala de depth por camada — o
  modelo de dados já nasce per-layer, então isto é **só o composite**, nunca uma reconstrução de topologia
  (regra two-strikes respeitada por construção).
- **Passe de luz na GPU** (`LayerOp` novo — há **8 slots livres** no `AdjustmentKind ≤ 32`, e código
  desconhecido no shader já é no-op identidade). Reconciliação bit-a-bit contra a CPU é **exigência da
  DIRETIVA §4**, não opcional.
- **`Plow`** (o traço **desloca** material alheio em vez de só empilhar) — é o anti-empilhamento
  fisicamente motivado do Painter.
- **Múltiplas luzes / IBL** (Krita tem 4 luzes; Rebelle tem environment maps). 1 luz fica chapado a longo
  prazo, mas 1 luz **já entrega** o efeito.
- **Persistência do `h` no `ProjectState`** — o `heights` entra no snapshot do Painter (undo) desde o dia 1,
  mas o save de projeto hoje **já não persiste** pixels de `SpriteSource::Individual` (gap conhecido).
  O `h` herda esse gap; fechá-lo é o mesmo work item que fechar o gap existente.

---

## 5. Kill-criterion (congelado ANTES do build — DIRETIVA §5)

O #15 mediu o Per-Layer Color e o Enio despriorizou porque *"performance muito boa"*. O impasto **não pode
regredir isso**. Portanto, congelo agora:

> **Cenário fixo:** canvas 2048², brush r=100, traço arrastado, `Show Impasto` ON, 1 camada com `h`.
> **Alvo:** o custo do passe (acumulação de `h` + normal + shading, sobre o dirty-rect) fica **≤ 4 ms/move**,
> mantendo o move inteiro dentro de 16,7 ms.
> **Kill:** se após **2 tentativas** de otimização em CPU o passe estourar 8 ms/move nesse cenário, a
> feature **não existe nesta forma** — ela vira GPU-only (o `LayerOp` da Fase 2) **antes** de fechar a
> linha, e o card fica atrás do toggle com aviso de perf (que é, literalmente, o que a Corel faz).

**Medição obrigatória ANTES de otimizar** ([[feedback_measure_perf_symptom_scale]]): fixar o número em ms
por-move, por-knob, em `--release`.

---

## 6. Plano de execução (se aprovado)

| # | Entrega | Gate vermelho-refutável |
|---|---|---|
| **1** | Canal `h` por camada + acumulação por-dab (`Depth`, `Depth Source`, `Draw To`) | traço com `Depth>0` escreve `h` não-nulo no dirty-rect certo; `Depth=0` ⇒ **byte-idêntico** (prova por igualdade de buffer) |
| **2** | Passe de luz (normal 4-tap + Blinn-Phong + LUT) no composite CPU | oráculo **derivado da DEFINIÇÃO** (uma rampa de altura analítica conhecida → normal/shading esperados), **não** espelhando o shader ([[feedback_oracle_must_model_appearance_not_implementation]]); `Show Impasto` OFF ⇒ byte-idêntico |
| **3** | Seam de UI (card + 5 sliders + 2 toggles) | teste `ph2d-ui-testkit` que **dirige o evento real** e afirma o efeito no `BrushSpec` — os 2 testes espelho de `seam.rs:425/452` |
| **4** | Perf no cenário do §5 | número em ms, em `--release`, antes de qualquer otimização |
| **5** | Smoke do Enio | exemplo pronto: canvas com traço de impasto + luz girável ([[feedback_ready_to_smoke_example]]) |

---

## 7. As 3 decisões que são do Enio

1. **Vale a pena o modelo "de verdade" (relevo + luz), sabendo que NÃO é o que o Procreate faz?**
   Recomendo **sim** — o caminho Procreate (3D pintado no grain) já está disponível hoje via Shape/Grain,
   então construí-lo seria construir o que já existe.
2. **O dente do papel passa a ter relevo iluminado** (é o mesmo campo `h`). É um ganho grátis, mas é
   **mudança visível** — fica atrás do toggle `Show Impasto`, OFF por default. **Confirma?**
3. **Fase 1 é CPU** (impasto ligado ⇒ preview cai no composite CPU, que já é o fallback existente), com o
   kill-criterion do §5 decidindo se vira GPU antes de fechar. **Confirma?**
