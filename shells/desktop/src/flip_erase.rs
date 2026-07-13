//! ADR-0114 W2 T2.9 — the Flip eraser (clean-room of GP `erase.cc`, 3 modes).
//!
//! - **Soft** (default, most paint-like): reduces per-point opacity by
//!   `strength · falloff(dist)` within the brush radius; on pen-up, points that
//!   fell below the threshold are removed (splitting the stroke at the gap).
//! - **Hard**: cuts the stroke — removes points inside the circle and splits the
//!   survivors into separate strokes.
//! - **Stroke**: erases the whole stroke if any point is inside the circle.
//!
//! A LOCKED layer refuses all three (materials/strokes preserved). Erasing an
//! empty frame (no key yet) is a no-op — the eraser never creates a drawing.
//! HR-5: the falloff is a clamped linear ramp (no transcendentals).

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, LayerId};
use ph2d_tool_flip::EraseMode;

/// Below this opacity a soft-erased point is dropped on pen-up (GP `erase.cc`).
const OPACITY_REMOVE_THRESHOLD: f32 = 0.05;

/// O desenho que a borracha edita — via o **autokey por-tool** (W3.T3.4): no rabo
/// de um hold, a borracha SEMPRE trabalha numa DUPLICATA do desenho que está na
/// tela (nunca num quadro em branco novo, que apagaria o nada e deixaria a arte
/// que o usuário vê intacta num quadro anterior). Camada travada e canvas vazio
/// continuam recusando (`None`) — a borracha nunca inventa um quadro.
fn active_drawing_mut<'a>(
    flip: &'a mut ph2d_flip::FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<LayerId>,
    strip: &crate::flip_strip::FlipStrip,
) -> Option<&'a mut FlipDrawing> {
    let (oid, _lid, did) = crate::flip_autokey::target_drawing(
        flip,
        playhead,
        active_layer,
        strip,
        crate::flip_autokey::FlipEdit::Modify,
    )?;
    flip.object_mut(oid)?.drawing_mut(did)
}

/// A fresh stroke carrying `src`'s per-curve attributes (empty points).
///
/// **Todo campo novo do `FlipStroke` tem de passar por aqui.** Este é um dos três
/// pontos de estrangulamento que definem o que sobrevive a uma operação (os outros são
/// `FlipStroke::clone_attrs` e o `cleanup_soft`); a W4 acrescentou `holes` e
/// `hide_stroke` ao modelo e atualizou só o primeiro — e a borracha passou a devolver
/// fragmentos de preenchimento SEM os furos (o "O" ficava sólido) e SEM o `hide_stroke`
/// (o fragmento virava fronteira do próximo balde e o Unpaint não o reconhecia mais).
fn new_like(src: &FlipStroke) -> FlipStroke {
    let mut s = FlipStroke::new();
    s.closed = src.closed;
    s.cap = src.cap;
    s.hardness = src.hardness;
    s.material = src.material;
    s.fill = src.fill;
    s.holes = src.holes.clone();
    s.hide_stroke = src.hide_stroke;
    // A W6 acrescentou a seleção: os PEDAÇOS de um traço selecionado continuam
    // selecionados (é o que o GP faz — cortar não desmarca). Sem isto, cortar um traço
    // selecionado o tiraria da seleção em silêncio, e o próximo ajuste do painel
    // atingiria outro traço.
    s.selected = src.selected;
    s
}

/// Um traço SEM tinta visível: um preenchimento (região) ou um fechamento de gap.
///
/// A borracha morde **tinta**, e um traço destes não tem nenhuma: o contorno de um fill
/// não é rasterizado, e um fechamento é invisível de propósito. Deixar as borrachas de
/// PONTO (Soft/Hard) mordê-los produzia só lixo — fragmentos de região com o furo
/// perdido, apagados num lugar onde o usuário não vê linha alguma. Uma região se remove
/// pelo **Unpaint** do balde, ou de uma vez pela borracha de traço.
fn is_region(s: &FlipStroke) -> bool {
    s.hide_stroke
}

/// Whether any point of `s` lies within `radius` of `center`.
fn touches(s: &FlipStroke, center: Vec2, radius: f32) -> bool {
    let r2 = radius * radius;
    s.positions().iter().any(|p| {
        let d = *p - center;
        d.x * d.x + d.y * d.y <= r2
    })
}

