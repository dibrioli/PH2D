//! The veil's DURATION HANDLE paint (Enio, 2026-07-23) — split from `ruler.rs`
//! under the panel LOC cap. A child module of `ruler`, so it reads the parent's
//! ruler geometry consts + helpers (`fill_triangle`, `RULER_H`, `BRACE_W`) through
//! `super::*`.

use super::*;

/// Horizontal offset of the ↔ glyph CENTRE from the veil edge — well to the RIGHT
/// (Enio, 2026-07-23: moved another 20 px out, *"pois ela conflita com a área do
/// loop quando o loop vai até o fim"*). The grab lives ON the glyph, so at this
/// distance neither the glyph nor its grab overlaps the loop brace sitting at the
/// edge when the loop runs to the end.
const DUR_ARROW_OFFSET: f32 = 40.0; // LITERAL-PX-OK: ↔ glyph offset from the duration edge
/// Half the shaft length of the ↔ glyph (2× — Enio, 2026-07-23: *"maior (2x)"*).
const DUR_ARROW_HALF_W: f32 = 8.0; // LITERAL-PX-OK: ↔ glyph shaft half-width
/// Half-thickness of the ↔ glyph shaft (2×).
const DUR_ARROW_SHAFT_HW: f32 = 2.0; // LITERAL-PX-OK: ↔ glyph shaft half-thickness
/// Arrowhead size of the ↔ glyph (2×).
const DUR_ARROW_HEAD: f32 = 6.0; // LITERAL-PX-OK: ↔ glyph arrowhead size
/// Padding around the ↔ glyph the grab still covers.
const DUR_ARROW_PAD: f32 = 4.0; // LITERAL-PX-OK: grab padding around the ↔ glyph

/// Paint the ↔ drag grip for the authored duration and register it as a
/// [`TimelineHitKind::DurationHandle`] surface. A no-op unless the view has an
/// authored duration whose end is on-screen — same gate as the veil
/// ([`beyond_end_shade`]).
///
/// **No bar on the edge; the ↔ glyph is the visual cue AND the grab** (Enio,
/// 2026-07-26 — REVERSES the earlier whole-veil grab of `d489f9d67`: *"o clique
/// para ajustar o véu é só na própria seta … qualquer região à direita dela … é
/// como se estivesse clicando nela"*). The whole-veil grab stole every press in
/// the dark dead-zone; now the grab is a tight box on the ↔ glyph only
/// ([`grab_rect`]), so clicking the veil to the right does nothing. The loop brace
/// sits ON the edge when the loop runs to the end, so the ↔ is drawn
/// `DUR_ARROW_OFFSET` to the right (clear of it). The drag is grab-relative
/// (`duration_drag`), so grabbing the glyph does not jump the duration to the pointer.
pub(super) fn paint_duration_handle(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    view_start: f64,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) {
    if !snap.view_length_explicit {
        return;
    }
    let right = region.x + region.w;
    let edge = region.x + ((snap.view_length_seconds - view_start) * px_per_s) as f32;
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    let Some(seta) = onde_a_seta(edge, (region.x, right), cartoes) else {
        return;
    };
    let color = resolve(ColorToken::TimelinePlayhead, theme);
    // The ↔ glyph, well to the right of the edge (over the dark veil), centred
    // vertically in the strip: a thin shaft with a triangle arrowhead at each end.
    let cy = region.y + RULER_H * 0.5;
    let gx = seta.gx;
    let half = DUR_ARROW_HALF_W;
    let haste = Rect::new(
        gx - half,
        cy - DUR_ARROW_SHAFT_HW,
        half * 2.0,
        DUR_ARROW_SHAFT_HW * 2.0,
    );
    let ponta = gx + half + DUR_ARROW_HEAD;
    let mut pontas = vec![
        [
            (gx - half - DUR_ARROW_HEAD, cy),
            (gx - half, cy - DUR_ARROW_HEAD),
            (gx - half, cy + DUR_ARROW_HEAD),
        ],
        [
            (ponta, cy),
            (gx + half, cy - DUR_ARROW_HEAD),
            (gx + half, cy + DUR_ARROW_HEAD),
        ],
    ];
    // Presa à borda com o fim além dela: um ▸ depois da ⟷ diz «o fim continua à direita».
    if seta.fora {
        let x0 = ponta + DUR_FORA_GAP;
        let h = DUR_FORA_SETA_HH;
        pontas.push([(x0, cy - h), (x0, cy + h), (x0 + DUR_FORA_SETA_W, cy)]);
    }
    let e = crate::alca_estado::EstadoAlca::nos_cartoes(ctx, ids::TIMELINE_DUR_HANDLE);
    crate::alca_estado::pinta(ctx, theme, e, color, &[haste], &pontas);
    // Presa, a leitura do arrasto vai à esquerda da ⟷ (à direita não há cartão).
    let presa = gx < edge + DUR_ARROW_OFFSET;
    let (lx, esq) = if presa {
        (gx - half - DUR_ARROW_HEAD, true)
    } else {
        (ponta, false)
    };
    let t = snap.view_length_seconds;
    crate::alca_estado::pinta_leitura(ctx, theme, e, t, lx, cy, esq);
    // The grab target: ONLY the ↔ glyph (Enio, 2026-07-26 — reversed the whole-veil
    // grab). Clicking the dark veil to the right must NOT resize the duration; only
    // the arrow does. Pinned, it is the same box plus the ▸, still inside the time area.
    let hit = grab_rect(seta, region.y);
    ctx.host.store_mut().register(
        ids::TIMELINE_DUR_HANDLE,
        InteractiveState::TimelineSurface {
            parent: ph2d_editor_core::ids::TIMELINE_PANEL,
            kind: TimelineHitKind::DurationHandle,
            canvas: hit,
        },
    );
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_DUR_HANDLE, hit);
}

