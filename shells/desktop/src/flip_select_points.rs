//! ADR-0114 W8 — **o domínio POINT do Edit Mode**, módulo-irmão de `flip_select` (cap
//! de LOC do HR-18): o pick de âncora, o plano do pen-down por ponto, o marquee por
//! ponto e a conversão de domínio na troca do toggle (`02_referencia §11`).
//!
//! As regras são as do domínio de traço, ponto a ponto — inclusive o **colapso
//! adiado** (clicar num ponto já selecionado não colapsa a seleção; soltar sem
//! arrastar colapsa). Quem decide o domínio é a TOOL (o snapshot); quem carrega o
//! dado é o DOCUMENTO (`FlipStroke::point_sel`).

use ph2d_core::Vec2;
use ph2d_flip::FlipDrawing;
use ph2d_vec_scene::Xform;

use crate::flip_select::visible_drawing;

/// Raio de pick de PONTO, em px de tela (domínio Point, W8). Um pouco mais generoso
/// que o piso do traço: o alvo é uma âncora, não uma linha.
const POINT_PICK_PX: f32 = 8.0; // LITERAL-PX-OK: folga de pick, nao metrica de design

/// **O PONTO sob o cursor** (domínio Point): o mais próximo dentro do raio, varrendo
/// todos os traços — devolve `(traço, ponto)`. O raio acompanha o zoom pela MESMA
/// conversão do pick de traço (px de tela → local); aproximar a câmera não pode exigir
/// mira mais fina.
#[must_use]
pub(crate) fn point_at(
    drawing: &FlipDrawing,
    local: Vec2,
    px_to_world: f32,
    w2l: &Xform,
) -> Option<(usize, usize)> {
    let px_to_local = px_to_world * w2l.mean_scale() as f32;
    let r = (POINT_PICK_PX * px_to_local).max(f32::EPSILON);
    let r2 = r * r;
    let mut best: Option<(f32, usize, usize)> = None;
    for (si, s) in drawing.strokes.iter().enumerate() {
        for (pi, p) in s.positions().iter().enumerate() {
            let d = local - *p;
            let d2 = d.x * d.x + d.y * d.y;
            if d2 <= r2 && best.is_none_or(|(bd, _, _)| d2 < bd) {
                best = Some((d2, si, pi));
            }
        }
    }
    best.map(|(_, si, pi)| (si, pi))
}

/// O que o pen-DOWN abre no domínio POINT — o espelho de [`Down`], com o alvo `(traço,
/// ponto)`. As regras são as mesmas do domínio de traço (inclusive o **colapso
/// adiado**: clicar num ponto já selecionado não colapsa a seleção; arrastar move o
/// grupo, soltar sem arrastar colapsa para este ponto).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DownPoints {
    Move { collapse_to: Option<(usize, usize)> },
    Click,
    Marquee { additive: bool },
}

/// **O plano do pen-DOWN no domínio Point** — espelho de [`plan_down`], ponto a ponto.
pub(crate) fn plan_down_points(
    drawing: &mut FlipDrawing,
    hit: Option<(usize, usize)>,
    shift: bool,
) -> DownPoints {
    match (hit, shift) {
        (None, shift) => {
            if !shift {
                drawing.clear_selection();
            }
            DownPoints::Marquee { additive: shift }
        }
        (Some((si, pi)), true) => {
            let on = !drawing.strokes[si].point_selected(pi);
            drawing.strokes[si].set_point_selected(pi, on);
            DownPoints::Click
        }
        (Some((si, pi)), false) if drawing.strokes[si].point_selected(pi) => DownPoints::Move {
            collapse_to: Some((si, pi)),
        },
        (Some((si, pi)), false) => {
            drawing.clear_selection();
            drawing.strokes[si].set_point_selected(pi, true);
            DownPoints::Move { collapse_to: None }
        }
    }
}

/// Aplica o marquee no domínio POINT: acende os pontos DENTRO da caixa (em LOCAL).
/// Devolve `true` se mudou. (Ponto é ponto: dentro-ou-fora, sem teste de segmento —
/// quem quer pegar a linha inteira usa o domínio Stroke.)
pub(crate) fn apply_marquee_points(
    drawing: &mut FlipDrawing,
    min: Vec2,
    max: Vec2,
    additive: bool,
) -> bool {
    let mut changed = false;
    if !additive {
        changed |= drawing.clear_selection();
    }
    for s in &mut drawing.strokes {
        for i in 0..s.len() {
            let p = s.positions()[i];
            if p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y {
                changed |= s.set_point_selected(i, true);
            }
        }
    }
    changed
}

/// **A troca de DOMÍNIO converte a seleção no documento** (W8 — a conversão explícita
/// do `02 §11`): Stroke→Point é broadcast (traço aceso ⇒ todos os pontos dele), Point→
/// Stroke é promoção `any()` + desmaterializa. Roda 1×/frame ao lado do
/// [`flip_edit_style_refresh`]; só age quando o toggle do painel MUDOU (a tool guarda a
/// escolha, o documento guarda o dado — e as duas pontas se encontram aqui).
pub(crate) fn flip_edit_domain_refresh(app: &mut crate::App) {
    let now = app.flip_style.map(|s| s.edit_domain);
    let prev = std::mem::replace(&mut app.flip_edit_domain, now);
    let (Some(prev), Some(now)) = (prev, now) else {
        return; // tool inativa (ou 1ª volta): nada a converter
    };
    if prev == now || !app.flip_wants_edit() {
        return;
    }
    let active_layer = app.flip_active_layer;
    let playhead = app.playhead;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let Some((oid, _lid, did)) = visible_drawing(&gfx.flip, &playhead, active_layer) else {
        return;
    };
    let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
        return;
    };
    match now {
        ph2d_tool_flip::EditDomain::Point => drawing.selection_to_point_domain(),
        ph2d_tool_flip::EditDomain::Stroke => drawing.selection_to_stroke_domain(),
    }
    app.title_dirty = true;
}
#[cfg(test)]
#[path = "flip_select_points_tests.rs"]
mod tests;
