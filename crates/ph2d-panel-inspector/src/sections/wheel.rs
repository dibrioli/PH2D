//! **§13 Pulley Wheel** — a roldana selecionada (W-Pulley W1).
//!
//! Seção própria porque **uma roldana é uma ENTIDADE**: quando estas rows
//! importam, ela é o objeto selecionado, e a §12 ao lado só existe com a CORDA
//! selecionada.
//!
//! ⚠️ **A POSIÇÃO não está aqui.** O centro da roldana É o
//! `Transform.translation`, que a §2 já pinta para toda entidade — e o dot do
//! canvas edita o mesmo campo. Uma row de posição aqui seria a segunda porta
//! para um fato que já tem dono.
//!
//! Das três rows, **duas são o único gesto que existe**: até esta seção, `Over`/
//! `Under` (o escape manual do pedido 7) e a `order` eram estado autorado,
//! serializado, que muda a rota — e que nenhum gesto do editor conseguia
//! alcançar. O raio é o caso oposto: a alça do aro no canvas já o edita, e as
//! duas portas escrevem o MESMO campo.

use super::rows::{num_row, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorWheelInfo;

/// Os chips do lado — a ordem de `WrapSide::ALL`, que é a ordem dos tags.
const WRAP_LABELS: [&str; 3] = ["Auto", "Over", "Under"];

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_wheel_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorWheelInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_WHEEL_SECTION);
    let color_id = ids::INSP_LIVE_WHEEL_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = SectionHeader::new(ids::INSP_LIVE_WHEEL_SECTION, "Pulley Wheel")
        .collapsible(!collapsed)
        .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    if collapsed {
        return y + header_h;
    }

    let mut yy = y + header_h;
    yy = paint_rope_row(scene, text_system, theme, x, w, yy, info);
    yy = num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Radius (m)",
        ids::INSP_WHEEL_RADIUS,
    );
    // **Order** — a rota é uma SEQUÊNCIA, e duas roldanas trocadas descrevem
    // outra corda. Sem esta row a única forma de reordenar seria apagar e
    // recriar; empate é resolvido pelo nome, então um número repetido é inerte,
    // nunca um erro.
    yy = num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Order",
        ids::INSP_WHEEL_ORDER,
    );
    // **Motor** — esta roldana é um TAMBOR. Graus por segundo porque a grandeza é
    // ANGULAR, e é isso que faz o diâmetro ser o câmbio: a corda anda `ω·r`,
    // então o mesmo motor num tambor maior recolhe mais depressa. Mesma unidade
    // que o motor do Pin usa na §12.
    yy = num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Motor (\u{00b0}/s)",
        ids::INSP_WHEEL_MOTOR,
    );
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Wrap",
        ids::INSP_WHEEL_WRAP_GROUP,
        &ids::INSP_WHEEL_WRAP,
        &WRAP_LABELS,
        info.wrap_tag,
    );
    yy + SECTION_BOTTOM_PAD_PX
}

/// **A que corda esta roldana pertence** — nome, e um aviso quando ele não
/// resolve.
///
/// Só leitura: a corda é escolhida no gesto que cria a roldana. Uma roldana
/// órfã (corda apagada ou renomeada) é **inerte, não quebrada** — ela some da
/// rota e nada mais acontece —, e dizê-lo em voz alta é a mesma escolha do
/// `bound` da §12: mostrar raio e lado como se ela estivesse na corda seria
/// mentira.
fn paint_rope_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorWheelInfo,
) -> f32 {
    let h = ROW_H_PX;
    let font = TypeToken::Sm.px();
    let label_w = (font * 3.0).min(w * 0.4); // LITERAL-PX-OK: label = 3 char-heights, capped at 0.4 of the row
    let text_y = y + (h - font) * 0.5;
    paint_text(
        text_system,
        scene,
        "Rope",
        x,
        text_y,
        font,
        label_w,
        resolve(ColorToken::Text2, theme),
    );
    let shown = if info.bound {
        info.rope_name.as_str()
    } else {
        "(no rope)"
    };
    paint_text(
        text_system,
        scene,
        shown,
        x + label_w,
        text_y,
        font,
        (w - label_w).max(0.0),
        resolve(
            if info.bound {
                ColorToken::Text1
            } else {
                ColorToken::Text3
            },
            theme,
        ),
    );
    y + h
}
