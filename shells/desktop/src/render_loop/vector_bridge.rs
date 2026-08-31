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
use ph2d_vec_scene::VecScene;

/// O estilo do traço (o registro que o painel edita, o detector e o escritor — juntos de
/// propósito). Módulo irmão pelo teto de LOC; o doc dele explica por que os três não se
/// separam.
#[path = "vector_bridge_style.rs"]
mod style;
use style::{
    RECOLOR_PRE, apply_fill_colour, rgba, seed_style_from_selection, selected_grad_color,
    set_selected_grad_color,
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
    // ⚠️ **O kind EFECTIVO do modo, não o botão aceso do catálogo.** Sem seleção, estes são os
    // campos que o painel semeia — e no modo Moldura os do catálogo não descrevem nada do que o
    // gesto vai desenhar. Não há contradição a criar: o botão do catálogo só acende quando
    // `snap.mode == DrawMode::Shape` (`paint_catalog`), então a Moldura já não reivindica nenhum.
    // É o que deixa o raio ser autorado ANTES do primeiro arrasto, e não só depois de selecionar.
    let k = tool
        .mode()
        .shape_kind(tool.shape())
        .unwrap_or_else(|| tool.shape());
    Some((k, tool.shape_values(k)))
}

/// Drena o *"o artista acabou de escolher uma forma no catálogo"* da tool
/// (`VectorTool::take_shape_armed`). `false` quando a tool não está em cena.
///
/// Downcast confinado a este bridge, como o [`set_mode`].
pub(crate) fn take_shape_armed(tools: &mut ToolRegistry) -> bool {
    tools
        .tool_by_id_mut(&ToolId::new("vector"))
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<ph2d_tool_vector::VectorTool>()
        })
        .is_some_and(ph2d_tool_vector::VectorTool::take_shape_armed)
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
    // ⭐ O CADEADO de proporção do padrão (plano 33 W10). Ele é da SESSÃO — descreve o gesto,
    // não o padrão —, então mora na shell e só atravessa aqui para o painel o desenhar.
    texpat_lock: [bool; 2],
    texpat_gap_link: [bool; 2],
    // ⭐ Os ladrilhos assados deste quadro (plano 33 W10) — atravessam só para o painel poder
    // dizer que uma arte **não encaixa consigo própria**. Passa o MAPA e não um `bool` por tinta
    // pela mesma razão que o `snap` passa o conjunto: o par (forma, tinta) resolve-se lá dentro,
    // onde ele já existe.
    texpat_tiles: &ph2d_vec_render::PatternTiles,
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
        // ⭐ **A cor de preenchimento viaja CRUA daqui para baixo** (W6). Ela era pré-digerida num
        // `Option<Rgba8>` — `alfa 0 => None` —, o que era a convenção do SÓLIDO aplicada antes de
        // saber a espécie da tinta; num padrão a mesma leitura destruiria a grade e o ladrilho.
        // *Uma decisão tomada antes de conhecer o sujeito é uma decisão tomada pelo caso errado.*
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
            let stroke_differs = p.stroke.as_ref().is_some_and(|s| {
                stroke_style.differs_from(s)
                    || (width_authored && (s.width - new_w).abs() > f64::EPSILON)
            });
            let fill_differs = if let Some(h) = active_handle {
                // Recolour the selected gradient slot (point / ramp stop).
                p.fill.as_ref().and_then(|f| selected_grad_color(f, h)) != Some(rgba(fill))
            } else {
                // ⭐ Sem alça de gradiente, o que um pick faz depende da ESPÉCIE da tinta, e a lei
                // inteira vive numa porta só (`apply_fill_colour`): gradiente intocável, padrão
                // recolorido e desvanecido, sólido/vazio substituído. Perguntar é aplicá-la num
                // clone — *a decisão escrita aqui e no escritor eram duas leis a manter iguais à
                // mão, e a do padrão faltava numa delas.*
                let mut sondagem = p.fill.clone();
                apply_fill_colour(&mut sondagem, p.closed, rgba(fill))
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
                } else {
                    // A MESMA porta que o `differs` sondou — nunca uma segunda cópia da decisão.
                    let closed = path.closed;
                    apply_fill_colour(&mut path.fill, closed, rgba(fill));
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

    // ── 5. **Publicar** — o que o painel mostra sobre a cena e a seleção.
    //
    // ⚠️ Esta primeira publicação fica AQUI porque é a única que lê a TOOL (`ui_snapshot`),
    // que é um empréstimo mútuo do registry; o resto do passo 5 mora no módulo irmão.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_vector_style(if vector_active {
        Some(tool.ui_snapshot())
    } else {
        None
    });
    // O resto do passo 5 (as fileiras, o Transform, o snap, o preenchimento e a régua do
    // arrasto) mora no irmão — teto de 600 LOC da shell (HR-18), cortado pela linha que o
    // doc-header já enumerava: *o que o frame FAZ* × *o que o painel é TOLD*.
    publish::publish(
        hero,
        scene,
        pen,
        xforms,
        sim,
        vec_entities,
        vector_active,
        active_handle,
        stroke,
        fill,
        px_to_world,
        pivot_edit,
        snap,
        texpat_lock,
        texpat_gap_link,
        texpat_tiles,
    );

    // Mirror the tool's mode + shape params so the input dispatch can route
    // canvas gestures (pen vs shape) + size the shapes without a downcast.
    tool.draw_config()
}

#[cfg(test)]
#[path = "vector_bridge_opacity_tests.rs"]
mod opacity_tests;

#[cfg(test)]
#[path = "vector_bridge_tests.rs"]
mod tests;

/// A TRADUÇÃO do vocabulário tool ⟷ documento — módulo irmão pelo teto de 600 LOC (HR-18).
#[path = "vector_bridge_vocab.rs"]
mod vocab;
use vocab::{line_cap, line_join};

/// **O que o painel é TOLD** (passo 5) — irmão pelo mesmo teto, cortado pela linha que o
/// doc-header acima já enumerava: os passos 1-4 mexem no documento, o 5 só o descreve.
#[path = "vector_bridge_publish.rs"]
mod publish;
