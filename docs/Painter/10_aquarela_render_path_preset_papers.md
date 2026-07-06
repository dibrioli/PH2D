# 10 — Aquarela: render-path óptico + preset + Papers + tagging de Layer/Group

> **Objetivo (Enio).** Um preset **"Aquarela Básica"** que configura TODO o painel para reproduzir
> **identicamente** o `docs/Painter/wet_edges_paint.html`, com **integração profunda** reusando tudo
> que já existe (Falloff, Grain, Layers, Texture Layers, RYB, coverage/overlay). O look **não** vem de
> colar efeitos no brush — vem de um **render-path óptico** (cobertura → densidade → Beer-Lambert sobre
> uma base congelada), que é a arquitetura do wet_edges. Supersede a abordagem "efeito bolt-on" do doc
> 08 (F1–F4) para o MODO aquarela; o brush digital normal fica intacto.
>
> Status: **A–D LANDARAM** (linha `line/Painter`, commits `3bd94798` A · `adc5fbda`+`76ca1e56` B ·
> `c1637c7e` C · `d7270b9b` D). A+B smoke-aprovados pelo Enio. F1–F4 (doc 08) foram a base
> reaproveitável (o edge/coverage/blur/RYB viraram peças do render-path óptico).
>
> **Follow-ups deferidos (§5 nota):** botões da modifier-toolbar (Mask/Clip/Lock/Ref-style) p/ tag de
> paper/granulation · tint da COR do papel na base do composite (hoje o tag alimenta o tooth/granulação,
> não a cor-base) · perf dirty-rect no composite p/ traços grandes.

---

## 0. A diferença que este doc resolve (recap da comparação exaustiva)

O wet_edges **reconstrói** a aparência a cada frame de uma **máscara de cobertura** + buffer de cor,
via `D = (cover·fill + edge)·granTex` → **Beer-Lambert por canal em luz linear** (`T = pigmento^(D·DEPTH)`,
`out = base·T + pigmento·(1−T)`) sobre a base congelada (o papel). Não há "dabs no canvas". Os F1–F4
colaram edge/granulação/pigment por cima do depósito normal de dab → herdam bolha per-dab + build-up de
alfa. **Este doc troca o DEPÓSITO** (no modo aquarela) por o composite óptico do wet_edges, reusando os
buffers e knobs que já construí.

---

## 1. Mapa de reuso — o que já existe e alimenta cada aspecto

| Aspecto da aquarela | Sistema existente reaproveitado | Estado |
|---|---|---|
| **Pincel macio** (dab radial) | `Falloff::Smooth` (procedural, sem imagem) — `falloff.rs:15`, `falloff_weight` `spec.rs:436` | ✅ pronto |
| **Cobertura do traço** | `stroke_coverage` + `accumulate_wet_coverage` (discos max-blend) — `wet_edges.rs` | ✅ pronto (F2) |
| **Live preview** | overlay restore/reapply por-frame — `wet_edges.rs` + lifecycle `paint.rs` | ✅ pronto (Fix 2) |
| **Edge darkening** | blur-diff da cobertura — `wet_edges.rs::box_blur` + apply | ✅ pronto (F2) |
| **Granulação (o gate)** | `granulation_gate`/`grain_coverage` — `texture.rs:485` | ✅ pronto (F3) |
| **Papel (height-field)** | slot **Grain** `Tiled` + um `TextureKind` de papel (novo, §4) | ⚠️ falta o kind Papers |
| **Mistura subtrativa** | RYB (Gossett) — `blend.rs::ryb_mix`/`blend_over_pigment` | ✅ pronto (F4) |
| **Base/papel como camada** | Layer system + Texture Layer + tagging "Use as Paper" (§5) | ⚠️ falta o tag |
| **Aplicar N settings de uma vez** | molde dos `reset_brush_*` (`jitter_settings.rs`, `watercolor_settings.rs:90`) | ✅ template |
| **Dropdown no topo** | `paint_dropdown_row` (`paint_brush.rs:385`) + popover | ✅ infra pronta |