/// Split `s` into runs of consecutive points that satisfy `keep`. Each run of
/// ≥2 points becomes a stroke (single-point runs are dropped — a dab, no line).
fn split_by<F: Fn(usize) -> bool>(s: &FlipStroke, keep: F) -> Vec<FlipStroke> {
    let mut out = Vec::new();
    let mut cur = new_like(s);
    for i in 0..s.len() {
        if keep(i) {
            if let Some(p) = s.point(i) {
                cur.push_point(p);
            }
        } else if cur.len() >= 2 {
            out.push(std::mem::replace(&mut cur, new_like(s)));
        } else {
            cur = new_like(s);
        }
    }
    if cur.len() >= 2 {
        out.push(cur);
    }
    out
}

/// Erase once at `center` (world) with `radius` (world) + `strength` (Soft only).
/// Returns `true` if the document changed.
#[allow(clippy::too_many_arguments)] // doc+playhead+camada+tira+modo+círculo+força
pub(crate) fn erase_at(
    flip: &mut ph2d_flip::FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<LayerId>,
    strip: &crate::flip_strip::FlipStrip,
    mode: EraseMode,
    center: Vec2,
    radius: f32,
    strength: f32,
) -> bool {
    let Some(dr) = active_drawing_mut(flip, playhead, active_layer, strip) else {
        return false;
    };
    match mode {
        EraseMode::Stroke => {
            let before = dr.strokes.len();
            dr.strokes.retain(|s| !touches(s, center, radius));
            dr.strokes.len() != before
        }
        EraseMode::Soft => {
            let mut changed = false;
            let r = radius.max(f32::EPSILON);
            for s in dr.strokes.iter_mut() {
                if is_region(s) {
                    continue; // região: não tem tinta para amaciar (ver `is_region`)
                }
                let ps: Vec<Vec2> = s.positions().to_vec();
                let ops = s.opacities_mut();
                for (i, p) in ps.iter().enumerate() {
                    let d = ((*p - center).x.powi(2) + (*p - center).y.powi(2)).sqrt();
                    if d < radius {
                        let falloff = (1.0 - d / r).clamp(0.0, 1.0); // linear ramp (HR-5)
                        let reduced = (ops[i] - strength * falloff).max(0.0);
                        if reduced != ops[i] {
                            ops[i] = reduced;
                            changed = true;
                        }
                    }
                }
            }
            changed
        }
        EraseMode::Hard => {
            let r2 = radius * radius;
            let mut changed = false;
            let mut out: Vec<FlipStroke> = Vec::new();
            for s in std::mem::take(&mut dr.strokes) {
                if is_region(&s) {
                    out.push(s); // região: não se corta em pedaços (ver `is_region`)
                    continue;
                }
                let inside: Vec<bool> = s
                    .positions()
                    .iter()
                    .map(|p| {
                        let d = *p - center;
                        d.x * d.x + d.y * d.y <= r2
                    })
                    .collect();
                if inside.iter().any(|&b| b) {
                    changed = true;
                    out.extend(split_by(&s, |i| !inside[i]));
                } else {
                    out.push(s);
                }
            }
            dr.strokes = out;
            changed
        }
    }
}

/// Soft-erase pen-up cleanup: drop only strokes that were erased AWAY entirely
/// (todos os pontos < limiar). **Não divide** o traço nos pontos parciais — os
/// pontos com opacidade reduzida (incl. ~0 no meio) ficam, pra o renderer
/// interpolar a opacidade por-ponto e o apagado seguir com borda MACIA. Dividir
/// (o 1º corte) trocava a queda suave por um corte com cap plano = borda dura
/// (Enio 2026-07-11: "no bake do traço o apagado fica com bordas duras").
/// No-op para os outros modos (o caller filtra por modo).
pub(crate) fn cleanup_soft(
    flip: &mut ph2d_flip::FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<LayerId>,
    strip: &crate::flip_strip::FlipStrip,
) -> bool {
    let Some(dr) = active_drawing_mut(flip, playhead, active_layer, strip) else {
        return false;
    };
    let before = dr.strokes.len();
    // **Uma REGIÃO nunca é coletada por opacidade de ponto.** A visibilidade dela não vem
    // dali (o contorno não é rasterizado): um fechamento de gap nasce com opacidade 0
    // porque é invisível de propósito — e este `retain` então o removia SEMPRE, a cada
    // pen-up da borracha macia, em qualquer lugar do canvas. O vão reabria sozinho e o
    // balde seguinte vazava; o fechamento persistente (o twist do Harmony) era desfeito
    // em silêncio por um toque de borracha do outro lado do desenho.
    dr.strokes
        .retain(|s| is_region(s) || s.opacities().iter().any(|o| *o >= OPACITY_REMOVE_THRESHOLD));
    dr.strokes.len() != before
}

