//! **A fileira `Pigment` — uma lei, uma porta, três meios.**
//!
//! Até 2026-09-20 a mistura subtractiva era um controlo da aguada: a fileira vivia dentro do cartão
//! **Water** da secção Watercolor e o motor a gateava com `watercolor && pigment`. A ordem do dono
//! desse dia (*«ligue o digital»*) tirou a cerca do meio — a lei passou a valer para todo meio que
//! componha um dab pela porta `blend_over_pigment` —, e com ela a fileira deixou de pertencer a uma
//! secção só.
//!
//! ⛔⛔ **Por isso ela mora AQUI, e num hospedeiro SÓ.** O valor que o slider mostra não é um campo:
//! ele é DERIVADO (`pigment ? pigment_mix : 0`, o par toggle+amount que a redesign de 2026-07-07
//! fundiu num controlo só), e *duas derivações do mesmo número divergem no dia em que alguém afina
//! uma delas*.
//!
//! ⚠️⚠️ **A 1.ª redacção tinha DOIS hospedeiros** — este cartão e o cartão *Water* da aguada, que
//! era onde a fileira nasceu — e um CENSO mandou-a juntar-se: com a fileira a poder desaparecer (o
//! 2.º report do dono), o `n_rows` do cartão Water passou a ser derivado e o
//! `o_numero_de_linhas_que_um_cartao_declara_e_o_que_ele_pinta` reprovou em voz alta (*«não é um
//! literal»*). ⛔ *A cura barata era cegar o censo; a certa era o cartão de altura variável ser
//! ESTE, que some inteiro.*
//!
//! ⚠️ **Quem responde «isto é oferecido AGORA?» é o `BrushSettings::pigment_offered`**, derivado
//! pela ferramenta em `PaintMedia::offers_pigment_mixing_in` — o MEIO ∧ o GESTO na mão —, e as duas
//! listas são MEDIDAS (`diag_pigmento_por_meio`), não raciocinadas: Digital · Watercolor · Impasto
//! sentem a lei e o **Wet Paint não**; e dos NOVE gestos que desenham esta metade do painel, só o
//! **pincel** deposita a cor de um dab. Um caso que lê a lei e não vê a fileira é um knob
//! INALCANÇÁVEL; um que a vê e não a lê é um knob MORTO — as duas leem-se igual numa tabela, e só a
//! medição as separa.

use crate::card::{card_frame, card_row};
use crate::number_field;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::BrushSettings;

/// O número que o slider **mostra**: a quantidade quando a mistura está ligada, senão `0`.
///
/// ⚠️ Ele não é o `pigment_mix` cru. O controlo é o par *toggle + amount* fundido (redesign
/// 2026-07-07): `0` desliga a mistura **guardando** a última quantidade, para que subir o slider a
/// devolva sem perda. Quem escreve é o `set_brush_pigment_mixing` do lado da ferramenta; esta função
/// é a metade de LEITURA da mesma lei, e existe para as duas não divergirem.
pub(crate) fn pigment_amount(brush: &BrushSettings) -> f32 {
    if brush.pigment {
        brush.pigment_mix
    } else {
        0.0
    }
}

/// Pinta a fileira `Pigment` numa linha de cartão já enquadrada e devolve o `y` da linha seguinte.
pub(crate) fn paint_pigment_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    ix: f32,
    iw: f32,
    ry: f32,
    brush: &BrushSettings,
) -> f32 {
    card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        tr("panel.painter_layers.watercolor.pigment"),
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_MIX,
        pigment_amount(brush),
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    )
}

/// O cartão **Mixing** — o hospedeiro ÚNICO da fileira, acima da secção do meio.
///
/// ⚠️ Ele fica acima porque o `Pigment` governa **três** meios, e a lei desta casa já diz que um
/// controlo que reinterpreta o que está abaixo dele se senta ACIMA (é o argumento escrito para o
/// chip do Paint Mode). Um cartão de uma linha é o preço de ela poder desaparecer inteira.
pub(crate) fn paint_mixing_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, ry, next_y) = card_frame(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.brush.mixing"),
        1,
    );
    let _ = paint_pigment_row(ctx, theme, ix, iw, ry, brush);
    next_y
}
