//! **A §11 Animation** — a última das doze seções do Inspector (spec
//! [`08_animation_inline.md`](../../../../docs/Sprite_projeto/08_animation_inline.md) §8.7).
//!
//! # As duas metades, e os dois donos
//!
//! - **O TOCADOR** (`SpriteAnimator`): o que está a tocar, a que velocidade, com que direção.
//! - **A BIBLIOTECA** (`SpriteAnimations`): as animações desta sprite — nome, intervalo, ritmo.
//!
//! São dois componentes distintos no motor, e a seção di-lo com dois rótulos. É a lição que a §12
//! pagou ao pôr as âncoras deste objeto e a âncora do pai na mesma lista sem dizer de quem eram.
//!
//! # ⚠️ Clicar numa linha ESCOLHE a animação — e isso difere da §12
//!
//! Na §12, clicar numa linha muda só a ficha aberta e não vai ao barramento. Aqui vai: numa
//! biblioteca de animações, *a que se vê* e *a que toca* serem a mesma é o que o artista espera
//! (o `AnimationPlayer` do Godot faz assim), e separá-las pediria um seletor com as mesmas
//! entradas da lista logo abaixo dele — 65 ids e uma máquina de popover para duplicar uma lista.
//!
//! # ⛔ Sem tocador, a seção mostra UM botão
//!
//! Pintar os controlos de reprodução sobre uma sprite que não tem estado de reprodução seria
//! oferecer knobs que não vão a lado nenhum — o defeito que o `Simple` do 9-slice era.

use super::*;
use ph2d_editor_core::screens::hero::InspectorAnimInfo;
use ph2d_editor_core::widget::SectionFold;

const BTN_H: f32 = 30.0; // LITERAL-PX-OK: altura de botão do Inspector
const CHECK_H: f32 = 18.0; // LITERAL-PX-OK: altura visual do Checkbox
const BAR_H: f32 = 6.0; // LITERAL-PX-OK: espessura da barra de progresso

/// Um segmentado de N botões numa linha, com o índice `sel` aceso. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn segmented_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    ids_: &[NodeId],
    labels: &[&str],
    sel: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let row_y = y + font + Spacing::Xs.px();
    let gap = Spacing::Xs.px();
    let n = ids_.len().max(1) as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, (&id, text)) in ids_.iter().zip(labels.iter()).enumerate() {
        let rect = Rect::new(x + (cw + gap) * i as f32, row_y, cw, ROW_H_PX);
        hit_index.register(id, rect);
        // ⚠️ **A seleção vem do SNAPSHOT**, não do store: o store guarda o visual do botão, e ler
        // dali qual está aceso faria o realce sobreviver à troca de sprite.
        let kind = if i == sel {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, *text)
                .kind(kind)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    row_y + ROW_H_PX + Spacing::Sm.px()
}

/// O bloco do TOCADOR. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn player_block(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cur_y = y;

    // As duas caixas, lado a lado.
    let half = (w - Spacing::Sm.px()) * 0.5;
    for (i, (id, label, on)) in [
        (ids::INSP_ANIM_PLAYING, "Playing", info.playing),
        (ids::INSP_ANIM_AUTOPLAY, "Autoplay", info.autoplay),
    ]
    .into_iter()
    .enumerate()
    {
        let rect = Rect::new(
            x + (half + Spacing::Sm.px()) * i as f32,
            cur_y,
            half,
            CHECK_H,
        );
        hit_index.register(id, rect);
        paint_checkbox(
            &Checkbox::new(id, label)
                .visual(store.checkbox_visual(id))
                .value(if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                }),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    cur_y += CHECK_H + Spacing::Sm.px();

    cur_y = super::anchors::field_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Speed (x)",
        &[ids::INSP_ANIM_SPEED],
        0.1,
    );

    cur_y = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Direction override",
        &ids::INSP_ANIM_DIR_OVERRIDE,
        &["Inherit", "Fwd", "Rev", "PP", "PP Rev"],
        usize::from(info.direction_override_tag),
    );
    cur_y = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Loop override",
        &ids::INSP_ANIM_LOOP_OVERRIDE,
        &["Inherit", "On", "Off"],
        usize::from(info.loop_override_tag),
    );

    // **O frame, e onde ele está dentro da animação.**
    //
    // ⚠️ O número é `frame + 1 / total` — o artista conta a partir de um, e a barra é o mesmo
    // facto desenhado. *Uma fonte, duas leituras*: os dois saem do `progress()` do snapshot.
    if let Some((step, span)) = info.progress() {
        let text = format!("Frame {} / {span}", step + 1);
        paint_text(
            text_system,
            scene,
            &text,
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text2, theme),
        );
        cur_y += font + Spacing::Xs.px();
        ph2d_editor_core::paint::fill_rounded_rect(
            scene,
            Rect::new(x, cur_y, w, BAR_H),
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
        let frac = (step + 1) as f32 / span as f32;
        ph2d_editor_core::paint::fill_rounded_rect(
            scene,
            Rect::new(x, cur_y, w * frac, BAR_H),
            Radius::Sm.px(),
            resolve(ColorToken::Accent, theme),
        );
        cur_y += BAR_H + Spacing::Sm.px();
    }

    if info.current_dangling() {
        paint_text(
            text_system,
            scene,
            "This sprite has no animation with that name, or the grid shrank under it.",
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Danger, theme),
        );
        cur_y += font + Spacing::Xs.px();
    }

    let rw = Rect::new(x, cur_y, w, BTN_H);
    hit_index.register(ids::INSP_ANIM_REWIND, rw);
    paint_button(
        &Button::new(ids::INSP_ANIM_REWIND, "Rewind")
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ANIM_REWIND)),
        rw,
        scene,
        text_system,
        theme,
    );
    cur_y + BTN_H + Spacing::Sm.px()
}

/// Pinta a §11 e devolve o `y` a seguir a ela.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_anim_section(
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
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = ids::INSP_LIVE_ANIM_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let title = if info.rows.is_empty() {
        String::from("Animation")
    } else {
        format!("Animation  ({})", info.rows.len())
    };
    let header = section_header(store, ids::INSP_LIVE_ANIM_SECTION, &title).color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        ids::INSP_LIVE_ANIM_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    let font = TypeToken::Sm.px();

    // ⛔ Sem tocador, UM botão — e mais nada. Ver o doc do módulo.
    if !info.player_present {
        paint_text(
            text_system,
            scene,
            "This sprite does not play animations yet.",
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + Spacing::Sm.px();
        let add = Rect::new(x, cur_y, w, BTN_H);
        hit_index.register(ids::INSP_ANIM_ADD_PLAYER, add);
        paint_button(
            &Button::new(ids::INSP_ANIM_ADD_PLAYER, "+ Add Animator")
                .kind(ButtonKind::Default)
                .visual(store.button_visual(ids::INSP_ANIM_ADD_PLAYER)),
            add,
            scene,
            text_system,
            theme,
        );
        return fold.finish(
            store,
            scene,
            hit_index,
            cur_y + BTN_H + SECTION_BOTTOM_PAD_PX,
        );
    }

    cur_y = player_block(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    cur_y = super::anim_rows::paint_library(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        selected,
    );
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
