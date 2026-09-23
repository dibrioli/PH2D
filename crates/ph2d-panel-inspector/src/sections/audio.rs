//! ⭐⭐⭐ **A secção AUDIO** — o som de um objecto da cena (TOP-20 #4, W3).
//!
//! # ⚠️ Ela nasce COM a wave, e isso é a lição que o `Timers` custou
//!
//! O `Timers` shipou anexável e sem linha de edição, e o report do dono foi *«timer sumiu do modal
//! de componente»* — o componente **estava** na paleta; o que não existia era o que acontece depois
//! de o anexar. *Um componente anexável sem painel é indistinguível de um que não foi anexado.*
//!
//! # ⚠️ Uma secção, DOIS corpos
//!
//! Um objecto pode ter a FONTE, as ORELHAS, ou as duas — dois componentes registados, uma pergunta
//! só para o artista. Ver o doc de [`ph2d_editor_core::ids::inspector_audio`].
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! Esta secção tem **quatro** avisos, e cada um responde a uma forma diferente de *«não ouço
//! nada»* — que é a única pergunta que um artista faz sobre som:
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `no file` | ainda não escolheu | **Browse…** |
//! | `file is gone` | o caminho não abre | escolher outra vez |
//! | `nothing plays this` | não arranca sozinha e **nenhum** sinal a manda tocar | ligar *Autoplay*, ou uma linha de *Signal Actions* |
//! | `no listener` | a cena não tem orelhas | pôr um `Audio Listener 2D` num objecto |
//!
//! ⚠️ **O terceiro é o que só o snapshot pode dizer**: ele é uma varredura das tabelas de acção da
//! cena. Um painel que só olhasse para o componente diria *«autoplay desligado»* sobre uma cena
//! perfeitamente correcta.

use super::*;
use ph2d_editor_core::screens::hero::{InspectorAudioInfo, InspectorAudioSource};
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

/// **As opções do seletor de barramento** — uma por `AudioBus::ALL`, na ordem dele.
///
/// ⚠️ **A posição é a tag**, e `zip` com os ids: uma lista de rótulos mais curta perde as
/// excedentes em vez de as pintar sem nome.
pub(crate) fn bus_options(labels: &[String]) -> Vec<DropdownOption<u8>> {
    ids::INSP_AUDIO_BUS_OPT
        .iter()
        .enumerate()
        .zip(labels.iter())
        .map(|((i, &id), label)| {
            DropdownOption::new(id, u8::try_from(i).unwrap_or(0), label.clone())
        })
        .collect()
}

/// A fileira `Browse…` / `Preview` / `Stop`, **pela porta do grupo**.
#[allow(clippy::too_many_arguments)]
fn buttons(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let fila = [
        (ids::INSP_AUDIO_BROWSE, tr("panel.inspector.audio.browse")),
        (ids::INSP_AUDIO_PREVIEW, tr("panel.inspector.audio.preview")),
        (ids::INSP_AUDIO_STOP, tr("panel.inspector.audio.stop")),
    ];
    // ⭐ A fileira mede as PALAVRAS — ver `segment_rects_for`.
    let rotulos: Vec<&str> = fila.iter().map(|(_, l)| *l).collect();
    let seg = ph2d_editor_core::widget::segment_rects_for(
        Rect::new(x, y, w, ALTURA_DE_BOTAO),
        &rotulos,
        ph2d_editor_core::widget::button_label_font(),
        text_system,
    );
    for (i, (id, label)) in fila.into_iter().enumerate() {
        let Some(&(rect, group)) = seg.get(i) else {
            continue;
        };
        hit_index.register(id, rect);
        paint_button(
            &Button::new(id, label)
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id))
                .in_group(group),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    y + ALTURA_DE_BOTAO + ph2d_tokens::control_gap_px()
}