**O que FALTA construir:** (a) o **render-path óptico** (buffer de cor + Beer-Lambert sobre base
congelada + skip do depósito normal), (b) 3 knobs novos (Fill/Depth/Warp) na seção Watercolor, (c) o
**preset dropdown** + `apply_brush_preset`, (d) o `TextureKind::Paper*` procedural, (e) o tagging
"Use as Paper"/"Use as Granulation" no Layer/Group.

---

## 2. O render-path óptico (Fase A — o coração do look)

**Gate:** ativo quando `brush.watercolor` (o toggle "Wet edges"). Off = brush digital normal (byte-idêntico).

**No modo aquarela, o depósito de dab é SUBSTITUÍDO.** Em `stamp_dabs` (`stamp_route.rs:37`), quando
`watercolor` ligado: em vez de `stamp_dabs_routed` (que blenda dab no canvas), acumular:
- **cobertura** (`stroke_coverage`, já faço) — a silhueta do traço (Falloff → discos).
- **cor** (`stroke_color: Vec<u8>` RGBA, NOVO) — a cor depositada por-pixel, `source-over` (dab recente
  vence), com pickup/mix opcional via RYB (F4) — é o `colC` do wet_edges.
- e **NÃO** depositar no canvas.

**A base congelada** = os pixels pré-traço (o papel + tinta anterior). Reuso o mecanismo do live overlay:
guardo a região do traço antes de compor e restauro antes de cada recomposição (já é o
`restore_wet_edge_overlay`, estendido para a base inteira do traço).

**Composite** (`apply_watercolor(commit)`, tool-side, molde do `apply_wet_edges`), por-pixel na bbox do
traço, sobre a base congelada `B`:
```
cover = smoothstep(SS0, SS1, sample(coverage, warp(x,y)))      // cobertura endurecida + warp (§ warp)
if cover <= 0 { out = B; continue }
inner = blur(coverage)                                          // ~1 dentro, →0 na borda
edge  = clamp(cover·(1−inner)·edge_gain, 0, 1)                  // franja
gran  = 1 + (paperHeight − 0.5)·2·granulation                  // granulação (Grain Tiled → paperHeight)
D     = (cover·fill + edge)·gran                                // densidade
color = sample(stroke_color, warp) or brush.color              // pigmento (smudge-aware / RYB)
od = D·depth
per canal i:  T_i = pigment_i^od       (LUT: exp(lnl[·]·od))    // Beer-Lambert, luz linear
out_i = l2s( s2l(B_i)·T_i + s2l(color_i)·(1−T_i) )
```
**LUTs (det-safe, HR-5):** `s2l[256]`, `lnl[256]=ln(linear)`, `l2s[4096]`, e uma LUT de `exp` sobre
`[lnl_min·depth_max, 0]` — construídas 1× (espelho do wet_edges `s2l`/`lnl`/`l2s`). `ph2d-color::srgb` já
dá `s2l`/`l2s`; a `exp`/`lnl` construo no tool. **Zero transcendental por-pixel.**

**Reuso:** `cover` = `stroke_coverage` (F2); `blur` = `box_blur` (F2); `gran` usa a amostra do Grain
(§4) via `texture::sample` (Tiled); `color` mixa via RYB (F4). Só o **buffer de cor** + o **Beer-Lambert
LUT** + o **skip-deposit** + a **base congelada** são novos.

**Knobs novos na seção Watercolor** (`BrushSpec` append, molde F1): **`fill`** (0.12, densidade do wash),
**`depth`** (1.2, profundidade óptica Beer-Lambert), **`warp`** (6 px, irregularidade da borda via campo
fractal — reusa o value-noise do Grain/paper). Os 5 de F1 (Edge/Spread/Granulation/Pigment/Mix) seguem.

