# Comentários históricos de `ph2d-app-painter`

Comentários históricos tirados do código de `ph2d-app-painter` (catraca the_shell_only_shrinks, 10/11); o código guarda a lei, aqui fica o porquê.

## `crates/ph2d-app-painter/src/painter_bridge.rs`

### cabeçalho do ficheiro (linhas 1–2 no original)

```text
// ⚠️ O `// ph2d-loc-cap:` que estava aqui SAIU: ele é **inerte** em `crates/` — porquê, e o corte
// que o substituiu, no header do irmão `painter_bridge_upload`.
```

### //! ## Sprite suppression (linhas 33–44 no original)

```text
//! ## Sprite suppression (replaces the Vello overlay)
//!
//! The live preview is the LAYER COMPOSITE (base layer = the sprite image
//! itself). It no longer paints as a Vello overlay ON TOP of the
//! still-rendered sprite (that duplicated the image: lowering the base
//! layer's opacity faded only the overlay, revealing the full-opacity sprite
//! underneath — Enio smoke 2026-06-01). Instead, mirroring BgRemoval, the
//! bridge uploads the premultiplied composite into an `IndividualTextureStore`
//! slot ([`PainterPreviewGpu`]); the next frame's `sim_extract` emits a
//! `PreviewOverride` that SUPPRESSES the source sprite and samples this
//! texture in its place. So the composite (incl. base-layer opacity) IS the
//! sprite, in-place, through the same sprite shader as Apply.
```

### const PREVIEW_DUMP_MAX_FRAMES (linhas 60–61 no original)

```text
/// Frame cap for the `PH2D_PREVIEW_DUMP` diagnostic trap (BUGS_painter.md #11). A stroke is tens of
/// frames, so this holds several gestures while keeping a long session from filling the disk.
```

### nota de mudança (queries) (linhas 64–65 no original)

```text
// `painter_has_unflushed_strokes` + `apply_layer_reparent` (tool-concrete downcast
// queries) moved to `painter_bridge_queries.rs` (HR-18 file-LOC cap).
```

### dispatch(present) (linhas 76–78 no original)

```text
    // ⭐⭐⭐ **O mundo de APRESENTAÇÃO (read-only)** — onde a malha posada de uma arte presa ao
    // esqueleto vive. O chrome do canvas desenha-se por cima dela, e o ponteiro que agarra as alças
    // já a consulta desde 2026-09-14: sem isto aqui, as duas metades do mesmo controlo discordam.
```

### dispatch(camera) (linhas 83–85 no original)

```text
    // The composite preview goes through the sprite pipeline (PreviewOverride),
    // not a Vello overlay — but these drive the on-canvas **brush cursor ring**
    // (a UI hint drawn into the overlay scene). See module header + the ring below.
```

### dispatch(painter_gpu_preview) (linhas 96–97 no original)

```text
    // GPU live-preview session (compositor + premul blit). `None` until the
    // first GPU-representable frame lazily builds it (ADR-0045 Phase 3 step 2).
```

### dispatch(pointer_held) (linhas 103–104 no original)

```text
    // `true` enquanto um botão de ponteiro está preso — o sinal de *"há um gesto em voo"* que o
    // `post_frame_undo` já usa. Aqui ele fecha o ciclo do rascunho de figura: ver `set_shape_draft_hold`.
```

### dispatch(fill_drag_armed) (linhas 106–109 no original)

```text
    // ⚠️ **O arrasto de cor do balde, RESOLVIDO pela shell** (W2 Fase D). O estado vive num
    // `thread_local` do `input_dispatch::fill_drag`, que é a camada de entrada — e era a última
    // aresta deste grupo de ficheiros para a `shells/desktop`. ⛔ Não é uma porta nova no
    // `AppHost`: escrito em TIPOS, o que a função precisa é de um `bool`.
```

### dispatch(read_source) (linhas 111–113 no original)

```text
    // ⚠️ **O LEITOR de pixels de um sprite, vindo da shell.** Devolve já em alfa DIRECTO
    // (`into_straight`), que é o que o `bind_document` quer. ⛔ Nenhum tipo do funil de
    // textura da shell (`SourceRead`) atravessa esta fronteira: o que chega é `Vec<u8>` + dims.
```

### dispatch(note_preview_px) (linhas 121–124 no original)

```text
    // ⚠️ **O contador do perfilador de quadro, entregue pela shell.** Ele vive no LAÇO
    // (`render_loop::note_preview_px`, atrás do `PH2D_FLUID_PROFILE`) e era a única aresta
    // `super::` deste ficheiro para fora da família. ⛔⛔ E era **invisível à régua do fecho**,
    // que não resolve `super::` — foi preciso varrer à mão para a achar.
```

### dispatch — PH2D_PAINT_PERF (linhas 127–131 no original)

```text
    // Diagnostic TRAP for the mask-path FPS report (2026-07-24): `PH2D_PAINT_PERF=1` logs, per frame
    // the painter is active, WHICH producer owned the preview + WHICH drain path ran + the phase
    // timings + the canvas dims. This is what tells the difference between "the GPU producer's partial
    // upload fired" and "the mask stroke fell to a full CPU composite every frame" — the two the
    // headless proxy cannot tell apart. Zero cost when the var is unset.
```

### dispatch — commit_requested (linhas 147–149 no original)

```text
    // W2.T2.5: consume the Cmd/Ctrl+Enter commit flag (set in
    // `handle_editor_key`). Taken unconditionally so it can't leak to a
    // later painter activation; only acted on in the downcast block below.
```

### dispatch — undo/redo (linhas 151–152 no original)

```text
    // W2.T2.2: same unconditional-take discipline for the stroke
    // undo/redo flags (Cmd+Z / Cmd+Shift+Z while Painter is active).
```

### dispatch — bind_document (linhas 174–176 no original)

```text
        // ⚠️ O trio de leitura (`sim`, `asset_db`, `atlas_asset_map`) é CAPTURADO aqui, e não
        // passado adiante: o bind não os usa para mais nada, e capturá-los tira três argumentos
        // de uma assinatura que a própria ferramenta já dizia ser larga demais.
```

### dispatch — panel_visibility (linhas 188–193 no original)

```text
    // Audit T1.5 round 1 B-H2: NO ghost `panel_visibility` insert. Painter
    // has no docked panel in T1.5 (sidebar lands W2 via
    // `ph2d-panel-painter`); inserting into the BTreeMap every frame just
    // to flip an unread bit risks colliding with the W2 sidebar's own
    // panel_visibility key. The Painter pill's pressed state is computed
    // directly off `tools.active().id()` in the topbar paint pass.
```

### dispatch — drain (FAST PATH) (linhas 197–211 no original)

```text
    // ── Drain current_preview (FAST PATH) + capture Apply ─────────────────
    //
    // **R4-LG-1 fix:** bypass `drive_preview_cache` (which does a 16 MB
    // `pixels.to_vec()` per dirty drain — at 60 fps painting that's ~960
    // MB/s of allocator churn). Painter-specific downcast lets us pull
    // the canvas as a 1-atomic-inc `Arc<Vec<u8>>` clone via
    // `take_preview_arc()`. Net: per-stroke painting touches the
    // allocator ONCE per `Arc::make_mut` cycle (i.e., once per preview-
    // drain frame), not every pointer event.
    //
    // `drive_pending_commit` stays on the generic `&mut dyn RasterEditTool`
    // path — it's only called once per Apply, not per frame.
    // B.1: carries the partial dirty bbox from the preview drain to the GPU
    // upload below (frame-local — same function scope, so no `PreviewCache`
    // field change needed). `Some` = upload only this sub-rect; `None` = full.
```

### dispatch — cache_version (linhas 213–218 no original)

```text
    // The tool's monotonic canvas CONTENT version this frame — the shell keys its GPU-slot upload on
    // this instead of the drained `Arc`'s pointer. Keying on the pointer meant the shell had to HOLD a
    // clone of the live canvas so `Arc::make_mut` would hand back a fresh pointer on a change; holding it
    // is exactly what made every `stamp_dabs` copy the whole canvas per move (measured: 0.34 ms/move @
    // 2048², 10 ms/move @ 4096², flat across brush size — the CPU-bound FPS drop). Captured whether the
    // drain produced a frame or not (an idle frame reads the SAME version → the plan Skips).
```

### dispatch — ph_ov_* (linhas 226–227 no original)

```text
    // …and the OVERLAY phase split by CALL, because 3,9 ms shared by three of them names none of
    // them (Enio, 2026-07-25, 4096²: o preview zerou e o custo mudou de lugar).
```

### dispatch — set_shape_draft_hold (linhas 241–246 no original)

```text
        // **O SOLTAR do arrasto de knob** (`shape_draft`, 2º fio): o `set_shape_draft_hold(true)` é
        // publicado no instante do edit de painel (`render_loop::mod`, o drain de `ToolPanelEvent`),
        // mas quando o artista SOLTA não chega evento nenhum — então a queda mora aqui, no passe que
        // roda todo quadro, e é ela que ASSENTA a figura de volta na tela. Chamar com o valor vivo nos
        // dois sítios não é duas respostas: é a mesma pergunta feita no momento em que ela decide algo
        // (antes do re-carimbo) e uma vez por quadro (para o soltar ser visto).
```

### dispatch — brush texture image (linhas 261–262 no original)

```text
        // The user picked the Image texture kind → open a native file picker, decode + install the
        // luminance as the brush texture (the pure engine has no file I/O). Cancel/failure reverts.
```

### dispatch — brush shape image (linhas 266–267 no original)

```text
        // Same path for the brush **Shape** (silhouette) slot when the user picks Image in the Shape
        // dropdown. Cancel/failure simply leaves the silhouette as the falloff (nothing to revert).
```

### dispatch — presets (linhas 279–280 no original)

```text
        // Os presets dos quatro meios (W1, L1): a biblioteca abre com o Painter, o pedido do painel é
        // atendido aqui (o disco é da ponte) e a vista é publicada.
```

### dispatch — try_drive (linhas 292–294 no original)

```text
        // GPU-vs-CPU preview decision (ADR-0045 Phase 3 step 2): representable
        // stack → GPU composite (fast slider drags), else CPU `take_preview_arc`
        // below. Both end in `painter_preview_gpu`. See `try_drive`.
```

### dispatch — note_preview_px (linhas 314–322 no original)

```text
            // O DIVISOR do `painter-dispatch` — a area publicada e' o que o
            // gather + premultiply + upload adiante tem de mover, e sem ela o
            // numero do log nao distingue um retangulo grande de um pequeno.
            // ⚠️ A bbox e' `(x, y, w, h)` -- NAO `(x0, y0, x1, y1)`, e o
            // doc-comment do `take_preview_upload_bbox` o diz. A 1a versao
            // subtraiu como se fossem cantos, o que da `(w - x) * (h - y)` e
            // SATURA EM ZERO para todo retangulo longe da origem: o log do
            // smoke veio `0.00 M px publicados em 80 quadros` -- o sitio de
            // contagem disparando e o valor sendo lixo.
