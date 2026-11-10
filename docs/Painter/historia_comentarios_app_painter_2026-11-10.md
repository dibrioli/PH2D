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