/// O seletor de barramento — um chip, como o do verbo. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn bus_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    labels: &[String],
    sel: u8,
) -> f32 {
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H_PX);
    let rect = Rect::new(x, y, control_w, ROW_H_PX);
    hit_index.register(ids::INSP_AUDIO_BUS_PICK, rect);
    let open = matches!(
        store.get(ids::INSP_AUDIO_BUS_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(ids::INSP_AUDIO_BUS_PICK, "", bus_options(labels))
        .open(open)
        .visual(store.dropdown_visual(ids::INSP_AUDIO_BUS_PICK));
    dd.select(sel);
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    // ⚠️ **O popover NÃO se pinta aqui** — ele sairia debaixo da secção seguinte. Ver
    // `crate::popovers`.
    if open {
        crate::state_popovers::set_pending_audio_dd(Some((sel, rect)));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + ph2d_tokens::row_pitch_px()
}

/// O corpo da FONTE. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn source_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    src: &InspectorAudioSource,
    info: &InspectorAudioInfo,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.audio.sound_label"),
        ids::INSP_AUDIO_SOUND,
        TextInput::new(ids::INSP_AUDIO_SOUND, "")
            .placeholder(tr("panel.inspector.audio.sound_file")),
        seccao,
    );
    cur_y = buttons(scene, text_system, theme, hit_index, store, x, w, cur_y);

    // ⚠️ **Os avisos vêm ANTES dos números**, e é deliberado: quem não ouve nada não quer afinar um
    // expoente de atenuação — quer saber porquê.
    if src.sound.trim().is_empty() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.audio.no_sound_file_yet_use"),
            ColorToken::Text3,
        );
    } else if src.file_missing {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.audio.that_file_is_gone_pick"),
            ColorToken::Danger,
        );
    }
    if src.never_sounds() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.audio.nothing_plays_this_autoplay_is"),
            ColorToken::Warn,
        );
    }
    if info.listener_count == 0 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.audio.no_audio_listener_2d_in"),
            ColorToken::Warn,
        );
    }

    let linhas = [
        (
            tr("panel.inspector.audio.volume_db"),
            ids::INSP_AUDIO_VOLUME,
            1.0,
            None,
        ), // LITERAL-PX-OK: passo em decibéis
        (
            tr("panel.inspector.audio.pitch"),
            ids::INSP_AUDIO_PITCH,
            0.05, // LITERAL-PX-OK: passo do factor de tom
            None,
        ),
        (
            tr("panel.inspector.audio.max_distance_m"),
            ids::INSP_AUDIO_MAX_DIST,
            0.5,
            Some(ph2d_editor_core::widget::Unit::Meters),
        ), // LITERAL-PX-OK: passo em metros
        (
            tr("panel.inspector.audio.attenuation"),
            ids::INSP_AUDIO_ATTENUATION,
            0.1, // LITERAL-PX-OK: passo do expoente
            None,
        ),
        (
            tr("panel.inspector.audio.non_spatialized_radius_m"),
            ids::INSP_AUDIO_RADIUS,
            0.1, // LITERAL-PX-OK: metros
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
        (
            tr("panel.inspector.audio.panning_strength"),
            ids::INSP_AUDIO_PANNING,
            0.05, // LITERAL-PX-OK: passo da fracção
            None,
        ),
        (
            tr("panel.inspector.audio.max_polyphony"),
            ids::INSP_AUDIO_POLYPHONY,
            1.0,
            None,
        ), // LITERAL-PX-OK: uma voz de cada vez
    ];
    // ⭐⭐ **A coluna é da SECÇÃO, medida uma vez sobre a TABELA que ela pinta** — ver
    //    [`ph2d_editor_core::property_row::Seccao`]. ⛔ A tabela deixou de ser um literal dentro
    //    do `for` porque ela é lida DUAS vezes: para medir o nome mais largo e para pintar.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &linhas.iter().map(|t| t.0).collect::<Vec<_>>(),
    );
    for (label, id, step, unit) in linhas {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &[id],
            step, // LITERAL-PX-OK: passo de scrub na UNIDADE do campo, não em pixels
            unit,
            seccao,
        );
    }

    cur_y = bus_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &info.bus_labels,
        src.bus_tag,
    );

    // ⭐⭐⭐ **As duas caixas partilham uma fileira SE couberem** — spec §6-quater; a tabela medida
    //    está no doc da porta. ⚠️ **O valor vem do SNAPSHOT**, nunca do store — a lei que a §11
    //    escreveu para o `Playing`.
    ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[
            (
                ids::INSP_AUDIO_LOOP,
                tr("panel.inspector.audio.loop"),
                src.looping,
            ),
            (
                ids::INSP_AUDIO_AUTOPLAY,
                tr("panel.inspector.audio.autoplay"),
                src.autoplay,
            ),
        ],
        seccao,
    )
}

/// A secção inteira — cabeçalho, dobra e corpo. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_audio_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAudioInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_AUDIO_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let header = section_header(
        store,
        core_ids::INSP_LIVE_AUDIO_SECTION,
        tr("panel.inspector.audio.audio"),
    )
    .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_AUDIO_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;

    if info.selected_count > 1 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.audio.multiple_selected_audio_edits_apply"),
            ColorToken::Warn,
        );
    }

    // **O corpo das ORELHAS** — um componente sem campo nenhum, e por isso uma linha que DIZ o que
    // ele faz. ⛔ Sem ela, anexar o `Audio Listener 2D` não muda nada na tela: exactamente o report
    // que o `Timers` custou.
    if info.is_listener {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.audio.these_are_the_scene_s"),
            ColorToken::Text2,
        );
        if info.listener_count > 1 {
            // ⚠️ **Ele NOMEIA qual manda**, em vez de recusar: uma cena a meio de ser montada pode
            // ter dois, e recusar tornaria o produto mudo enquanto o artista pensa.
            let texto = if info.is_active_listener {
                tr_with(
                    "panel.inspector.audio.listeners_this_one",
                    &[("n", &info.listener_count)],
                )
            } else {
                tr_with(
                    "panel.inspector.audio.listeners_another_one",
                    &[("n", &info.listener_count)],
                )
            };
            cur_y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                &texto,
                ColorToken::Warn,
            );
        }
        cur_y += ph2d_tokens::control_gap_px();
    }

    if let Some(src) = &info.source {
        let sec_fonte = ph2d_editor_core::property_row::Seccao::medida(
            text_system,
            1,
            &[tr("panel.inspector.audio.sound_label")],
        );
        cur_y = source_body(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            src,
            info,
            sec_fonte,
        );
    }

    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
