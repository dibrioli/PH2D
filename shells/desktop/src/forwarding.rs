//! Forward winit-derived input events into the hero screen's
//! interaction dispatcher.
//!
//! Extracted from [`main`] (Track B7). Five thin orchestrators that
//! each take `Option<&mut AppGfx>`, call into the editor's
//! `dispatch_*` entry point, and apply the emitted `WidgetEvent`s.
//! Also bridges Cmd+C/X/V → `arboard` (OS clipboard) for the key
//! forwarder.

use crate::AppGfx;
use ph2d_editor_core::WidgetEvent;
use ph2d_editor_core::interaction::{PanelRowDrop, PanelRowFamily};
use ph2d_host::{KeyEvent, PointerEvent};

#[path = "forwarding_persist.rs"]
mod persist;
/// ⭐ O diagnóstico da escolha do conta-gotas (`PH2D_PICK_LOG=1`) — ver o `//!` do módulo. ⚠️ Ele
/// mora AQUI, e não na raiz: o sujeito dele é esta escolha, e a raiz é o ficheiro que o tecto de
/// LOC da shell aperta primeiro.
#[path = "pick_log.rs"]
mod pick_log;
/// ⭐ As duas coisas que o selector de cor não sabe fazer sozinho — ver o `//!` do módulo.
#[path = "forwarding_picker.rs"]
mod picker;
use picker::{handle_palette_io, painter_eyedropper_sample};