impl crate::App {
    /// The Flip tool wants the canvas for ERASING now (active + Erase mode). The
    /// `input_dispatch` reads the published style cache — no downcast.
    #[must_use]
    pub(crate) fn flip_wants_erase(&self) -> bool {
        self.flip_active
            && matches!(
                self.flip_style.map(|s| s.mode),
                Some(ph2d_tool_flip::FlipMode::Erase)
            )
    }

    /// Pen-down of the eraser: begin the gesture + erase at the cursor. Returns
    /// `true` if consumed (so the caller doesn't fall into the gizmo/pick).
    pub(crate) fn flip_erase_canvas_down(&mut self, x: f32, y: f32) -> bool {
        // Apagar é "fazer outra coisa": o alvo vivo (o último traço/preenchimento) deixa
        // de ser o alvo dos ajustes do painel.
        self.flip_live_clear();
        if !self.flip_wants_erase() {
            return false;
        }
        self.flip_erasing = true;
        self.flip_erase_apply(x, y);
        true
    }

    /// Move while erasing: erase at the cursor. `true` while a gesture is live.
    pub(crate) fn flip_erase_canvas_move(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_erasing {
            return false;
        }
        self.flip_erase_apply(x, y);
        true
    }

    /// Pen-up: end the gesture + (Soft mode) drop the faded points.
    pub(crate) fn flip_erase_canvas_up(&mut self) -> bool {
        if !self.flip_erasing {
            return false;
        }
        self.flip_erasing = false;
        let soft = matches!(
            self.flip_style.map(|s| s.erase),
            Some(ph2d_tool_flip::EraseMode::Soft)
        );
        if soft {
            let active_layer = self.flip_active_layer;
            if let Some(gfx) = self.gfx.as_mut() {
                cleanup_soft(
                    &mut gfx.flip,
                    &self.playhead,
                    active_layer,
                    &self.flip_strip,
                );
            }
        }
        true
    }

