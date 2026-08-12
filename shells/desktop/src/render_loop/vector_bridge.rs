//! Vector tool ⟷ shell bridge (ADR-0108 cutover).
//!
//! Per-frame jobs (mirror of the Padding / Painter panel bridges):
//!
//! 1. **Panel visibility** — show the docked `ph2d-panel-vector` Style panel
//!    (right dock) iff the `vector` tool is active; hide the real Inspector
//!    (edge-triggered) so they don't both claim the slot.
//! 2. **Picker read-back** — a Down on the Stroke / Fill swatch opened the
//!    shared OKLCH picker (generic `is_picker_swatch` dispatch). While the
//!    picker targets one of our swatches, feed the live picked colour into the
//!    tool via [`VectorTool::set_stroke_rgba`] / [`VectorTool::set_fill_rgba`].
//! 3. **Style sync** — copy the tool's stroke / fill / width into the Pen so
//!    newly drawn paths honour the Style.
//! 4. **Recolour selected** — when a colour changed (`take_apply_to_selected`),
//!    recolour the selected path. ONE undo step per gesture: a picker drag
//!    commits on close; a discrete pick (Fill "None") commits the same frame.
//! 5. **Publish** — sync the swatches' `widget_color` to the live colour (seeds
//!    the picker on open) + publish the Style snapshot the panel paints.
//!
//! The concrete-tool downcast lives HERE (allowlisted:
//! `architecture_no_downcast_to_concrete_tool_in_shell`), so the central render
//! loop stays downcast-free — mirror of `painter_bridge`.

use ph2d_editor::{HeroScreen, ToolId, ToolRegistry};
use ph2d_tool_vector::VectorDrawConfig;
use ph2d_vec_edit::{History, PenStyle, PenTool, ShapeTool};
use ph2d_vec_render::GradHandle;
use ph2d_vec_scene::{Paint, VecScene};

/// O estilo do traço (o registro que o painel edita, o detector e o escritor — juntos de
/// propósito). Módulo irmão pelo teto de LOC; o doc dele explica por que os três não se
/// separam.
#[path = "vector_bridge_style.rs"]
mod style;
use style::{
    RECOLOR_PRE, rgba, seed_style_from_selection, selected_grad_color, set_selected_grad_color,
    sync_opacity_slider,
};
/// ⚠️ `StrokeStyle` sai junto porque o gate de CONSEQUÊNCIA do [`crate::vec_selection`]
/// **restiliza de verdade** em vez de contar caminhos — o artista vê cores, não listas.
pub(crate) use style::{StrokeStyle, restyle_selected_strokes};

/// Troca o modo de desenho da tool Vector (a tool é a dona; o shell só espelha). O
/// downcast fica confinado a este bridge (allowlist da gate
/// `no_downcast_to_concrete_tool_in_shell`); o resto do shell chama por aqui. No-op se
/// a tool Vector não está no registry.
pub(crate) fn set_mode(tools: &mut ToolRegistry, mode: ph2d_tool_vector::DrawMode) {
    if let Some(tool) = tools.tool_by_id_mut(&ToolId::new("vector")).and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_vector::VectorTool>()
    }) {
        tool.set_mode(mode);
    }
}

/// A tool ADOTA os parâmetros (unidade de UI) da forma `kind`. A shell chama isto quando
/// o usuário seleciona uma forma VIVA: eles viram os correntes DAQUELA forma, então o
/// painel (que pinta a partir da tool) para de mentir e a próxima desenhada os herda.
///
/// Downcast confinado a este bridge, como o [`set_mode`].
/// **O que o CATÁLOGO oferece agora** — o tipo activo da tool e os valores que ela guarda para
/// ele (já em unidade de UI).
///
/// Existe para o sítio de decisão da semente não repetir o downcast: uma 2ª cópia dele é uma 2ª
/// resposta a *"quem é a ferramenta de vetor?"*. `None` = a tool não está em cena, e aí não há
/// catálogo nenhum a mostrar.
pub(crate) fn shape_catalog(
    tools: &mut ToolRegistry,
) -> Option<(ph2d_vec_scene::ShapeKind, ph2d_vec_scene::ShapeValues)> {
    let tool = tools.tool_by_id_mut(&ToolId::new("vector")).and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_vector::VectorTool>()
    })?;
    let k = tool.shape();
    Some((k, tool.shape_values(k)))
}