/// Forward a pointer event to the hero screen's interaction
/// dispatcher when the hero is active. Drains emitted
/// [`WidgetEvent`]s into `HeroScreen::apply_event` (consumed events
/// drive hero-level state mutations) and logs unconsumed ones to
/// stderr for the developer to verify wiring.
///
/// Returns a `PanelRowReparent` payload `(dragged, drop)` **da família `PainterLayer`** quando o
/// despacho emitiu um — quem chama (que tem o `ToolRegistry`) encaminha-o para o
/// `PainterTool::handle_layer_reparent` activo. `None` nos outros casos. (O hero/chrome não pode
/// possuir isto: é uma mutação de FERRAMENTA, e o `forward_to_hero` não tem `ToolRegistry`.)
///
/// ⚠️ **As outras famílias CAEM para o `hero.apply_event`** e são resolvidas pelo painel que as
/// desenhou — a `TagTree` é do documento e o painel *Tags* traduz a queda num `TagTreeEdit::Move`.
/// *Interceptar aqui todas as famílias faria esta função ter de conhecer cada uma delas.*
#[must_use]
pub fn forward_to_hero(
    gfx: Option<&mut AppGfx>,
    event: PointerEvent,
) -> Option<(ph2d_editor_core::NodeId, PanelRowDrop)> {
    let gfx = gfx?;
    let hero = gfx.hero_screen.as_mut()?;
    // Snapshot events before applying — apply_event may mutate hero,
    // but the events slice itself lives in the arena (immutable view).
    // Threads the live TextSystem so click→caret on text widgets
    // snaps to the nearest glyph boundary (real measurement) instead
    // of the dispatch's char-count heuristic.
    let snapshot: Vec<WidgetEvent> = hero
        .handle_pointer_fisico(event, &mut gfx.text_system, &gfx.hero_arena)
        .to_vec();
    let mut reparent = None;
    for e in snapshot {
        // Eyedropper pick. The generic readback samples `vello_pass`'s intermediate texture — the
        // Vello UI layer, which is TRANSPARENT over the canvas (sprites live in the surface, not the
        // intermediate), so picking a painted pixel always returned transparent. When the Painter is
        // active and the click lands on the selected sprite, sample the painted layer COMPOSITE
        // instead (`PainterTool::sample_composite_at_uv`) so the eyedropper reads the real colour;
        // otherwise fall back to the rendered-overlay pixel.
        if let WidgetEvent::EyedropperPick { parent, .. } = e {
            // ⚠️ O ponto do evento é LÓGICO (o hero viu o clique na escala da interface); a cor lê-se
            // na JANELA, então o ponto é o do clique físico que o fez nascer.
            let (px, py) = (event.x.max(0.0) as u32, event.y.max(0.0) as u32);
            // Disjoint-field borrows (`hero` already holds `&mut gfx.hero_screen`): read the selection
            // + panel hit from `hero` first, then pass the render fields by value/ref to the helper.
            let selection = hero.gizmo.selection;
            let on_panel = hero.chrome_panel_at(px as f32, py as f32).is_some();
            let autorada = painter_eyedropper_sample(
                &mut gfx.tools,
                &gfx.sim,
                gfx.present.world_mut(),
                &gfx.camera,
                crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size()),
                selection,
                on_panel,
                px as f32,
                py as f32,
            );
            // ⭐⭐⭐ **A COR QUE O ARTISTA VÊ** (2026-09-15) — a reserva lia SÓ a camada do Vello,
            // que sobre o canvas é transparente por construção: o conta-gotas devolvia
            // `#00000000` em quase todo o ecrã (report do dono: *«não funciona de maneira nenhuma
            // e em nenhum lugar»*). O quadro tem DUAS metades em texturas diferentes — o mundo
            // (sprites, imagens, a prévia do Painter) e o chrome (os painéis **e a arte vectorial
            // do documento**) —, e a cor do ecrã é a composição delas. Ver
            // [`ph2d_render::screen_pick`].
            let mundo =
                ph2d_render::world_source(gfx.compositor_reads_world, &gfx.world_rt, &gfx.tonemap);
            let do_ecra = ph2d_render::screen_color(
                gfx.surface.gpu(),
                mundo,
                gfx.vello_pass.intermediate_texture(),
                px,
                py,
            );
            // ⚠️ **O DIAGNÓSTICO da escolha** (`PH2D_PICK_LOG=1`) — ele imprime as DUAS respostas e
            // de que textura veio o mundo. *Um report de «não funciona» sobre um caminho que tem
            // duas fontes e dois modos não tem como ser diagnosticado por leitura.*
            if pick_log::armado() {
                pick_log::diz(
                    px,
                    py,
                    gfx.compositor_reads_world,
                    selection,
                    on_panel,
                    autorada,
                    do_ecra,
                    ph2d_render::read_texel(gfx.surface.gpu(), mundo, px, py),
                    ph2d_render::read_texel(
                        gfx.surface.gpu(),
                        gfx.vello_pass.intermediate_texture(),
                        px,
                        py,
                    ),
                );
            }
            let picked = autorada.or(do_ecra);
            if let Some([r, g, b, a]) = picked {
                hero.store
                    .set_blender_value(parent, ph2d_tokens::ColorValue::from_rgba8(r, g, b, a));
            }
            continue;
        }
        // O arrasto das CAMADAS do Painter (W3 T3.8): sobe a quem chama, que tem o
        // `ToolRegistry`, para ser aplicado na ferramenta activa. ⚠️ **Só esta família** — as
        // outras caem para o `apply_event` e são do painel que as desenhou.
        if let WidgetEvent::PanelRowReparent {
            family: PanelRowFamily::PainterLayer,
            dragged,
            drop,
        } = e
        {
            reparent = Some((dragged, drop));
            continue;
        }
        if !hero.apply_event(e)
            && !ph2d_editor_core::interaction::unhandled_is_expected(&e, &hero.store)
        {
            // O detector de SEAM MORTO: um widget pintado mas não-fiado aparece aqui primeiro.
            eprintln!("[hero] unhandled event: {e:?}");
        }
    }
    // Palette Import / Export: a click on the picker's button flagged a host file-I/O request (the
    // picker can't open files). Import REPLACES the active palette from a chosen file; Export saves
    // it. The format is the file extension — .gpl / .hex / .ase / .aco (via `ph2d_color::palette`).
    if let Some((parent, io_kind)) = hero.store.take_palette_io_pending() {
        handle_palette_io(hero, parent, io_kind);
    }
    // Cross-session persistence: the palette CRUD (new / delete / select / import / swatch edits) is
    // pointer-driven, so this pointer-dispatch hook catches every change. Cheap hash-gate → save only
    // when the set actually changed.
    persist::palettes_if_changed(hero);
    // Idem para as preferências de UI (carácter da UI viva + reduced motion): a escolha é um clique
    // numa row do pill Settings → Motion, logo este mesmo hook de ponteiro apanha-a.
    persist::prefs_if_changed(hero);
    reparent
}

