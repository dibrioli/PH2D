# 07 — Rendering Modes + Wet Mix (Glaze / Blending / Wet Edges / Burnt Edges)

> **Pré-requisitos:** [`04_pesquisa_shape_grain_procreate.md`](04_pesquisa_shape_grain_procreate.md) (modelo de dab/textura) + [`01_arquitetura_e_decisoes.md`](01_arquitetura_e_decisoes.md).
> **Entregável:** arquitetura + algoritmos + plano faseado para os modos de renderização **por-traço** estilo Procreate, com back-compat **byte-idêntico** (modo default = pipeline atual intacto). **Não implementa** — o código é a rodada seguinte ([`HANDOFF_rendering_modes_wet_mix.md`](HANDOFF_rendering_modes_wet_mix.md)).
> **Origem:** pesquisa multi-agente (Procreate Handbook + Krita Color Smudge + libmypaint + Curtis 1997) com **verificação adversarial** das afirmações algorítmicas (ver §0).

---
> **Crates:** `ph2d-painter-brush` (engine CPU), `ph2d-tool-painter` (host/lifecycle), `ph2d-panel-painter-layers` (UI), `ph2d-editor-core` (IDs).
> **Referência comportamental:** Procreate Brush Studio (Rendering + Wet Mix), Krita Color Smudge (Dulling), libmypaint smudge, Curtis 1997 (edge darkening). GPL → clean-room (comportamento, nunca código).
> **Restrição de norte:** sem re-simulação de fluido (ADR-0096). Tudo é *per-stroke-buffer* + um único `blur` separável. UI 100% em inglês via tokens/i18n (HR-15).

---

## 0 — Status de verificação (Handbook verbatim vs. inferência de engenharia)

As definições abaixo passaram por **verificação adversarial**: 12 afirmações de alto/médio impacto, cada uma checada por um agente instruído a **refutar**. Honestidade epistêmica é regra do projeto ([no-industrial-claims-without-verification]) — por isso separamos fato publicado de reconstrução plausível.

**Verbatim do Procreate Handbook (fato, alta confiança):**
- *Intense Blending:* "the heaviest rendering mode offered and gives a **full flow effect to the paint's Wet Mix**" → **prova o acoplamento Rendering ⇄ Wet Mix** (§1).
- *Wet Edges:* "**Soften and blur the edges** of your brushstrokes to mimic pigment bleeding into paper" → na Procreate é **blur da borda**, *não* acúmulo de pigmento na borda (isso é a semântica do Photoshop — divergência real, §2.5).
- *Burnt Edges:* "creates a '**color burn**' effect around the edges of the stroke when you layer brushstrokes. Burnt Edges also **darkens the edges where colors overlap**." Tem **seu próprio Blend Mode configurável** no painel (§2.6).
- *Fronteira do Glaze = pen-lift:* um modo Glaze deposita **um único tom** pela extensão de um traço contínuo e só acumula quando você **levanta a caneta** e repinta; um modo Blending interage consigo mesmo **dentro** do mesmo traço.

**Inferência de engenharia (recipe plausível, NÃO spec publicada — tratar como reconstrução fiel):**
- Stroke buffer premultiplicado com acumulação **MAX** (Uniform) vs **additive** (Intense): coerente com a teoria de compositing e confirmado contra Krita "Wash" + Photoshop build-up, mas a Procreate é *closed-source*.
- Fórmulas de Wet/Burnt Edges (`blur(α)`, `rim = max(0, α − blur(α))`, ColorBurn `1−(1−b)/s`): a fórmula de ColorBurn é a canônica (W3C), mas **não há fonte de que a Procreate use exatamente ela** — é a recipe mais barata consistente com os textos.
- Se a Wet Mix é literalmente *inerte* nos modos Glaze: **não-verificável** (a Procreate nunca publicou). Por isso o design **não** faz hard-gate — modela como ganho contínuo (§1, §4).

**Correção incorporada (erro a evitar):** não diga "Burnt Edges é inter-stroke-aware *ao contrário de* Wet Edges" — **ambos** são efeitos de borda; a diferença real é **escurecer (Burnt) vs. suavizar/sangrar (Wet)**.

---

## 1. Resumo executivo

Vamos adicionar ao pincel um **modo de renderização por-traço** (Glaze vs Blending, Uniform vs Intense) mais dois efeitos de borda (**Wet Edges**, **Burnt Edges**) e o grupo **Wet Mix** (smudge/mixer). O *enabler* central é um **buffer RGBA premultiplicado por traço** ("stroke buffer"): os dabs acumulam nele e o traço é composto **uma única vez** sobre a camada no pen-up. Esse buffer é o que torna Glaze (cap de cobertura uniforme), Burnt/Wet Edges (um `blur` sobre a cobertura do traço inteiro) e o composite-único todos viáveis e baratos.

