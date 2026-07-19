//! **A alça de fade do strip** (ADR-0115 B4) — geometria e ordem de hit.
//!
//! Mora fora do `stack_lane_paint` por duas razões: o arquivo de lá estourou o cap de 600 LOC dos
//! painéis, e — a de verdade — isto é uma unidade própria. Os dois grips **se constrangem
//! mutuamente**, então quem os coloca tem de ver os DOIS: colocados um de cada vez, eles se
//! atropelam exatamente na forma mais ordinária que existe (um fade-in e um fade-out que quase se
//! encontram no meio), e o hit index, que é last-wins, entrega o clique da ponta VISÍVEL de um
//! para o outro.

use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::zones::Rect;

use crate::graph::TimeView;
use crate::stack_lane_paint::EDGE_W;

/// Grab size of the ease handle at a strip's top corner (B4).
pub(crate) const EASE_W: f32 = 12.0; // LITERAL-PX-OK: a comfortable pointer target — 7 px was too thin to click (Enio, 2026-07-19)
/// The ease handle's resting inset from the corner, when the strip has NO fade yet.
///
/// It rests just PAST the trim grip instead of on the corner, so the two grips never
/// share a pixel: a handle stacked on the trim edge would have to win the click (it is
/// registered later), and the artist reaching for the trim would author a fade instead.
/// Dragging it back to here means zero fade.
pub(crate) const EASE_REST_X: f32 = EDGE_W; // LITERAL-PX-OK: exactly clear of the trim grip
/// The blend window's width in pixels — the wedge the panel already draws, measured.
pub(crate) fn blend_px(view: TimeView, t_start: f64, blend: f64) -> f32 {
    view.x(t_start + blend) - view.x(t_start)
}

/// Largura da BARRA que se vê (o grip que se agarra é mais largo — folga para o ponteiro).
pub(crate) const EASE_BAR_W: f32 = 2.0; // LITERAL-PX-OK: uma linha, como o traço de uma borda

/// Hit/paint code for the fade-in grip (the strip's start corner).
pub(crate) const EASE_IN: u8 = 3;
/// …and the fade-out grip (its end corner).
pub(crate) const EASE_OUT: u8 = 4;