pub(crate) fn adopt_shape_values(
    tools: &mut ToolRegistry,
    kind: ph2d_vec_scene::ShapeKind,
    values: ph2d_vec_scene::ShapeValues,
) {
    if let Some(tool) = tools.tool_by_id_mut(&ToolId::new("vector")).and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_vector::VectorTool>()
    }) {
        tool.adopt_shape_values(kind, values);
    }
}

/// Per-frame Vector-tool plumbing. Safe to call every frame; a no-op when the
/// Vector tool is absent from the registry.
/// Returns the tool's current [`VectorDrawConfig`] so the shell can mirror it
/// into `App` (the input dispatch reads it to route canvas gestures + size the
/// shapes without a downcast). Defaults when the Vector tool is absent.
#[allow(clippy::too_many_arguments)] // per-frame bridge inputs, each distinct
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    scene: &mut VecScene,
    pen: &mut PenTool,
    shape: &mut ShapeTool,
    // **O LÁPIS** — o 3º produtor de geometria a receber o Style corrente. Está aqui porque um
    // traço de mão livre nasce com o mesmo traço/preenchimento que a caneta e a forma: uma 2ª
    // fonte de estilo faria o mesmo pincel desenhar diferente em modos vizinhos.
    pencil: &mut ph2d_vec_edit::Pencil,
    history: &mut History,
    // World units per screen pixel (from the camera) — converts the tool's px
    // stroke width into the path's world-space width when restyling.
    px_to_world: f64,
    // Selected gradient handle (a multi-point point OR a linear/radial endpoint) —
    // the Fill swatch shows its colour and the picker recolours THAT slot instead of
    // replacing the whole fill with a solid colour.
    grad_handle: Option<GradHandle>,
    // Onde cada path ESTÁ (ADR-0111): o readout de posição/tamanho é em MUNDO.
    xforms: &ph2d_vec_scene::VecXforms,
    // O mundo ECS + a ponte path↔entidade: é neles que mora o `VecConnector` da linha
    // selecionada (a seção Connector do painel lê daqui).
    sim: &ph2d_ecs::SimWorld,
    vec_entities: &crate::vec_entities::VecEntityMap,
    // "Set Center" armado (ADR-0112): só muda o rótulo do botão.
    pivot_edit: bool,
    // Whether the transform gizmo's "Set Center" pivot-edit mode is armed.
    // Os ajustes de snap do módulo, espelhados na seção Snap do painel. ⚠️ Passa o CONJUNTO e
    // não um `bool` por interruptor: a lista já é longa, e a W6 acrescentou dois — o próximo
    // custaria mais uma posição numa assinatura que ninguém lê ao chamar. A GRADE não está
    // aqui (ela é do painel universal de Grid Snap).
    snap: crate::vec_snap::VecSnapSettings,
) -> VectorDrawConfig {
    let vector_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("vector"));

    // ── 1. Panel visibility (mirror of the Padding dock takeover) ─────────
    hero.panel_visibility.insert("vector", vector_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(vector_active, Ordering::Relaxed);
        if was != vector_active {
            hero.panel_visibility.insert("inspector", !vector_active);
        }
    }

    // The tool persists in the registry whether or not it is active, so its
    // Style survives tool switches (mirror of the painter bridge).
    let Some(tool) = tools.tool_by_id_mut(&ToolId::new("vector")).and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_vector::VectorTool>()
    }) else {
        #[cfg(feature = "panel-vector")]
        ph2d_panel_vector::set_current_vector_style(None);
        return VectorDrawConfig::default();
    };

    // ── 2. Picker read-back: which swatch is the picker targeting? ────────
    let target = hero.store.picker_target();
    let stroke_open = target == Some(ph2d_editor::ids::VECTOR_STROKE_SWATCH);
    let fill_open = target == Some(ph2d_editor::ids::VECTOR_FILL_SWATCH);
    if (stroke_open || fill_open)
        && let Some((value, _, _, _)) = hero
            .store
            .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
    {
        // The picker owns RGB **and alpha**: its alpha flows into the tool's
        // stroke/fill alpha, and the bridge pushes that back onto the Opacity
        // slider each frame (below) so the picker's alpha and the panel's
        // Opacity stay in sync (Enio 2026-07-07).
        let picked = value.rgba;
        if stroke_open {
            tool.set_stroke_rgba(picked);
        } else {
            tool.set_fill_rgba(picked);
        }
    }

    // O ESTILO do caminho selecionado vira o corrente quando a seleção muda (antes de qualquer
    // leitura do Style abaixo, senão o frame da seleção ainda pintaria o antigo).
    //
    // ⚠️ A largura viaja em px de TELA na tool e em MUNDO no documento; a conversão tem um dono
    // só, e é este (`px_to_world` é o recíproco).
    seed_style_from_selection(
        tool,
        &mut hero.store,
        pen,
        scene,
        if px_to_world > 0.0 {
            1.0 / px_to_world
        } else {
            0.0
        },
    );

    // **Uma cor autorada SOLTA o token** (plano UI/UX W4a) — o one-shot do tool, drenado aqui e
    // consumido pelo passe que tem o mundo e a seleção na mão.
    let (fill_authored, stroke_authored) = tool.take_colour_authored();
    if fill_authored {
        crate::vec_bindings::note_authored(ph2d_ecs::BoundProp::Fill);
    }
    if stroke_authored {
        crate::vec_bindings::note_authored(ph2d_ecs::BoundProp::StrokeColor);
    }
    let stroke = tool.stroke_rgba();
    let fill = tool.fill_rgba();
    let cap = line_cap(tool.cap());
    let join = line_join(tool.join());
    // Sem `match` de conversão: o alinhamento usa o tipo do DOCUMENTO (o precedente das
    // PONTAS), então não há tabela de três braços a manter em dia como em cap/join.
    let align = tool.stroke_align();
    let (marker_start, marker_end) = (tool.marker_start(), tool.marker_end());
    // Tamanho da cabeça + arredondamento das quinas dela: Style, como as pontas — valem para
    // o próximo caminho desenhado (via `PenStyle`) E para TODOS os selecionados (abaixo).
    let (marker_scale, marker_round) = (tool.marker_scale(), tool.marker_round());
    // Dash + gap are MULTIPLES of the stroke width (width-aware) — the render
    // scales them by the path's own width, so no px→world conversion here.
    // `dash = 0` ⇒ solid; otherwise `(dash, gap)` sizes the dash and the space.
    let dash = (tool.dash() > 0.0).then_some((tool.dash(), tool.gap()));

    // ── 3. New paths honour the tool's Style (pen + shape share it). ──────
    let style = PenStyle {
        stroke: rgba(stroke),
        stroke_w_px: tool.stroke_width_px(),
        fill: rgba(fill),
        cap,
        join,
        align,
        dash,
        marker_start,
        marker_end,
        marker_scale,
        marker_round,
    };
    pen.set_style(style);
    shape.set_style(style);
    pencil.set_style(style);
    // A Fidelity autorada (a tolerância do decimador) vem pela MESMA rota do estilo: o tool é o
    // dono do valor, e o `Pencil` é quem o consome. O estabilizador NÃO passa aqui — ele é aplicado
    // na ENTRADA, por movimento de ponteiro, e viaja no `VectorDrawConfig` que o `input_dispatch` lê.
    pencil.set_fidelity_px(tool.pencil_fidelity_px());

    // ── 4. Restyle the selected path — colour + width (undoable, one step per
    //    gesture). A width-slider DRAG is a gesture like a picker drag, so scope
    //    its undo the same way (one step per drag).
    //
    // ⚠️ **Duas perguntas diferentes, e elas dividiam uma resposta.** *"Há um gesto em curso?"*
    // (para AGRUPAR o undo) é sobre um arrasto, e o estado do slider responde certo. *"A largura
    // foi autorada?"* (para ESCREVÊ-LA na seleção) não é sobre arrasto nenhum — a caixa numérica
    // ao lado autora pelo mesmo `SetValue(VECTOR_WIDTH)` e nunca põe o slider em `Dragging`, então
    // enquanto as duas partilhavam `width_dragging` digitar um número mudava o tool e **não mudava
    // a forma selecionada** (Enio 2026-08-01). Quem sabe a segunda é o TOOL, que recebeu o evento.
    let width_dragging = matches!(
        hero.store.slider(ph2d_editor::ids::VECTOR_WIDTH),
        Some((ph2d_editor::widget::SliderState::Dragging, _))
    );
    let width_authored = tool.take_width_authored();
    // **Digitar uma espessura SOLTA o token dela** (W4c.4) — a mesma lei da cor, pelo mesmo canal.
    if width_authored {
        crate::vec_bindings::note_authored(ph2d_ecs::BoundProp::StrokeWidth);
    }
    let session = stroke_open || fill_open || width_dragging;
    // The selected gradient handle, kept only if it still addresses a colour on the
    // current fill (a stale handle after a kind switch resolves to `None` and falls
    // through to the solid-fill path). The picker recolours THIS slot.
    let active_handle = grad_handle.filter(|&h| {
        pen.selected()
            .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
            .and_then(|p| p.fill.as_ref())
            .and_then(|f| selected_grad_color(f, h))
            .is_some()
    });
    if tool.take_apply_to_selected() {
        // Restyle EVERY selected path, not just the primary — a multi-selection
        // (marquee, Shift+click, or all the glyphs of a text block) recolours as one
        // (Enio 2026-07-11). A gradient-handle recolour is the one exception: the
        // handle addresses the PRIMARY path's fill slot, so it stays single-path.
        let sel_ids: Vec<ph2d_vec_scene::VecPathId> = if active_handle.is_some() {
            pen.selected().into_iter().collect()
        } else {
            pen.selected_paths().to_vec()
        };
        let new_stroke = rgba(stroke);
        let new_fill = if fill[3] == 0 { None } else { Some(rgba(fill)) };
        let new_w = tool.stroke_width_px() * px_to_world;
        // A ficha ÚNICA do traço: a mesma que detecta a mudança e a que a grava (as pontas,
        // o tamanho e o arredondamento delas viajam aqui — um campo só num dos dois lados
        // seria um controle que mexe no número e não muda nada na tela).
        let stroke_style = StrokeStyle {
            color: new_stroke,
            cap,
            join,
            align,
            dash,
            marker_start,
            marker_end,
            marker_scale,
            marker_round,
        };
        let differs = |p: &ph2d_vec_scene::VecPath| {
            let stroke_differs = p.stroke.is_some_and(|s| {
                stroke_style.differs_from(&s)
                    || (width_authored && (s.width - new_w).abs() > f64::EPSILON)
            });
            let fill_differs = if let Some(h) = active_handle {
                // Recolour the selected gradient slot (point / ramp stop).
                p.fill.as_ref().and_then(|f| selected_grad_color(f, h)) != Some(rgba(fill))
            } else {
                // No gradient handle selected → a fill pick only replaces a Solid /
                // None fill; it must NEVER clobber a gradient (use Fill Type → Solid
                // for that). Guards the linear/radial→solid regression (Enio 2026-07-08).
                p.closed
                    && matches!(p.fill, None | Some(Paint::Solid(_)))
                    && p.fill.as_ref().map(Paint::primary_color) != new_fill
            };
            stroke_differs || fill_differs
        };
        let will_change = sel_ids.iter().any(|&id| {
            scene
                .paths()
                .iter()
                .find(|p| p.id == id)
                .is_some_and(&differs)
        });
        if will_change {
            RECOLOR_PRE.with(|c| {
                if c.borrow().is_none() {
                    *c.borrow_mut() = Some(scene.clone());
                }
            });
            // O TRAÇO de todos os selecionados, de uma vez (a largura só acompanha quando FOI
            // autorada — arrastando o slider ou digitando na caixa; uma escolha de cor nunca pode
            // reengrossar a linha, e é o `None` daqui que a impede).
            restyle_selected_strokes(
                scene,
                &sel_ids,
                &stroke_style,
                width_authored.then_some(new_w),
            );
            for &id in &sel_ids {
                let Some(path) = scene.path_mut(id) else {
                    continue;
                };
                if let Some(h) = active_handle {
                    // Recolour the selected gradient slot (never converts to solid).
                    if let Some(paint) = path.fill.as_mut() {
                        set_selected_grad_color(paint, h, rgba(fill));
                    }
                } else if path.closed && matches!(path.fill, None | Some(Paint::Solid(_))) {
                    // Otherwise a fill pick sets a SOLID fill — but only over a solid /
                    // empty fill, never over a gradient.
                    path.fill = new_fill.map(Paint::solid);
                }
            }
        }
    }
    // Commit the gesture's undo when it ends (no picker / width-drag session):
    // a discrete pick (None) commits immediately; a drag commits on release.
    if !session {
        RECOLOR_PRE.with(|c| {
            if let Some(pre) = c.borrow_mut().take() {
                history.push_undo(pre);
            }
        });
    }

    // ── 5. Sync swatch colours (seeds the picker on open) + Opacity sliders
    //    (so a picker alpha shows on the panel) + publish. ──────────────────
    hero.store
        .set_widget_color(ph2d_editor::ids::VECTOR_STROKE_SWATCH, stroke);
    // The Fill swatch shows the selected gradient point's colour (so the picker
    // opens seeded on it) when a MultiPoint point is selected, else the tool fill.
    let fill_swatch_col = active_handle
        .and_then(|h| {
            pen.selected()
                .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
                .and_then(|p| p.fill.as_ref())
                .and_then(|f| selected_grad_color(f, h))
                .map(|c| [c.r, c.g, c.b, c.a])
        })
        .unwrap_or(fill);
    hero.store
        .set_widget_color(ph2d_editor::ids::VECTOR_FILL_SWATCH, fill_swatch_col);
    // Push the tool's alpha onto the Opacity sliders (unless being dragged) so
    // an alpha set in the colour picker reflects on the panel, and vice-versa.
    sync_opacity_slider(
        &mut hero.store,
        ph2d_editor::ids::VECTOR_STROKE_OPACITY,
        stroke[3],
    );
    sync_opacity_slider(
        &mut hero.store,
        ph2d_editor::ids::VECTOR_FILL_OPACITY,
        fill[3],
    );
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_vector_style(if vector_active {
        Some(tool.ui_snapshot())
    } else {
        None
    });
    // Publish the selected vertex's type so the panel shows the Vertex section
    // (Corner/Smooth/Symmetric) + highlights the active one. `None` hides it.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_selected_vertex_type(if vector_active {
        pen.selected_vertex_kind(scene).map(vertex_sel_of)
    } else {
        None
    });
    // **Quantos** nós estão selecionados — o tipo acima diz *uniforme ou misto*, não a contagem, e
    // o Average precisa exactamente dela (com um nó só não há o que mediar).
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_vertex_count(if vector_active {
        pen.selected_verts().len()
    } else {
        0
    });
    // **Existe LÂMINA?** — o fato que decide se os dois botões do corte são oferecidos. A verdade
    // mora no ECS (`VecCutPath`); isto é a projeção, como toda a fronteira deste painel.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_cut_line_exists(
        vector_active && crate::vec_cut_line::cut_line(sim, vec_entities).is_some(),
    );
    // NOTA: o estilo de quina não é mais publicado para um toggle na seção Vertex — ele virou o
    // par de ferramentas Fillet / Chamfer (o SINAL do `corner_radius` é escrito pelo arrasto).

    // Publish the selected path's anchor bbox `[x, y, w, h]` so the panel shows + seeds the
    // numeric Transform fields. `None` hides the section.
    //
    // ⚠️ **Os quatro números cruzam a fronteira de DISPLAY aqui**, e a razão é que este painel
    // era a QUARTA superfície a responder *onde está esta coisa?* — a régua, o Inspector e o
    // painel de Grid Snap já convertiam, e este publicava metros de mundo: com os defaults
    // (100 px/m, Pixels) os três diziam `150` e este dizia `1.5`.
    //
    // Posição e tamanho atravessam pela MESMA porta porque a conversão é uma escala pura (sem
    // deslocamento) — `x` e `w` não precisam de leis diferentes.
    #[cfg(feature = "panel-vector")]
    {
        let display = ph2d_editor::LengthDisplay::of(&hero.project);
        ph2d_panel_vector::set_length_suffix(display.suffix());
        ph2d_panel_vector::set_current_transform(if vector_active {
            pen.selected()
                .and_then(|sel| scene.path_world_curve_bbox(xforms, sel))
                .map(|(lo, hi)| {
                    [
                        display.value(lo[0]),
                        display.value(lo[1]),
                        display.value(hi[0] - lo[0]),
                        display.value(hi[1] - lo[1]),
                    ]
                })
        } else {
            None
        });
    }

    // **O CONECTOR selecionado** — a seção Connector do painel (Route / Jetty / Spread).
    // Publica os valores **EFETIVOS** (o automático, quando o usuário não fixou nada);
    // `None` ⇒ nenhum conector na seleção ⇒ a seção inteira some. O corpo mora no módulo
    // dono do assunto (teto de 600 LOC por arquivo da shell, HR-18).
    crate::vec_connector_panel::publish(
        sim,
        vec_entities,
        scene,
        xforms,
        pen.selected_paths(),
        vector_active,
    );

    // Publish the object-selection path count so the panel shows Align (≥2) /
    // Distribute (≥3).
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_selection_count(if vector_active {
        pen.selected_paths().len()
    } else {
        0
    });
    // Publish the pivot-edit ("Set Center") armed state for the button label.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_pivot_edit(vector_active && pivot_edit);
    // Publish shape-snapping so the Snap section reflects (and drives) it.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_snap(snap.on);
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_snap_position(snap.path, snap.crossings);
    // ⚠️ A régua vem do HERO, não do `snap`: ela é chrome de canvas (aparece com qualquer
    // ferramenta) e o seu dono é a vista, não a ferramenta vetorial. O painel só a alcança.
    ph2d_panel_vector::set_current_guides(snap.guides, hero.view.rulers_visible);

    // Publish the selected path's fill rule — `Some` ONLY when it is a compound
    // path, since with a single contour both rules paint identically and the row
    // would be a no-op control.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_fill_rule(
        vector_active
            .then(|| pen.selected())
            .flatten()
            .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
            .filter(|p| p.is_compound())
            .map(|p| match p.fill_rule {
                ph2d_vec_scene::FillRule::NonZero => ph2d_panel_vector::PathFillRule::NonZero,
                ph2d_vec_scene::FillRule::EvenOdd => ph2d_panel_vector::PathFillRule::EvenOdd,
            }),
    );

    // Publish the selected path's closed flag so the panel labels the toggle
    // "Close Path" / "Open Path" correctly.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_path_closed(if vector_active {
        pen.selected()
            .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
            .map(|p| p.closed)
    } else {
        None
    });

    // Publish the selected path's fill kind (+ linear angle) so the Fill-type
    // selector reflects + drives it.
    #[cfg(feature = "panel-vector")]
    {
        use ph2d_panel_vector::FillKind;
        use ph2d_vec_scene::Paint;
        let (kind, angle) = if vector_active {
            match pen
                .selected()
                .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
                .and_then(|p| p.fill.as_ref())
            {
                Some(Paint::Solid(_)) => (Some(FillKind::Solid), None),
                Some(Paint::Linear { start, end, .. }) => {
                    // Angle of the ramp direction, normalized to [0, 360).
                    let mut deg = (end[1] - start[1]).atan2(end[0] - start[0]).to_degrees();
                    if deg < 0.0 {
                        deg += 360.0;
                    }
                    (Some(FillKind::Linear), Some(deg))
                }
                Some(Paint::Radial { .. }) => (Some(FillKind::Radial), None),
                Some(Paint::MultiPoint { .. }) => (Some(FillKind::MultiPoint), None),
                None => (None, None),
            }
        } else {
            (None, None)
        };
        ph2d_panel_vector::set_current_fill(kind, angle);
        // Publish the selected multi-point point's influence + jitter (drive the sliders).
        let sel_point = active_handle.and_then(GradHandle::point).and_then(|i| {
            pen.selected()
                .and_then(|sel| scene.paths().iter().find(|p| p.id == sel))
                .and_then(|p| match &p.fill {
                    Some(Paint::MultiPoint { points }) => points.get(i).copied(),
                    _ => None,
                })
        });
        ph2d_panel_vector::set_current_grad_influence(sel_point.map(|gp| gp.influence));
        ph2d_panel_vector::set_current_grad_jitter(sel_point.map(|gp| gp.jitter));
    }

    // Calibrate the Transform fields' drag scrub to the camera: value-units per cursor pixel ⇒
    // dragging a chip N px moves the shape N px on screen at any zoom (unbounded — no clamp).
    // Live each frame so zoom in/out keeps the 1:1 feel.
    //
    // ⚠️ **A TAXA cruza a MESMA fronteira que o valor, e esquecê-la é o defeito que compila.**
    // Ela é *comprimento por pixel de cursor*, logo é um comprimento — com o valor em pixels de
    // display e a taxa em metros de mundo, arrastar um chip um pixel moveria o número em `0,01`
    // enquanto ele mostra centenas: o chip pareceria travado. Uma porta, os dois lados.
    if vector_active {
        let px_to_world = ph2d_editor::LengthDisplay::of(&hero.project).value(px_to_world);
        for id in [
            ph2d_editor::ids::VECTOR_TRANSFORM_X,
            ph2d_editor::ids::VECTOR_TRANSFORM_Y,
            ph2d_editor::ids::VECTOR_TRANSFORM_W,
            ph2d_editor::ids::VECTOR_TRANSFORM_H,
        ] {
            hero.store.set_number_drag_rate(id, px_to_world);
        }
        // The Angle (R) field is in DEGREES, not world units — a fixed, gentle
        // scrub (a full drag across the screen ≈ a couple turns), zoom-independent.
        const ROT_DRAG_DEG_PER_PX: f64 = 0.5;
        hero.store
            .set_number_drag_rate(ph2d_editor::ids::VECTOR_TRANSFORM_R, ROT_DRAG_DEG_PER_PX);
    }

    // Mirror the tool's mode + shape params so the input dispatch can route
    // canvas gestures (pen vs shape) + size the shapes without a downcast.
    tool.draw_config()
}

#[cfg(test)]
#[path = "vector_bridge_tests.rs"]
mod tests;

/// A TRADUÇÃO do vocabulário tool ⟷ documento — módulo irmão pelo teto de 600 LOC (HR-18).
#[path = "vector_bridge_vocab.rs"]
mod vocab;
use vocab::{line_cap, line_join, vertex_sel_of};