**Pergunta central — Rendering Mode e Wet Mix são interdependentes?** **Sim, são acoplados, mas não com um gate liga/desliga duro.** A evidência canônica é o Handbook da Procreate descrevendo *Intense Blending* como "the heaviest rendering mode offered and gives a **full flow effect to the paint's Wet Mix**" (rx:procreate-wet-mix-interdependence). O Rendering Mode é o **ganho sobre a auto-interação dentro do traço**: em modos Glaze o traço não se mistura consigo mesmo no meio do traço (a cobertura é capada por MAX e composta só no pen-up), então os parâmetros de Wet Mix (Pull/Dilution/Blur) **têm pouco em que morder**; nos modos Blending o dab lê o destino e faz lerp a cada passada, então o Wet Mix "ganha vida". **Conclusão de design:** modele Wet Mix como um mixer/smudge state independente, mas faça o RenderingMode **selecionar a regra de acumulação do stroke buffer** (MAX vs additive) e **ligar a leitura de destino** (Blending). Eles compartilham o mesmo buffer e a mesma regra de cobertura — daí o acoplamento natural, sem precisar de flags cruzadas.

---

## 2. As 6 features pedidas

O eixo carregador (rx:procreate-rendering-modes, rx:stroke-buffer-glaze-theory) é **dois eixos ortogonais**:
- **Glaze vs Blending** = *o dab lê o destino?* Glaze: não (filme de cor puro, composto 1× no fim). Blending: sim (lerp com o pixel embaixo, a cada dab).
- **Uniform vs Intense** = *como a cobertura acumula no buffer?* Uniform = `max(aᵢ)` (Alpha-Darken / Wash). Intense = `Σaᵢ` clamped (additive).

E o caveat verificado **de alto valor**: a diferença entre os modos **só é visível com Opacity baixa**; em opacidade plena os seis convergem (rx:procreate-rendering-modes). Isso vira um teste (§11).

> Nota de mapeamento de nomes: o pedido cita 6 (Uniform/Intense × Glaze/Blending + Wet/Burnt Edges). A Procreate expõe 4 Glaze (Light/Uniform/Intense/Heavy) + 2 Blending. Implementamos o **eixo** (Uniform/Intense × Glaze/Blending = 4 combinações) e tratamos Light/Heavy como presets de teto de alpha do mesmo eixo Glaze (`glaze_ceiling` derivado do nome). Wet/Burnt Edges são toggles ortogonais (§5).

### 2.1 Uniform Glaze
- **Definição:** filme de cor uniforme; auto-overlap **não** acumula dentro do traço; build-up só acontece quando você levanta a caneta e repinta. "Photoshop-like" (deposição par, previsível) (rx:procreate-rendering-modes).
- **Math (premultiplicado):** acumula no stroke buffer via Alpha-Darken (rx:stroke-buffer-glaze-theory §3a):
  ```
  por dab:  Ba' = max(Ba, aᵢ)                  // aᵢ = falloff × grain × flow
            B'  = (Ba'>Ba) ? lerp(B, C, …) : B  // cor com flow/opacity=100%, só alpha usa O
  no pen-up: α = min(O, maxᵢ aᵢ); Layer ← B·α + Layer·(1−α)   // um único over
  ```
- **Mapeamento PH2D:** Esta é **quase exatamente** o caminho `stroke_mask` já existente (Accumulate-OFF). O `stroke_mask` (`paint.rs:201`, clear em `paint.rs:287`, cap em `dab.rs:532–541`) já implementa `m_buf[mi] = max-toward-cap` por pixel: `if m >= coverage { continue }`, `add = w*(coverage−m)`, `a = add/(1−m)`. Isso é o teto de cobertura uniforme **direto na camada**. **O que falta:** hoje ele compõe direto na camada (`dst[i..i+4]`, `dab.rs:551`), sem buffer separado, então não há um composite-único e o color-channel não é "100% flow". Uniform Glaze = mover esse cap para o stroke buffer + composite-único.

### 2.2 Intense Glaze
- **Definição:** mesma regra glaze (sem build-up intra-traço), mas **teto de alpha mais alto / menos diluição** — depósito mais denso por passada (rx:procreate-rendering-modes).
- **Math:** idêntico a Uniform Glaze, mas `glaze_ceiling` maior e a curva de alpha do dab menos diluída. Heavy Glaze = "maintains the paint's opacity when mixing" → `glaze_ceiling ≈ 1.0`, dilution mínima.
- **Mapeamento PH2D:** mesmo caminho do §2.1; só muda o escalar de teto (`O` efetivo) e a curva de falloff→alpha. Zero código novo além do escalar.

### 2.3 Uniform Blending
- **Definição:** **não** é glaze. O dab lê o destino e mistura ("caustic approach", "very pronounced Wet Mix effect"); acumula mesmo sem levantar a caneta (rx:procreate-rendering-modes).
- **Math:** o dab faz lerp com o pixel embaixo (mixer/smudge, §4), depositado no stroke buffer com acumulação **MAX** (uniforme):
  ```
  pickup = sample_dest(p)                       // do stroke buffer ∪ camada
  dab_color = (s·state_rgb + (1−s)·brush_rgb)/tα   // s = charge-complement (§4)
  Ba' = max(Ba, aᵢ); B' segue dab_color
  ```
