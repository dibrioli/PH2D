//! **A BIBLIOTECA da §11** — a lista de animações e o editor da que está aberta.
//!
//! ⚠️ **Irmão de [`super::anim`] por CAP de FICHEIRO** (600), o mesmo corte que a §12 fez entre
//! [`super::anchors`] e [`super::anchor_mount_row`].
//!
//! ⚠️ **A linha selecionada É a animação que toca** — ver o doc de [`super::anim`]. Por isso ela
//! desenha-se com o mesmo realce que a §12 dá à ficha aberta, e o clique nela vai ao barramento.

use super::*;
use ph2d_editor_core::screens::hero::InspectorAnimInfo;

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector
const ROW_H: f32 = 22.0; // LITERAL-PX-OK: altura de uma linha da lista
/// Onde o resumo (`2-5 · Forward`) começa, como fração da largura.
const SUMMARY_COL_FRAC: f32 = 0.52; // LITERAL-PX-OK: fração de layout

/// A lista mais o editor. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_library(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    selected: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;
    paint_text(
        text_system,
        scene,
        "This sprite's animations",
        x,
        cur_y,
        font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    cur_y += font + Spacing::Xs.px();

    if info.rows.is_empty() {
        paint_text(
            text_system,
            scene,
            "No animations yet.",
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + Spacing::Sm.px();
    } else {
        // ⚠️ `zip` com o array de ids: uma biblioteca com mais tags do que ids (impossível
        // enquanto o gate `the_anim_row_ids_cover_the_model_cap` viver) perde as excedentes em
        // vez de as pintar umas sobre as outras.
        for (i, (row, &id)) in info.rows.iter().zip(ids::INSP_ANIM_ROW.iter()).enumerate() {
            let rect = Rect::new(x, cur_y, w, ROW_H);
            hit_index.register(id, rect);
            // **Duas coisas diferentes, dois realces:** a que está ABERTA no editor (fundo), e a
            // que está a TOCAR (o texto aceso). Elas são a mesma na esmagadora maioria dos
            // casos — mas não quando a que toca deixou de caber na grelha.
            let is_open = i == selected;
            let is_current = info.current == row.name;
            if is_open {
                ph2d_editor_core::paint::fill_rounded_rect(
                    scene,
                    rect,
                    Radius::Sm.px(),
                    resolve(ColorToken::Bg2, theme),
                );
            }
            let color = if is_current {
                resolve(ColorToken::Accent, theme)
            } else if is_open {
                resolve(ColorToken::Text1, theme)
            } else {
                resolve(ColorToken::Text2, theme)
            };
            paint_text(
                text_system,
                scene,
                &row.name,
                x + Spacing::Sm.px(),
                cur_y + (ROW_H - font) * 0.5,
                font,
                w,
                color,
            );
            // ⚠️ **O resumo diz o que a linha FAZ**, e o alcance sai da grelha de hoje: uma tag
            // que deixou de caber mostra-o aqui, e não só quando alguém carrega em Play.
            let dir = ph2d_ecs_dir_label(row.direction_tag);
            let summary = if row.fits(info.cells) {
                format!(
                    "{}-{} \u{b7} {dir}",
                    row.from.min(row.to),
                    row.from.max(row.to)
                )
            } else {
                String::from("out of grid")
            };
            let summary_color = if row.fits(info.cells) {
                resolve(ColorToken::Text3, theme)
            } else {
                resolve(ColorToken::Danger, theme)
            };
            paint_text(
                text_system,
                scene,
                &summary,
                x + w * SUMMARY_COL_FRAC,
                cur_y + (ROW_H - font) * 0.5,
                font,
                w,
                summary_color,
            );
            cur_y += ROW_H;
        }
        cur_y += Spacing::Sm.px();
        if let Some(row) = info.rows.get(selected.min(info.rows.len() - 1)) {
            cur_y = editor(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                row,
            );
        }
    }

    let add = Rect::new(x, cur_y, w, BTN_H);
    hit_index.register(ids::INSP_ANIM_ADD, add);
    paint_button(
        &Button::new(ids::INSP_ANIM_ADD, "+ Add Animation")
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ANIM_ADD)),
        add,
        scene,
        text_system,
        theme,
    );
    cur_y += BTN_H + Spacing::Xs.px();
    if !info.rows.is_empty() {
        let rm = Rect::new(x, cur_y, w, BTN_H);
        hit_index.register(ids::INSP_ANIM_REMOVE, rm);
        paint_button(
            &Button::new(ids::INSP_ANIM_REMOVE, "x Remove Animation")
                .kind(ButtonKind::Default)
                .visual(store.button_visual(ids::INSP_ANIM_REMOVE)),
            rm,
            scene,
            text_system,
            theme,
        );
        cur_y += BTN_H;
    }
    cur_y + Spacing::Sm.px()
}

/// O rótulo de uma direção, pelo tag. ⚠️ **Espelha `ph2d_ecs::AnimDirection::label`** porque este
/// painel não vê o motor; o gate da shell prende os dois, como no `kind_label` da §12.
fn ph2d_ecs_dir_label(tag: u8) -> &'static str {
    match tag {
        1 => "Reverse",
        2 => "Ping-Pong",
        3 => "Ping-Pong Rev",
        _ => "Forward",
    }
}