**Warp** (borda orgânica): dois campos fractais `warpX/warpY` (value-noise, cell ~8–22px) deslocam a
amostragem da cobertura — idêntico ao wet_edges. Det-safe (integer-hash noise).

---

## 3. Preset dropdown + `apply_brush_preset` (Fase B — a UX)

**Dropdown no topo do painel** (header em `paint.rs:50`; helper `paint_dropdown_row`): `Preset ▾` com 2
opções — **Digital Básico** e **Aquarela Básica**. Emite `PanelEvent::SelectOption(PRESET_ID, "0|1")` →
`PainterTool::apply_brush_preset(kind)`.

`apply_brush_preset(Aquarela)` (molde dos `reset_brush_*`) seta o `BrushSpec` INTEIRO para os valores do
wet_edges de uma vez:
- `falloff = Smooth`, `hardness = 0` (dab radial macio); `blend = Mix`; `spacing ≈ 0.05` (denso).
- `texture.kind = Paper*` (ou `Grain`), `texture.mapping = Tiled`, `grain_depth`, size do papel.
- `watercolor = true`, `fill 0.12`, `depth 1.2`, `edge_gain 3.0`, `edge_spread 7`, `warp 6`,
  `granulation 0.30`, `pigment` on, `pigment_mix`.
- `strength/flow` calibrados p/ wash translúcido.
`apply_brush_preset(Digital)` = `BrushSpec::default()` (o brush atual).

Isso satisfaz "ao escolher Aquarela Básica todas as propriedades do painel são configuradas".

---

## 4. `TextureKind::Paper*` procedural (Fase C — o papel)

Novo(s) kind(s) no enum `TextureKind` (`texture.rs:45`, apende discriminante 26+, `to_u8`/`from_u8`/
`name`/`COUNT`), sampler em `texture/patterns/`, params em `specs.rs`. **Aparece automaticamente no
dropdown do Grain E do Texture Layer** (ambos usam o mesmo enum — confirmado no mapa).

**Receita (det-safe, transcendental-free — pesquisa §5):** height-field `h(x,y)∈[0,1]`:
1. **transform anisotrópico** `p = R(θ)·diag(1/sx,1/sy)·(p/scale)` (θ/sx/sy pré-computados) — cold vs rough.
2. **fBm** value/gradient noise (integer-hash `lowbias32`, quintic fade), 3–5 oitavas — o tooth base.
3. **high-pass** `h = clamp(0.5 + contrast·(n − boxblur(n,R)))` — grão crocante (não blob).
4. **ridged fold** `(1−|grad|)²` (rough) — fibras/grooves.
5. **Worley F1** blend — célula de feltro.
Presets **Cold/Rough/Hot** = os mesmos passos com valores diferentes (tabela na pesquisa). Params
expostos: Contrast/Tooth, Scale, Anisotropy, Orientation, Fiber, Cellular, HP-radius, Seed.

**Como o papel modula o pigmento** (a granulação): no composite (§2), `paperHeight = sample(Grain Tiled)`
entra no termo `gran` (Curtis §4.5: pigmento assenta nos vales `(1−h·γ)`). Já é o `granulation_gate` (F3),
agora alimentado pelo Paper. `c = h·(cmax−cmin)+cmin` (capacidade) é follow-up p/ backruns (fora do básico).

---

## 5. Tagging "Use as Paper" / "Use as Granulation" (Fase D — integração Layer/Group)

Duas vias, ambas clonando padrões existentes:

