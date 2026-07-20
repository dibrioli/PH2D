# 08 — Plano de aquarela: Edge-darkening + Granulação + Build-up subtrativo

> **Escopo.** Três camadas *image-space / por-stroke* que dão paridade visual com o Krita e com
> `docs/Painter/wet_edges_paint.html` **sem simulação de fluido**. Ordem por impacto÷custo (literatura
> Curtis 1997 §4.3.3 · Bousseau 2006 · Montesdeoca 2016): **#1 edge-darkening**, **#2 granulação**,
> **#3 mistura subtrativa**. O tier fluido (Curtis 3-camadas / `ph2d_wet_paint/`) fica **fora** deste
> plano (alto custo, retorno decrescente; já removido de propósito por ADR-0096).
>
> **Princípio-mestre (pedido do Enio):** integrar *profundamente* ao engine existente, **zero
> redundância**. Reaproveitar Grain (granulação), Shape/Stroke (mecânica), `ph2d-color` (cor). Cada
> feature entra por **um único seam de execução**, forçando o caminho per-pixel quando ativa (mesma
> tática já usada por `accumulate_cap`) para não duplicar os caminhos cacheados.
>
> Status: **IMPLEMENTADO (F1–F4) — linha `line/Painter`, aguardando smoke do Enio + integração.**
> Ver §8 (tracker do que landou) no fim do doc.

---

## 0. O que já existe e vamos reusar (nada de crate nova)

| Peça | Onde | Reuso |
|---|---|---|
| Blur separável de máscara | `ph2d-painter-brush/src/blur.rs` | núcleo do blur-diff de **#1** |
| Precedente de pós-processo de pen-up dentro da transação de undo | `heal_inpaint()` em `tool/paint.rs:560` | padrão de inserção de **#1** |
| Acumulação `over` de cobertura por-camada | `stamp_color/accumulate.rs:22` (`accumulate_color_stamp_coverage`) | modelo do coverage-buffer de **#1** |
| Slot **Grain** (`TextureSettings`, `grain_depth`) | `spec.rs:108/112`, amostrado em `dab.rs:466-497` | é o height-field de **#2** |
| `TextureKind::Grain` ("paper/canvas tooth") + mapping `Tiled` (canvas-fixo) | `texture.rs:77` + `texture.rs:504-507` | paper-tooth pronto p/ **#2** — **sem novo kind** |
| Forçar per-pixel (bypass dos caches) | `accumulate_cap` em `stamp_route.rs:409/424/453/459` | tática de seam único p/ **#2** e **#3** |
| Composição de cor do dab | `blend_over` em `dab.rs:549` (blend.rs:132) | único seam de **#3** |
| Transfers sRGB↔linear por LUT | `ph2d-color::srgb` (`SRGB_DECODE_LUT`/encode), `LinearRgba` | building blocks de **#3** |
| **Técnica K–M residual clean-room** (não a lib Mixbox) | `km.rs` recuperável: `git show 90a2d068~1:crates/ph2d-painter-wash/src/km.rs` (`unmix`/`pigment_residual`/`compose_km_mixbox`, ADR-0091) | receita license-clean de **#3** |
| Padrão de seção colapsável + chrome ids | `paint_shape.rs:29` + `ids/chrome/painter_shape.rs` | UI de todas as 3 |

