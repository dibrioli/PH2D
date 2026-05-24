# 09 — Export e Interop

## 9.1 Native format: `.ph2d-painter`

Container postcard binário com layers + brush refs + stroke history + time-lapse opcional. HR-6 (content-addressed via blake3), HR-14 (versionado, migration chain).

### 9.1.1 Schema v1

```rust
#[derive(Serialize, Deserialize)]
pub struct PainterFileV1 {
    pub version: u32,                        // = 1
    pub uuid: Uuid,                          // canvas identity
    pub blake3_payload: [u8; 32],            // hash do payload (excluding este campo)

    // Canvas metadata
    pub canvas_meta: CanvasMeta,             // dimensions, DPI, color profile, background

    // Layer stack
    pub layers: Vec<LayerRecord>,            // ver §9.1.2

    // Brush references (NOT inline brush data; só refs)
    pub brush_refs: Vec<BrushRef>,           // (uuid, blake3_self, name_hint)

    // Stroke history (último 250)
    pub stroke_history: VecDeque<StrokeRecord>,  // ring trimmed

    // Selection persistente
    pub saved_selections: [Option<Selection>; 4],

    // Color state
    pub color_state: ColorState,             // primary, secondary, history, active palette ref

    // Active drawing guide (opcional)
    pub active_guide: Option<DrawingGuide>,

    // Animation Assist state (opcional, se canvas usou anim)
    pub animation: Option<AnimationAssistState>,

    // Time-lapse (opcional, embedded video)
    pub timelapse: Option<TimelapseEmbed>,   // bytes do MP4/HEVC; compactado externamente

    // Painter Preferences snapshot (per-doc overrides)
    pub doc_prefs: DocPainterPreferences,
}
```

### 9.1.2 LayerRecord

```rust
#[derive(Serialize, Deserialize)]
pub struct LayerRecord {
    pub id: LayerId,
    pub name: String,
    pub kind: LayerKind,                    // Raster | Mask | Group | Reference | ...
    pub visible: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub alpha_lock: bool,
    pub clipped_by: Option<LayerId>,
    pub mask: Option<MaskData>,
    pub children: Vec<LayerRecord>,         // se Group; nesting cap 8

    // Bitmap data — guardado como blake3-addressed asset
    pub pixels_blake3: Option<[u8; 32]>,    // hash do pixel data
    pub pixels_codec: PixelCodec,           // RawRgba8 | RawRgba16F | PngLossless | Zstd
}
```

Bitmap data: blake3-addressed em **asset DB** (HR-6). Múltiplos LayerRecords podem referenciar mesmo blake3 (clones, undo states). PNG lossless preferido (smaller); Zstd raw para non-image data.

### 9.1.3 Container

`.ph2d-painter` é tar.zst:
- `manifest.postcard` — `PainterFileV1` serialized (sem pixels inline).
- `assets/{blake3}.png` — bitmap data.
- `assets/brushes/{blake3}.ph2d-brush` — brush data inline (se brush não está na library global).
- `timelapse.mp4` — opcional.

Tar permite stream-read; zstd permite compress.

### 9.1.4 Migration

`PainterFileV2 = ...` futura → `fn migrate_v1_to_v2(v1: PainterFileV1) -> PainterFileV2`. HR-14: build de release não compila sem migrator quando schema muda.

## 9.2 Raster export

| Formato | Color depth | Alpha | Layers preserved | Animation |
|---------|-------------|-------|-------------------|-----------|
| **PNG** | 8-bit / 16-bit | ✓ | — (flattened) | — |
| **JPEG** | 8-bit | — (composite on white default) | — | — |
| **TIFF** | 8/16/32-bit | ✓ | ✓ (multi-layer TIFF) | — |
| **WebP** | 8-bit | ✓ | — | ✓ (animated WebP) |

### 9.2.1 PNG export

- Default 8-bit RGBA com alpha.
- Toggle "16-bit" para canvases P3 (preserva wide gamut precision).
- Color profile (sRGB / Display P3) embed via `iCCP` chunk.
- Padrão: lossless. Sub-toggle "Compression level" (0–9) — default 6.
- Atalho: `Ctrl+Shift+S` para Export PNG default location.

### 9.2.2 JPEG export

