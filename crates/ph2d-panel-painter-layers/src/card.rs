//! The **card** — a titled, bordered box of `label · number-box` rows. The shape the Painter's brush
//! panel states a technique in: Wash, Brush, Water, Impasto, Lighting.
//!
//! Extracted from `paint_watercolor.rs` (a pure move, no behaviour) when Impasto needed the same box.
//! Two sections hand-rolling their own card is how two sections quietly stop looking alike — see
//! [[feedback_ui_source_of_truth_gallery_inspector]].

use crate::number_field;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};

/// Draw a titled bordered **card** (the Composite/Clone-card idiom) sized for `n_rows` number rows, and
/// return `(inner_x, inner_w, first_row_y, y_after_card)` — the caller paints the rows into
/// `[inner_x, inner_w]` starting at `first_row_y` with [`card_row`], then continues at `y_after_card`.
pub(crate) fn card_frame(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    title: &str,
    n_rows: usize,
) -> (f32, f32, f32, f32) {
    let pad = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let title_h = font + Spacing::Sm.px();
    let row_adv = ph2d_tokens::row_pitch_px();
    let card_h = pad + title_h + n_rows as f32 * row_adv + pad;
    let card = Rect::new(x, y, content_w, card_h);
    // ⭐ Raio e moldura pela porta do TEMA: o cartão é plano num tema moderno.
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Md.px());
    // ⭐⭐⭐ **O tom de uma SUBSECÇÃO, não o de uma secção** — report do dono, 2026-09-06, com as
    //    duas telas lado a lado: *«em Audio Editor: Effects temos o card. Já o card de Painter:
    //    Jitter não se vê mais.»*
    //
    //    ⚠️ **Ele não sumiu: foi ENGOLIDO.** Este cartão sempre pintou `Bg1`, e em 2026-09-06 a
    //    wave 12 pôs um cartão de SECÇÃO por trás dele — também `Bg1`. Dois `Bg1` encostados são
    //    um só. *Um cartão dentro de um cartão da mesma cor não é um cartão; é o mesmo cartão.*
    //
    //    ⇒ a cura é a lei que o próprio dono descreveu na wave 9, e que existia sem um único
    //    chamador: *«uma subsecção está com o seu título dentro do card da secção mas o seu
    //    conteúdo fica dentro de outro card/container de COR DIFERENTE»*. O tom vem da porta, e
    //    não de um token escolhido aqui — senão a escada volta a ter duas respostas.
    fill_rounded_rect(
        ctx.scene,
        card,
        radius,
        resolve(
            ph2d_editor_core::widget::section_cards::CardDepth::Subsection.token(),
            theme,
        ),
    );
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        card,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    // Card title — a discreet caption in the card's top-left.
    paint_text(
        ctx.text_system,
        ctx.scene,
        title,
        x + pad,
        y + pad,
        font,
        content_w - 2.0 * pad,
        resolve(ColorToken::Text2, theme),
    );
    (
        x + pad,
        content_w - 2.0 * pad,
        y + pad + title_h,
        y + card_h + ph2d_tokens::control_gap_px(),
    )
}

/// One `label · number-box` row inside a card — **pela linha numérica deste painel**
/// ([`number_field::paint_num_row`], que passa pela porta da casa) e com a coluna da SECÇÃO da chave.
///
/// ⛔⛔ **Até 2026-09-23 o cartão tinha a coluna dele: `CARD_LABEL_W = 96`**, escrita aqui, com o
/// rótulo encostado à esquerda — e as caixas de marcar e as escolhas do MESMO cartão já viviam na
/// coluna da secção. *Duas colunas de nome dentro de um cartão*, que é o defeito que a
/// `seccoes.rs` existe para matar, e a última entrada da catraca `COLUNAS_A_MAO` que era uma linha
/// de propriedade. ⚠️ A razão escrita para ela (*«descriptive technique names fit»*) é a razão da
/// secção declarada: a coluna mede o nome mais largo do cartão, e nenhum literal cresce com o dock.
///
/// ⭐ **Recebe a CHAVE e não o texto**, pela lei do [`crate::seccoes::seccao_da_chave`]: *quem só
/// tem o texto traduzido não sabe a que secção pertence*.
#[allow(clippy::too_many_arguments)]
pub(crate) fn card_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    chave: &str,
    id: ph2d_a11y::NodeId,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
) -> f32 {
    let seccao = crate::seccoes::seccao_da_chave(ctx.text_system, chave);
    number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr(chave),
        id,
        value,
        min,
        max,
        step,
        decimals,
        seccao,
    )
}