```

### dispatch — PH2D_PREVIEW_DUMP (linhas 327–332 no original)

```text
            // Diagnostic TRAP for the per-layer-colour "rectangle residue" (BUGS_painter.md #11 — OPEN,
            // intermittent). `PH2D_PREVIEW_DUMP=<dir>` writes each frame's CPU composite (the exact bytes
            // about to be uploaded, BEFORE any overlay) to `<dir>/preview_NNNN.png`, capped so a long
            // session can't fill the disk. If the rectangle shows in these PNGs it is the composite; if
            // they are clean while the artifact is on screen, it is an on-top overlay or the GPU producer
            // (neither of which this lane touches). Zero cost when the var is unset.
```

### dispatch — own_preview_buffer (linhas 342–349 no original)

```text
            // Own the preview buffer: patch the dirty region out of the drained composite into the
            // shell's OWN buffer (in-place — the shell is its sole owner), then let `drained` drop. The
            // old code stashed `drained` (a clone of the tool's live `canvas_rgba` on the trivial path,
            // of its `composited` cache otherwise) and held it across the frame, so the tool's next
            // `Arc::make_mut` saw a second owner and copied the WHOLE plane per move. Patching a
            // shell-owned mirror is O(dirty bbox); the seed (no prior buffer / dims-or-entity change /
            // full recompose) copies the composite once. Either way the tool is left the SOLE owner of
            // its canvas, so its next stamp writes in place.
```

### dispatch — canvas_version (linhas 365–367 no original)

```text
        // Capture the content version for the upload plan below — bumped by the drain above on a
        // change, unchanged on an idle frame (so the plan Skips). Read here, inside the downcast, in
        // both the drained and idle cases.
```

### dispatch — PH2D_PREVIEW_DIAG (linhas 385–392 no original)

```text
        // Diagnostic TRAP, half 1 (BUGS_painter.md #11 — OPEN): `PH2D_PREVIEW_DIAG=1` logs which producer
        // owns the preview slot each frame + the CPU partial-upload bbox. This is what proved the per-layer
        // shape edits run on the CPU lane (`gpu_owns=false`) while a slider drag hands the slot to the GPU
        // producer — the CPU↔GPU handoff the headless harness cannot reach. Zero cost when unset.
        // ⚠️ Logs on CHANGE, not per frame. Per-frame it emitted 60 lines a second and the interesting
        // ones — a producer flip, a bbox going `None` — scrolled off before anyone could read them
        // (Enio, 2026-07-25: *"Muitos logs. tente diminuir"*). What this trap exists to show is a
        // TRANSITION, and a line per frame is the one format that hides transitions.
```

### dispatch — m_p (linhas 402–405 no original)

```text
        // ⚠️ **DUAS declarações, e não um `allow(unused_mut)`.** Só o bloco do painel de camadas
        // reatribui este relógio, logo sem a feature ele nunca muta. Silenciar o diagnóstico seria
        // armengo mesmo com a ferramenta defeituosa — e aqui a ferramenta tem razão: nesta build a
        // variável **não** precisa de ser mutável.
```

### dispatch — layers snapshot publish (linhas 416–430 no original)

```text
        // (B.5 perf) Layers snapshot publish pro docked layers panel.
        // The panel paints a row per layer off this clone. **Gated on
        // `layers_revision()`:** the `LayerStack` is metadata-only, but the clone
        // (N rows × name `String`) ran EVERY frame Painter was active — including
        // every mouse-move during a layer drag, which made the reparent feel
        // sluggish (Enio 2026-06-02 "muito lenta"). `layers_revision` bumps only
        // on structural/metadata edits (`invalidate_composite` + `set_source`),
        // NOT strokes and NOT cursor moves. So during an in-flight drag the
        // structure is stable → we skip the clone entirely (the panel keeps its last snapshot and
        // reads the live `panel_row_drag()` cursor). First activation always publishes (sentinel
        // `u64::MAX`); `layers_revision` is monotonic for the app lifetime (never a stale skip).
        // ⚠️ **A reatribuição do relógio vive DENTRO do `cfg`, e é obrigatório que viva** (W2 Fase
        // D): só este bloco o volta a ler, e na shell a feature era `default` — logo o código morto
        // sem ela nunca aparecia. Uma crate própria compila-se também SEM a feature, e aí um
        // `m_p = …` fora daqui é uma atribuição que ninguém lê.
```

### dispatch — set_current_selection (linhas 442–447 no original)

```text
            // (W3 multi-select) Publish the selection set every frame — a tiny
            // BTreeSet (≤ HARD_CAP_LAYERS u64s). NOT gated on `layers_revision`:
            // a plain re-click that collapses a multi-selection onto the
            // already-active layer changes the selection WITHOUT a structural
            // edit, so a revision gate would miss it. The panel reads this for
            // the multi-row highlight (active = strong outline, others = wash).
```

### dispatch — brush snapshot (linhas 453–457 no original)

```text
            // (Brush UI) Publish the active brush snapshot every frame — a tiny
            // Copy struct (size/colour/blend). The panel's Brush section reads it
            // to position the Size/RGB sliders + the blend chip. Not revision-
            // gated: brush edits don't bump `layers_revision`, and the cost is a
            // few floats.
```

### dispatch — wet tuning (linhas 463–467 no original)

```text
            // Wet Tuning side panel (doc 22): same snapshot, same cadence — and
            // its visibility MIRRORS the tool's authored Tuning checkbox
            // (painter active + wet armed + tuning_open). Edge-triggered z
            // bump: `WET_TUNING_PANEL` rides the z fallback list, but bumping
            // on open keeps it above its dock neighbours.
```

### dispatch — tool rail (linhas 480–485 no original)

```text
            // (Tool rail) The rail radio FOLLOWS the painter's mode rather than remembering which button
            // was clicked. The Impasto section's unified TOOL list can change the mode too (picking
            // "Chisel" there enters Sculpt), and a rail that only learned about its own clicks would go
            // on highlighting "Brush" while the artist sculpts — two answers to "which tool am I
            // holding?", with the wrong one on screen. Self-gating: writes only when it actually moved.
            // O meio decide que ferramentas o trilho oferece (as da água no lugar do Smear/Blur do Digital).
```

### dispatch — eyedropper (linhas 491–494 no original)

```text
            // (Eyedropper) When the on-canvas colour pick completes (armed → not armed), snap the tool
            // rail radio back to Brush — the pick is a MOMENTARY tool, so its button stops looking
            // selected once a colour is sampled. Edge-detected via a static so arming the pick (which
            // sets armed=true) never triggers an immediate reset (only the true→false pick does).
```

### dispatch — shapes rail (linhas 505–508 no original)

```text
            // (Shapes rail) Keep the tool-rail's active button in sync with the stroke method: choosing a
            // shape in the Brush panel's Method dropdown moves the rail to the matching Shapes button
            // (a non-shape method leaves the radio alone — returning to Brush is the Brush button's job).
            // Edge-detected so it fires only on a real method change, not every frame.
```

### dispatch — refresh_shape_source_if_changed (linhas 525–528 no original)

```text
            // (Shape source linkage) If the multi-layer Shape was captured from the ACTIVE sprite, re-capture
            // it when that sprite changed (paint / opacity / visibility / undo) — keeping the per-layer
            // colours. Cheap revision compare per frame; re-captures only on a change. Before the preview
            // refresh so the preview reflects the re-captured Shape the same frame.
```

### dispatch — shape colour preview (linhas 532–535 no original)

```text
            // (Shape preview, Per-Layer Color) Publish the multi-layer COLOURED composite so the Shape
            // preview shows the per-layer colours — the colours need the per-layer pixels, which only the
            // tool has. The tool re-bakes the composite ONLY when the Shape appearance changes (a cheap
            // key-compare per frame), so we publish (and pay the bake) on an edit, never per frame.
```

### dispatch — refresh_shape_grab_tol (linhas 548–550 no original)

```text
        // Keep the shape-editor grab tolerance in sync with the live camera every frame (not just on a
        // painter Down/Move/Up), so the on-canvas handles are drawn where they'll be grabbed — no snap
        // when the first grab after a zoom refreshes the tol.
```

### dispatch — draw_repeat_image (linhas 560–564 no original)

```text
        // Repeat Image FIRST: the 3×3 tile preview is canvas CONTENT (the composite at the 8 neighbour
        // positions), so it must sit UNDER all editing chrome. Drawn after the overlays it covered any
        // chrome extending past the sprite border — a shape crossing the seam lost its editor overlay
        // beyond the edge, and the brush ring / marching ants vanished over the neighbour tiles
        // (the "overlay stops at the seam" bug, Enio 2026-07-11).
```

### dispatch — GPU lifecycle (linhas 619–631 no original)

```text
    // Mirror of `bgremoval_preview`: upload the premultiplied composite into a
    // transient `IndividualTextureStore` slot; NEXT frame's `sim_extract` reads
    // `painter_preview_gpu` to emit a `PreviewOverride` that SUPPRESSES the
    // source sprite and samples THIS texture in its place. The composite is
    // STRAIGHT sRGB8 (the canvas / `take_preview_arc`); byte-space
    // `premultiply_rgba8` matches EXACTLY what Apply's
    // `SpriteImage::into_premultiplied` produces, so the live preview is
    // byte-for-byte identical to the committed result on the same
    // `Rgba8UnormSrgb` + premul-blend sprite shader (no Vello gamma/blend
    // divergence, no image duplication). 1-frame lag is imperceptible.
    //
    // On a GPU-owned frame the GPU producer fills the slot; hide the CPU cache
    // from this block so it neither re-uploads nor releases that slot.
```

### dispatch — record_dispatch (linhas 646–647 no original)

```text
        // Record this frame's dispatch info + sub-phase split; the frame timer (`run_render_frame`)
        // pairs it with the whole-frame time and the aggregator prints ONE summary per window.
```

### dispatch — fold_ms (linhas 660–662 no original)

```text
            // ⚠️ Preenchidos pelo `record_dispatch` a partir do acumulador: o fold é anotado de DENTRO
            // do `try_drive` (que roda na fase `preview` acima), onde a janela dele é resolvida — aqui
            // não há como saber se ela foi um retângulo ou a tela.