/// **Both fade grips of one strip, placed together** — because they CONSTRAIN each other.
///
/// Each sits at the tip of its wedge, on the strip's top edge, so the thing you grab is the
/// thing you see. But two fades that nearly meet in the middle — the most ordinary fade-in /
/// fade-out shape there is — put the two tips within a grip's width of each other, and then:
///
/// - placed independently, the rects **coincide**, and the hit index (last-wins) hands every
///   click to the fade-OUT: grabbing the visible fade-in tip would drag the other fade;
/// - further in, the rects **cross**, and the fade-in grip is painted to the RIGHT of the
///   fade-out one.
///
/// So they are laid out as a pair: when the two would touch, they are set side by side about
/// the midpoint of the tips — still under the wedge tips, still in the right ORDER, never the
/// same pixel. Placing them one at a time is the bug; the signature is the fix.
///
/// `None` when the strip is too narrow to hold both trim grips AND both handles: on a strip
/// that small the handles would sit on top of the trim edges and steal them (an ease grip
/// outranks a trim grip). A strip you cannot fade at this zoom is honest; a strip you can no
/// longer TRIM is a bug.
///
/// With no fade authored the tip is at the corner, where the trim grip already is, so the
/// handle RESTS one grip-width in ([`EASE_REST_X`]). Dragging it back there means zero fade.
pub(crate) fn ease_grips(body: Rect, in_px: f32, out_px: f32) -> Option<(Rect, Rect)> {
    // Room for: trim grip | fade-in | fade-out | trim grip, none of them overlapping.
    if body.w < (EASE_REST_X + EASE_W) * 2.0 {
        return None;
    }
    let (lo, hi) = (
        body.x + EASE_REST_X,                   // leftmost either grip may sit
        body.x + body.w - EASE_REST_X - EASE_W, // rightmost either grip may sit
    );
    // `safe_clamp`: os limites saem da GEOMETRIA (o corpo do strip), não de constantes — e um
    // NaN vindo de um zoom degenerado viraria um retângulo que o hit index aceita e ninguém vê.
    let mut a = safe_clamp(body.x + in_px.max(EASE_REST_X), lo, hi);
    let mut b = safe_clamp(body.x + body.w - out_px.max(EASE_REST_X) - EASE_W, lo, hi);
    if b < a + EASE_W {
        // The tips met. Sit the pair side by side about their midpoint, in order.
        let mid = (a + b + EASE_W) * 0.5;
        a = safe_clamp(mid - EASE_W, lo, hi - EASE_W); // `hi - lo >= EASE_W` pela guarda acima
        b = a + EASE_W;
    }
    // **Altura INTEIRA do strip** — como a borda de aparar, e pela mesma razão.
    //
    // A primeira versão fazia um quadradinho de 7×7 na quina de cima. Medida na pintura real: o
    // strip tem 240×22 e a borda de aparar 6×22, então a alça era **um décimo da área** do grip
    // vizinho, numa faixa de 7 px no topo. O ponteiro do Enio caía no CORPO e arrastava o clipe
    // inteiro: *"não consigo arrastar a alça"*. O gate de hit passava — o alvo existia, ganhava o
    // pixel, e era pequeno demais para alguém acertar.
    //
    // Isso não é calibragem, é o modelo errado ([[feedback_ergonomics_verdict_is_a_design_bug]]):
    // a alça de fade É uma borda arrastável, igual à de aparar, e tem de ser tão agarrável quanto.
    // O corpo perde 7 px perto de cada quina — e continua com 90% do strip para o slide.
    Some((
        Rect::new(a, body.y, EASE_W, body.h),
        Rect::new(b, body.y, EASE_W, body.h),
    ))
}

/// Both fade grips, honouring an **outward** fade on either edge (`lead_in_px`,
/// `lead_out_px`).
///
/// When a side's lead is `> 0` its grip sits in the GAP beyond that edge — the fade-in at
/// `t_start - lead_in`, the fade-out at `t_end + lead_out` — where it cannot collide with
/// the other grip, so that side is placed independently (no pairing dance). Only the sides
/// WITHOUT an outward lead go through the inward-pair placement, which is the one that has
/// to keep two nearly-meeting tips apart. With both leads `== 0` this is exactly
/// [`ease_grips`] — the inward pair, byte-for-byte unchanged.
pub(crate) fn strip_grips(
    body: Rect,
    lead_in_px: f32,
    in_px: f32,
    out_px: f32,
    lead_out_px: f32,
) -> Option<(Rect, Rect)> {
    // The inward pair still decides the width guard, and provides the grip on any side
    // that has NO outward lead. A side WITH a lead leaves its inward slot at rest (`0.0`)
    // in the pair — its grip is placed in the gap below, where nothing can collide.
    let (in_base, out_base) = ease_grips(
        body,
        if lead_in_px > 0.0 { 0.0 } else { in_px },
        if lead_out_px > 0.0 { 0.0 } else { out_px },
    )?;
    let fade_in = if lead_in_px > 0.0 {
        Rect::new(body.x - lead_in_px, body.y, EASE_W, body.h)
    } else {
        in_base
    };
    let fade_out = if lead_out_px > 0.0 {
        // The grip's RIGHT edge lands on the wedge's outer tip (`t_end + lead_out`), so
        // the bar (drawn at `g.x + g.w`) marks the tip — the mirror of the fade-in's `g.x`.
        Rect::new(
            body.x + body.w + lead_out_px - EASE_W,
            body.y,
            EASE_W,
            body.h,
        )
    } else {
        out_base
    };
    Some((fade_in, fade_out))
}

/// Do two rects share a pixel?
pub(crate) fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}
