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
    yy = paint_rope_row(scene, text_system, theme, hit_index, store, x, w, yy, info);
    yy = paint_mount_row(scene, text_system, theme, hit_index, store, x, w, yy, info);
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
    // **Out Radius** — o SEGUNDO diâmetro do eixo (W4). `0` é uma roldana comum;
    // qualquer outro valor faz dela um TAMBOR DIFERENCIAL, e a corda ganha
    // vantagem mecânica `Radius / Out Radius`.
    //
    // ⚠️ Row SEMPRE pintada, e não só quando já há um diferencial: ela é o único
    // gesto que CRIA um, e um controle que só aparece depois de a coisa existir
    // não pode ser o que a faz existir. (É a mesma lei da §11 vazia do W2a.)
    yy = num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Out Radius (m)",
        ids::INSP_WHEEL_RADIUS_OUT,
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
    yy = paint_break_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);
    yy + SECTION_BOTTOM_PAD_PX
}

/// **O card de ruptura do EIXO** — o switch, o limiar que ele gateia, e a carga
/// que torna o limiar afinável.
///
/// ⚠️ **O limiar é por-RODA enquanto o da corda é um só**, e a razão é física: a
/// tensão é uniforme ao longo da corda, mas a carga do eixo depende do ÂNGULO do
/// enlace — 2T num enlace de 180°, quase nada numa roldana que só encosta na
/// corda.
///
/// ⚠️ **A CARGA VIVA não está aqui, de propósito.** Ela é uma anotação de
/// OVERLAY, ao lado da roda, exatamente como a do joint — o `publish` do
/// snapshot não recebe a ponte (está escrito lá), e um segundo lugar mostrando o
/// mesmo número seria a segunda resposta que diverge.
#[allow(clippy::too_many_arguments)]
fn paint_break_rows(
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
    let yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        "Axle Breaks",
        ids::INSP_WHEEL_BREAK_GROUP,
        &ids::INSP_WHEEL_BREAK,
        &["Off", "On"],
        u8::from(info.break_enabled),
    );
    if !info.break_enabled {
        return yy;
    }
    num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Break Force (N)",
        ids::INSP_WHEEL_BREAK_FORCE,
    )
}

/// **Em que CORPO esta roldana se monta** (W3) — o nome vigente, o eyedropper que
/// arma o pick de canvas, e o botão de desmontar.
///
/// ⚠️ **É a row que traz a vantagem mecânica**, e ela não tem número nenhum: uma
/// roldana montada num corpo que se move é a *cadernal móvel* de uma talha, e o
/// corpo passa a ser sustentado por DOIS ramos da mesma corda. O "2" não é
/// autorado em lugar nenhum — ele é a geometria do enlace.
///
/// ⚠️ **O eyedropper ARMA**, ele não escolhe: o alvo vem do próximo clique no
/// canvas, o idiom de pick deste app (o eyedropper de cor, o `vec_path_pick`, o
/// re-pick de corpo do joint). Ele pinta `Pressed` enquanto espera.
///
/// ⚠️ **O botão de desmontar só existe quando há o que desmontar** — a lei do
/// botão-morto. Sem ele a única saída de uma montagem errada seria apagar a
/// roldana e refazê-la, que é a queixa (4) que abriu esta wave.
#[allow(clippy::too_many_arguments)]
fn paint_mount_row(
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
    let h = ROW_H_PX;
    let font = TypeToken::Sm.px();
    let icon_w = (h * 0.82).min(w); // LITERAL-PX-OK: icon inset ratio (compact square in the row)
    let label_w = (font * 5.0).min(w * 0.42); // LITERAL-PX-OK: label = 5 char-heights, capped at 0.42 of the row
    let gap = Spacing::Sm.px();
    let mounted = !info.mount_name.is_empty();
    let icons = if mounted { 2.0 } else { 1.0 };
    let text_y = y + (h - font) * 0.5;
    paint_text(
        text_system,
        scene,
        "Mounted On",
        x,
        text_y,
        font,
        label_w,
        resolve(ColorToken::Text2, theme),
    );
    // "(scenery)" e não um vazio: *pregada no cenário* é uma resposta, e uma
    // lacuna onde devia haver um nome não é.
    paint_text(
        text_system,
        scene,
        if mounted {
            info.mount_name.as_str()
        } else {
            "(scenery)"
        },
        x + label_w,
        text_y,
        font,
        (w - label_w - icons * (icon_w + gap)).max(0.0),
        resolve(
            if mounted {
                ColorToken::Text1
            } else {
                ColorToken::Text3
            },
            theme,
        ),
    );
    let mut bx = x + w - icon_w;
    if mounted {
        let brect = Rect::new(bx, y + (h - icon_w) * 0.5, icon_w, icon_w);
        paint_icon_button(
            brect,
            IconGlyph::Builtin(IconId::Trash),
            IconButtonStyle::Compact,
            store
                .button_state(ids::INSP_WHEEL_UNMOUNT)
                .unwrap_or(ButtonState::Normal),
            scene,
            theme,
        );
        hit_index.register(ids::INSP_WHEEL_UNMOUNT, brect);
        bx -= icon_w + gap;
    }
    let brect = Rect::new(bx, y + (h - icon_w) * 0.5, icon_w, icon_w);
    paint_icon_button(
        brect,
        IconGlyph::Builtin(IconId::Eyedropper),
        IconButtonStyle::Compact,
        if info.mount_pick_armed {
            ButtonState::Pressed
        } else {
            store
                .button_state(ids::INSP_WHEEL_MOUNT_PICK)
                .unwrap_or(ButtonState::Normal)
        },
        scene,
        theme,
    );
    hit_index.register(ids::INSP_WHEEL_MOUNT_PICK, brect);
    y + h
}

