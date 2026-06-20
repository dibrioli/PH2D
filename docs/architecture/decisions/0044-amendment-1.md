# ADR-0044 Amendment 1 — `Stamp._pad` → `roundness` (W2.9 shape squash)

**Status:** Accepted · 2026-06-17 · Coordenador
**Amends:** [0044-brush-engine-gpu.md](0044-brush-engine-gpu.md) §2.3 (`Stamp` ABI)

## Contexto

W2.9 do Brush Engine (paridade Procreate — [`docs/Novo Painter/03`](../../Novo%20Painter/03_plano_implementacao.md))
exige **Roundness**: o shape carimbado vira uma **elipse** (nib caligráfico) em vez de
um círculo. O fator de squash é **por-dab** — base do brush, modulado por pressão,
tilt e jitter — então precisa viajar no `Stamp` (96 B, ABI congelada, ADR-0044 §2.3)
do scheduler (CPU) até o consumer (`cpu_render` + `stamp.wgsl`).

O `Stamp` estava **cheio**: 92 dos 96 bytes em uso; só o último `u32` (`_pad`,
offset 92..96) sobrava, reservado p/ alinhamento e mantido `0` em det-mode
(audit D-F7). Não havia slot livre — daí a nota ratificada em `sections.rs`
("squashing needs a roundness field in the FROZEN 96B Stamp ABI (Coord+ADR)").

## Decisão

**Repurposar `Stamp._pad` (offset 92..96) → `pub roundness: f32`.** O `Stamp`
permanece **exatamente 96 B, `repr(C, align(16))`, `Pod + Zeroable`** — o slot
trocado é o mesmo `u32`→`f32` no fim da struct; **nenhum offset anterior muda**.

- **Semântica:** `roundness ∈ [ROUNDNESS_MIN, 1]`; `1.0` = círculo (sem squash),
  valores menores comprimem o eixo perpendicular (`v`) do shape dentro do mesmo
  footprint quadrado. `ROUNDNESS_MIN = 0.05` é um **guard numérico** (não física)
  que impede `vr / roundness` de colapsar numa linha de largura-zero (razão de
  aspecto máx. 20:1). Consumer trata `≤0`/non-finite como `1.0` (defesa p/ slot
  zeroed/garbage).
- **`Stamp::zeroed()`** agora nasce com `roundness: 1.0` (como `grain_layer`/
  `grain_scale` já têm sentinels) — um stamp todo-zero teria `roundness == 0`
  (linha degenerada). O construtor deixa de ser "bit-zero".
- **Det-mode (`_pad == 0`):** o offset 92 deixa de ser padding e passa a carregar
  dado **determinístico legítimo** (entra no hash de replay como qualquer outro
  campo). O método `assert_pad_zero()` e o invariante `_pad == 0` são **removidos**
  (zero callers externos — confirmado por grep cross-workspace). HR-5 preservado:
  o valor é função determinística de `(brush, sample, stamp_index)`.

### Avaliação por-dab (scheduler)

`StampScheduler::push_one_stamp` computa o squash a partir do modelo `ShapeParams`
(já existente desde ADR-0044) — cobre **W2.9 + W2.11 + W2.12 + W2.13(vertical)**
num único valor:

```
r  = shape_roundness                       // base [0,1]
r *= 1 - shape_pressure_roundness · pressure   // pressão achata o nib
r *= 1 - shape_tilt_roundness · (tilt / (π/2)) // tilt achata
r *= 1 - shape_vertical_jitter · rand(0xD5)    // jitter por-dab (eixo registrado)
roundness = clamp(r, ROUNDNESS_MIN, 1)
```

Cada termo é `1 − amount·input`: `amount=0` **ou** `input=0` ⇒ sem efeito. Um brush
default (`roundness=1`, moduladores `0`) emite `1.0` ⇒ o squash CPU/GPU é a
**identidade** ⇒ render **byte-idêntico** ao caminho pré-amendment (os guards `>0`
também deixam o stream PRNG 0xD5 intocado). Eixo `0xD5` registrado na tabela
`det_random` + gate `det_random_axis_tags_match_registry`.

### Squash (consumer, CPU + WGSL espelhados)