- **Mapeamento PH2D:** precisa do smudge state (§4). A leitura de destino entra no laço `stamp_band` (`dab.rs:508–512` já lê `prev` = destino — hoje só para blend; reaproveitar como pickup). É a diferença real entre os mapas: hoje a cor é fixa por traço (`spec.color`, `dab.rs:462`), aqui ela vira função do destino.

### 2.4 Intense Blending
- **Definição:** o mais pesado; "squash and mix", "thick paint", "full flow effect to the paint's Wet Mix" (rx:procreate-rendering-modes, rx:procreate-wet-mix-interdependence). Acumula e mistura mais agressivamente.
- **Math:** mixer lerp (§4) com **charge/attack altos**, acumulação **additive** (`Ba' = clamp(Ba+aᵢ,0,1)`) → atinge opacidade plena rápido, smear forte.
- **Mapeamento PH2D:** smudge state (§4) + regra additive no stroke buffer. O acoplamento Rendering↔WetMix vive aqui: `Intense Blending` é o ponto onde o ganho de Wet Mix = máximo.

### 2.5 Wet Edges
- **Definição (canônica Procreate):** "Soften and blur the edges of your brushstrokes to mimic pigment bleeding into paper" (rx:procreate-wet-burnt-edges). **Divergência verificada:** o framing "pigmento acumula na borda" é semântica Photoshop; a Procreate documentada é **feather/blur**. Escolhemos a recipe que bate verbatim com a Procreate (blur da cobertura), com a variante rim-pool disponível como knob.
- **Math (sobre o stroke buffer, §5):** `α' = blur(α)` (1–2 px separável) para a leitura Procreate; ou variante mass-conserving `density = α·(1−k·blur) + k_pool·rim` (rx:watercolor-edge-darkening-cheap §2 Recipe B).
- **Mapeamento PH2D:** roda no *finalize* do stroke buffer (pen-up), antes do composite-único. Sem o stroke buffer isso seria impossível (não há "o traço inteiro" para borrar na camada).

### 2.6 Burnt Edges
- **Definição (canônica Procreate):** "color burn effect around the edges… darkens the edges where colors overlap" (rx:procreate-wet-burnt-edges).
- **Math:** rim = lobo positivo do unsharp `rim = max(0, α − blur(α))` (= termo de Curtis eq.3 `mask − blur(mask)`, rx:watercolor-edge-darkening-cheap §1, §2 Recipe A); aplica color-burn ponderado por `rim`:
  ```
  ColorBurn(b,s) = 1 − (1−b)/s
  dst = mix(dst, ColorBurn(dst, strokeColor), rim·strength·overlap)
  ```
  onde `overlap` reforça onde já havia tinta (`dst_alpha>0`), atendendo "where colors overlap".
- **Mapeamento PH2D:** mesmo `blur(α)` da Wet Edges (uma passada serve aos dois toggles); aplicado no finalize. ColorBurn já existe no dispatch de blend (`BrushBlend::ColorBurn` em `blend.rs:33`, dispatch em `blend_rgb` `blend.rs:186`), reusável.

---

## 3. Arquitetura: o per-stroke "stroke buffer"

### 3.1 O buffer
Novo campo no estado de paint do tool (não no `BrushSpec`):
```rust
// ph2d-tool-painter, junto de stroke_mask (paint.rs:201)
struct Paint {
    stroke_mask: Vec<u8>,            // já existe (Accumulate-OFF cap)
    wet_buffer: Vec<f32>,            // NOVO: RGBA premultiplicado-LINEAR, w*h*4
    wet_bbox: Option<Rect>,          // NOVO: dirty-rect do traço
    // ...
}
```
- **Formato:** **premultiplicado, linear-sRGB**, `[f32;4]` por pixel. Premul porque `max`/additive/over são associativos e sem divisão em premul (rx:stroke-buffer-glaze-theory §3: "do the MAX in premultiplied space"; `max` em straight injeta lixo quando `a≈0`). Linear porque é o working space canônico do compositor (`ph2d-color/src/linear.rs:63–70`; o compositor decodifica sRGB→linear via `SRGB_DECODE_LUT`, `compose.rs:42`). A camada permanece sRGB8 straight; convertemos só na borda (decode na leitura de destino para Blending; encode no composite-único).
- **Custo:** 4×f32 = 16 B/px → 64×64 = 256 KiB; 4K = 256 MiB. **Mitigação:** alocar **lazy e bbox-local** (não canvas inteiro) quando viável; mínimo viável aloca canvas-size como o `stroke_mask` faz hoje. Ver §9.

