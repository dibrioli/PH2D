//! **O que o painel é TOLD** — o passo 5 do bridge do vetor (o `dispatch` faz os passos 1-4).
//!
//! # Por que este corte, e não outro
//!
//! O doc-header do pai já enumerava cinco trabalhos, e os quatro primeiros MEXEM no documento
//! (visibilidade do dock · leitura do picker · o Style que a caneta herda · o restyle da
//! seleção) enquanto o quinto só **conta ao painel o que já é verdade**. Quando o arquivo
//! cruzou o teto de 600 LOC (HR-18) essa era a linha de corte que já estava escrita.
//!
//! ⚠️ **Uma publicação ficou no pai, e não por acidente:** `set_current_vector_style` lê o
//! `ui_snapshot()` da TOOL, e a tool é um empréstimo mútuo do registry que este módulo não
//! recebe. Ela roda **antes** desta função; a ordem entre as duas é irrelevante (uma escreve
//! num estático do painel, a outra no store e noutros estáticos), e é por isso que separá-las
//! é seguro.
//!
//! ⚠️ **A fronteira de DISPLAY mora aqui** (doc 88 do Motion, e a lição que este painel pagou
//! por ser a QUARTA superfície a responder *onde está esta coisa?*): todo comprimento sai
//! UMA vez pela `LengthDisplay`, e a TAXA de arrasto cruza a mesma porta que o valor.

use ph2d_editor::HeroScreen;
use ph2d_vec_edit::PenTool;
use ph2d_vec_render::GradHandle;
use ph2d_vec_scene::VecScene;

use super::style::{selected_grad_color, sync_opacity_slider};
use super::vocab::vertex_sel_of;

/// Empurra para o painel tudo o que ele mostra sobre a cena e a seleção correntes.
///
/// Chamada uma vez por frame pelo [`super::dispatch`], **depois** de os passos 1-4 terem
/// deixado o documento no estado que este passe descreve.
#[allow(clippy::too_many_arguments)] // per-frame publish inputs, each a distinct fact
pub(super) fn publish(
    hero: &mut HeroScreen,
    scene: &VecScene,
    pen: &PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    sim: &ph2d_ecs::SimWorld,
    vec_entities: &crate::vec_entities::VecEntityMap,
    // A ferramenta Vector está em mãos? Fora dela o painel não pinta nada disto.
    vector_active: bool,
    // A alça de gradiente que ainda endereça uma cor no preenchimento corrente.
    active_handle: Option<GradHandle>,
    stroke: [u8; 4],
    fill: [u8; 4],
    // Unidades de mundo por pixel de cursor (da câmera) — a régua do arrasto dos chips.
    px_to_world: f64,
    pivot_edit: bool,
    snap: crate::vec_snap::VecSnapSettings,
    // O cadeado de proporção do padrão — estado de SESSÃO da shell (ver `dispatch`).
    texpat_lock: bool,
) {
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
    // Publish the selected vertex's type so the panel shows the Vertex section
    // (Corner/Smooth/Symmetric) + highlights the active one. `None` hides it.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_selected_vertex_type(if vector_active {
        pen.selected_vertex_kind(scene).map(vertex_sel_of)
    } else {
        None
    });
    // **ONDE o nó está** — a mediana das âncoras selecionadas, em MUNDO e na unidade do artista.
    //
    // ⚠️ A conversão mundo→display é a MESMA porta da bbox do Transform, três linhas acima: os
    // dois readouts descrevem o mesmo canvas, e um deles em metros enquanto o outro está em
    // pixels seria a quinta superfície a discordar.
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_vertex_pos(if vector_active {
        let d = ph2d_editor::LengthDisplay::of(&hero.project);
        pen.selected_anchor_world(scene)
            .map(|p| [d.value(p[0]), d.value(p[1])])
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
                Some(Paint::Pattern(_)) => (Some(FillKind::Pattern), None),
                None => (None, None),
            }
        } else {
            (None, None)
        };
        ph2d_panel_vector::set_current_fill(kind, angle);
        // ⭐⭐ **UMA LEI POR TINTA** (plano 35, wave F; Enio 2026-08-28: *"cada seção deve ter seus
        // ajustes próprios"*). `None` numa tinta esconde a secção DELA, e só a dela.
        //
        // ⛔ **Não há mais ALVO a resolver.** A wave D publicava a lei *do sujeito aceso* e um chip
        // dizia qual era; o artista mexia num knob e via o outro sujeito mudar. Com duas secções, o
        // sujeito está no id do controlo — e a preferência de sessão que os coagia deixou de
        // existir, com a classe inteira de defeitos que ela trazia.
        for (slot, pat) in [
            (
                0,
                pen.selected()
                    .and_then(|sel| {
                        crate::texture_pattern_edit::pattern_at(
                            scene,
                            sel,
                            ph2d_vec_render::PatternSlot::Fill,
                        )
                    })
                    .cloned(),
            ),
            (
                1,
                pen.selected()
                    .and_then(|sel| {
                        crate::texture_pattern_edit::pattern_at(
                            scene,
                            sel,
                            ph2d_vec_render::PatternSlot::Stroke,
                        )
                    })
                    .cloned(),
            ),
        ] {
            ph2d_panel_vector::set_current_texture_pattern(
                slot,
                pen.selected().zip(pat.as_ref()).map(|(sel, pat)| {
                    ph2d_panel_vector::TexturePatternRow {
                        kind: crate::texture_pattern_edit::tile_index(pat.kind),
                        offset_denom: f64::from(pat.offset_denom.max(1)),
                        size: pat.size,
                        lock_aspect: texpat_lock,
                        gap: pat.gap[0],
                        angle_deg: pat.angle.to_degrees(),
                        // ⚠️ A fase mede-se do canto da CAIXA da forma — a MESMA base que a
                        // escrita usa (`TexPatCmd::Shift`). Sem uma caixa não há fase, e `0` é a
                        // resposta honesta: é onde o padrão nasce.
                        shift_pct: scene
                            .path_bbox(sel)
                            .map_or([0.0, 0.0], |(lo, _)| pat.shift(lo).map(|s| s * 100.0)),
                        mode: crate::texture_pattern_edit::mode_index(pat.mode),
                    }
                }),
            );
        }
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
}