/// Onde a ⟷ se pinta: o centro `gx` e se está presa com o fim além da borda (`fora`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Seta {
    pub(crate) gx: f32,
    pub(crate) fora: bool,
}

/// O alcance da ⟷ a partir do centro: haste, ponta e a folga do alvo.
const DUR_ARROW_REACH: f32 = DUR_ARROW_HALF_W + DUR_ARROW_HEAD + DUR_ARROW_PAD;

/// Quanto a ⟷ pede à direita do fim para caber inteira — o enquadrar reserva-o (`view.rs`).
pub(crate) const DUR_ARROW_ROOM: f32 = DUR_ARROW_OFFSET + DUR_ARROW_REACH;

/// A ⟷ para o fim em `edge` na faixa `(x0, right)`. Fim à esquerda da faixa: nenhuma (arrastaria de
/// um tempo onde o fim não está). Clássico: só com o fim à vista, `DUR_ARROW_OFFSET` à direita dele.
/// ⭐ Cartões (dono 10/11): a ⟷ nunca sai da faixa — encosta na borda direita, e com o fim além
/// dela fica PRESA com o ▸; o arrasto é relativo ao agarrar (`duration_drag`), logo continua a mudar
/// a duração dali.
pub(crate) fn onde_a_seta(edge: f32, (x0, right): (f32, f32), cartoes: bool) -> Option<Seta> {
    if edge < x0 || (edge > right && !cartoes) {
        return None;
    }
    let fora = edge > right;
    let gx = edge + DUR_ARROW_OFFSET;
    if !cartoes {
        return Some(Seta { gx, fora });
    }
    let extra = if fora { DUR_FORA_EXTRA } else { 0.0 };
    let limite = right - DUR_ARROW_REACH - extra;
    Some(Seta {
        gx: gx.min(limite),
        fora,
    })
}

/// O ▸ depois da ⟷ presa: folga, largura e meia-altura.
const DUR_FORA_GAP: f32 = 3.0; // LITERAL-PX-OK: gap between the ↔ and its ▸ when pinned
const DUR_FORA_SETA_W: f32 = 5.0; // LITERAL-PX-OK: ▸ width when the ↔ is pinned
const DUR_FORA_SETA_HH: f32 = 4.0; // LITERAL-PX-OK: ▸ half-height when the ↔ is pinned
/// O que o ▸ acrescenta ao alcance da ⟷ presa.
const DUR_FORA_EXTRA: f32 = DUR_FORA_GAP + DUR_FORA_SETA_W;