### 3.2 Pontos de inserção (citados do map:stroke-lifecycle-tool)
| Evento | Site | Ação |
|---|---|---|
| **pen-down** | `paint.rs:287` (logo após `stroke_mask.clear()`) | `wet_buffer` clear/resize-zero; `wet_bbox = None`. Só se `rendering_mode != Direct` (modo default = caminho atual intacto). |
| **routing dos dabs** | `paint.rs:524–533` (dispatcher `stamp_stroke_dabs`) | branch novo: se modo usa buffer → `stamp_dabs_to_wet(&dabs)`; senão caminho atual. |
| **stamp no buffer** | nova fn `stamp_dabs_to_wet` | escreve em `wet_buffer` (não `canvas_rgba`); acumula `wet_bbox`; **não** chama `mark_dirty`. Para Blending, lê destino (camada+buffer) como pickup. |
| **pen-up finalize** | `paint.rs:370` (após o `stamp_dabs` do `finish()`) | `finalize_wet_buffer()`: aplica Wet/Burnt Edges (§5) sobre `wet_bbox`, depois `composite_wet_to_canvas()` (um único over), depois `mark_dirty(wet_bbox)`. |
| **undo** | inalterado (`paint.rs:385–386`) | snapshot é por-traço; `wet_buffer` é transiente, fora do snapshot. |

### 3.3 Seleção MAX (Uniform) vs additive (Intense)
No `stamp_dabs_to_wet`, por pixel tocado (premul):
```rust
let a = falloff * grain * flow;                 // cobertura do dab
match accum {
    Uniform => {                                 // Alpha-Darken
        let ba2 = ba.max(a);
        if ba2 > ba { rgb = lerp(rgb, src_rgb, (ba2-ba)/ba2.max(EPS)); }
        ba = ba2.min(ceiling);                    // teto = O efetivo
    }
    Additive => {                                 // build-up indireto
        rgb = src_rgb*a + rgb*(1.0-a);            // premul over
        ba  = (ba + a).min(1.0);
    }
}
```
`src_rgb` é premul = `color·a` (Glaze: `color` = brush color; Blending: `color` = `dab_color` do mixer, §4).

### 3.4 Blending lê a camada-abaixo; Glaze não
- **Glaze:** `src_rgb` = brush color puro. A camada só é lida no composite-único final. Difere por *não* amostrar o destino — daí ser deferível num buffer source-only (rx:stroke-buffer-glaze-theory §4).
- **Blending:** o dab amostra o destino (camada+buffer) e faz `lerp(dst, brush, t)`; depende da história → não-deferível como source-only, mas ainda escreve no stroke buffer para o composite-único e para o finalize de edges. A leitura de destino reusa `dab.rs:508–512` (`prev` já decodificado) somando o que já está no `wet_buffer` naquele pixel.

---

## 4. Wet Mix — é essencial?

**Veredito:** Wet Mix (smudge/mixer state) é **essencial para os dois modos Blending** (§2.3, §2.4) e **dispensável para os modos Glaze e para Wet/Burnt Edges**. Glaze é cor pura composta 1× (não amostra destino); Wet/Burnt Edges operam sobre a cobertura `α` do buffer, não sobre mistura de pigmento. Logo, **Fase de Wet Mix pode vir por último** (§10) e os 4 sub-features Glaze + 2 edges shippam antes.

### 4.1 Modelo de smudge (clean-room MyPaint/Krita-Dulling)
Estado por-traço (rx:mypaint-smudge-source §1, rx:krita-color-smudge-source §1 Dulling):
```rust
struct Smudge { rgb: [f32;3], a: f32, recentness: f32 }  // premul linear
```
Por dab:
1. `update_factor = max(0.01, pull)`; `recentness *= update_factor`;
2. se `recentness < min(1, (0.5·update_factor)^len_log)+ε`: amostra média do destino sob o disco `radius·e^(blur_log)` → `(r,g,b,a)`; então
   `state = update_factor·state + (1−update_factor)·a·sampled` (running-average); `recentness = 1`.
   (Dulling = uma cor média achatada no dab; é o análogo do "Blending" da Procreate, rx:krita §1.)
3. depósito: `s = charge_complement`; `tα = clamp((1−s)+s·state_a, 0,1)`; `dab_color = (s·state_rgb + (1−s)·brush_rgb)/tα`; opacidade do dab × `tα`.

### 4.2 Mapa parâmetro → escalar (do que é necessário)
| Procreate | Escalar engine | Fórmula | Necessário p/ as 6? |
|---|---|---|---|
| **Dilution** | `wet_dilution` | lerp da cor depositada → transparência: `flow_eff = flow·(1−dilution)` | Blending: sim |
| **Charge** | `wet_charge` | reservatório de cor fresca, depleta no traço: `s = 1 − charge·depletion(t)` | Blending: sim |
| **Pull** | `wet_pull` | `smudge_length` (lag/retenção) = taxa de resample | Blending: sim |
| **Grade** | `wet_grade` | contraste/chunkiness da textura do depósito | **Defer** (cosmético; não muda look dos 6) |
| **Blur** | `wet_blur` | `smudge_radius_log` (tamanho do disco de pickup) | Blending: sim (mais suave) |
| **Wet Jitter** | `wet_jitter` | randomização per-stamp de dilution | **Defer** (variação, não core) |