```

## `crates/ph2d-app-painter/src/painter_gpu_preview.rs`

### //! ## Why it wins (linhas 13–20 no original)

```text
//! ## Why it wins
//!
//! An adjustment-slider drag changes only `gpu_params` (no layer pixels), so the
//! compositor keeps every layer slice cached (their pixel versions are stable —
//! see `PainterTool::layer_pixel_versions`) and re-runs just the compute
//! (~1.7 ms @1024² vs ~55 ms for the CPU HSB recompose). The straight→premul
//! handoff is the one piece of fresh render work — see [`ph2d_render::premul`]
//! and `HANDOFF_painter_gpu_preview_coord.md`.
```

### struct PainterGpuPreview (linhas 33–38 no original)

```text
/// Per-session GPU state for the Painter live preview: the layer compositor, the
/// straight→premultiplied blit, and a cached [`GpuContext`] handle (cheap
/// `Arc`-backed clone). Created lazily on the first GPU-representable frame and
/// kept for the tool session: the compositor's own slice cache invalidates on a
/// canvas-dims change, and the tool's monotonic pixel versions handle layer-key
/// reuse across sources, so the same instance stays correct across edits.
```

### fn prewarm (linhas 51–65 no original)

```text
/// **Constrói a sessão ANTES do primeiro traço precisar dela** (doc 28 §4.8).
///
/// ⚠️ O `get_or_insert_with` do [`drive`] a criava no primeiro frame que precisasse do preview GPU — que
/// é o **primeiro traço do artista** — e as três peças dela **COMPILAM shaders**. Medido na RTX
/// (`ph2d-render/tests/it/measure_first_stroke_pipelines.rs`, driver já quente): `LayerCompositor` **6,01
/// ms** + `ImpastoLightPass` **16,30** + `PreviewPremul` **5,70** = **28,01 ms**, ou seja quase dois
/// quadros de 60 fps, pagos exatamente no gesto em que o artista está esperando a tinta aparecer.
///
/// É custo ÚNICO e por isso invisível a toda sonda do tool (elas medem o `PainterTool`, que não tem
/// GPU) — e invisível também a uma sonda que meça o SEGUNDO traço, que foi o que me fez chamar o
/// problema de *"o delay de todo pen-down"* quando o Enio o chamava de *"o delay do PRIMEIRO traço"*.
///
/// O gatilho é o **bind do documento**: o artista escolhe o sprite, depois leva o mouse até a tela e
/// clica — há tempo HUMANO nesse vão, e é ali que os 28 ms cabem sem ninguém ver. Fazê-lo no boot
/// cobraria os mesmos 28 ms de quem nunca pinta; fazê-lo por frame seria pior que o lazy.
```

### fn prewarm — corpo (linhas 79–102 no original)

```text
    // ⚠️ E os RECURSOS do COMPOSITOR e do premul, que nascem do TAMANHO DO CANVAS e que só a 1ª
    // execução aloca — medido em `ph2d-render/tests/it/measure_first_stroke_pipelines.rs`.
    //
    // ⚠️ **O passe de LUZ fica de fora, e isto é uma limitação NOMEADA, não um descuido.** Uma pilha
    // recém-bindada não tem relevo, então `impasto_gpu_planes_in` recusa, a luz não roda, e nem as
    // texturas dela nem a semeadura (`planes_seeded`) acontecem aqui. Consequência medida: o **primeiro
    // traço com relevo dobra o CANVAS INTEIRO** em vez da própria região — 14,55 ms contra 0,38 a
    // 4096² depois de o fold passar a caminhar por linhas (`measure_the_fold_the_product_runs`), mais
    // a alocação das 4 planes + a out (~218 MB a 4096², 13,21 ms medidos).
    //
    // Fechá-lo é uma DECISÃO DE PRODUTO, não uma correção mecânica: qualquer pré-aquecimento da luz
    // cobra VRAM canvas-sized de TODO bind, inclusive de quem nunca liga o impasto — o mesmo argumento
    // que já mantém este pré-aquecimento fora do boot, num tamanho dez vezes maior. O número que
    // decide é o que o `PH2D_PAINT_PERF` reportar no frame do primeiro traço agora que o fold caiu de
    // 201,5 para 14,55 ms.
    //
    // ⚠️ **A `gpu_eligible` NÃO pode ser consultada aqui, e é isso que fazia o custo cair no 1º traço:**
    // uma pilha recém-bindada é TRIVIAL e sem relevo, então ela recusa, nada é alocado, e o primeiro
    // traço com relevo — que a torna não-trivial — aloca tudo de uma vez. O pré-aquecimento passa por
    // cima dela DE PROPÓSITO e cozinha um frame que ninguém vê, só para as texturas existirem.
    //
    // O slot do renderer é liberado no frame seguinte (a pilha ainda é trivial, então o produtor CPU
    // reassume) e isso está certo: o que precisava sobreviver são as texturas INTERNAS dos passes, que
    // moram no `session_slot` e que o `release_slot` não toca.
```

### fn try_drive (linhas 159–179 no original)

```text
/// Decide GPU-vs-CPU for this frame and, when GPU, recomposite into the preview
/// slot. Returns `true` iff the GPU producer owns the slot (the stack is
/// GPU-representable) — the caller then gates its CPU upload block off. On the
/// CPU branch (no selection, or `flatten_for_gpu` returns `None` — its module doc
/// owns the refusal list, so this one cannot go stale) this is a no-op and the
/// caller runs `take_preview_arc` + the CPU upload path.
///
/// Drains the preview-dirty flag WITHOUT a CPU composite, so it recomposites on
/// the GPU only when the preview actually changed; an idle representable stack
/// keeps its slot. Also drains the tracked dirty-bbox so it can't leak into a
/// later CPU frame.
///
/// A TRIVIAL stack (single visible opaque Normal raster — the composite IS
/// `canvas_rgba`) stays on the CPU path on purpose: the CPU producer there is
/// zero-composite (`take_preview_arc` hands back the canvas `Arc`) with the B.1
/// partial dirty-bbox upload, strictly cheaper than a full-slice re-upload +
/// composite + premul + copy per stroke frame.
///
/// **Unless the canvas carries relief**, which makes the composite something other
/// than `canvas_rgba` and takes the zero-composite premise away — see
/// [`gpu_eligible`].
```

### fn try_drive — seed (linhas 199–201 no original)

```text
        // CPU/hover path: a full recomposite (no live wet envelope) — seed the whole
        // canvas (`seed_full = true`, full slot copy), byte-identical to the old
        // `Region::full` composite.
```

### fn gpu_eligible (linhas 221–224 no original)

```text
/// The GPU-vs-CPU gate, pure half (headless-testable): `Some(ops, adj_luts)` iff
/// this frame's preview should composite on the GPU. The trivial-stack check
/// runs FIRST — `flatten_for_gpu` happily represents a single plain raster, but
/// the CPU path is strictly better there (see [`try_drive`]).
```

### fn gpu_eligible — trivial (linhas 226–237 no original)

```text
    // The trivial bow-out rests on the CPU path being ZERO-composite there — and relief falsifies that
    // premise. `take_preview_arc` already refuses the fast lane when `impasto_visible()`, because that
    // lane hands back the raw `canvas_rgba` Arc and the light may never write into the artist's own
    // pixels. So a sculpted single-layer document — the most ordinary way to use Impasto — pays a FULL
    // CPU composite plus a full CPU light on every dirty frame, and the GPU is strictly better.
    //
    // The two lanes have to agree about this or the work goes to whichever one is worse: the guard here
    // mirrors `runtime.rs`'s, deliberately and visibly.
    //
    // ⚠️ **E o papel não tira o documento da pista de CPU** (2026-10-06): com uma camada lisa sobre o
    // papel a CPU recompõe só a região suja (o papel incluído) — medido `1,85–4,5×` mais rápido que esta
    // pista, que refaz a tela inteira. A GPU compõe sobre o papel onde já seria ela a produzir.
```

### fn gpu_eligible — repeat_image (linhas 241–246 no original)

```text
    // Repeat Image draws the 3×3 tile preview from the CPU composite (`PainterPreview` — the GPU
    // slot has no CPU bytes to blit), and when the GPU owns the slot the bridge clears that cache:
    // the 8 neighbour tiles silently vanish (Enio 2026-07-20, "em impasto Tiling as imagens
    // repetidas desaparecem" — impasto because the GPU light pass made relief documents
    // GPU-eligible, but any GPU-owned stack loses the tiles the same way). While the artist is
    // LOOKING at the tiling preview, the CPU lane must produce.
```

### fn gpu_eligible — mask_scratch_active (linhas 250–261 no original)

```text
    // A PROTEÇÃO é chrome que mora no COMPOSITE: `apply_mask_overlay` tinge o composto por-pixel pela
    // cobertura do scratch, na `runtime.rs`, e o produtor de GPU **não a desenha** — ele composita e
    // ilumina, e o overlay não é uma `LayerOp`. Enquanto não havia relevo isso não aparecia: um
    // documento com scratch vivo é quase sempre trivial, e a bow-out acima o mandava para a CPU.
    // Com o relevo o documento vira elegível (a luz na GPU, 2026-07-18) e o overlay SOME — o artista
    // pinta sob uma proteção que ele não vê (Enio, 2026-08-07: *"a máscara funcionou, mas fica
    // invisível ao usar impasto"*).
    //
    // ⚠️ **A cura é a mesma do Repeat Image acima, e pelo mesmo motivo:** quem desenha aquele chrome é
    // a pista de CPU, então enquanto o artista está OLHANDO para ele a CPU produz. A alternativa —
    // portar o tinte para o shader — seria uma segunda resposta a *"como uma proteção se parece"*,
    // divergindo no único lugar onde ninguém lê um número.
```

### fn gpu_eligible — papel_atravessa_vidro (linhas 265–267 no original)

```text
    // ⚠️ **O VIDRO da aguada sob papel de cor** (doc 48): o papel atravessa cada canal pelo alfa dele,
    // e o compositor da GPU só conhece um alfa por texel — compô-lo ali mostraria a aguada de antes
    // (a que soma luz ao papel). Enquanto isto for verdade a CPU produz, como com a proteção acima.
```

### fn drive (linhas 274–278 no original)

```text
/// Composite `ops` on the GPU, premultiply, and copy into the preview slot,
/// pointing `painter_preview_gpu` at it. The slot is (re)acquired empty when
/// missing / resized. On any GPU error the slot is released and a toast queued
/// (the CPU path can take over next frame). Returns `true` iff a preview texture
/// is now live in the slot.
```

### fn drive — copy (linhas 334–337 no original)

```text
    // 4) Ensure a preview slot of the right size, then COPY the premultiplied
    //    result into it (rgba8unorm → Rgba8UnormSrgb is copy-compatible). The slot
    //    persists across the stroke, so a seed frame copies the WHOLE canvas and
    //    later frames copy only the wet envelope rect on top of it.