/// **A que corda esta roldana pertence** — o nome, um aviso quando ele não resolve,
/// e o eyedropper que a RE-ESCOLHE (W1).
///
/// Uma roldana órfã (corda apagada ou renomeada) é **inerte, não quebrada** — ela
/// some da rota e nada mais acontece —, e dizê-lo em voz alta é a mesma escolha do
/// `bound` da §12. O que faltava era a **volta**: até esta wave a corda só era
/// escolhida no gesto que CRIA a roldana, então o caminho de cura era apagar e
/// refazer.
///
/// ⚠️ **O eyedropper é irmão do de montagem no GESTO e não no ALVO** (`rope_at_world`
/// vs `pick_sprites_at_world`): ele arma, o próximo clique no canvas escolhe, e
/// pinta `Pressed` enquanto espera — mas o alvo é a **ROTA desenhada**, porque a
/// entidade-corda não tem sprite e o gesto do corpo resolveria `None` para sempre.
///
/// ⚠️ **Sempre oferecido, inclusive na roldana ÓRFÃ** — e é ali que ele mais serve:
/// gatear em `bound` esconderia o botão exatamente no estado que ele existe para
/// consertar.
#[allow(clippy::too_many_arguments)]
fn paint_rope_row(
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
    let h = ROW_H_PX;
    let font = TypeToken::Sm.px();
    let icon_w = (h * 0.82).min(w); // LITERAL-PX-OK: icon inset ratio (compact square in the row)
    let label_w = (font * 5.0).min(w * 0.42); // LITERAL-PX-OK: label = 5 char-heights, capped at 0.42 of the row
    let gap = Spacing::Sm.px();
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
    paint_text(
        text_system,
        scene,
        if info.bound {
            info.rope_name.as_str()
        } else {
            "(no rope)"
        },
        x + label_w,
        text_y,
        font,
        (w - label_w - (icon_w + gap)).max(0.0),
        resolve(
            if info.bound {
                ColorToken::Text1
            } else {
                ColorToken::Text3
            },
            theme,
        ),
    );
    let brect = Rect::new(x + w - icon_w, y + (h - icon_w) * 0.5, icon_w, icon_w);
    paint_icon_button(
        brect,
        IconGlyph::Builtin(IconId::Eyedropper),
        IconButtonStyle::Compact,
        if info.rope_pick_armed {
            ButtonState::Pressed
        } else {
            store
                .button_state(ids::INSP_WHEEL_ROPE_PICK)
                .unwrap_or(ButtonState::Normal)
        },
        scene,
        theme,
    );
    hit_index.register(ids::INSP_WHEEL_ROPE_PICK, brect);
    y + h
}