**Defer explícito:** `Grade` e `Wet Jitter` **não** são necessários para reproduzir os 6 looks; entram como polish pós-MVP. `Attack` da Procreate dobramos em `charge`/`flow` (deposit gain) para não inflar a UI.

---

## 5. Wet Edges / Burnt Edges — algoritmo

Roda **uma vez** no `finalize_wet_buffer()` (pen-up), sobre `wet_bbox` expandido pelo raio de blur `r` (a banda vaza `r` px para fora — rx:watercolor-edge-darkening-cheap §3).

```rust
// α = canal alpha (premul) do wet_buffer, recortado a (wet_bbox ± r)
let b = box_blur_separable(&alpha, bbox_pad, r);   // 2 passadas 1D, sliding-window O(N)

if wet_edges {
    // Procreate verbatim: feather. (variante rim-pool atrás de knob)
    for px { alpha[px] = b[px]; }                  // ou: α·(1−k·b)+k_pool·rim (mass-conserving)
}

if burnt_edges {
    for px {
        let rim = (alpha[px] - b[px]).max(0.0);    // Curtis mask − blur(mask)
        let overlap = if layer_alpha[px] > 0.0 { 1.0 } else { 0.0 };
        let w = (rim * burnt_strength * (0.5 + 0.5*overlap)).clamp(0.0, 1.0);
        rgb[px] = lerp(rgb[px], color_burn(rgb[px], stroke_color), w);
    }
}
```
- **Blur:** box separável com running-sum (O(área), independente do raio — rx:watercolor-edge-darkening-cheap §3, "biggest perf lever"). 3 passadas box ≈ gaussiana.
- **Ordem:** Wet Edges (redistribui/borra α) **antes** de Burnt Edges (lê `α` já borrado) é aceitável; para fidelidade, computar `b` uma vez do `α` original e alimentar ambos.
- **Onde:** `finalize_wet_buffer()` inserido em `paint.rs:370`, antes do composite-único. `color_burn` reusa `blend_rgb(BrushBlend::ColorBurn, …)`.

---

## 6. Modelo de cor

- **Working math:** linear-premul no `wet_buffer` (`ph2d-color/src/linear.rs:63–70`, `premultiplied.rs:48–101`). Composite-único usa o caminho já provado: encode linear→sRGB via `SRGB_ENCODE_THRESH` (`compose.rs:53–69`, gate `encode_via_threshold_matches_linear_to_srgb_byte`), decode via `SRGB_DECODE_LUT`. Zero `powf` no hot loop.
- **Pigment mixing (Mixbox/K–M) é necessário para Blending?** **Não no MVP.** O map:color-compositor-history confirma: **não há código Mixbox/K–M alcançável** — a impl espectral vive só em backup (`backups/watercolor_v2_2026-06-12/.../pigment_mix.rs`), removida por ADR-0096; o residual Mixbox de ADR-0091 é só documentação. Portanto **default = lerp RGB linear** (`lerp(dst, brush, t)` no smudge, §4). É determinístico, reproduzível, e bate o look "squash and mix" o suficiente.
- **Upgrade opcional (não-MVP):** reintroduzir o residual Mixbox (`c=unmix(rgb)`, `r=rgb−mix(c)`, decode `mix(c)+r`) como uma flag `pigment_mode` no smudge mix — única mudança seria trocar o `lerp` linear por `mix_colors` espectral em §4.1 passo 2 (mesmo peso `update_factor`). Fica fora do escopo das 6 features.

---

## 7. Mudanças no contrato / BrushSpec

`BrushSpec` é `Copy`, sem serde (map:brushspec-config §4: serialização vive upstream em `BrushSettings`, sem `SCHEMA_VERSION` no engine). Adicionar campos é seguro (inline, Copy-safe).

