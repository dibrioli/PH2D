# Novo Painter — Algoritmos de referência (clean-room)

> **Como usar:** o implementador codifica **a partir deste doc** (comportamento + matemática),
> **não** lendo os `.cc` do Blender enquanto escreve Rust. A árvore GPL em
> [`reference/blender-texture-paint/`](../../reference/blender-texture-paint/) é o **mapa de
> onde** cada comportamento vive; a matemática abaixo é padrão da literatura de pintura raster
> (falloff, spacing, pressão, source-over), **não** expressão GPL. Cada item linka o arquivo
> Blender só como *behavioral reference*. Valide as formas contra as imagens em
> [`blender_ui_reference/`](blender_ui_reference/).

---

## 0. Convenções

- Buffer do layer = RGBA8 **straight**, sRGB-encoded, row-major, `w*h*4`.
- Trabalho de cor em **linear**: decode sRGB8→linear (LUT canônica `ph2d-color`), opera, encode de volta.
- Dab centrado em `c=(cx,cy)` (coords de imagem, float). `R = size_px/2`. Para pixel `p`: `d=|p−c|`, `t=clamp(d/R,0,1)`.
- Blend reusa `ph2d-painter-effects::blend::apply` — **não** reimplementar math de blend.

---

## 1. Falloff do dab (máscara radial) — `Falloff::weight(t)`

**Behavioral reference:** Blender `blenkernel/intern/brush.cc` (curve presets) + editor
`mesh/paint_image_2d_curve_mask.cc` (aplicação da máscara). **Math = curvas padrão**; alvo visual =
`blender_ui_reference/brush_falloff_*.png`.

Peso `w(t) ∈ [0,1]`, `w(0)=1` (centro cheio), `w(1)=0` (borda zero), monotônico decrescente:

| Preset | `w(t)` | Forma |
|---|---|---|
| Constant | `1` (e 0 em t≥1) | disco duro |
| Linear | `1 − t` | cone |
| Smooth | `1 − (3t² − 2t³)` | smoothstep (default macio) |
| Smoother | `1 − (6t⁵ − 15t⁴ + 10t³)` | smootherstep (mais macio) |
| Sphere | `sqrt(1 − t²)` | ombro arredondado |
| Root | `sqrt(1 − t)` | borda suave, centro cheio |
| Sharp | `(1 − t)²` | concentrado no centro |
| Sharper | `(1 − t)⁴` | muito concentrado |
| Custom | curva editável (2D) | ver §1.1 |

**Hardness `h ∈ [0,1]`** (plateau central antes do falloff) — remap antes da curva:
`t' = clamp((t − h)/(1 − h), 0, 1)` e então `w = curve(t')`. `h=1` ⇒ disco duro; `h=0` ⇒ curva pura.
(Comportamento idêntico ao "hardness" de Krita/Photoshop; não é número mágico — é a definição.)

### 1.1 Curva custom (widget 2D)
Tabela de pontos `(x,y) ∈ [0,1]²` amostrada (LUT 256). Editor 2D = `InteractiveState`+dispatch em
editor-core (padrão BlenderHit — Slider 1D não basta; memória `panel 2D-drag precisa dispatch`).
Ref. visual: `brush_falloff_brush-curve.png`.

---

## 2. Aplicação do dab — `stamp_dab`

**Behavioral reference:** `mesh/paint_image_2d.cc` (soft-brush 2D, anti-alias de borda).

Para cada pixel `p` no bbox `[c−R, c+R]` clampado ao canvas:
```
t  = clamp(|p−c|/R, 0, 1)
a  = falloff.weight(t) * flow * coverage     // coverage = dynamics/pressão deste dab
if alpha_locked: a *= dst.alpha               // pinta só onde já há alpha
if mask presente:  a *= mask(p)               // máscara do layer ativo
Cs = brush_color_linear                       // cor do brush em linear, alpha = a
dst_linear = decode_srgb(dst_rgba8)
out_linear = blend::apply(spec.blend, Cs, dst_linear)   // a já dobrada no alpha de Cs
dst_rgba8  = encode_srgb(out_linear)
```
- **Anti-alias de borda:** o próprio `w(t)→0` em t→1 dá borda macia; para brush "hard" (hardness alto),
  aplicar AA de ~1px no `t≈1` (cobertura fracionária) para não serrilhar.
- **HR-3:** bbox pré-calculado uma vez; itera slice sem alloc por pixel.
- **Acúmulo dentro do MESMO stroke:** "strength" limita a opacidade máxima acumulada por stroke
  (Blender: build-up vs. cap). Implementar como teto por-pixel por-stroke (buffer de "pintado neste
  stroke") quando `strength<1` e blend=Normal — senão sobrepor dabs satura. (Ver `paint_image_2d.cc`
  accumulation buffer — comportamento, não código.)

### 2.1 Paridade obrigatória (DIRETIVA §4)
`stamp_dab(Normal, flow=1, hardness=1, coverage=1)` de cor opaca sobre fundo opaco
== `blend::apply(Normal, Cs, Cb)` **bit a bit**. É a asserção-vermelha da Fase 1.