```

### fn compose_light_premul (linhas 364–367 no original)

```text
/// Extracted so the end-to-end parity gate can drive the REAL chain
/// (`the_gpu_producer_shows_what_the_cpu_producer_shows`) rather than a mirror of it. A gate that
/// re-assembled these three calls itself would go on passing after this function stopped doing them in
/// that order — which is exactly the class of bug it exists to catch.
```

### fn compose_light_premul — Errors (linhas 371–372 no original)

```text
/// A description of which stage refused. Every one of them is a reason to hand the frame back to the CPU
/// producer, which can always draw it.
```

### fn compose_light_premul — 2) (linhas 404–415 no original)

```text
    // 2) Impasto: light the freshly-composited region. Same place in the chain as the CPU pass, and
    //    for the same reason — lighting is NOT idempotent, so it must see pixels that were composited
    //    from the layers this frame, never pixels that were already lit. The compositor rewrites
    //    `region` from scratch on every dispatch, so what it feeds here is always fresh.
    //
    //    `None` planes is the ordinary case, not an error: no relief, the pass switched off, or every
    //    lamp dark. Those are the CPU pass's own bails, and they leave the composite untouched.
    //
    //    The match YIELDS the finished texture rather than setting a flag the next step re-reads. That
    //    is not style: "which texture holds the finished canvas" must have exactly one answer, and a
    //    second derivation of it that got the condition backwards would ship an unlit painting with
    //    every gate still green — the failure has no symptom except the artist seeing flat paint.
```

### fn compose_light_premul — plane_win (linhas 419–427 no original)

```text
    // **Which window the fold walks.** Two questions, each answered by whoever owns the fact:
    //
    // * the TOOL knows whether this frame's change was confined to a rect (`preview_gpu_region` is
    //   `Some` only then — a structural edit routes through `invalidate_composite`, which drops it);
    // * the PASS knows whether its persistent plane textures have ever held the whole painting, because
    //   it owns them and a resize rebuilt them.
    //
    // Either answering "no" means fold the canvas. Measured, at 4096²: 202 ms a frame whole against
    // 2,8 ms for a 512² window, and the walk is the entire cost (`measure_what_the_fold_is_made_of`).
```

### fn compose_light_premul — m_fold (linhas 432–434 no original)

```text
    // O DIVISOR: este fold é a única metade de CPU do caminho GPU, e a única cujo custo depende de a
    // janela ser um retângulo ou a tela. O relógio SOZINHO não decide a cura (device lento × canvas
    // dobrado), então a janela viaja junto com ele.
```

### fn compose_light_premul — region (linhas 458–468 no original)

```text
                // The WHOLE canvas, even when the composite refreshed only a region — and the asymmetry
                // is the point. The light owns a SECOND persistent canvas, so its freshness cannot be
                // inherited from whichever rectangle the compositor happened to touch: one frame with
                // the relief hidden (planes `None`, the compositor refreshes a region, the light never
                // runs) and the next partial lit frame would carry pixels from before that update, in
                // the corner nobody was looking at.
                //
                // It costs nothing today — every dispatch here is already full-canvas — and it stays
                // correct if a partial lane is ever added, merely not optimal, which is the safe
                // direction to be wrong in. It is also always sound: the pass reads the compositor's
                // output (valid everywhere, unlit) and writes its own, so it never re-lights a pixel.
```

### fn compose_light_premul — plane_region (linhas 470–472 no original)

```text
                // …but the PLANES are only the window the fold walked. The textures persist, so the rest
                // of the canvas keeps the last upload — which is exactly why the full-canvas dispatch
                // above stays correct while the CPU-side fold shrinks.
```

### fn compose_light_premul — light.run (linhas 489–490 no original)

```text
            // Falling through UNLIT would composite the layers perfectly and drop the shading on the
            // floor: the artist would sculpt and see nothing, with no error anywhere.
```

### fn compose_light_premul — 3) (linhas 503–506 no original)

```text
    // 3) Premultiply the straight `rgba8unorm` output into a COPY_SRC texture
    //    (the sprite preview slot samples PREMULTIPLIED — see `ph2d_render::premul`).
    //    Full-canvas: a single cheap pass; out-of-region texels are the unchanged
    //    (still-valid) prior composite, so re-premultiplying them is a no-op.
```

### fn ensure_slot (linhas 516–521 no original)

```text
/// Ensure `painter_preview_gpu` holds a slot sized `width × height`; reuse the
/// existing slot when the dims match (the copy overwrites its contents),
/// otherwise release the old slot and acquire a fresh EMPTY one. Returns the
/// slot's `texture_id`. `arc_token` is `0` — the GPU producer has no CPU content
/// version, and the next CPU frame's `arc_token != cache_version` test then
/// correctly forces a full re-upload on a GPU→CPU transition.
```

### fn release_slot (linhas 556–559 no original)

```text
/// Release the preview slot (if any) and clear the GPU cache — on a GPU error so
/// the next frame re-acquires from scratch (or the CPU path takes over). Mirror
/// of `painter_bridge::release_preview_texture`, kept local so the two preview
/// producers don't reach into each other.
```

## `crates/ph2d-app-painter/src/paint_perf.rs`

### //! cabeçalho (linhas 3–9 no original)

```text
//! The per-frame log drowned the terminal. Instead, `painter_bridge::dispatch` records this frame's
//! dispatch cost — split into sub-phases — plus the producer + flags via [`record_dispatch`], and
//! `run_render_frame`'s frame timer reports the whole-frame cost via [`end_frame`]; the two are paired
//! and a compact summary is printed once every [`WINDOW`] painter frames. It reports the p50 of each
//! sub-phase (so the dominant one is obvious) and the flags of the WORST-dispatch frame (so they
//! describe the expensive case, not an idle tail frame). Painting a scenario for ~2 s yields one or
//! two lines to paste — enough to locate the cost inside dispatch.
```

### FrameInfo.ov_tol_ms (linhas 38–39 no original)

```text
    /// The OVERLAY phase, split by call — it aggregated three of them, and a 3,9 ms number that could
    /// belong to any of the three is a number nobody can act on (Enio, 2026-07-25, 4096²).
```

### FrameInfo.panel_sub (linhas 43–44 no original)

```text
    /// The PANEL phase split by step (labels in [`PANEL_LABELS`]) — a 3,8 ms number shared by a
    /// revision-gated clone, a `brush_settings` snapshot and two shape re-bakes names none of them.
```

### FrameInfo.fold_ms (linhas 50–54 no original)

```text
    /// **O DIVISOR do `preview` na rota GPU** — dentro do `try_drive` o composite e a luz correm no
    /// device, mas o FOLD dos três planos de relevo roda na CPU, e ele é o único item ali cujo custo
    /// depende de a janela ser um retângulo ou a tela inteira (§4.8.2: 0,38 ms contra 14,55 a 4096²).
    /// Sem esta linha um `preview 54,3 ms` com `branch=idle` não distingue *o device demorou* de *a
    /// CPU dobrou o canvas*, que são curas opostas.
```

### FrameInfo.fold_full (linhas 56–57 no original)

```text
    /// `true` quando o fold percorreu a TELA INTEIRA — a pergunta que decide a cura, e que nenhum
    /// relógio sozinho responde.
```

### Agg.pending (linhas 79–80 no original)

```text
    /// **L0 — o relógio que o artista SENTE.** O instante do evento de ponteiro mais ANTIGO que ainda
    /// não foi mostrado; `None` quando não há nada pendente. Ver [`stamp_pointer`].
```

### Agg.events (linhas 84–90 no original)

```text
    /// **Quantos eventos chegaram na janela** — TODOS, não só os que abriram um lote.
    ///
    /// ⚠️ Sem ele o relatório não distingue as duas causas de uma latência alta, e elas têm curas
    /// OPOSTAS: se chegam ~1 evento por frame, o atraso é do **pipeline** (o frame demora a sair); se
    /// chegam muitos, eles **ENFILEIRAM** e a cura é consumir o lote inteiro por frame. A 1ª leitura
    /// real (Enio, 4096², 2026-07-25) deu `p50 34,3 ms` com `frame p50 4,7 ms` — aritmética que só
    /// fecha se uma das duas for verdade, e o relatório não dizia qual.
```

### Agg.input_ms (linhas 92–103 no original)

```text
    /// **O trabalho de pintar, que acontece FORA do frame.** Acumulado dentro da janela entre dois
    /// `end_frame`, em ms.
    ///
    /// ⚠️ **O `PaintFrameTimer` cobre o `run_render_frame` inteiro — e o `on_canvas_pointer` NÃO roda
    /// lá dentro.** Ele roda no handler de input do winit, então carimbar dabs (a 4096², com impasto)
    /// nunca apareceu em `frame`, nem em `dispatch`, nem em nenhum dos 17 sub-slots. A 1ª leitura com
    /// período real mostrou o buraco: `frame p50 16,7 ms` e **período 99,9 ms/frame**, com 8,8 eventos
    /// por frame — 83 ms por frame fora de tudo que o relatório media.
    /// ⚠️ **Por FASE (`[down, move, up]`), e não um número só.** O log de 2026-08-01 trouxe
    /// `INPUT p50=0,0 max=1016,5 ms`: mediana grátis e UM evento de um segundo — que admite *"um
    /// pen-up custa um segundo"* e *"um move custa um segundo"*, **curas opostas**. Um custo sem a sua
    /// fase é inatribuível, exatamente como `stamps` era antes de publicar as entregas (§5.48).
```

### Agg.window_start (linhas 109–113 no original)

```text
    /// Início da janela, para o **PERÍODO** real do frame (`span / frames`).
    ///
    /// ⚠️ O `frame p50` mede o **TRABALHO** dentro do frame, não o intervalo entre frames — e a
    /// diferença é o que o app passa esperando o vsync, a fila de eventos e o compositor. Uma latência
    /// de 34 ms sobre um trabalho de 4,7 é impossível de ler sem o período ao lado.