**Correções a suposições antigas (verificadas no código):**
- **Não há módulo `mixbox`** em `ph2d-painter-brush` — foi deletado com o engine antigo. `ph2d-color::PigmentLinearSrgb` é **só um alias de tipo** (`= LinearRgba`), sem funções nem LUT. A mistura precisa ser **restaurada de `km.rs` (git)** ou reescrita (K–M single-constant, SPEC §14).
- **Mixbox a lib é non-commercial** (mesmo bloqueio do Krita, KDE#446759). Usamos **a técnica**, não a lib — como o ADR-0091 já fez. License-clean.
- **Não existe buffer de cobertura por-stroke universal**; o `stroke_mask` só é alocado no caminho Accumulate-OFF e é um mapa de *cap*, não `over`.
- Painel vivo = **`ph2d-panel-painter-layers`**. Ids `PAINTER_STUDIO_*` são **órfãos inertes** — não usar.

---

## 1. Feature #1 — Edge-darkening (a "fringe"): o efeito nº 1, o mais barato

**Teoria.** É o mecanismo do Curtis §4.3.3 eq.(3) `p ← p − η·(1 − blur(M))·M` — pigmento migra
para a frente que seca e empoça na borda. Versão image-space (idêntica ao `wet_edges_paint.html`):
`edge = clamp(cov − blur(cov))` → multiply-darken na borda. **1 blur + 1 pass**, sem solver.

**Estado a criar.** Um `Vec<u8>` de **cobertura por-stroke, sempre-ligado**, espelhando
`self.paint.stroke_mask`:
- Reset junto em `paint.rs:449` (`paint_begin`).
- Acumular `over` a cada dab — **copiar** `accumulate_color_stamp_coverage` (`stamp_color/accumulate.rs:22`): `cov[i] = encode(a + prev·(1−a))`. Alimentar do mesmo alfa efetivo (`w·g·coverage`) que o dab já calcula, no dirty-rect do stroke.

**Seam de execução.** Em **`paint_end`, antes de `close_stroke()` (`paint.rs:562`)** — exatamente
onde `heal_inpaint()` (`paint.rs:560`) já roda um pós-processo dentro da mesma transação de undo:
1. Blur da cobertura sobre o dirty-rect (`ph2d-painter-brush/src/blur.rs`), raio = slider **Spread**.
2. `edge = clamp01(cov − blur(cov)) · EdgeGain`.
3. Multiply-darken em `self.canvas_rgba` só onde `cov > 0` (escurece a cor já depositada; opcional:
   deslocar para K–M se **#3** ligado, ver §3).

**Por que no pen-up e não por-dab:** a borda só é definida pela silhueta *final* do stroke; fazer por-dab
escureceria juntas internas. Igual ao bake do `wet_edges_paint.html` (`endStroke → composite`).

**UI:** sliders **Edge** (ganho) e **Spread** (raio do blur) na nova seção Watercolor (§4).

**Custo:** 1 buffer canvas-size (já temos o `stroke_mask` do mesmo tamanho) + 1 blur no dirty-rect no
pen-up. Trivial. **Determinismo:** blur é soma/deslocamento — HR-5 ok.

---

## 2. Feature #2 — Granulação por papel: **reaproveitar o Grain quase inteiro**

**Intuição do Enio confirmada.** O slot Grain, no mapping **`Tiled`** com **`TextureKind::Grain`**, já
é um **height-field de papel ancorado em canvas-px** (`texture.rs:504-507`; `is_canvas_fixed()`
texture.rs:281). O pattern paper-tooth **já existe**. **Não criamos novo `TextureKind`.**

**O que falta = a curva de resposta do depósito.** Hoje o Grain **multiplica linearmente** a cobertura
(`dab.rs:486-492`: `g *= 1 + (s−1)·depth`). A granulação de aquarela é um **gate não-linear** (Curtis
§4.5 / SPEC §10: `deposit = stamp − (1 − tooth)·gate` → picos do papel pegam pigmento, vales rejeitam;
esse pass/reject *é* a granulação). Um multiply suaviza demais; o gate cria o speckle.

**Seam de execução — um único site.** `dab.rs:486-492` (o combine `g_eff`). Sob um novo escalar
**`granulation: f32`** em `BrushSpec`:
- `granulation == 0` → comportamento atual (multiply linear) — **byte-idêntico**, default neutro.
- `granulation > 0` → gate: `g_eff = clamp01(g − (1 − s)·granulation)` (ou lerp entre multiply e gate
  por `granulation`), onde `s` é a amostra Grain (a "tooth").

**Evitar redundância nos 3 sites de combine.** O `g_eff` é replicado em (a) per-pixel `dab.rs:486`,
(b) cached `stamp.rs render_stamp_mask`, (c) per-layer `stamp_color/accumulate.rs`. Tática já provada:
**granulation ativa → força per-pixel** (bypass dos caches, como `accumulate_cap` em
`stamp_route.rs:409/424/453/459`) ⇒ só `dab.rs:486-492` muda. Um flag/amount é bem menos redundante
que um novo kind.

**UI:** slider **Granulation** na seção Watercolor. (O *tipo* de textura de papel e a *escala* já são os
controles Grain existentes — Kind=Grain, Mapping=Tiled, Size; nada novo.)

**Custo:** ~5 linhas no combine + gating de cache. **Determinismo:** subtração/clamp — HR-5 ok.

---

## 3. Feature #3 — Build-up subtrativo (cor de pigmento real): acima do Krita

**Teoria.** Krita mistura em **média RGB linear** (azul+amarelo→cinza). Aquarela real é **subtrativa**
(azul+amarelo→verde). Duas opções, ambas license-clean (a lib Mixbox é non-commercial — proibida):
- **(a) K–M single-constant (SPEC §14):** `KS(R)=(1−R)²/(2R)`; mistura linear em K/S; inverte
  `R = 1 + KS − √(KS²+2KS)`. Simples, sem NNLS. Uma cor sozinha **não** reproduz exata (leve drift).
- **(b) Residual Mixbox clean-room (`km.rs`, ADR-0091):** `c=unmix(rgb)` (sem `K_REF`) + residual
  `r=rgb−mix(c)`; decode `mix(c̄)+r̄`. **Cor sozinha = identidade exata** (o requisito do Enio de
  2026-06-14). Mais fiel, um pouco mais pesado. **Recuperável do git**, já validado no Metal.

**Recomendação:** começar por **(b) restaurando `km.rs`** (é código do próprio projeto, já auditado e
com a fidelidade "cor pintada = cor escolhida" que o Enio exigiu) reduzido ao caminho CPU per-pixel;
(a) fica como fallback se quisermos menos custo.

**Seam de execução — um único site.** `dab.rs:549`:
```rust
let out = crate::blend::blend_over(blend, prev, color, a);   // ← trocar sob flag pigment
```
Sob **`pigment: bool`** novo em `BrushSpec` (e `pigment_mix: f32` de intensidade):
1. decode `prev` e `color` sRGB→linear (LUT `ph2d-color::srgb`),
2. mistura em espaço pigmento (`compose_km_mixbox` de `km.rs`) ponderada por `a·pigment_mix`,
3. re-encode linear→sRGB.
Os caminhos cached (`stamp.rs:217`, `stamp.rs:463`) **não** mudam: `pigment` ativo **força per-pixel**
(mesma tática), então `dab.rs:549` é o **único** ponto. Building blocks: `LinearRgba` +
`ph2d-color::srgb` (já usados no `compositor/mod.rs:42-97`).

**Interação com #1:** se `pigment` ligado, o multiply-darken do edge (§1) deve escurecer **em K/S**
(aumentar concentração na borda) em vez de multiply sRGB — mais fisicamente correto, borda ganha o
matiz espesso. Um `if pigment` no pass de edge.

**UI:** toggle **Pigment** + slider **Mix** na seção Watercolor.

**Custo:** restaurar `km.rs` (~150 linhas, git) + troca de 1 site. **Determinismo/HR-5:** K–M usa
`sqrt` — verificar escopo do gate `determinism_sweep` (o brush já usa `smoothstep`; se o sweep grepar
`.sqrt(`, tabelar via LUT como o `wet_edges_paint.html` faz com `s2l/l2s`, ou confirmar que o caminho
não é det-gated). **Flag antes de codar.**

---

## 4. UI — uma nova seção "Watercolor" (padrão existente, sem improvisar chrome)

Espelhar `paint_shape_section` (`paint_shape.rs:29`) no painel `ph2d-panel-painter-layers`:

```
paint_collapsible_section(ctx, theme, x, w, y, "Watercolor",
    PAINTER_WATERCOLOR_SECTION, _SECTION_COLOR, _SECTION_RESET)
  ├─ paint_checkbox_row  "Wet edges"   → PAINTER_WATERCOLOR_ENABLE
  ├─ slider "Edge"        → PAINTER_WATERCOLOR_EDGE        (#1 ganho)
  ├─ slider "Spread"      → PAINTER_WATERCOLOR_SPREAD      (#1 raio blur)
  ├─ slider "Granulation" → PAINTER_WATERCOLOR_GRANULATION (#2)
  ├─ paint_checkbox_row  "Pigment"     → PAINTER_WATERCOLOR_PIGMENT (#3 toggle)
  └─ slider "Mix"         → PAINTER_WATERCOLOR_MIX         (#3 intensidade)
```

**Passos (padrão `PAINTER_SHAPE_*`):**
1. Novas `NodeId` const em `ids/chrome/painter_watercolor.rs` via `hash_node_id("painter_brush.watercolor_*")`; incluir resets em `PAINTER_BRUSH_SECTION_RESETS` (`painter_shape.rs:53`).
2. Seção colapsável espelhando `paint_shape.rs`; sliders com o helper de `PAINTER_SHAPE_ANGLE`; toggles com `paint_checkbox_row` (como `PAINTER_SHAPE_RAKE`, `paint_shape.rs:145`).
3. Fios `PanelEvent` (canal congelado) → novos setters em `PainterTool` espelhando `set_brush_grain_depth` / `toggle_brush_shape_rake`.
4. Labels **em inglês** (HR-15 + memória `feedback_app_ui_english_only`): "Wet edges", "Edge", "Spread", "Granulation", "Pigment", "Mix".

---

## 5. Contrato de dados (campos novos)

**`BrushSpec` (`ph2d-painter-brush/src/spec.rs`)** — todos default-neutros (byte-idêntico com watercolor OFF):
```rust
pub watercolor: bool,      // gate mestre (força per-pixel quando true)
pub edge_gain: f32,        // #1  default 0.0
pub edge_spread: f32,      // #1  raio do blur (px), default ~7
pub granulation: f32,      // #2  default 0.0 (multiply linear = hoje)
pub pigment: bool,         // #3  default false
pub pigment_mix: f32,      // #3  default 0.0
```
Espelhar em `BrushSettings` (`tool/paint/brush_settings.rs`, cópia panel-facing) + serialização
(cuidado com SCHEMA_VERSION / postcard posicional — **append no fim**, nunca no meio).

**Estado runtime** (`PaintState`/`paint.rs`): `stroke_coverage: Vec<u8>` (§1), alocado/reset como
`stroke_mask`.

---

## 6. Ordem de implementação e gates

Fases pequenas, cada uma fecha com `cargo check -p` no inner loop e gate no fim (§DIRETIVA):

1. **F1 — plumbing neutro:** campos em `BrushSpec`+`BrushSettings`+serialização + seção UI vazia
   (toggle "Wet edges" inerte). Gate: round-trip de serialização + byte-idêntico com OFF.
2. **F2 — #1 Edge-darkening:** coverage-buffer + pós-processo pen-up + sliders Edge/Spread.
   Verify: pintar traço, soltar → borda escura (comparar com `wet_edges_paint.html`).
3. **F3 — #2 Granulação:** gate no `g_eff` + force-per-pixel + slider. Verify: Grain=Tiled+paper,
   Granulation>0 → speckle nos vales.
4. **F4 — #3 Pigment:** restaurar `km.rs` (git) → CPU per-pixel; troca `dab.rs:549` + toggle/Mix +
   integração K/S com edge (§3). Verify: azul sobre amarelo molhado → verde.
5. **Fecho de módulo:** `scripts/nextest-impacted.sh` + clippy `--all-targets` + auditoria ≥2 lentes
   sobre o diff acumulado; **flag HR-5** do `sqrt` do K–M resolvida (LUT ou confirmação de escopo).

**Isolamento (§0.2):** tudo cai em `ph2d-painter-brush`, `ph2d-tool-painter`, `ph2d-panel-painter-layers`
e novos ids em `ph2d-editor-core/src/ids/chrome/`. `ph2d-color` só é **lido** (alias já existe). Se
precisar tocar contrato congelado (`Tool`/`PanelEvent`) ou `ph2d-color` além de leitura → **parar e
reportar ao Coordenador**.

---

## 7. Resumo dos seams (uma linha cada)

| # | Feature | Único seam de execução | Estado novo | Reuso |
|---|---|---|---|---|
| 1 | Edge-darkening | `paint.rs:562` (pré-`close_stroke`, padrão `heal_inpaint`) + blur.rs | `stroke_coverage: Vec<u8>` | blur.rs, accumulate.rs:22 |
| 2 | Granulação | `dab.rs:486-492` (gate no `g_eff`, force-per-pixel) | `granulation: f32` | Grain `Tiled`+`Grain` kind |
| 3 | Build-up subtrativo | `dab.rs:549` (troca `blend_over`, force-per-pixel) | `pigment`/`pigment_mix` | `km.rs` (git) + `ph2d-color::srgb` |

**Sem crate nova. Sem novo `TextureKind`. Cada feature = 1 seam + gate para não duplicar os caches.**
Fluido (Curtis / `ph2d_wet_paint/`) permanece fora de escopo.

---

## 8. Tracker — o que landou (line/Painter, Modo L / ADR-0107)

Quatro fases, cada uma commitada localmente + gate batched no fim (clippy `--all-targets` limpo,
suítes verdes, gates de arquitetura verdes). **Byte-idêntico com a seção OFF** em todas — a
neutralidade é o gate `watercolor` (não zeros nos params), então ligar "Wet edges" mostra efeito na
hora. Toda a matemática é **det-safe (HR-5)**: só somas/clamps/`sqrt`/`min`/`max` — nenhum
`exp`/`ln`/`pow` (por isso NÃO revivemos `km.rs`, que usa `exp`/`ln`).

| Fase | Feature | Seam(s) | Prova (asserção-vermelha) |
|---|---|---|---|
| **F1** | Seção **Watercolor** (plumbing) | `BrushSpec`+`BrushSettings`+`snapshot` · chrome ids `painter_watercolor.rs` · `paint_watercolor.rs` · router `watercolor_settings.rs` · wiring 7-pontas | `tests/seam.rs` (Click+SetValue forward) + `watercolor_settings::tests` (PanelEvent→campo) + gate `panel_wiring_parity` |
| **F2** | **Edge-darkening** (#1) | coverage por-stroke (`stroke_coverage`, discos max-blend em `stamp_dabs`, clear em `stamp_drag_preview`) + blur-diff no pen-up `wet_edges.rs::apply_wet_edges` (antes de `close_stroke`, padrão `heal_inpaint`) | `watercolor_edge_darkens_the_rim_not_the_interior` (rim < interior; uniforme OFF) + `box_blur` units |
| **F3** | **Granulação** (#2) | helper único `texture::grain_coverage`/`granulation_gate` (valley-gate multiplicativo) nos **5** sites de combine do Grain + `StampKey.granulation` (re-baka o cache) | `watercolor_granulation_rejects_valley_deposit` (Tiled Grain: gran↑ ⇒ tinta↓, mas >0) + units byte-idêntico |
| **F4** | **Pigment build-up** (#3) | `blend::blend_over_pigment` (RYB Gossett & Chen 2004, subtrativo em sRGB) nos **3** sites de cor sólida (`dab`/`blit_stamp`/`canvas_blit_band`) | `pigment_mixes_blue_and_yellow_toward_green` + `watercolor_pigment_mixes_wet_on_wet_toward_green` (verde sobe vs plain) + byte-idêntico OFF |

### Decisões de implementação (desvios do plano, com razão)

1. **Pigment = RYB subtrativo em sRGB, NÃO K–M/`km.rs`.** O `km.rs` (residual Mixbox) usa `exp`/`ln`
   → conflita com HR-5, e `ph2d-painter-brush` não linka `ph2d-color`/LUT sRGB. **RYB (Gossett & Chen
   2004)** — o mesmo modo "realista" do `wet_edges_paint.html` — é algoritmo publicado, opera direto em
   sRGB `[0,1]`, **det-safe** (min/max/mul), sem dep nova, e dá azul+amarelo→verde. É o caminho
   license-clean e det-safe. K–M linear (via LUT) fica como upgrade futuro se quisermos fidelidade
   espectral.
2. **Granulação em TODOS os sites de combine (não "force per-pixel").** Com Grain `Tiled` (paper) o
   roteador usa o cache canvas-fixo, não o per-pixel — então o gate entra via um helper único
   (`grain_coverage`) chamado nos 5 sites, mantendo os caches (sem perda de perf) e consistência total.
   Forma **multiplicativa** do valley-gate para compor com o path de Color-Ramp.
3. **Defaults sensatos-quando-ligado** (`edge_gain 3`, `spread 7`, `granulation 0.3`, `mix 0.5`): a
   neutralidade vem do gate `watercolor=false`, então ligar a seção mostra efeito imediato.

### Escopo conhecido (honesto, não "silently dead")

- **Pigment** aplica aos 3 caminhos de **cor sólida** (o pincel de aquarela típico: cor + Grain).
  Os caminhos de **Color-Ramp / Per-Layer-Color** (`stamp_color.rs`/`stamp_ramped.rs`) seguem em
  blend normal — misturar pigmento com um stamp de cor espacialmente variável é semântica distinta,
  follow-up. O toggle Pigment funciona para o brush padrão (99% do uso).
- **Granulação** precisa de um **Grain atribuído** (`Tiled` + kind `Grain` p/ paper); sem Grain, o
  gate opera sobre sample constante 1.0 = sem efeito (esperado).
- **Fluido / backruns / K–M espectral**: fora de escopo (tier caro; `ph2d_wet_paint/` é a referência
  se um dia formos por aí).

### Pendente para fechar (DoD DIRETIVA §5)

- **Smoke visual do Enio** (efeito perceptual — DIRETIVA §4 exige OLHAR): pintar traços com "Wet
  edges" on, testar Edge/Spread, um Grain `Tiled` + Granulation, e Pigment (azul→amarelo molhado).
- **Integração** ao main via `bash scripts/foundational-integrate.sh` (toca foundational em
  `ph2d-editor-core` → gate da árvore combinada `cargo check --workspace`). Só após o smoke.
