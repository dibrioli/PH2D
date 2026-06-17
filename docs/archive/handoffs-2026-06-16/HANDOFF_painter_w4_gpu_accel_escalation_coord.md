═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · ESCALAÇÃO (prioridade): GPU dos 5 kinds CPU-fallback
Autor: Implementador Painter (jornada 2026-06-06) · supersede o "non-urgent" dos
       handoffs anteriores — o Enio pediu MÁXIMO (zero perf na mesa)
═══════════════════════════════════════════════════════════════════

## §0 — Mandato
O Enio foi explícito: **"não é uma escolha deixar aquém do máximo."** Fiz o CPU chegar
no **máximo de hardware** (multithread em todos os kernels + downsample no Bloom, commits
`aa3228e`/`3a9dbe7`). Mas o **teto real** é a GPU (~30× vs CPU, memória `project_painter_composite_perf`).
Hoje **5 kinds caem no CPU fallback** porque não têm código GPU — e um único deles numa
stack joga TUDO pro CPU (flatten devolve None). Pra fechar o máximo, esses 5 precisam de
kernel GPU. **As refs CPU estão completas e testadas** — é espelhar.

## §1 — OS 5 KINDS (prioridade por impacto de perf)
| Kind | Tipo | Ref CPU (minha, testada) | Infra GPU que falta (tua) |
|---|---|---|---|
| **Bloom** | spatial | `spatial::apply_bloom` | `SPATIAL_BLOOM` + mip/Kawase pyramid (bright-pass→down→blur→up→add) |
| **ShadowsHighlights** | spatial-tonal | `spatial::apply_shadows_highlights` | blur-de-luma (2 raios) + combine variante (lê 2 mapas locais) |
| **Noise** | per-pixel coord | `spatial::apply_noise` | `ADJ_NOISE` no `layer_composite.wgsl` (hash na coord `global_id` + flip do `gpu_code`) |
| **Halftone** | per-pixel coord | `spatial::apply_halftone` | `ADJ_HALFTONE` (screen-function na coord) |
| **ColorLookup** | per-pixel | `lut::apply_color_lookup` (8 looks analíticos) | `ADJ_COLOR_LOOKUP` (switch de preset OU 3D-LUT texture sampler p/ .cube) |

Noise/Halftone/ColorLookup são **per-pixel** → entram no teu switch escalar
`apply_adjustment` WGSL (igual Vibrance/Curves), + flip do `gpu_code()` de `None` pro
código novo. **Eu ligo o lado do tool** (o `gpu_code()` em `ph2d-painter-brush` + os
`gpu_params`) no instante em que o teu WGSL landar — me pinga.

## §2 — RECONCILIAÇÃO (sem quebrar gates)
`ph2d-render` é dev-dep de `ph2d-painter-brush`, então tua ref CPU dos gates de paridade
pode CHAMAR as minhas direto (`apply_bloom`/`apply_shadows_highlights`/`apply_noise`/
`apply_halftone`/`apply_color_lookup`) — zero duplicação. Todas premultiplicadas onde
relevante (Bloom feathera cobertura; ver `feathers_coverage()` + meu §3 do handoff premul).
Determinismo: os kernels CPU são bit-idênticos multithread (split por linha), então servem
de golden estável.

## §3 — O QUE EU JÁ DEIXEI PRONTO DO MEU LADO
- `gpu_spatial_code()` (Gaussian/Sharpen/Motion/Chroma já emitem `SpatialAdjustment`).
  Pra Bloom/SH eu adiciono o código espacial assim que tua pyramid/combine landar.
- `feathers_coverage()` (combine adota alpha feathered: blur-family + **Bloom**; SH/tonais preservam).
- UI completa (sliders/segments/toggles) dos 24 kinds — a UI não muda quando o compute
  vira GPU (mesmo `*_params`).
- Flatten do bridge (`painter_gpu_flatten.rs`) — adiciono os emits assim que houver código GPU.

## §4 — POSSE
`ph2d-render` (WGSL + pyramid + combine) = TEU. `ph2d-painter-brush`/`tool`/bridge = meu
(ligo o tool-side). Sem push (tu shipas). **Pedido: priorizar esses 5** — é o delta entre
"W4 funciona" e "W4 no máximo". Ordem sugerida por dor de FPS: Bloom → S/H → Noise/Halftone/ColorLookup.
═══════════════════════════════════════════════════════════════════