```

### fn stamp_pointer (linhas 122–134 no original)

```text
/// **L0: carimba a chegada de um evento de ponteiro de canvas** (o plano 26, frente L).
///
/// O módulo media `ms/frame` e `ms/dispatch` e **nunca mediu do evento até o pixel** — que é o único
/// número que a pergunta *"por que o Procreate parece mais rápido?"* de fato cobra, e cujo alvo é
/// público: **9 ms** (o Apple Pencil saiu de 20 para 9, e não foi compute — foi pipeline).
///
/// ⚠️ **Carimba o MAIS ANTIGO não-servido, nunca o mais recente.** Entre dois frames chegam vários
/// eventos; o que o artista percebe como atraso é o do PRIMEIRO deles, porque é ele que está na tela
/// esperando há mais tempo. Carimbar o último mediria a latência do evento mais sortudo do lote e
/// reportaria um número que ninguém sente.
///
/// ⚠️ **É o relógio da SHELL, não do tool** — o contrato `CanvasPaintTool` está congelado (§6) e um
/// método novo nele exigiria ADR. A shell já é dona do evento; ela o carimba na entrega.
```

### fn note_gpu_fold (linhas 148–154 no original)

```text
/// **Quanto este evento custou DENTRO do `on_canvas_pointer`** — o trabalho de carimbar dabs.
///
/// Irmão do [`stamp_pointer`]: aquele diz quando o evento CHEGOU, este diz quanto ele CUSTOU. Os dois
/// juntos são a diferença entre *"o frame demora a sair"* e *"pintar é caro e ninguém estava olhando"*.
/// Registra o custo do FOLD dos planos de relevo (a metade de CPU do `try_drive`) e se ele percorreu a
/// tela inteira. Chamado de [`super::painter_gpu_preview`], onde a janela é resolvida — o `record_dispatch`
/// o drena para o `FrameInfo` do quadro. No-op sem a env var.
```

### InputPhase::of (linhas 183–184 no original)

```text
    /// A fase de um evento de canvas. Mora AQUI, ao lado do enum que ela produz, e não no sítio de
    /// chamada — o `painter_canvas_input.rs` está no teto de 600 LOC, e o mapeamento é do balde.
```

### fn record_dispatch (linhas 208–210 no original)

```text
        // ⚠️ O fold é anotado DENTRO do `try_drive`, que roda na fase `preview` deste mesmo quadro —
        // então drená-lo aqui o credita ao quadro certo. Zerar depois de ler é o que impede um quadro
        // sem fold de herdar o número do anterior.
```

### fn end_frame — latência (linhas 263–265 no original)

```text
        // ⚠️ Ele mede *evento → fim do frame*, e o present acontece a uma fração de ms daqui; a
        // diferença é honesta e menor que a resolução com que o artista percebe atraso. Prometer
        // "→ pixel" quando o instrumento para no fim do frame seria vender o que ele não mede.
```

### fn pq (linhas 304–306 no original)

```text
/// O quantil `q` de `v` (0..1). ⚠️ A latência se julga pela **CAUDA**: uma mediana de 8 ms com um
/// p95 de 40 é exatamente o que o artista descreve como *"às vezes trava"*, e a mediana sozinha diz
/// que está tudo bem.
```

### fn emit — WORST split (linhas 363–367 no original)

```text
    // ⚠️ **E o SPLIT do frame PIOR, não só a magnitude dele.** O `max=` diz QUANTO o pior frame custou
    // e as flags dizem em que estado ele estava — mas as fases eram todas p50, ou seja descreviam o
    // frame TÍPICO. Um custo de UMA VEZ (o primeiro traço alocando as texturas do preview no tamanho
    // do canvas) é invisível numa mediana por construção: ele é exatamente o outlier que a mediana
    // existe para descartar. Sem esta linha, "muito delay no primeiro traço" não tem onde ser lido.
```

### fn emit — AQUARELA (linhas 399–406 no original)

```text
    // A AQUARELA do lado do TOOL, quando ela pintou nesta janela. Mora aqui e não no `[frame]` porque
    // ⚠️ **é este o relatório que o artista roda** (`PH2D_PAINT_PERF`): a 1ª versão desta linha nasceu
    // no bloco `[frame]`, que é de outra env, e **não apareceu no log do smoke** — um instrumento no
    // logger errado é indistinguível de um instrumento que não existe.
    //
    // A linha `CHROME wet` acima é a metade do SHELL (o véu); esta é a metade do TOOL, e as duas
    // juntas fecham o quadro. O `ns/texel` separa dois mundos com curas OPOSTAS entre duas janelas:
    // constante = o pincel/documento cresceu (TRABALHO, e há sliders) · subindo = contenção.
```

### fn emit — EVENTO->FRAME (linhas 429–430 no original)

```text
    // L0: o número que a pergunta do Enio cobra, com o alvo público ao lado dele — e com as DUAS
    // grandezas que dizem de onde ele vem (o período real do frame, e quantos eventos por frame).
```

### fn preview_diag_changed (linhas 461–467 no original)

```text
/// The trap answers *"which producer owns the slot, and what bbox did the CPU upload"* — both of which
/// are interesting exactly when they MOVE. Logged per frame they emitted 60 lines a second, and the
/// transition everyone was hunting scrolled past between two identical neighbours.
///
/// State lives here, next to the other diagnostic's accumulators, rather than in the bridge: a `static`
/// declared inside a `dispatch` that already carries a dozen `dbg_` locals is a thirteenth thing to read
/// past, and this one has to persist across frames.
```

### fn preview_diag_changed — poisoned (linhas 472–473 no original)

```text
        // A poisoned lock must not silence a diagnostic — logging one extra line is the harmless
        // direction to be wrong in.