/// ⭐⭐⭐ **O CLIQUE NO CANVAS SOLTA O TECLADO DO PAINEL** — o irmão de
/// [`forward_to_hero`] para quem TOMA o gesto e devolve antes dele.
///
/// # O report que a criou
///
/// Enio, 2026-09-07: *«a tecla del para deletar o mesh parou de funcionar e não temos undo/redo
/// para Cloth»*. **Os dois são o MESMO defeito, e nenhum é do tecido.** Tocar num chip numérico
/// do painel de escultura (são 37; o pincel de tecido acrescentou cinco) põe o foco do teclado
/// nele. Voltar ao barro NÃO o tirava: o `sculpt3d_pointer_down` toma o clique e devolve `return`
/// **antes** do [`forward_to_hero`], então a partida de foco do despachante nunca corria e
/// `focus_id` ficava preso naquele chip **para o resto da sessão**. Dali em diante o
/// `sculpt3d_key` recusa na primeira linha (`text_entry_focused`) e morrem, juntos, `Delete`,
/// `Ctrl+Z`, `Ctrl+Shift+Z` e todo atalho da cena 3D. (O 3D saiu — ADR-0179; a lei ficou com quem
/// ainda toma o gesto antes do despachante, a alça do gizmo de âncora.)
///
/// ⚠️ **A cura é chamar a MESMA lei, nunca repeti-la** — [`ph2d_editor_core::interaction::blur_focus`]
/// compromete o buffer numérico por confirmar, repõe o visual do widget e emite o `Blur`. Um
/// `set_focus(None)` à mão aqui perderia o número que o artista digitou e deixaria o campo a
/// desenhar o cursor de texto sem ter o teclado.
///
/// ⚠️ **Idempotente**: sem foco é um no-op exacto, e depois dela o bloco do `dispatch_down` não
/// tem o que fazer. É isso que a torna segura à frente de um consumidor que talvez não consuma.
pub fn forward_blur_to_hero(gfx: Option<&mut AppGfx>) {
    let Some(gfx) = gfx else {
        return;
    };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    // Mesmo idioma do irmão: fotografa antes de aplicar, porque o `apply_event`
    // muta o hero e a fatia vive na arena.
    let snapshot: Vec<WidgetEvent> = hero.blur_focus(&gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e)
            && !ph2d_editor_core::interaction::unhandled_is_expected(&e, &hero.store)
        {
            eprintln!("[hero] unhandled event: {e:?}");
        }
    }
}

/// Forward a translated [`KeyEvent`] (with editor-canonical
/// `keycode` from [`crate::keymap::winit_to_editor_keycode`]) into
/// the hero dispatcher so focused widgets see Tab/Enter/Backspace/
/// arrows etc. Also drains any clipboard copy/paste requests the
/// dispatcher set for this key event and bridges to `arboard`.
pub fn forward_key_to_hero(gfx: Option<&mut AppGfx>, event: KeyEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let snapshot: Vec<WidgetEvent> = hero.handle_key(event, &gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled key event: {e:?}");
        }
    }
    // Drain clipboard requests set by Cmd+C / Cmd+X / Cmd+V.
    if let Some(text) = hero.store.take_clipboard_copy()
        && let Some(cb) = gfx.clipboard.as_mut()
        && let Err(err) = cb.set_text(text)
    {
        eprintln!("[ph2d] clipboard set_text failed: {err}");
    }
    if let Some(target) = hero.store.take_clipboard_paste_request() {
        let text = gfx
            .clipboard
            .as_mut()
            .and_then(|cb| cb.get_text().ok())
            .unwrap_or_default();
        if !text.is_empty()
            && ph2d_editor_core::interaction::apply_clipboard_paste(&mut hero.store, target, &text)
        {
            // Mimic the TextChanged path so sliders/links update.
            let _ = hero.apply_event(WidgetEvent::TextChanged(target));
        }
    }
}