/// The duration handle's grab rect: a tight box on the ↔ glyph ONLY (Enio,
/// 2026-07-26 — reverses the earlier whole-veil grab), plus the ▸ when pinned. The glyph
/// reaches `DUR_ARROW_HALF_W + DUR_ARROW_HEAD` either side of `gx`; the grab adds
/// `DUR_ARROW_PAD`. Pure, so the geometry has an oracle the paint cannot give.
pub(crate) fn grab_rect(seta: Seta, region_y: f32) -> Rect {
    let reach = DUR_ARROW_REACH;
    let extra = if seta.fora { DUR_FORA_EXTRA } else { 0.0 };
    Rect::new(seta.gx - reach, region_y, reach * 2.0 + extra, RULER_H)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The grab is ONLY the ↔ glyph — it covers the arrow, stays clear of the loop
    /// brace at the edge, and does NOT spill into the veil to the right** (Enio,
    /// 2026-07-26: clicking the dark veil right of the arrow must not resize the
    /// duration; only the arrow does). Reverses the earlier whole-veil grab.
    #[test]
    fn the_grab_is_only_the_arrow_not_the_whole_veil() {
        let (edge, right) = (200.0, 800.0);
        let seta = onde_a_seta(edge, (0.0, right), false).expect("o fim à vista");
        let g = grab_rect(seta, 0.0);
        let gx = edge + DUR_ARROW_OFFSET;
        // Covers the arrow glyph...
        assert!(
            g.x <= gx && gx <= g.x + g.w,
            "the grab must cover the arrow at {gx} (got x={}..{})",
            g.x,
            g.x + g.w
        );
        // ...clear of the loop brace at the edge (registered last = topmost)...
        assert!(
            g.x > edge + BRACE_HIT_HW,
            "grab starts at {} but the loop brace reaches to {}",
            g.x,
            edge + BRACE_HIT_HW
        );
        // ...and does NOT reach the dark veil to the right (the bug): a click well
        // right of the arrow misses the grab.
        assert!(
            g.x + g.w < right,
            "the grab must NOT cover the veil to the right edge {right} (only the arrow), got {}",
            g.x + g.w
        );
        assert!(
            gx + 100.0 > g.x + g.w,
            "a click 100px right of the arrow must miss the grab"
        );
    }

    /// ⭐ Dono 10/11 (cartão estreito): nos cartões a ⟷ nunca sai da área de tempo — encosta na
    /// borda com o fim perto dela, e PRESA com o ▸ com o fim além; o alvo cabe e não encolhe.
    #[test]
    fn nos_cartoes_a_seta_fica_dentro_e_presa_com_o_fim_alem() {
        let faixa = (100.0, 400.0);
        let antigo = grab_rect(onde_a_seta(200.0, faixa, false).expect("à vista"), 0.0);
        for (edge, fora) in [
            (200.0, false),
            (390.0, false),
            (400.0, false),
            (900.0, true),
        ] {
            let seta = onde_a_seta(edge, faixa, true).expect("nos cartões há sempre ⟷");
            assert_eq!(seta.fora, fora, "fim em {edge}");
            let g = grab_rect(seta, 0.0);
            assert!(
                g.x >= faixa.0 && g.x + g.w <= faixa.1,
                "fim em {edge}: alvo {}..{} fora da faixa",
                g.x,
                g.x + g.w
            );
            assert!(g.w >= antigo.w && g.h >= antigo.h, "o alvo não encolhe");
        }
        assert!(onde_a_seta(900.0, faixa, false).is_none(), "clássico igual");
        assert!(
            onde_a_seta(50.0, faixa, true).is_none(),
            "fim à esquerda: nenhuma"
        );
    }
}