/// O editor da animação aberta.
#[allow(clippy::too_many_arguments)]
fn editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &ph2d_editor_core::screens::hero::InspectorAnimRow,
) -> f32 {
    let mut cur_y = y;
    cur_y = text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_ANIM_NAME,
        TextInput::new(ids::INSP_ANIM_NAME, "").placeholder("animation_name\u{2026}"),
    );

    cur_y = super::anchors::field_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "From / To (cell)",
        &[ids::INSP_ANIM_FROM, ids::INSP_ANIM_TO],
        1.0,
    );
    cur_y = super::anchors::field_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Frame ms / Repeat (0 = forever)",
        &[ids::INSP_ANIM_FRAME_MS, ids::INSP_ANIM_REPEAT],
        1.0,
    );
    // ⚠️ **Uma animação com ritmo PRÓPRIO por célula (§8.12) tem de o DIZER.** Sem esta linha o
    // campo `Frame ms` acima mente sobre ela — mostra a duração mais comum e nada explica por que
    // a animação não anda naquele ritmo. O Inspector não a edita (a §8.8 põe essa edição no editor
    // de timeline futuro); ele diz que ela existe, que é a diferença entre um dado e um mistério.
    if row.has_per_frame_timing() {
        paint_text(
            text_system,
            scene,
            "\u{2022} this animation has per-frame timing (imported)",
            x,
            cur_y,
            TypeToken::Sm.px(),
            w,
            resolve(ColorToken::Warn, theme),
        );
        cur_y += TypeToken::Sm.px() + Spacing::Xs.px();
    }
    cur_y = super::anchors::field_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Hold ms / Repeat delay ms",
        &[ids::INSP_ANIM_HOLD_MS, ids::INSP_ANIM_DELAY_MS],
        10.0, // LITERAL-PX-OK: passo de scrub em MILISSEGUNDOS, não em pixels
    );

    // A direção, como quatro botões. ⚠️ A seleção vem do SNAPSHOT, e não do store.
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        "Direction",
        x,
        cur_y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    cur_y += font + Spacing::Xs.px();
    let gap = Spacing::Xs.px();
    let n = ids::INSP_ANIM_DIR.len() as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, (&id, label)) in ids::INSP_ANIM_DIR
        .iter()
        .zip(["Fwd", "Rev", "PP", "PP Rev"].iter())
        .enumerate()
    {
        let rect = Rect::new(x + (cw + gap) * i as f32, cur_y, cw, ROW_H_PX);
        hit_index.register(id, rect);
        let kind = if usize::from(row.direction_tag) == i {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, *label)
                .kind(kind)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    cur_y += ROW_H_PX + Spacing::Sm.px();

    // **OS SINAIS** (spec §8.10) — no FIM, e depois da direção, porque eles são o que a animação
    // diz para FORA. Tudo acima descreve o que ela faz; isto descreve o que ela anuncia.
    //
    // ⚠️ **Dois campos e não um mais uma fase**: acabar e dar a volta distinguem-se por serem
    // nomes diferentes — a lei que a física já escreveu para os contatos. Vazio = calada.
    paint_text(
        text_system,
        scene,
        "Signals (empty = silent)",
        x,
        cur_y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    cur_y += font + Spacing::Xs.px();
    cur_y = text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_ANIM_SIGNAL_FINISH,
        TextInput::new(ids::INSP_ANIM_SIGNAL_FINISH, "").placeholder("on finish\u{2026}"),
    );
    text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_ANIM_SIGNAL_LOOP,
        TextInput::new(ids::INSP_ANIM_SIGNAL_LOOP, "").placeholder("on loop\u{2026}"),
    )
}

/// **Uma linha de texto da §11**, pintada do `WidgetStore` — o nome da animação e os dois nomes de
/// sinal são a MESMA coisa vista de três sítios.
///
/// ⚠️ Um helper e não três blocos iguais: o nome já vivia inline aqui, e copiá-lo duas vezes era a
/// forma de o *placeholder*, o `hover_live` ou o caret divergirem entre campos que o artista lê
/// como irmãos.
///
/// ⚠️ **O `TextInput` chega PRONTO do chamador, e o `placeholder` não é um parâmetro `&str`.** A
/// primeira versão recebia a string — e o gate do HR-15 reprovou: ele conta `.placeholder("…")` no
/// código de widget, e passar a string por um argumento tira **três** strings de UI da vista dele.
/// *Um helper que esconde literais do scanner é uma isenção silenciosa da regra que ele scanneia.*
#[allow(clippy::too_many_arguments)]
fn text_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    input: TextInput,
) -> f32 {
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H_PX);
    let host = Rect::new(x, y, control_w, ROW_H_PX);
    hit_index.register(id, host);
    let (state, text, caret, sel_anchor) = match store.get(id) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let input = input.visual((state, store.hover_live(id)));
    paint_text_input_with_buffer(
        &input,
        text,
        Some(caret),
        sel_anchor,
        host,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + ROW_H_PX + Spacing::Sm.px()
}