**(a) Menu de contexto da Hierarchy** (clonar a cadeia de 6 passos do "Use as Brush Shape/Grain"):
`menus.rs` (ids `CTX_MENU_HIER_USE_AS_PAPER`/`_GRANULATION`) → `context_menu_overlay.rs` (entradas) →
`hierarchy/event.rs` (push) → `action_bus.rs` (variantes `HierUseAsPaper`/`_Granulation`) →
`shells/desktop/render_loop/mod.rs` (dreno: lê pixels da layer/grupo → luminância → setter) →
`shape_settings.rs` (setter novo `set_brush_paper_image` que grava `PaintState.paper_image` + força
`texture.mapping = Tiled` + liga `watercolor`+`granulation`). Grupo = compõe os filhos primeiro (o
compositor já anda a árvore). Isso cobre "papéis de mais de uma textura via Group".

**(b) Botões na modifier-toolbar** (clonar Mask/Clip/Lock/Ref — a foto do Enio): `paint_modifier_toolbar`
(`paint.rs:519`) — adicionar 2 tuplas `(PAINTER_LAYERS_PAPER, "Paper", on)` / `(..._GRANULATION,
"Grain", on)` + flags `is_paper`/`is_granulation` em `LayerModifiers` (`layers/mod.rs:145`) + handlers em
`trait_impls.rs` (molde de `set_layer_clipping`). A layer/grupo marcada vira a fonte de papel/granulação
do render-path.

**Papers como Texture Layer:** com o kind `Paper*` (Fase C), criar um Texture Layer de papel já é o fluxo
existente (+ "Use as Paper" para ligá-lo ao brush).

---

## 6. Fases (ordem por valor; cada uma fecha com testes + gate, DIRETIVA §3–§5)

| Fase | Entrega | Reusa | Novo | Prova |
|---|---|---|---|---|
| **A** | **Render-path óptico** (o look = wet_edges) | coverage, overlay, blur, Grain, RYB | color-buffer, Beer-Lambert LUT, base congelada, skip-deposit, knobs Fill/Depth/Warp | teste: wash liso sem bolha; rim de pigmento; Beer-Lambert bit-exato vs LUT; neutro off |
| **B** | **Preset dropdown** Digital/Aquarela + `apply_brush_preset` | `paint_dropdown_row`, molde `reset_brush_*` | id preset, opções, handler, apply | seam test (SelectOption → BrushSpec setado); smoke: "Aquarela Básica" ≈ wet_edges |
| **C** | **`TextureKind::Paper*`** (cold/rough/hot) procedural | enum/sampler/specs, Grain+TextureLayer dropdowns | 3 presets de papel + params + LUTs de noise | teste: estatísticas do height-field por preset; det-safe |
| **D** | **Tagging "Use as Paper/Granulation"** (menu + toolbar + Group) | cadeia "Use as Brush Shape", modifier-toolbar, LayerModifiers | ids/actions/setters/flags | seam test (tag → paper do render-path); smoke |

**A+B** entregam o núcleo "Aquarela Básica idêntico ao wet_edges". **C+D** são a riqueza da integração
profunda (papel selecionável + camadas/grupos como papel/granulação).

**Isolamento (Modo L / ADR-0107):** tudo cai em `ph2d-painter-brush`, `ph2d-tool-painter`,
`ph2d-panel-painter-layers`, `ph2d-editor-core` (ids/chrome + action_bus + context menu = foundational
editável na linha), `shells/desktop/render_loop` (foundational, gate testado). Nenhum contrato congelado
(`Tool`/`PanelEvent` caps) precisa mudar — o preset usa `SelectOption` genérico; o tagging usa novas
variantes `EditorAction` **internas** (Hierarchy) que não são o canal congelado de tools. Confirmar no
build combinado.

---

## 7. Referências
- Alvo: `docs/Painter/wet_edges_paint.html` (composite + Beer-Lambert).
- Tier fluido (fora daqui): `docs/Painter/ph2d_wet_paint/` SPEC §4 (papel) + Curtis 1997.
- Papel procedural: IQ (fBm/voronoise/gradient noise), `lowbias32` (nullprogram), Curtis §4.5 granulação,
  Bousseau 2006 (darkening screen-space). Fontes completas no relatório de pesquisa desta sessão.