    /// Erase once under the cursor (screen coords → world + radius from the brush).
    fn flip_erase_apply(&mut self, x: f32, y: f32) {
        let Some(style) = self.flip_style else {
            return;
        };
        let active_layer = self.flip_active_layer;
        // Fronteira MUNDO→LOCAL (ADR-0111): a geometria de um objeto já movido pelo
        // gizmo é LOCAL, então o cursor (mundo) desce ao espaço local e o raio recua
        // pela escala. Identidade num objeto não-movido (o comum) → no-op.
        let w2l = self.flip_active_world_to_local();
        if let Some(gfx) = self.gfx.as_mut() {
            let win = gfx.surface.size();
            let w = gfx.camera.screen_to_world((x, y), win);
            let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
            // Eraser radius = the brush size (px → world); strength = brush opacity.
            let radius = (style.width_px as f32 * 0.5) * px_to_world;
            let local = w2l.apply([f64::from(w[0]), f64::from(w[1])]);
            let radius_local = radius * w2l.mean_scale() as f32;
            erase_at(
                &mut gfx.flip,
                &self.playhead,
                active_layer,
                &self.flip_strip,
                style.erase,
                Vec2::new(local[0] as f32, local[1] as f32),
                radius_local,
                style.opacity,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Playhead;
    use ph2d_flip::{FlipDoc, Hold, KeyKind, Point, Rgba};

    fn doc_with_line() -> (FlipDoc, LayerId) {
        let mut doc = FlipDoc::new();
        let oid = doc.push_object("O");
        let obj = doc.object_mut(oid).unwrap();
        let l = obj.add_layer("L");
        let d = obj
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let mut s = FlipStroke::new();
        for x in 0..5 {
            s.push_point(Point {
                pos: Vec2::new(x as f32, 0.0),
                width: 0.1,
                opacity: 1.0,
                color: Rgba::WHITE,
            });
        }
        obj.drawing_mut(d).unwrap().strokes.push(s);
        (doc, l)
    }

    #[test]
    fn stroke_mode_removes_the_whole_touched_stroke() {
        let (mut doc, l) = doc_with_line();
        let hit = erase_at(
            &mut doc,
            &Playhead::default(),
            Some(l),
            &crate::flip_strip::FlipStrip::default(),
            EraseMode::Stroke,
            Vec2::new(2.0, 0.0),
            0.5,
            1.0,
        );
        assert!(hit);
        let obj = doc.objects().first().unwrap();
        let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
        assert_eq!(obj.drawing(did).unwrap().strokes.len(), 0);
    }

    #[test]
    fn hard_mode_splits_the_stroke_at_the_gap() {
        let (mut doc, l) = doc_with_line();
        // Erase the middle point (x=2) → two runs [0,1] and [3,4].
        let hit = erase_at(
            &mut doc,
            &Playhead::default(),
            Some(l),
            &crate::flip_strip::FlipStrip::default(),
            EraseMode::Hard,
            Vec2::new(2.0, 0.0),
            0.5,
            1.0,
        );
        assert!(hit);
        let obj = doc.objects().first().unwrap();
        let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
        assert_eq!(obj.drawing(did).unwrap().strokes.len(), 2, "split into two");
    }

    #[test]
    fn soft_mode_reduces_opacity_then_cleanup_removes_faded() {
        let (mut doc, l) = doc_with_line();
        // Full-strength soft erase over the whole line → all points to ~0.
        for _ in 0..2 {
            erase_at(
                &mut doc,
                &Playhead::default(),
                Some(l),
                &crate::flip_strip::FlipStrip::default(),
                EraseMode::Soft,
                Vec2::new(2.0, 0.0),
                10.0,
                1.0,
            );
        }
        let obj = doc.objects().first().unwrap();
        let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
        let min_op = obj.drawing(did).unwrap().strokes[0]
            .opacities()
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min);
        assert!(
            min_op < OPACITY_REMOVE_THRESHOLD,
            "center faded below threshold"
        );
        assert!(cleanup_soft(
            &mut doc,
            &Playhead::default(),
            Some(l),
            &crate::flip_strip::FlipStrip::default(),
        ));
    }

    #[test]
    fn locked_layer_refuses_erase() {
        let (mut doc, l) = doc_with_line();
        doc.objects().first().unwrap(); // sanity
        let oid = doc.objects().first().unwrap().id;
        doc.object_mut(oid).unwrap().layer_mut(l).unwrap().locked = true;
        let hit = erase_at(
            &mut doc,
            &Playhead::default(),
            Some(l),
            &crate::flip_strip::FlipStrip::default(),
            EraseMode::Stroke,
            Vec2::new(2.0, 0.0),
            0.5,
            1.0,
        );
        assert!(!hit, "locked layer preserved");
    }

    /// Um desenho com line-art + um PREENCHIMENTO (com furo) + um FECHAMENTO de gap.
    fn doc_with_fill_and_closure() -> (FlipDoc, LayerId) {
        let (mut doc, l) = doc_with_line();
        let oid = doc.objects().first().unwrap().id;
        let obj = doc.object_mut(oid).unwrap();
        let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
        let dr = obj.drawing_mut(did).unwrap();

        // O preenchimento: contorno invisível, com um furo.
        let mut fill = FlipStroke::new();
        for p in [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ] {
            fill.push_point(Point {
                pos: p,
                width: 0.0,
                opacity: 1.0,
                color: Rgba::WHITE,
            });
        }
        fill.closed = true;
        fill.hide_stroke = true;
        fill.holes = vec![vec![
            Vec2::new(3.0, 3.0),
            Vec2::new(7.0, 3.0),
            Vec2::new(7.0, 7.0),
            Vec2::new(3.0, 7.0),
        ]];
        fill.fill = Some(ph2d_flip::Fill {
            color: Rgba::WHITE,
            opacity: 1.0,
        });
        dr.strokes.push(fill);

        // O fechamento de gap: invisível, sem cor, opacidade ZERO (é assim que nasce).
        let mut clo = FlipStroke::new();
        for p in [Vec2::new(20.0, 20.0), Vec2::new(22.0, 20.0)] {
            clo.push_point(Point {
                pos: p,
                width: 0.0,
                opacity: 0.0,
                color: Rgba::TRANSPARENT,
            });
        }
        clo.hide_stroke = true;
        dr.strokes.push(clo);
        (doc, l)
    }

    fn strokes_of(doc: &FlipDoc) -> Vec<FlipStroke> {
        let obj = doc.objects().first().unwrap();
        let l = obj.layers().first().unwrap().id;
        let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
        obj.drawing(did).unwrap().strokes.clone()
    }

    /// **A borracha macia NÃO pode apagar os fechamentos de gap.**
    ///
    /// Um fechamento nasce com opacidade 0 (é invisível de propósito), e o `cleanup_soft`
    /// coletava lixo por opacidade de ponto: `retain(any opacity >= 0.05)` matava TODOS
    /// eles — a cada pen-up, em qualquer lugar do canvas. O vão reabria e o balde seguinte
    /// vazava. O "fechamento persistente" (o twist do Harmony) era desfeito em silêncio.
    #[test]
    fn the_soft_eraser_does_not_delete_gap_closures_anywhere_on_the_canvas() {
        let (mut doc, l) = doc_with_fill_and_closure();
        let ph = Playhead::new(1.0 / 12.0);
        let strip = crate::flip_strip::FlipStrip::default();
        let before = strokes_of(&doc).len();

        // Um toque de borracha macia LONGE de tudo (em (100, 100)).
        erase_at(
            &mut doc,
            &ph,
            Some(l),
            &strip,
            EraseMode::Soft,
            Vec2::new(100.0, 100.0),
            1.0,
            1.0,
        );
        cleanup_soft(&mut doc, &ph, Some(l), &strip);

        let after = strokes_of(&doc);
        assert_eq!(
            after.len(),
            before,
            "a borracha apagou tracos do outro lado do canvas (o fechamento de gap sumiu)"
        );
        assert!(
            after.iter().any(|s| s.hide_stroke && s.fill.is_none()),
            "o fechamento de gap invisivel foi coletado como lixo"
        );
    }

    /// **A borracha de PONTO não morde uma região.** Um preenchimento não tem tinta
    /// visível (o contorno não é rasterizado): mordê-lo produzia fragmentos com o furo
    /// PERDIDO (o "O" ficava sólido) e sem `hide_stroke` — o fragmento virava fronteira
    /// do próximo balde e o Unpaint não o reconhecia mais.
    #[test]
    fn a_point_eraser_does_not_shred_a_filled_region() {
        for mode in [EraseMode::Soft, EraseMode::Hard] {
            let (mut doc, l) = doc_with_fill_and_closure();
            let ph = Playhead::new(1.0 / 12.0);
            let strip = crate::flip_strip::FlipStrip::default();

            // Apaga bem no MEIO da região preenchida.
            erase_at(
                &mut doc,
                &ph,
                Some(l),
                &strip,
                mode,
                Vec2::new(5.0, 5.0),
                4.0,
                1.0,
            );
            cleanup_soft(&mut doc, &ph, Some(l), &strip);

            let fills: Vec<FlipStroke> = strokes_of(&doc)
                .into_iter()
                .filter(|s| s.hide_stroke && s.fill.is_some())
                .collect();
            assert_eq!(
                fills.len(),
                1,
                "{mode:?}: a regiao foi picada em {} pedacos",
                fills.len()
            );
            assert_eq!(
                fills[0].holes.len(),
                1,
                "{mode:?}: o furo do \"O\" foi perdido (a regiao virou solida)"
            );
            assert!(
                fills[0].hide_stroke,
                "{mode:?}: o fill perdeu o hide_stroke"
            );
        }
    }
}