No `apply_one_stamp` / `apply_one_stamp_wash` (e em `cs_stamp`), após rotacionar o
sample p/ shape-space, antes do un-center: `v = vr / roundness + 0.5`. Como
`roundness ≤ 1`, `v` alcança a borda `[0,1]` mais cedo ⇒ elipse mais fina no eixo
`v`, alinhada ao `rotation_rad` do stamp (acompanha follow-angle/scatter — base do
nib caligráfico). O eixo maior (`u`) mantém o `size_px` cheio; a elipse cabe sempre
dentro do footprint quadrado (squash só encolhe).

### Spacing direcional (scheduler — fix dirigido pelo smoke 2026-06-17)

O squash sozinho deixava **listras horizontais** ("venetian-blind"): o `spacing` era
derivado do diâmetro cheio, mas um nib achatado é fino no eixo menor — quando o traço
viaja **ao longo** desse eixo, dabs finos ficam espaçados de um eixo-maior → gaps. Fix:
o passo de spacing encolhe p/ a **extensão da elipse na direção do traço**, fator
`√(cos²Δ + r²·sin²Δ)` (Δ = ângulo entre traço e o eixo maior do nib). Construído da
`stroke_dir` unitária + `sin_cos(base_rotation)` (sem `atan2`); `r ≥ 1` faz early-return
de `spacing_px` exato (default byte-idêntico). Com Follow Rotation o nib segue o traço
⇒ Δ = −base_rotation ⇒ independente da direção. Guardado pelo golden
`low_roundness_stroke_across_the_thin_axis_has_no_horizontal_gaps` (sem o fix: 20 rows
vazias; com: 0).

## Consequências / gates

- **`architecture_painter_contract_surface`** (gate `stamp_size_is_96_bytes_aligned_16`):
  **VERDE sem edição** — checa textualmente `size_of::<Stamp>() == 96` +
  `align_of == 16`, ambos preservados.
- **`architecture_studio_slider_wiring`:** `SHAPE_ROUNDNESS_SLIDER` **sai da
  allowlist `DORMANT`** (era stub forward; agora o engine lê o campo e a UI o
  expõe — slider VIVO, fiado nos 8 sites). `ALPHA_THRESHOLD_SLIDER` segue dormente.
- **`shader_stamp_struct_size_matches_rust_abi`** (naga): VERDE — o WGSL declara
  `roundness: f32` no offset 92.
- **Persistência:** `Stamp` é efêmero (per-frame), não serializado — savefiles
  referenciam o brush por `BrushParamsHash`. Nenhuma mudança de `SCHEMA_VERSION`.

### Aceitação (golden, headless — DIRETIVA §4)

- `roundness_one_paints_a_round_dab_and_low_roundness_squashes_in_y` — anisotropia:
  `roundness=1` ⇒ dab circular; `0.3` ⇒ extent-Y encolhe, extent-X mantém.
- `roundness_squash_axis_rotates_with_the_stamp` — a `0°` fino em Y; a `90°` o eixo
  fino troca p/ X (prova que o squash acompanha o stamp = nib caligráfico).
- `roundness_zero_or_garbage_is_treated_as_round_not_a_degenerate_line` — defesa do floor.
- `default_brush_emits_round_dabs` / modulação `pressure`/`tilt`/`vertical_jitter`
  determinística (scheduler).
- **Smoke manual** (Apple Pencil, W9-adjacent): tilt achatando o nib em tempo real.

## Escopo / follow-ups nomeados (DIRETIVA §5 — NÃO meia-costura, próximos passos)

Este amendment fecha **W2.9/W2.11/W2.12/W2.13(vertical)**. Ficam nomeados:

1. **W2.10 `shape_angle`** (nó de ângulo-base do grafo Roundness, `shapeAngle`) —
   campo novo em `ShapeParams` (há headroom: 15/20) que **dobra em `rotation_rad`**
   (como `follow_angle`); 2ª superfície de contrato (Brush) + widget de **dial de
   ângulo** próprio (não pct-slider). Baixo risco de persistência (sem golden-hash
   pinado), mas merece passo focado.
2. **W2.13 horizontal jitter** — exige um **2º carrier de anisotropia** no Stamp
   (squash independente em `u`); não cabe no slot único de `roundness`. Defer até
   nova folga de ABI ou bitpacking.
3. **W2.2 `input_style`** (Azimuth/Barrel) — bloqueado no input de tilt/azimuth
   (dormente no caminho vivo; T-input/W9).
4. **W2.14 `shape_filtering`** — argumento da render-call (como `paper_grain`), sem
   tocar a ABI; passo separado de qualidade de AA.