- Slider quality 1–100 (default 90).
- Lossy.
- Background color: white default ou current canvas bg.

### 9.2.3 TIFF export

- Lossless. 8/16/32 bit float.
- Multi-layer: layers viram TIFF pages (Photoshop reads).
- Color profile embedded.
- Para print workflow.

### 9.2.4 WebP export

- 8-bit, alpha supported.
- Quality slider (lossy) ou toggle "lossless".
- Animated WebP para anim assist export.

## 9.3 PSD export (interop crucial)

### 9.3.1 Layer mapping

PSD format é well-documented (open spec). Painter mapeia 1:1 onde possível.

| Painter | PSD |
|---------|-----|
| Raster layer | Raster layer |
| Mask layer | Layer mask channel (attached to parent) |
| Clipping mask | "Knockout" + group; PSD não tem clipping mask 1:1, **emula** com clip group |
| Reference layer | Discarded (Painter-only concept; PSD has nothing equivalent) |
| Group | Layer group |
| Alpha-locked | Discarded (PSD has no equivalent; pintura already restricted) |
| Layer opacity | Layer opacity |
| Layer visibility | Layer visibility |
| Blend mode | Mapped via table (§9.3.2) |
| Layer name | Layer name (UTF-16) |

### 9.3.2 Blend mode mapping