/// Forward a wheel / trackpad scroll into the hero dispatcher.
/// Routes to whichever panel registered its rect under the cursor.
pub fn forward_wheel_to_hero(gfx: Option<&mut AppGfx>, event: ph2d_host::WheelEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let _ = hero.handle_wheel_fisico(event, &gfx.hero_arena);
}

/// Forward a single printable character into the hero text-input
/// dispatcher (focused TextInput/NumberInput/Combobox buffer).
pub fn forward_text_to_hero(gfx: Option<&mut AppGfx>, ch: char) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let snapshot: Vec<WidgetEvent> = hero.handle_text_input(ch, &gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled text-input event: {e:?}");
        }
    }
}

/// `true` when `(x, y)` lies inside ANY panel rect published by the most recent
/// `paint_hero_screen` pass — the wheel then scrolls/zooms the panel instead of the camera.
///
/// ⭐ **Derived, never listed** (rolagem única, W3 — `docs/UI_New_and_Simple/spec/04_a_rolagem_unica.md`).
/// Until 2026-09-29 this was a hand-written `inside(ID) ||` chain of 26 ids, and it was the
/// fourth of the four edits a scrollable panel needed — the only one that did not fail loud.
/// It cost, in order: the Audio Mixer (07/09), the Asset Browser (30/08), the Model3D panel
/// (27/08), the Widget Lab (03/09), the Tags panel — and the Skeleton panel, which was still
/// missing on the day it died (the wheel over the bones ZOOMED the camera underneath).
/// Every panel publishes its rect while visible and clears it when it closes
/// (`clear_panel_rect`), so the published table IS the list — a new panel gets the wheel
/// without writing a line. Gate: `shells/desktop/tests/it/scrollable_panels_intercept_the_wheel.rs`.
///
/// Returns false when no hero is active — the demo's fixture mode shows raw sprites with no
/// panels, so the whole window is "canvas" and the wheel zooms the camera.
pub fn cursor_over_hero_panel(gfx: Option<&AppGfx>, x: f32, y: f32) -> bool {
    let Some(gfx) = gfx else { return false };
    let Some(hero) = gfx.hero_screen.as_ref() else {
        return false;
    };
    hero.chrome_panel_at(x, y).is_some()
}

/// ADR-0029 Phase C.2: resolve canvas-picked entity bits to a live
/// Hierarchy entry via the panel-owned thread-local snapshot. Takes
/// the `hero_live` field directly (not `&AppGfx`) so the caller can
/// keep a `&mut gfx.hero_screen` borrow live alongside this read.
#[cfg(feature = "panel-hierarchy")]
pub(crate) fn resolve_live_entry(
    hero_live: Option<&crate::HeroLive>,
    picked: Option<u64>,
) -> Option<ph2d_editor_core::screens::hero::fixture::HierarchyEntity> {
    let node = hero_live?.bridge.node_for(picked?)?;
    ph2d_panel_hierarchy::current_live_entries()?
        .get(&node)
        .cloned()
}
#[cfg(not(feature = "panel-hierarchy"))]
pub(crate) fn resolve_live_entry(
    _hero_live: Option<&crate::HeroLive>,
    _picked: Option<u64>,
) -> Option<ph2d_editor_core::screens::hero::fixture::HierarchyEntity> {
    None
}