### 7.1 Enum novo (`spec.rs`, após import de `Falloff`, ~linha 11)
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderingMode {
    #[default] Direct,     // 0 — caminho ATUAL, byte-idêntico (default não-destrutivo)
    UniformGlaze,          // 1
    IntenseGlaze,          // 2
    UniformBlending,       // 3
    IntenseBlending,       // 4
}
impl RenderingMode {
    pub const fn to_u8(self) -> u8 { self as u8 }
    pub fn from_u8(v: u8) -> Self { /* fallback Direct */ }
    pub const fn uses_stroke_buffer(self) -> bool { !matches!(self, Self::Direct) }
    pub const fn reads_destination(self) -> bool { matches!(self, Self::UniformBlending|Self::IntenseBlending) }
    pub const fn is_additive(self) -> bool { matches!(self, Self::IntenseGlaze|Self::IntenseBlending) }
    pub const fn name(self) -> &'static str { /* English */ }
}
```

### 7.2 Campos novos em `BrushSpec` (após `grain_depth`, `spec.rs:111`)
```rust
// ── Master gates (default false) — a engine atual é INTOCÁVEL sem opt-in explícito ──
pub use_rendering: bool,             // +1 B — liga a seção Rendering (senão = Direct)
pub use_wet_mix: bool,               // +1 B — liga o smudge/mixer (seção Wet Mix)
// ── Rendering ──
pub rendering_mode: RenderingMode,   // +1 B  (só tem efeito com use_rendering = true)
pub wet_edges: bool,                 // +1 B
pub burnt_edges: bool,               // +1 B
pub burnt_strength: f32,             // +4 B  (peso do color-burn)
pub edge_blur_px: f32,               // +4 B  (raio do blur dos edges)
// Wet Mix (Blending):
pub wet_dilution: f32,               // +4 B
pub wet_charge: f32,                 // +4 B
pub wet_pull: f32,                   // +4 B
pub wet_blur: f32,                   // +4 B
// deferidos (presentes, default neutro): wet_grade, wet_jitter — opcional Fase 5
```

### 7.3 Defaults BYTE-IDÊNTICOS ao hoje (garantia não-destrutiva)
```rust
use_rendering: false, use_wet_mix: false,   // ← master gates OFF: TUDO desligado por default
rendering_mode: RenderingMode::Direct,
wet_edges: false, burnt_edges: false,
burnt_strength: 0.0, edge_blur_px: 0.0,
wet_dilution: 0.0, wet_charge: 1.0, wet_pull: 0.0, wet_blur: 0.0,
```
O modo **efetivo** é `if use_rendering { rendering_mode } else { Direct }` e `smudge_on = use_wet_mix && effective_mode.reads_destination()`. Com os master gates OFF (o default), o dispatcher (`paint.rs:524–533`) cai no caminho existente — nenhum dab toca o `wet_buffer`, nenhum finalize/smudge roda, **não importa o valor salvo de Mode/sliders**. **Output bit-a-bit igual ao atual.** Esse é o invariante testado em §11 (incl. o caso adversarial: master OFF + parâmetros não-default ⇒ mesmo hash). **A engine atual só muda de comportamento por escolha explícita do usuário (marcar `Use Rendering` / `Use Wet Mix`).**

- **Copy/size:** +~25 B em struct já ~400+ B (2× `TextureSettings`). Sem alocação, Copy preservado (map:brushspec-config §5).
- **Save:** espelhar em `BrushSettings` (map:brushspec-config §4, `brush_settings.rs:68+`) + setters em `PainterTool`. Sem quebra de schema no engine (versionamento é upstream).

---

## 8. UI — DUAS seções ("Rendering" + "Wet Mix"), cada uma com master toggle

**Decisão (Enio, 2026-06-27):** os parâmetros novos vivem em **duas seções novas e separadas**, cada uma com um **master checkbox** (default **OFF**). A engine atual **não muda em nada** sem o usuário marcar o master toggle — é a forma visível da garantia byte-idêntica (§7.3).

- **Seção "Rendering"** — `Use Rendering` (master, OFF) · Mode dropdown · `Wet Edges` · `Burnt Edges` · `Burnt Strength` · `Edge Blur`.
- **Seção "Wet Mix"** — `Use Wet Mix` (master, OFF) · `Dilution` · `Charge` · `Pull` · `Wet Blur`.

**Todos os 5 sites obrigatórios** por widget (senão clique/drag dropam silenciosamente — feedback_panel_populate_register):

1. **IDs** — `ph2d-editor-core/src/ids/chrome/painter_brush_sections.rs`:
   - Rendering: `RENDERING_SECTION`, **`RENDERING_USE`** (master), `RENDERING_MODE` (dropdown), `RENDERING_WET_EDGES`/`RENDERING_BURNT_EDGES`, `RENDERING_BURNT_STRENGTH`/`RENDERING_EDGE_BLUR` (+`_CHIP` cada) + factory `painter_brush_rendering_mode_option_id(u8)`.
   - Wet Mix: `WET_MIX_SECTION`, **`WET_MIX_USE`** (master), `WET_MIX_DILUTION`/`_CHARGE`/`_PULL`/`_WET_BLUR` (+`_CHIP` cada).
2. **Register** (`populate.rs`): os 2 master checkboxes + dropdown + toggles no array de buttons (~128–172); os 6 sliders em `register_brush_slider_chips` (~303–317) — **cada slider chama `set_number_range(chip, 0.0, 1.0, 0.01)`** (reference_number_input_register_range); as **2 seções** em `register_collapsible_sections` (~326–354).
3. **Paint** (novo `paint_rendering.rs`, as duas seções): `paint_collapsible_section("Rendering", …)` com `paint_checkbox_row("Use Rendering")` no topo → `paint_dropdown_row("Mode", …)` → `paint_checkbox_row("Wet Edges")`/`("Burnt Edges")` → `paint_slider_chip_row("Burnt Strength"/"Edge Blur")`; `paint_collapsible_section("Wet Mix", …)` com `paint_checkbox_row("Use Wet Mix")` no topo → `paint_slider_chip_row("Dilution"/"Charge"/"Pull"/"Wet Blur")`. Integrar em `paint_brush.rs:224–253` (após Eraser) + `mod paint_rendering;` no `lib.rs`.
4. **Event** (`event.rs`): os 2 master toggles + Wet/Burnt Edges no match `Click` (~418–441); sliders no `ValueChanged` (~520–537); opção do Mode via `route_brush_dropdown_option` (~480).
5. **Tool** (`trait_impls.rs` + `brush_settings.rs`): handlers `Click`/`SetValue`/`SelectOption` + setters `toggle_brush_use_rendering`, `toggle_brush_use_wet_mix`, `set_brush_rendering_mode`, `toggle_brush_wet_edges`, `toggle_brush_burnt_edges`, `set_brush_*` (clamp 0..1).

**HR-15:** labels em **English** (rx confirma: "Use Rendering", "Use Wet Mix", "Wet Edges", "Burnt Edges", "Dilution", "Charge", "Pull"). O map:panel-ui flagra que os labels hoje estão **hardcoded** (`paint_brush.rs:220` "Accumulate" etc.) — dívida pré-existente, não introduzida aqui; sigo o padrão vigente da crate (labels inline) e registro a migração label→i18n token como follow-up cross-cutting. Zero hex / zero f32-literal de UI: tudo via `ph2d-tokens` + `ColorToken`.

**Mode dropdown — labels (English):** `Uniform Glaze`, `Intense Glaze`, `Uniform Blending`, `Intense Blending`. (`Direct` **não** aparece — é o estado com `Use Rendering` OFF; o master toggle é o liga/desliga, o dropdown só escolhe QUAL modo ativo.)

**Affordance do master toggle:** `Use Rendering` OFF ⇒ esmaeça/colapse o resto da seção Rendering (token `Text2`); idem `Use Wet Mix` OFF para a seção Wet Mix. **Não** desabilite o `register` dos widgets internos (evita drop de hit) — só o estado visual muda.

---

## 9. Performance

- **Stroke buffer:** o maior custo é a memória/clear. Mitigações em ordem: (a) **alocar só com `rendering_mode != Direct`** (default não paga nada); (b) **bbox-local** — alocar/clearar só a bounding-box do traço crescida sob demanda, não canvas inteiro (o `stroke_mask` hoje já cresce lazy, `stamp_cache.rs:293`). Para canvas 64px (demo) é 256 KiB, irrelevante; em 4K, bbox-local é mandatório (canvas inteiro = 256 MiB premul — proibitivo).
- **Composite-único no pen-up:** 1 passada over sobre `wet_bbox` — mais barato que blend per-dab em modos pesados.
- **Blur dos edges:** box separável O(área do bbox), independente do raio (running-sum). Para um traço típico (~10³–10⁵ px) é sub-frame (rx:watercolor-edge-darkening-cheap §3), bem dentro do budget de dirty-rect que o painter já usa (`project_painter_composite_perf`).
- **Smudge (Blending):** pickup é amostragem de disco + lerp por dab; `pull`/`wet_blur` controlam frequência de resample (rx:mypaint — não reamostra todo dab quando `pull` alto). Custo dominado pelo `get_color` médio; subsample o disco em raios grandes.
- **Interação com caches existentes:** o ramp-stamp cache (`stamp_color.rs`) e Shape/Grain caches (map:brushspec-config §3, `dab_mask_cacheable`) **continuam válidos** — eles produzem `w` (falloff×shape×grain), que entra como `aᵢ` no buffer. O stroke buffer é *downstream* do cache; não invalida nada. Blending desabilita o cache de cor (cor vira função do destino) mas mantém o cache de *máscara* (silhueta).
- **Alvos:** interativo 60 Hz em canvas demo; pen-up finalize (edges+composite) < 1 frame no bbox típico. Medir em `--release` (dev=opt0 mente — project_painter_composite_perf).

---

## 10. Plano de implementação faseado

Cada fase: independentemente shippável + `cargo check -p` verde + default byte-idêntico preservado.

- **Fase 0 — Contrato + UI esqueleto.** `RenderingMode` enum + campos em `BrushSpec` (default `Direct`) + espelho `BrushSettings` + seção UI "Rendering" completa (5 sites) com Mode dropdown + checkboxes + sliders, **sem efeito ainda** (setters mutam spec). Gate: default Direct ⇒ output inalterado. `cargo check -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers`.
- **Fase 1 — Stroke buffer (foundation) atrás de modo identity.** `wet_buffer`/`wet_bbox`; `stamp_dabs_to_wet` + `composite_wet_to_canvas`; routing em `paint.rs:524–533`. Implementar **Uniform Glaze** (MAX) e **Intense Glaze** (additive) — ambos source-only (sem destino). Teste: Glaze ≈ caminho `stroke_mask` atual, mas composto 1×.
- **Fase 2 — Blending + smudge state.** `Smudge` struct + pickup (reusa `dab.rs:508–512`); **Uniform/Intense Blending**; Wet Mix sliders (Dilution/Charge/Pull/Wet Blur) ativos. Acoplamento Rendering↔WetMix entra aqui.
- **Fase 3 — Wet Edges / Burnt Edges.** `box_blur_separable` + `finalize_wet_buffer` (rim, color-burn, overlap-gate). Toggles ativos.
- **Fase 4 (opcional) — Polish Wet Mix.** `Grade`, `Wet Jitter` (deferidos §4.2).
- **Fase 5 (opcional/upgrade) — Mixbox residual** no smudge mix (§6), atrás de flag.

---

## 11. Riscos + verificação

**Trap #1 — unit-green ≠ alive (feedback_tool_unit_green_integration_dead, project_painter_canvas_res_64).** Um teste de buffer pode passar e a UI estar morta (pill não registrada / input não wirado). **Mitigação:** teste e2e do dispatch (populate↔register) + verificação visual no canvas demo 64px **antes** de declarar pronto.

**Trap #2 — o caveat "só visível em opacity baixa".** Em opacity plena os 6 modos convergem; um teste ingênuo em opacity=1 "passa" para todos e prova nada. **Teste obrigatório:** pintar X-overlap em opacity≈0.3 e assertar: Glaze mantém α uniforme no cruzamento (`α_cross ≈ α_single`), Blending acumula (`α_cross > α_single`), Direct (legado) acumula como hoje.

**Testes por modo:**
- **Default byte-idêntico (o mais importante):** mesmo traço com `use_rendering=false`+`use_wet_mix=false` produz buffer **bit-a-bit** igual ao baseline pré-feature. Hash do canvas antes/depois.
- **Master gate adversarial:** master toggles OFF **mas** `rendering_mode=IntenseBlending` + `wet_dilution=1.0` setados ⇒ MESMO hash do baseline (prova que o valor salvo do parâmetro não muda nada sem o opt-in). E `use_rendering=true` ⇒ hash diferente (o opt-in ativa de fato).
- **Uniform Glaze:** N dabs sobrepostos em opacity 0.3 → `α = min(O, max aᵢ)`, não `1−(1−a)ⁿ`. Assert `α ≤ O+ε`.
- **Intense Glaze:** mesma sobreposição → `α` cresce (additive), mas composto 1× (sem double-composite contra a camada).
- **Uniform/Intense Blending:** pintar cor B sobre região de cor A → resultado é lerp(A,B,t) com `t` função de charge/dilution; verificar que **lê o destino** (pintar sobre transparente vs sobre cor dá resultados diferentes — prova pickup, rx:mypaint §3 fade-out em transparência).
- **Wet Edges (Procreate):** borda do traço fica mais suave que sem o toggle (variância do gradiente de α na borda cai).
- **Burnt Edges:** rim mais escuro que interior; `rim = max(0, α−blur(α))` > 0 só na banda; reforço onde `layer_alpha>0`.
- **Premul correctness:** `max` em premul não injeta cor em pixels `a≈0` (teste com dab quase-transparente sobre cor saturada — straight falharia, premul não).

**Trap #3 — medir perf em dev.** dev=opt0; medir edges/composite em `--release`, bbox-local, 4K (project_painter_composite_perf).

**Lentes adversariais (≥2, feedback_audit_lens_diversity):** (1) correção numérica premul/linear vs straight/sRGB; (2) costura e2e dispatch+visual; rodar gate batched 1× no fim do módulo (não por task).

---

**Arquivos-âncora para o implementador:**
- Engine: `crates/ph2d-painter-brush/src/spec.rs` (enum+campos, ~11/111/157), `dab.rs` (`stamp_band` 423, cap 532–541, destino 508–512, write 551), `blend.rs` (ColorBurn / `blend_rgb`), novo `wet_buffer.rs` (blur+finalize).
- Tool: `crates/ph2d-tool-painter/src/tool/paint.rs` (down 287, routing 524–533, finalize 370), `paint/brush_settings.rs` (setters + `BrushSettings`), `tool/trait_impls.rs` (handlers).
- UI: `crates/ph2d-panel-painter-layers/src/{populate.rs,event.rs,paint_brush.rs,lib.rs}` + novo `paint_rendering.rs`; `crates/ph2d-editor-core/src/ids/chrome/painter_brush_sections.rs`.
- Cor/compositor (reuso, não tocar contrato): `crates/ph2d-color/src/{linear.rs:63,premultiplied.rs:48}`, `crates/ph2d-tool-painter/src/compositor/compose.rs` (encode/decode LUT 42–79).