```

### fn force_on (linhas 489–491 no original)

```text
/// ⚠️ `std::env::set_var` é `unsafe` na edição 2024 e a shell tem `forbid(unsafe_code)`; e mesmo que
/// não tivesse, mutar o ambiente de um processo de teste multi-thread é uma corrida. O `ON` já é
/// `thread_local`, então armá-lo direto é a porta certa.
```

### fn fold_hist (linhas 508–510 no original)

```text
/// ⚠️ `cfg(test)`: o RELATÓRIO passou a imprimir as três fases separadas (é esse o ponto do divisor),
/// então quem soma é só o gate que afirma *"as fases não se perderam"*. Um `pub` sem chamador de
/// produto seria uma segunda resposta esperando alguém chamá-la.
```

## B20a — painter_gpu_preview{,_tests}.rs (verbatim, 2026-11-10)

`painter_gpu_preview_tests.rs` (antes de `fn sculpted_tool() -> PainterTool {`):
```text
/// A single-layer canvas with a real impasto stroke on it. Big enough that a brush actually deposits
/// relief — a 4x4 canvas would leave `heights` empty and every gate below would pass by doing
/// nothing.
```

`painter_gpu_preview_tests.rs` (antes de `let t = sourced_tool();`):
```text
// A single plain raster IS GPU-representable (flatten = Some), but the
// gate must bow out FIRST: the CPU path is zero-composite + partial
// bbox upload there, and the fluid E4 texture mode (trivial-only)
// must never share the frame with a GPU layer recomposite.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **Repeat Image keeps the CPU producer** (Enio 2026-07-20, "em impasto Tiling as imagens
/// repetidas desaparecem"): the 3×3 tile preview blits the CPU composite (`PainterPreview`),
/// and when the GPU owns the slot the bridge clears that cache — the tiles silently vanish.
/// The fixture is the sculpted document (GPU-eligible since the light port — exactly the case
/// the report came from); the presence half proves the refusal is the TOGGLE's, not the stack's.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **A PROTEÇÃO mantém o produtor de CPU** (Enio 2026-08-07: *"a máscara funcionou, mas fica invisível
/// ao usar impasto"*): o overlay dela é um tinte por-pixel sobre o COMPOSTO (`apply_mask_overlay`, na
/// `runtime.rs`), e o produtor de GPU não o desenha — ele composita e ilumina, e o overlay não é uma
/// `LayerOp`.
///
/// ⚠️ **A fixture tem de ser o documento ESCULPIDO**, e é isso que torna o gate o do report: sem relevo
/// um documento com scratch é trivial e a bow-out do topo já o mandava para a CPU, então o defeito não
/// existia. Foi o relevo — elegível desde o port da luz — que o expôs.
///
/// **Mutação que deve sangrar:** apagar o `mask_scratch_active` do `gpu_eligible`.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// A sculpted SINGLE-LAYER document — the most ordinary way to use Impasto — must now take the GPU
/// path. It is the whole point of the light port, and it is the case the old `impasto_visible()` bail
/// sent to the CPU compositor together with every layer, blend mode and adjustment in the document.
///
/// The premise this rests on is checked here rather than assumed: the stack IS trivial, so without
/// the relief the gate would (correctly) bow out.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// …and the light has something to hand the shader. A gate on eligibility alone would stay green if
/// the planes came back `None`, which is the shape of "the GPU path claims the frame and draws it
/// unlit" — the exact failure the old bail existed to prevent.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// Switching the pass OFF must put the canvas back on the CPU's zero-composite fast lane. Otherwise
/// hiding the relief would silently cost a full GPU recomposite per frame to draw a picture the CPU
/// hands over by cloning an `Arc`.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **The masked document goes to the GPU now** — the headline of
/// `docs/Painter/25_avaliacao_gpu.md` achado A, asserted where the routing
/// actually happens rather than one layer down in the flatten.
///
/// Measured before this landed: a single mask took a 4096² adjustment drag
/// from 0,738 ms to 652,9 ms — 885×, triggered by a checkbox, with nothing on
/// screen to say why. A mask is how you paint inside a shape; it is not an
/// exotic feature to leave on the slow producer.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// The refusal that REMAINS, so "eligible" does not quietly become "always".
///
/// A clipped GROUP stays on the CPU deliberately: the CPU reference's Group
/// arm reads neither `mask` nor `clipping`, and a GPU that honoured them would
/// make the picture depend on which producer won the frame.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **Com PAPEL a GPU produz** (2026-10-06; era `o_papel_mantem_o_produtor_de_cpu`): o compositor
/// compõe cada saída sobre o papel com a lei inteira da CPU (`LayerCompositor::set_paper`), então um
/// documento que a GPU produziria deixa de cair na CPU por ter papel. ⚠️ Uma camada LISA com papel
/// fica na CPU, MEDIDO: ali ela recompõe só a região suja e é `1,85–4,5×` mais rápida
/// (`measure_o_quadro_com_papel_nos_dois_produtores`). A paridade ao byte é a do
/// `the_gpu_producer_shows_what_the_cpu_producer_shows`, que corre o documento sem e com papel.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **Com o VIDRO da aguada sob papel de cor a CPU produz** (doc 48, BUGS #46): o compositor da GPU só
/// conhece um alfa por texel, e mostraria a aguada que soma luz ao papel. Um desenho novo (papel branco)
/// esculpido e com uma aguada continua na GPU; o papel castanho tira-o dela; sem aguada, o castanho
/// não tira.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **G20 (F36 §1) — no fim de um episódio sem memória o preview na placa refaz a sessão e o slot e
/// recompõe a tela, sem esperar uma pincelada.** O que a sessão e o slot criaram sob recusa é inválido;
/// a fonte são as camadas em CPU. Um quadro parado sem episódio não refaz nada.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **B16.12 — a troca GPU→CPU sai do quadro do toque.** Com a aguada ARMADA sobre o papel de cor, a 1.ª
/// aguada criaria o vidro e tiraria o documento da placa NO toque (4K: a tela inteira composta na CPU e
/// 33 MB enviados, 154,5 ms). O portão decide a CPU antes do 1.º vidro, e o toque não muda a pista; o
/// Digital no mesmo papel fica na GPU.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **B20.A2 — a versão de uma camada é um relógio do DOCUMENTO** (`bump_layer_pixels`: sobe a cada toque, de
/// qualquer camada), não «+1 por envio». O retângulo sujo do instrumento cobre só o que mudou desde a drenagem
/// ANTERIOR, logo só remenda a fatia que subiu nessa drenagem; uma camada que lá não esteve (fora da pilha,
/// prewarm, falha) sobe inteira — senão o que mudou antes do retângulo ficava velho na placa.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **B20.A1 — a falha da placa entrega o quadro à CPU.** Uma tela mais larga do que a placa aceita: o
/// compositor recusa, o `drive` devolve `false`, e antes o `try_drive` dizia `true` com o slot largado —
/// ninguém produzia o quadro. Agora a CPU compõe a tela inteira nesse quadro e fica dona (um aviso só).
```

`painter_gpu_preview.rs` (antes de `let Some((ops, adj_luts)) = ph2d_tool_painter::flatten_for_gpu(painter`):
```text
// Also allocates the canvas-sized compositor/premul resources (first execution only).
// ⚠️ The LIGHT pass stays out (named limitation, a product decision): a fresh stack has no
// relief, so the first relief stroke folds the whole canvas. `gpu_eligible` is bypassed ON
// PURPOSE (a fresh stack is trivial and refuses). The slot is released next frame; the passes'
// internal textures live in `session_slot` and survive.
```

`painter_gpu_preview.rs` (antes de `struct PainterLayerProvider<'a> {`):
```text
/// Adapts [`PainterTool::preview_layer_pixels`] to the render crate's
/// [`LayerPixelProvider`] — the tool stays decoupled from `ph2d-render`, so the
/// bridge owns this glue.
```

`painter_gpu_preview.rs` (antes de `pub(super) fn try_drive(`):
```text
/// Decide GPU-vs-CPU for this frame and, when GPU, recomposite into the preview slot. Returns
/// `true` iff the GPU producer owns the slot; otherwise a no-op and the caller runs the CPU path.
/// Drains the preview-dirty flag (recomposite only on change) and the dirty bbox (no leak into a
/// later CPU frame). A TRIVIAL stack stays on the CPU (zero-composite + partial upload) unless
/// the canvas carries relief — see [`gpu_eligible`].
```

`painter_gpu_preview.rs` (antes de `if painter.preview_layer_stack_is_trivial() && !painter.impasto_visibl`):
```text
// Trivial bow-out unless relief: relief forces a CPU full composite + light, where the GPU is
// better (mirrors `runtime.rs`). Paper alone keeps the document on the CPU lane (it recomposes
// only the dirty region).
```

`painter_gpu_preview.rs` (antes de `if painter.papel_pede_a_cpu() {`):
```text
// Wash GLASS under coloured paper (doc 48): paper crosses each channel by its alpha and the GPU
// compositor knows one alpha per texel — the CPU produces, from the moment the wash is ARMED on
// that paper (B16.12: never on the touch frame).
```

`painter_gpu_preview.rs` (antes de `#[allow(clippy::too_many_arguments)]`):
```text
/// Composite `ops` on the GPU, premultiply, copy into the preview slot ((re)acquired when
/// missing/resized). On a GPU error the slot is released and a toast queued. Returns `true` iff
/// a preview texture is live.
```

`painter_gpu_preview.rs` (antes de `fn compose_light_premul(`):
```text
/// Composite, light, premultiply — everything that turns the layer stack into the bytes the preview
/// slot will hold, with no slot and no `SpriteRenderer` in sight. The finished pixels are
/// `session.premul.output_texture()`.
///
/// Extracted so the parity gate (`the_gpu_producer_shows_what_the_cpu_producer_shows`) drives
/// the REAL chain.
///
/// # Errors
///
/// Which stage refused — each hands the frame back to the CPU producer.
```

`painter_gpu_preview.rs` (antes de `let mut novas = std::mem::take(&mut session.a_entregar);`):
```text
// 1) GPU composite into the PERSISTENT canvas-sized `out`, writing at canvas
//    coords — a region dispatch refreshes only `region`, leaving the rest from
//    the prior frame (the 4K multi-layer cost goes O(canvas×layers) → O(env)).
```

`painter_gpu_preview.rs` (antes de `let comp_out = session.compositor.output_texture().ok_or(ph2d_i18n::tr`):
```text
// 2) Impasto light on the freshly composited region (lighting is NOT idempotent). `None` planes
//    = no relief / pass off / lamps dark. The match YIELDS the finished texture: one answer to
//    "which texture holds the canvas".
```

`painter_gpu_preview_tests.rs` (antes de `let mut t = sourced_tool();`):
```text
// Opacity < 1 on the single layer breaks triviality without leaving
// GPU-representability → the GPU path owns the preview.
```

`painter_gpu_preview_tests.rs` (antes de `#[test]`):
```text
/// **A placa larga o documento e a CPU compõe a tela inteira NESSE quadro** (B16.12): sem isto a pista
/// da CPU só compunha no próximo sujo — o toque — e pagava ali a tela fria inteira e o envio cheio.
```

`painter_gpu_preview.rs` (antes de `pub struct PainterGpuPreview {`):
```text
/// Per-session GPU state for the Painter live preview: compositor, straight→premul blit and a
/// cached [`GpuContext`]. Created on bind ([`prewarm`]) or lazily, kept for the tool session.
```

`painter_gpu_preview.rs` (antes de `light: ImpastoLightPass,`):
```text
/// Impasto's light — the relief made visible. Runs BETWEEN the composite and the premultiply, which
/// is exactly where the CPU pass runs (`runtime::take_preview_arc`: composite, light, overlay).
```

`painter_gpu_preview.rs` (antes de `geracao: u64,`):
```text
/// A geração de memória da placa em que a sessão nasceu (F36): no fim de um episódio sem memória
/// a sessão e o slot refazem-se e a tela recompõe-se inteira (a fonte são as camadas em CPU).
```

`painter_gpu_preview.rs` (antes de `planos: Option<ph2d_tool_painter::ImpastoPlanes>,`):
```text
/// Os planos do último envio, cujos buffers a próxima dobra reescreve (B15-cont: sem os ~107 MB alocados e
/// soltos a cada quadro do Ctrl+Z 4K).
```

`painter_gpu_preview.rs` (antes de `entregue: Vec<(u64, u64)>,`):
```text
/// B20.A2: `(camada, versão)` entregues ao compositor na última drenagem da placa. O sujo do instrumento
/// cobre só o que mudou desde a drenagem ANTERIOR: a camada que não subiu nela sobe inteira.
```

`painter_gpu_preview.rs` (antes de `pub(crate) fn prewarm(`):
```text
/// Builds the session BEFORE the first stroke needs it (doc 28 §4.8): its passes compile
/// shaders (~28 ms), paid on the document bind — human time — never on the first pen-down.
```

`painter_gpu_preview.rs` (antes de `fn recusa(session_slot: &mut Option<PainterGpuPreview>, painter: &mut `):
```text
/// B20.A1: a placa recusou a tela. A CPU compõe-na inteira JÁ neste quadro e fica dona até a tela ou a
/// geração de memória mudarem.
```

`painter_gpu_preview.rs` (antes de `let contigua = self.anteriores.iter().any(|&(k, _)| k == key);`):
```text
// A versão é um relógio do documento (sobe a cada toque, de qualquer camada): o sujo só remenda a
// fatia que subiu na drenagem anterior; uma camada que lá não esteve sobe inteira.
```

`painter_gpu_preview.rs` (antes de `fn gpu_eligible(painter: &PainterTool) -> Option<(Vec<LayerOp>, Vec<f3`):
```text
/// The GPU-vs-CPU gate, pure half: `Some(ops, adj_luts)` iff this frame composites on the GPU.
/// The trivial-stack check runs FIRST (see [`try_drive`]).
```

`painter_gpu_preview.rs` (antes de `if painter.repeat_image() {`):
```text
// Repeat Image draws its 3×3 tiles from the CPU composite, which a GPU-owned slot clears:
// while the tiling preview is on, the CPU lane produces.
```

`painter_gpu_preview.rs` (antes de `if painter.mask_scratch_active() {`):
```text
// The mask PROTECTION tint (`apply_mask_overlay`) lives in the CPU composite and the GPU
// producer does not draw it: while a scratch is live, the CPU lane produces.
```

`painter_gpu_preview.rs` (antes de `let region = if seed_full {`):
```text
// The composite region: the WHOLE canvas on a seed frame (so the persistent
// `out` + the slot hold a valid full backdrop), else just the wet envelope.
```

`painter_gpu_preview.rs` (antes de `if let Err(e) = compose_light_premul(session, tool, &ops, &adj_luts, w`):
```text
// 1-3) Composite, light, premultiply — everything that turns the layer stack into the bytes the
//      slot will hold. On any failure the slot is released and the CPU producer takes the frame.
```

`painter_gpu_preview.rs` (antes de `let slot = ensure_slot(renderer, painter_preview_gpu, entity_bits, wid`):
```text
// 4) Ensure a right-sized slot, then COPY the premultiplied result in: the whole canvas on a
//    seed frame, else only the wet envelope rect.
```

`painter_gpu_preview.rs` (antes de `session.compositor.set_paper(tool.papel());`):
```text
// O PAPEL do documento: o compositor compõe cada saída SOBRE ele com a lei inteira da CPU
// (`compoe_sobre_o_papel`), no mesmo lugar da cadeia — depois do composite, antes da luz.
```

`painter_gpu_preview.rs` (antes de `let plane_win = tool`):
```text
// Fold window: the tool's `preview_gpu_region` (Some only for a rect-confined change) AND the
// pass's `planes_seeded` AND a lit last frame (an unlit one wrote no planes); any no folds the whole canvas.
```

`painter_gpu_preview.rs` (antes de `region: Region::full(width, height),`):
```text
// The WHOLE canvas: the light owns a second persistent canvas, so its freshness
// cannot be inherited from the compositor's region (reads unlit, writes its own).
```

`painter_gpu_preview.rs` (antes de `session`):
```text
// 3) Premultiply into a COPY_SRC texture (the slot samples PREMULTIPLIED);
//    full-canvas, re-premultiplying out-of-region texels is a no-op.
```

`painter_gpu_preview.rs` (antes de `fn ensure_slot(`):
```text
/// Ensure a `width × height` slot (reuse on matching dims, else release + acquire EMPTY);
/// returns its `texture_id`. `arc_token` is `0`, so a GPU→CPU transition forces a full re-upload.
```

`painter_gpu_preview.rs` (antes de `fn release_slot(`):
```text
/// Release the preview slot (if any) — local mirror of
/// `painter_bridge::release_preview_texture`, so the producers don't reach into each other.
```

`painter_preview_handoff_tests.rs` (antes de `#[allow(clippy::too_many_arguments)]`):
```text
/// One frame of the app's preview lifecycle, in `dispatch`'s exact order: the GPU producer gets
/// first refusal (`try_drive`), a GPU-owned frame clears the CPU cache, a CPU-owned frame drains
/// the tool and runs the real upload door ([`crate::painter_bridge_upload::upload_cpu_preview`]).
/// Returns whether the GPU producer owned the slot this frame.
```

`painter_preview_handoff_tests.rs` (antes de `let mirror = crate::painter_bridge_upload::own_preview_buffer(`):
```text
// The shell owns its preview buffer (drives the REAL helper), so the tool stays the sole
// owner of its canvas — exactly as `dispatch` does it.
```

`painter_preview_handoff_tests.rs` (antes de `let cache_version = painter.canvas_version();`):
```text
// Idle (drain None) reads the unchanged version → the plan Skips; a dirty frame reads the bumped
// one → it uploads.
```

`painter_preview_handoff_tests.rs` (antes de `#[test]`):
```text
/// **The screen survives the producer handoffs — CPU→GPU→CPU, on real hardware.**
///
/// The producer handoff dance: the GPU producer takes the slot when the stack is representable, the
/// CPU producer reclaims it when it is not, and **the first CPU frame after a GPU stretch must
/// re-seed the slot whole** — the two defects this phase found (`take_preview_dirty` leaving the CPU
/// composite stale + the partial plan patching a GPU-seeded slot) both live exactly on that seam,
/// latent today only because every eligibility-flipping door happens to invalidate the composite.
/// This gate runs the seam end-to-end — real `try_drive`, real upload door, real wgpu textures, real
/// readback — and holds the final screen bytes to a from-scratch recompose.
///
/// ## The lever keeps moving, the gate does not
///
/// It first flipped eligibility with `impasto_show` (premise: *"the GPU compositor cannot light
/// relief"*), then with a layer **mask** (2026-07-18, once the light port made relief GPU-eligible).
/// Then the GPU Ondas made a mask an OP too (`docs/Painter/25_avaliacao_gpu.md`), so a mask no longer
/// moves a document between producers either — the lever is now an **unported adjustment**
/// (`ColorBalance`: no scalar and no spatial GPU code, so `flatten_for_gpu` still refuses it). Nothing
/// this gate proves has changed: it needs a door that flips eligibility, not any particular door — and
/// each time a wave widened what the GPU can represent, this gate's lever had to be the thing STILL
/// outside it (⚠️ and this gate is GPU-adapter-only, so `ship.sh` never runs it — the mask lever went
/// stale-green-latent for exactly one wave before this run caught it).
///
/// `#[ignore]`: needs a GPU adapter (none on CI); run locally with `--ignored`.
```

`painter_preview_handoff_tests.rs` (antes de `assert!(`):
```text
// 1. Trivial stack, nothing painted yet → the CPU producer owns (the trivial bow-out: its lane
//    is zero-composite there, strictly cheaper than a GPU round trip).
```

`painter_preview_handoff_tests.rs` (antes de `let l2 = t.add_raster_layer("Layer 2").expect("layer 2");`):
```text
// 2. The stroke put RELIEF on the canvas, which takes the trivial bow-out away (the CPU lane
//    refuses its zero-composite fast path once `impasto_visible`) → CPU→GPU handoff, with the GPU
//    doing the lighting. This is the step the light port exists for.
```

`painter_preview_handoff_tests.rs` (antes de `t.add_adjustment_layer(AdjustmentKind::ColorBalance)`):
```text
// 4. An UNPORTED adjustment (ColorBalance has no scalar and no spatial GPU code) → outside the
//    GPU op-list → GPU→CPU handoff; then one more impasto stroke so the partial lane runs over
//    the re-seeded slot. `add_adjustment_layer` leaves the previously-active RASTER as the paint
//    target, so the follow-up strokes still deposit. (The lever moved off the mask: this line's
//    own wave made masks GPU-representable, so a mask no longer flips eligibility — see the module
//    doc.)
```

`painter_preview_handoff_tests.rs` (antes de `#[test]`):
```text
/// **A slot the GPU producer seeded is never PATCHED — the handoff must re-seed it whole.**
///
/// The GPU preview producer stamps its slots with `arc_token: 0` ("no CPU Arc") precisely so the
/// first CPU frame after a GPU-owned stretch forces a full re-upload — the slot holds the GPU
/// compositor's output (unlit, another era), and a rect patched over it leaves every other pixel
/// to a different producer. The dims/entity guards can't tell the producers apart (the GPU lane
/// reuses the same slot at the same dims), so the token is the ONLY witness. Today this is latent
/// — every door that flips producer eligibility happens to invalidate the tool's composite, so no
/// bbox reaches this plan on a handoff frame — but that safety is an enumeration of doors, and
/// door N+1 (a future eligibility flip that forgets to invalidate) lands on exactly this line.
///
/// **Mutation that must bleed:** drop the `g.arc_token != 0` arm of the partial guard in
/// [`plan_upload`] — the plan below comes back `Partial`.
```

`painter_preview_handoff_tests.rs` (antes de `plan_upload(&preview, Some(gpu_seeded), Some((4, 4, 8, 8)), 1, false),`):
```text
// A real CPU content version (1) against the GPU stamp (0): the version differs so an upload
// is due, and the `g.arc_token != 0` arm must still force it FULL rather than Partial.
```

`painter_preview_handoff_tests.rs` (antes de `#[test]`):
```text
/// **What the artist sees through the GPU producer is what they would have seen through the CPU one
/// — on a SCULPTED canvas, with the relief lit by the shader.**
///
/// This is the gate the GPU light port is answerable to. Everything else about the port is checked a
/// level down (`ph2d-render`'s `impasto_light_gpu` reconciles the shader against
/// `apply_impasto_light` pixel for pixel), but that gate hands both sides the same synthetic buffer.
/// This one runs the PRODUCT: the real `try_drive`, the real eligibility gate, the real flatten, the
/// real GPU compositor, the real light pass, the real premultiply, the real slot texture — and reads
/// the bytes the sprite shader will sample straight back off the device.
///
/// The oracle is the OTHER producer, which is the only oracle that answers the question actually
/// being asked. Moving a document between two lanes is only safe if the lanes draw the same picture;
/// a gate comparing the GPU lane to a re-derivation of what it ought to draw would agree with any bug
/// the two share, and the whole risk of this change is that they might not share one.
///
/// **Why this could not have been written before 2026-07-18:** the eligibility gate sent every
/// sculpted canvas to the CPU, so there was no GPU-owned frame with relief on it to read back.
```

`painter_preview_handoff_tests.rs` (antes de `if let Some([r, g, b]) = papel {`):
```text
// O PAPEL do documento (2026-10-06): a GPU compõe sobre ele com a lei inteira da CPU, então o
// mesmo documento, com papel de cor, tem de dar os mesmos bytes nos dois produtores.
```

`painter_preview_handoff_tests.rs` (antes de `t.on_canvas_pointer(cp([60.0, 90.0], PointerPhase::Down));`):
```text
// Sculpt: a curved stroke, so the relief carries slopes in every direction. A straight one would
// leave the normal pointing the same way down the whole stroke and a sign error could hide.
```

`painter_preview_handoff_tests.rs` (antes de `let mut gpu_owns = false;`):
```text
// Run frames until the producers settle. The GPU must be the one holding the slot: if the
// eligibility gate ever sends a sculpted canvas back to the CPU this reads as a plain failure
// here rather than as a silent loss of the speed the port was built for.
```

`painter_preview_handoff_tests.rs` (antes de `let truth = screen_truth(&mut t);`):
```text
// …against the CPU producer's own answer for the same document. `screen_truth` drains the tool,
// so it runs AFTER the readback.
```

`painter_preview_handoff_tests.rs` (antes de `for angulo in [120.0, 200.0] {`):
```text
// B15.3: o tique da lâmpada não redobra (janela de 1 texel) e a tela é a da CPU com a luz nova. O 1.º
// redobra: a `screen_truth` acima foi um quadro da CPU.
```

`painter_preview_handoff_tests.rs` (antes de `#[test]`):
```text
/// **O PAPEL atravessa o produtor da GPU — o gate que faltava, e a razão de o Enio ver o papel morto.**
///
/// Irmão exato do de cima, com a única alavanca trocada: nenhuma pincelada, **só o substrato**. É a
/// forma que o defeito de 2026-08-10 tinha (*"Paper parece não funcionar para Digital"*): a wave somava
/// a inclinação do dente dentro do laço da **CPU**, e um documento que a **GPU** compõe — o caso normal
/// — nunca passa por lá. Sete gates de unidade verdes, e o papel invisível na tela.
///
/// ⚠️ **A fixture NÃO pinta, e é isso que a torna o oráculo certo.** O gate irmão esculpe um traço, e
/// sobre relevo de tinta o dente é um detalhe que se perde no ruído; um documento nu isola a única coisa
/// que esta wave acrescenta. É também literalmente o caso do artista: o Digital não tem plano de
/// `covers` nenhum.
///
/// **Duas mutações têm de sangrar, e cada uma sozinha:**
///  * tirar o dente do `ReliefFields::height_at` (devolvê-lo ao laço da CPU) — o plano sobe chato e a
///    tela da GPU volta a ser papel branco enquanto a da CPU tem grão;
///  * zerar o `paper_body` no uniform — o plano sobe CERTO e o shader o apaga, multiplicando por uma
///    cobertura de tinta que num documento digital é zero.
```

`painter_preview_handoff_tests.rs` (antes de `let lit = shown`):
```text
// ⚠️ O CONTROLE vem antes da paridade: sem tinta nenhuma, duas telas brancas casariam
// perfeitamente e o gate passaria por vácuo — que é exatamente o estado que ele existe para
// reprovar. O dente TEM de ter acendido alguma coisa.
```

## B20.1 — preview tests (verbatim, 2026-11-10)

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 1–19 no original)

```text
//! Display gates — **what the artist SEES is what the tool painted** (phase D, 2026-07-15).
//!
//! Enio's 2026-07-14 smoke reported two display bugs with impasto on screen: (a) the **Anchored**
//! method's relief vanishes at pen-up, (b) a **jittered** stroke's relief comes out stretched. The
//! tool was proven innocent the same day (its own gate holds the pen-up frame byte-equal to a full
//! recompose, *inside* the tool). So the defect — if it is reachable without a window — lives in
//! the seam these tests own: the shell's preview drain → upload plan → slot texture, the bytes the
//! sprite shader actually samples.
//!
//! Everything here is the REAL pipeline: the real `PainterTool` under the smoke's arming, the real
//! drain (`take_preview_arc` / `take_preview_upload_bbox`), the real decision ([`plan_upload`]),
//! the real gather ([`extract_region`]) and the real `premultiply_rgba8`. The only substitution is
//! the wgpu copy at the very end, modelled as what a texture upload is — a byte copy into an array
//! ([`Screen::frame`]). That is not a mirror of the product; it is the product minus the driver.
//!
//! The oracle models the APPEARANCE, never the mechanism: after any frame, the slot's bytes must
//! equal the premultiplied composite the tool just claimed; after a whole gesture, they must equal
//! a from-scratch recompose of the document. Anything the protocol drops, misplaces or staleness
//! leaks — vanished relief, displaced patch rects, missed settle frames — is a byte diff here.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 41–43 no original)

```text
/// A white canvas + the impasto smoke's exact arming (`impasto_smoke::arm_brush_once`: size 40,
/// impasto ON, everything else the shipped default) — the brush in Enio's hand when he saw both
/// symptoms.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 52–54 no original)

```text
/// The screen's half of the CPU preview lane, run exactly as `painter_bridge::dispatch` runs it
/// each frame: drain if dirty (a clean frame keeps the previous cache), plan, apply the plan to
/// the slot bytes, update the bookkeeping.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 63–66 no original)

```text
    /// The tool's composite for the last drained frame, kept WHOLE and INDEPENDENT of the shell's
    /// mirror — the oracle. `assert_shows_the_cache` compares the slot against this, so a bug in
    /// `own_preview_buffer`'s region patch (the shell's mirror diverging from the tool's composite)
    /// shows up as a mismatch instead of hiding behind a slot derived from the same mirror.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 91–93 no original)

```text
            // The shell's OWN buffer — the REAL function, driven exactly as `dispatch` drives it. It
            // patches the dirty region into the prior buffer (or seeds a fresh one) and drops
            // `drained`, so the tool is left the sole owner of its canvas.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 169–172 no original)

```text
    /// The pair oracle: the slot must hold exactly the premultiplied form of the composite the
    /// tool most recently handed the bridge. On a Full frame that is trivially the upload; on a
    /// Partial frame it additionally asserts the *untouched* slot pixels still match — i.e. the
    /// tool's dirty bbox really covered everything that changed, lighting included.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 175–177 no original)

```text
        // Compare against the INDEPENDENT truth (the tool's composite), not the shell's mirror the
        // slot was patched from — so a region-patch bug in `own_preview_buffer` can't hide behind a
        // slot and mirror that agree with each other while both diverge from what the tool produced.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 184–188 no original)

```text
/// What the artist SHOULD be seeing: the document composited from scratch (the public structural
/// door forces the full lane — re-setting the active layer's opacity to its CURRENT value is a
/// byte-neutral `invalidate_composite`; reading it first matters, or the oracle would silently
/// EDIT a half-opacity layer to 1.0 and compare the screen against a different document — it did,
/// for one red run of the handoff gate), premultiplied the way the upload premultiplies.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 230–238 no original)

```text
/// **D(a) — the Anchored stroke's relief must still be on screen after pen-up.**
///
/// The symptom (Enio, live app): drag with the Anchored method — the drag sizes one disc from the
/// press point — and the lit relief is there; lift the pen and it vanishes. The tool keeps it (its
/// own pen-up gate, `impasto_smoothing_settles_every_stroke_...`, holds the post-up frame equal to
/// a full recompose). Here the SAME gesture runs through the shell's drain → plan → slot protocol,
/// frame by frame at the app's cadence, and the slot is held to the from-scratch truth after the
/// up — plus one idle frame, because a one-frame lag is invisible but a slot nobody updates again
/// IS the bug.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 273–278 no original)

```text
/// **The relief the screen shows is LIT relief — the presence sibling.**
///
/// Every equality above is vacuous if the composite never carried shading in the first place (an
/// unlit pipeline agrees with an unlit truth). So: the same committed Anchored stroke must render
/// DIFFERENTLY with the light pass on vs off. This is the anti-vacuity anchor for the whole file
/// ([[feedback_absence_gate_needs_a_presence_sibling]]).
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 305–314 no original)

```text
/// **D(b) — a jittered stroke's screen must track the tool frame by frame.**
///
/// The symptom (Enio, live app): with per-dab Jitter armed the relief comes out stretched. Jitter
/// Scale + Rotate churn small, scattered rects all along the stroke — the heaviest client of the
/// B.1 partial-upload lane. A patch rect that is displaced, undersized, or that misses pixels the
/// LIGHT changed (shading reaches past the stamped bbox: the normal reads neighbours) leaves stale
/// bytes standing next to fresh ones — which on a relief reads exactly as smearing/stretching.
/// The pair oracle after every frame holds every slot byte, patched and untouched alike, to the
/// composite the tool just handed over; the end oracle holds the final screen to a from-scratch
/// recompose.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 343–351 no original)

```text
/// **Every stroke method's screen survives its own commit path — the table.**
///
/// The tool's pen-up bug lived in the paths nobody connected (the five shape methods never
/// committed), and its gate is a table over all ten methods for exactly that reason. The screen's
/// gate takes the same shape: for every method, the same drain-per-event cadence, the pair oracle
/// on every frame and the from-scratch truth at the end — through pen-up for the freehand five,
/// and through the OPEN shape for the editor five (an open shape is a live preview with an Apply
/// button; the screen must show it too). A method whose commit forgets to tell the preview — or
/// tells it with a bbox smaller than what the settle re-lit — goes red here by name.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 399–402 no original)

```text
/// Deposit two thick ridges (relief for the spatula to work), then enter **Sculpt** — the actual
/// context of the 2026-07-14 smoke: both display symptoms were reported while SCULPTING, and the
/// sculpt's commit path (`sculpt_session`, per-stroke law, settle-at-up) is entirely different
/// plumbing from the deposit's. Every event is followed by a frame, at the app's cadence.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 422–426 no original)

```text
/// **D(a), in the smoke's own context: a SCULPT stroke through the Anchored method must still be
/// on screen after pen-up.** The sculpt re-renders from a frozen `pre` and commits at the up — a
/// commit that swaps buffers without touching a pixel of pigment is exactly the shape of bug that
/// leaves the tool's buffers right and the screen stale (the tool's own 19601 lesson, one path
/// over). Frame-per-event cadence; pair oracle mid-gesture; from-scratch truth after the up.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 456–459 no original)

```text
/// **D(b), in the smoke's own context: a JITTERED sculpt tracks the tool frame by frame.** Jitter
/// Scale/Rotate on the spatula churns the same scattered small rects as on the brush, but through
/// the sculpt's own render (memo tiles + light). Any patch displaced or under-covering what the
/// light re-shaded reads as smeared/stretched relief on screen.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 490–492 no original)

```text
/// **The plan itself refuses the stale-skip and the mismatched patch** — the two protocol
/// mutations that would silently freeze or misplace the screen. Pure-decision checks (no tool):
/// a newer content version must never plan `Skip`; a dims change must never plan `Partial`.
```

### `crates/ph2d-app-painter/src/painter_preview_pipeline_tests.rs` (linhas 510–512 no original)

```text
    // Same version → Skip (the idle-frame no-op). The buffer's pointer is irrelevant now — the shell
    // patches its mirror in place, so the pointer never changes even as the pixels do; only the
    // version says whether the content moved.
```

### `crates/ph2d-app-painter/src/painter_preview_undo_tests.rs` (linhas 1–11 no original)

```text
//! **O que a tela mostra DEPOIS de um undo** — o report de resquício do smoke `PH2D_IMPASTO_SMOKE=2`.
//!
//! Enio, 2026-07-25, com foto: *"UNDO deixa resquícios na tela"* — um retângulo branco, axis-aligned,
//! recortado no traço perto do cursor. Um retângulo na saída é a impressão digital de trabalho limitado
//! por REGIÃO, e a pista GPU tem três: o upload por-camada (Onda 5b), o fold do relevo (esta wave) e o
//! `dirty` que os dois leem.
//!
//! Este módulo reproduz a sequência no caminho REAL (o mesmo `app_frame` que o gate de paridade usa:
//! `try_drive` de verdade, compositor de verdade, passe de luz de verdade, slot de verdade, readback do
//! device) e confronta o que o produtor GPU mostra com o que o produtor CPU mostraria para o MESMO
//! documento — o único oráculo que responde à pergunta que o report faz.
```

### `crates/ph2d-app-painter/src/painter_preview_undo_tests.rs` (linhas 20–31 no original)

```text
/// **Um undo deixa a tela idêntica ao que o outro produtor desenharia.**
///
/// A hipótese que este gate testa é a que a foto sugere: o undo troca o canvas INTEIRO
/// (`restore_model` reinstala `canvas_rgba`, `heights`, `covers`, `mats`), então qualquer upload ou fold
/// que ainda esteja limitado ao retângulo do último dab mostraria a arte nova dentro dele e a VELHA
/// fora — um retângulo, exatamente como o da foto.
///
/// ⚠️ O fixture tem de conter DOIS traços e desfazer o segundo: desfazer o único traço de um canvas
/// deixa a tela branca, e branco casa com branco por qualquer caminho errado
/// ([[feedback_a_fixture_only_proves_what_it_contains]]).
///
/// `#[ignore]`: precisa de adapter; rode com `--release --ignored`.
```

### `crates/ph2d-app-painter/src/painter_preview_undo_tests.rs` (linhas 136–149 no original)

```text
/// **O produtor devolve o frame para a GPU depois de a CPU ter sido dona — e os planos têm de estar
/// atuais.** Report do Enio (2026-07-25, smoke 4096²): o retângulo «aparece pintando» e fica até nova
/// pintura, depois de alternar Digital ↔ Impasto.
///
/// Mecanismo (bisseção; cura `gpu_lane_stale`): o `dirty_rect` é compartilhado pelas duas pistas e
/// consumido por quem drena; enquanto a CPU é dona ela o leva, e no frame em que a GPU retoma o rect
/// descreve só o ÚLTIMO frame da CPU. Os dois consumidores de `preview_dirty_region` (compositor e fold)
/// adoeciam juntos. A cura espelha a da Fase D: toda drenagem da CPU levanta `gpu_lane_stale` e a
/// próxima da GPU o consome declarando não-confinado (`None` = faça inteiro).
///
/// Mutação que sangra: `self.gpu_lane_stale = true` → `false` em `take_preview_arc` devolve os 197.172
/// bytes exatos. O oráculo é o OUTRO produtor (re-derivar o esperado concordaria com o bug).
///
/// `#[ignore]`: precisa de adapter; rode com `--release --ignored`.
```
