//! ⭐⭐⭐ **A seção COMPONENT** (ADR-0164 / F5) — *«o que esta cópia tem de diferente da receita»*.
//!
//! Ver [`ph2d_editor_core::screens::hero::InspectorInstanceInfo`] para o buraco que ela fecha: o
//! modelo de override existe desde a F4.4 e era **inteiramente invisível** — o artista lia-o pelo
//! COMPORTAMENTO (a receita deixou de alcançar aquela peça), nunca por um sinal.

use super::*;
use ph2d_editor_core::screens::hero::InspectorInstanceInfo;

/// Pinta a seção. Devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_instance_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorInstanceInfo,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let mut y = y;
    let font = TypeToken::Base.px();
    let small = TypeToken::Sm.px();
    let line = font + Spacing::Xs.px();
    // A linha de proveniência: de que receita esta cópia nasceu. É a única superfície que o diz —
    // a Hierarquia mostra a árvore, não o vínculo.
    paint_text(
        text_system,
        scene,
        &format!("Instance of \u{201c}{}\u{201d}", info.master_name),
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text1, theme),
    );
    y += line;
    paint_text(
        text_system,
        scene,
        &info.summary(),
        x,
        y,
        small,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y += line;

    // ⚠️ **Uma linha por componente overridado, pelo NOME que o `+` usa.** Sem elas o artista sabe
    // que «alguma coisa» está diferente e não sabe o quê — que é metade do defeito.
    for name in &info.overridden {
        paint_text(
            text_system,
            scene,
            &format!("\u{2022} {name}"),
            x + Spacing::Md.px(),
            y,
            font,
            w,
            resolve(ColorToken::Text1, theme),
        );
        y += line;
    }

    // ⭐ O gesto dos ÓRFÃOS — e ele **só aparece quando existem**: um botão permanentemente inerte
    // é ruído que o artista aprende a ignorar.
    if info.orphans > 0 {
        let host = Rect::new(x, y, w, ROW_H_PX);
        hit_index.register(ids::INSP_INSTANCE_CLEAR_ORPHANS, host);
        let button = Button::new(
            ids::INSP_INSTANCE_CLEAR_ORPHANS,
            format!("Clear {} unused override(s)", info.orphans),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_INSTANCE_CLEAR_ORPHANS));
        paint_button(&button, host, scene, text_system, theme);
        y += ROW_H_PX;
    }
    y + SECTION_BOTTOM_PAD_PX
}