---

## 3. Motor de stroke — `Stroke::push(point) -> [Dab]`

**Behavioral reference:** `editors/sculpt_paint/paint_stroke.cc` (`paint_space_stroke`, spacing/airbrush).
Ref. visual: `brush_stroke_stroke-panel.png` (Method/Spacing/Jitter/Smooth/Input Samples).

### 3.1 Spacing = "Space" (default)
Emite dabs a intervalos de distância constante ao longo do caminho:
```
step = spacing_frac * diameter          // diameter = 2R; spacing_frac default 0.10 (10%)
para cada novo segmento (last_point → point):
    seg_len = |point − last_point|
    while accum_dist + seg_len >= step:
        f = (step − accum_dist) / seg_len     // parâmetro no segmento
        emit_pos      = lerp(last_point, point, f)
        emit_pressure = lerp(last_pressure, pressure, f)
        emitir Dab{ pos: emit_pos + jitter(), coverage: dyn(emit_pressure) }
        last_point = emit_pos; seg_len -= (step − accum_dist); accum_dist = 0
    accum_dist += seg_len
```
- **Jitter:** deslocamento aleatório `≤ jitter*R`. RNG com seed por-stroke (determinismo HR-5 se algum
  dia entrar em replay; usar `Pcg64Mcg` seedado, não `thread_rng`).
- **Input Samples:** média de N amostras de entrada antes de processar (suaviza ruído do device).

### 3.2 Outros métodos (Fase 2+/DEFER)
`Line`, `Curve`, `Dots`, `Anchored`, `Airbrush`. Airbrush = emissão por **tempo** (rate dabs/seg) via
`Tool::on_tick` enquanto o ponteiro está down e parado.

### 3.3 Smooth Stroke (stabilizer)
**Behavioral reference:** mesmo painel. O cursor "real" puxa um cursor "atrasado" por mola:
`smoothed += (input − smoothed) * factor` só quando `|input − smoothed| > radius`. Dabs usam `smoothed`.
Pode ser DEFER nomeado se a Fase 2 inchar.

---

## 4. Dynamics — pressão/tilt → size/strength

**Behavioral reference:** `DNA_brush_types.h`/`brush.cc` (pressure mappings).
```
dab_size     = base_size * lerp(size_min,     1, pressure)   // se "Size" mapeado à pressão
dab_strength = strength  * lerp(strength_min, 1, pressure)   // idem "Strength"
```
- `pressure ∈ [0,1]`; mouse/sem-Pencil ⇒ `pressure = 1.0` (sem efeito).
- Curva de pressão opcional (mesma infra de curva custom §1.1) reescala `pressure` antes do lerp.
- Tilt → futuro (ângulo/achatamento do dab); DEFER nomeado.

---

## 5. Undo de stroke (tiles)

**Não há equivalente direto reutilizável** (o WAL/stroke do Blender é outro modelo). Design PH2D:
- Canvas dividido em tiles (ex. 128×128). No 1º pixel que um dab toca num tile durante o stroke,
  snapshot do tile "before" (`Box<[u8]>`). Ao fechar o stroke: 1 `UndoEntry::Stroke{layer, tiles}`.
- Undo: restaura cada tile "before"; bump `layer_pixel_versions` (re-upload GPU). Redo: reaplica
  "after" (snapshot no close, ou re-stamp determinístico). Integra no `UndoController` existente
  (`ph2d-tool-painter/src/undo.rs`).

---

## 6. Mapa Blender (arquivo) → alvo PH2D

| Blender (referência comportamental) | LOC | Vira no PH2D |
|---|---:|---|
| `editors/sculpt_paint/paint_stroke.cc` | 1777 | `ph2d-painter-brush/src/stroke.rs` + `dynamics.rs` (§3,§4) |
| `mesh/paint_image_2d.cc` | 2168 | `ph2d-painter-brush/src/dab.rs` (§2) |
| `mesh/paint_image_2d_curve_mask.cc` | 193 | `ph2d-painter-brush/src/falloff.rs` (§1) |
| `blenkernel/intern/brush.cc` (curve presets) | 2022 | presets de `falloff.rs` + defaults de `spec.rs` |
| `makesdna/DNA_brush_types.h` / `_enums.h` | 501/615 | campos de `BrushSpec` (§1.1 doc 02) — **derivar, não copiar** |
| `imbuf/intern/rectop.cc` (`IMB_blend_*`) | 1323 | **NÃO portar** — usar `ph2d-painter-effects::blend` |
| `mesh/paint_image_proj.cc` (projection 3D) | 7217 | **DEFER** (fora da 1ª entrega) |
| `editors/sculpt_paint/paint_cursor.cc` | 1333 | ring do cursor (overlay) — Fase 4/host |
| `mesh/paint_image.cc` (undo de imagem) | 980 | inspira o undo de tiles (§5) — modelo PH2D próprio |

**Regra final (DIRETIVA §1):** achou constante mágica no comportamento (um `*_MAX`, um fator solto)?
PARE e ache/derive a fonte — não transcreva número do Blender sem entender.
