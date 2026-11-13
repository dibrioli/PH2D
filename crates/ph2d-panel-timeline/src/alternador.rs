//! ⭐ **Um botão que acende** (dono 13/11, «Edit Path»): só o nome, sem caixa de marcar; ligado leva a
//! tinta do chip escolhido da barra. O clique é o da caixa de sempre (o store espelha-a).

use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;

/// Um botão que ACENDE (o chip escolhido da barra) quando ligado, só com o nome; o clique é o da caixa
/// de sempre (o store espelha-a em `prepara`).
pub(crate) fn pinta(
    c: &mut CardCtx<'_>,
    (id, nome, on): (ph2d_a11y::NodeId, &str, bool),
    r: Rect,
    theme: Theme,
) {
    let t = crate::transport::cartoes::tons_da_barra(theme);
    c.hit_index.register(id, r);
    let (fundo, texto) = if on {
        (t.ativa_fundo, t.ativa_texto)
    } else if c.store.hover_live(id) > 0.0 {
        (t.sob_o_rato, t.inativa_texto)
    } else {
        (t.corpo_da_peca, t.discreta_texto)
    };
    ph2d_editor_core::paint::fill_rounded_rect(c.scene, r, m::FIELD_RADIUS, fundo);
    let (x, y) = (
        r.x + m::FIELD_PAD_R,
        r.y + ph2d_editor_core::paint::folga_do_texto(r.h, m::VALUE_PX),
    );
    let w = r.w - m::FIELD_PAD_R * 2.0;
    ph2d_editor_core::paint::paint_text_elided(
        c.text_system,
        c.scene,
        nome,
        x,
        y,
        m::VALUE_PX,
        w,
        texto,
    );
}