| Painter | PSD code | Notes |
|---------|----------|-------|
| Normal | `norm` | 1:1 |
| Multiply | `mul ` | 1:1 |
| Darken | `dark` | 1:1 |
| Color Burn | `idiv` | 1:1 |
| Linear Burn | `lbrn` | PSD 1:1 |
| Lighten | `lite` | 1:1 |
| Screen | `scrn` | 1:1 |
| Color Dodge | `div ` | 1:1 |
| Add | `lddg` | Linear Dodge (Add) |
| Overlay | `over` | 1:1 |
| Soft Light | `sLit` | 1:1 |
| Hard Light | `hLit` | 1:1 |
| Vivid Light | `vLit` | 1:1 |
| Linear Light | `lLit` | 1:1 |
| Difference | `diff` | 1:1 |
| Exclusion | `smud` | 1:1 |
| Hue | `hue ` | 1:1 |
| Saturation | `sat ` | 1:1 |
| Color | `colr` | 1:1 |
| Luminosity | `lum ` | 1:1 |
| Behind | (PS-only mode "Behind") | PSD has limited support; export as Normal w/ warning |
| Clear | — | PSD has no equivalent (it's a tool state, not blend); export as Normal w/ warning |

### 9.3.3 Pixel data

- Color profile: PSD format suporta ICC profiles → Display P3 embedável.
- Bit depth: PSD suporta 8/16/32 — match canvas profile (RGBA8 → PSD 8-bit; RGBA16F → PSD 16-bit; nunca 32-bit float).
- Compression: PSD ZIP (preferred — smaller) ou RLE (faster, larger).

### 9.3.4 Compatibility flatten

PSD spec mandates "merged image data" — composição completa final. Painter genera e inclui (necessário para PSD readers que não suportam layers, como preview thumbs).

### 9.3.5 Warnings

PSD export retorna `Vec<ExportWarning>`:
- "Reference layer dropped — PSD has no equivalent"
- "Clipping mask emulated via clip group (visual result preserved)"
- "Blend mode 'Behind' approximated as Normal"
- "Wide gamut P3 colors converted; some saturation loss possible if recipient app reads as sRGB"

Dialog mostra warnings + opção "Save anyway" / "Cancel".

## 9.4 PSD import (lossy)

Read PSD via `psd` crate (existing Rust crate, mature).

Mapping reverse:
- Raster layer → Raster layer.
- Layer mask → Mask layer.
- Layer group → Group.
- Blend modes mapped via reverse table.

**Limitações:**
- Adjustment layers PSD → discarded (com warning).
- Smart Objects → flatten para Raster (com warning).
- Text layers PSD → flatten para Raster (Painter não tem text editing).
- Vector layers PSD → flatten para Raster.
- Style layer effects (drop shadow etc.) → flatten incluido no raster (com warning).

Workflow: usuária recebe PSD, importa para Painter para finishes, exporta PSD de volta → preserva a maioria das layers, mas adjustment layers e text não roundtrip.

## 9.5 Animation export

Detalhes em [10_animation_assist.md](10_animation_assist.md). Resumo dos formatos:

| Formato | Use case |
|---------|----------|
| **Animated GIF** | Web sharing, low-res, lossy palette 256 colors |
| **APNG** | Web sharing, higher quality, transparency preserved |
| **MP4** | Standard video; bigger but universal |
| **HEVC** | Higher compression; modern devices |
| **Animated WebP** | Modern web, transparent + small |

Frame rate, loop mode (Loop/Ping-Pong/One Shot), resolution configurável no export dialog.

## 9.6 Time-lapse export

Vide [04_canvas_guides.md](04_canvas_guides.md) §4.6. Export options:
- **Full length** — todo o video capturado durante painting sessions.
- **30 Seconds** — auto-edited highlights.

Output sempre MP4 ou HEVC (escolhido na canvas creation). Quality já fixa pelo canvas setting.

## 9.7 Brush export / import

### 9.7.1 Export brush

Single brush: `.ph2d-brush` (vide [01_brush_engine.md](01_brush_engine.md) §1.9).

Brushset: `.ph2d-brushset`.

Acesso: Brush Library → tap-and-hold brush → "Share..." → file dialog.

### 9.7.2 Import brush

Suporta:
- `.ph2d-brush` / `.ph2d-brushset` — native.
- `.brush` / `.brushset` (Procreate) — lossy mapper (§1.9.2).
- Krita `.kpp` — futuro pós-v1.0 (lossy).

Drag-and-drop em Brush Library importa. Após import, registry update + nova categoria criada se brushset.

### 9.7.3 CLI: `ph2d-painter-brush-cli`

**Decisão estrutural (2026-05-23, fechada em [README §11](README.md) #1):** brush format é **postcard binário**. CLI provê camada textual derivada para auditoria, edit à mão, e diff em PR review:

```bash
# Inspect — imprime TOML legível em stdout (sem texturas; refs por blake3)
cargo run -p ph2d-painter-brush-cli inspect my.ph2d-brush

# Edit — extrai my.toml + textures/{shape,grain}.png em pasta temp; abre $EDITOR
cargo run -p ph2d-painter-brush-cli edit my.ph2d-brush
# (após salvar e fechar, re-compila automaticamente ao .ph2d-brush original)

# Compile — empacota my.toml + textures → my.ph2d-brush (hash determinístico)
cargo run -p ph2d-painter-brush-cli compile my.toml --out my.ph2d-brush

# Diff — comparação textual de 2 brushes (útil em PR review e A/B testing)
cargo run -p ph2d-painter-brush-cli diff a.ph2d-brush b.ph2d-brush

# Validate — schema check (HR-14 version field, params em range, texturas presentes)
cargo run -p ph2d-painter-brush-cli validate my.ph2d-brush

# Convert — Procreate .brush → .ph2d-brush (lossy; vide §1.9.2)
cargo run -p ph2d-painter-brush-cli convert procreate.brush --out mine.ph2d-brush
```

**TOML schema (read-only side):** mesma struct `Brush` do [01 §1.3](01_brush_engine.md) em formato `toml::Value`, com texturas representadas como:
- `shape_source = { kind = "builtin", name = "round_hard" }`, OU
- `shape_source = { kind = "imported", blake3 = "abc123...", path = "./textures/shape.png" }` (em edit-mode).

**Determinismo:** `compile` produz bytes bit-identical para mesmo input (HR-6). Gateado em `cli_compile_deterministic` test — diff em 2 compiles consecutivos = 0 bytes.

**Casos de uso:**
1. **PR review:** quando um agente importa brushset Procreate, o reviewer roda `inspect` no resultado para auditar o que landou.
2. **Tweaking via texto:** power-user que quer ajustar 1 param (e.g., `spacing = 0.05`) abre `edit`, modifica, fecha — texto reabilita "git-friendly" iteration sem GUI.
3. **CI golden brushes:** os 12 default brushes ([01 §1.6](01_brush_engine.md)) viram fixtures `.toml` no repo; CI valida que `compile` deles produz `.ph2d-brush` com hash esperado.
4. **Migrations (HR-14):** quando schema bumpear a v2, `validate` reporta brushes legados; `compile --migrate` reescreve para v2.

## 9.8 Palette export / import

### 9.8.1 Export

- `.ph2d-palette` (native) — postcard.
- `.swatches` (Procreate) — binário simples (RGBA bytes em ordem).
- `.ase` (Adobe Swatch Exchange) — binário Adobe-compat.
- `.gpl` (GIMP palette) — text-based.

### 9.8.2 Import

Mesmos formatos. `.swatches` lossy (Procreate só guarda RGB; OKLCH lightness aproximada do RGB sRGB).

## 9.9 Asset sharing fluxo

PH2D não tem cloud sync nativo (decisão consciente §12.6). Workflow:
- Save canvas como `.ph2d-painter` em local de escolha (Files.app, FS, etc.).
- Os usuários transferem via Dropbox / Drive / Files.app / AirDrop / etc.
- Reabertura em outro device: `Ctrl+O` → file picker → load.

Reproducibility: blake3 garante "este `.ph2d-painter` é o mesmo arquivo em qualquer device" (CI test).

## 9.10 Export performance

| Operação | Canvas 4K, 20 layers, 100 strokes | Performance |
|----------|-----------------------------------|-------------|
| Save .ph2d-painter | ~40 MB compressed | 300–500ms off-thread |
| Export PNG flattened | ~16 MB | 200–400ms (composite + encode) |
| Export PSD multi-layer | ~80 MB | 800–1500ms off-thread (PSD encoding pesado) |
| Export TIFF multi-layer | ~120 MB | 600–1000ms off-thread |
| Export MP4 (anim 24 frames) | ~5 MB | 2–5s (encoder native) |

Todas as exports off-thread. Game thread mostra progress + permite cancel.

## 9.11 Export dialog UX

```
┌────────────────────────────────────────────┐
│ Export Canvas                              │
├────────────────────────────────────────────┤
│ Format:                                    │
│   ● PNG  ◯ JPEG  ◯ TIFF  ◯ WebP            │
│   ◯ PSD  ◯ .ph2d-painter  ◯ Animation     │
├────────────────────────────────────────────┤
│ Options:                                   │
│   Color profile: [sRGB ▼] (Display P3 / sRGB) │
│   Bit depth:     [8-bit ▼]                 │
│   Quality (JPEG/WebP): [90 ────●─────]     │
│   ☐ Include color profile (ICC embed)      │
│   ☐ Strip metadata (EXIF, XMP)             │
├────────────────────────────────────────────┤
│ Destination:                               │
│   [Choose...]                              │
├────────────────────────────────────────────┤
│         [ Cancel ]    [ Export ]           │
└────────────────────────────────────────────┘
```

## 9.12 MCP export (HR-10)

Export exposto via MCP para LLMs:
- `painter_export_png(canvas_id, options)` → returns blake3 + path.
- `painter_export_psd(canvas_id, options)`.
- `painter_export_animation(canvas_id, format, options)`.

Não-destrutivo (não exige confirmation token HR-11).

## 9.13 Gates de teste

| Gate | Crate | Valida |
|------|-------|--------|
| `export_native_roundtrip` | `ph2d-tool-painter` | Save `.ph2d-painter` → load → state idêntico |
| `export_png_color_profile_embed` | idem | PNG export em P3 canvas embed iCCP chunk válido |
| `export_psd_layer_mapping` | idem | Painter canvas → PSD → reopened in Photoshop test fixture preserva layers/blend |
| `export_psd_blend_mode_table` | idem | Cada blend mode mapeia para PSD code correto |
| `export_psd_warning_for_reference_layer` | idem | Reference layer presente → warning na export |
| `import_psd_basic` | idem | Fixture PSD com 3 layers + masks → carrega corretamente |
| `import_psd_text_warning` | idem | PSD com text layer → flattened com warning |
| `import_procreate_brushset_smoke` | idem | Fixture .brushset → registry gets brushes |
| `palette_swatches_roundtrip` | `ph2d-painter-color` | .swatches save → load → palette idêntica |
| `export_anim_gif_loops` | `ph2d-painter-anim` | GIF generated tem loop count correto |
| `export_blake3_deterministic` | idem | Mesma canvas data → mesmo `.ph2d-painter` hash |

**Continua em:** [10_animation_assist.md](10_animation_assist.md) — Animation Assist (timeline frame-based).
