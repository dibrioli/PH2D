//! ⭐⭐⭐ **O que o Inspector mostra do SEGUIDOR DE CAMINHO** (suplente #23).
//!
//! # ⭐⭐ O painel DIZ porque é que ele não anda, e são CINCO razões
//!
//! | aviso | a cura |
//! |---|---|
//! | `Type the name of a drawn shape` | escrever o nome |
//! | `No object in the scene has that name` | corrigir o nome |
//! | `That object has no drawn shape` | apontar a uma forma |
//! | `There is no Timer at that slot` | baixar o índice, ou acrescentar um relógio |
//! | `Timer N has no duration` | pôr uma duração |
//!
//! ⚠️ **São cinco e não uma**, e a razão é que as curas são diferentes — a escada vive no
//! [`ph2d_editor_core::path_follow_edits::InspectorPathFollowInfo::queixa`], e é ela que garante
//! que o artista é mandado à metade certa.
//!
//! # ⭐ O RELÓGIO mora AQUI, e é o mesmo `Timers[i]` da secção TIMERS
//!
//! É a lição da W9 do tween, herdada inteira: *o painel não manda o artista a outra secção para
//! escolher o tempo*. ⛔ E não são duas superfícies sobre um valor — é **a mesma porta com dois
//! chamadores** (`TimerFieldEdit`), como o teclado e o menu do `project_io`.
//!
//! # ⚠️ Os chips saem da PORTA do tween
//!
//! O [`super::tween_editor::grupo`] mede quantos cabem numa fileira e pinta o realce a partir do
//! SNAPSHOT. Copiá-lo aqui daria duas respostas à mesma pergunta de largura, que é o defeito que a
//! foto do dono de 19/09 pagou.

use super::tween::warn;
use super::tween_editor::grupo;
use super::*;
use ph2d_editor_core::path_follow_edits::{InspectorPathFollowInfo, PathFollowQueixa};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::{tr, tr_with};

/// A chave da frase de cada queixa. ⚠️ Um `match` e não uma tabela indexada: a posição de uma
/// variante não é a tag de nada aqui, e uma tabela criaria essa dependência do nada.
const fn chave_da_queixa(q: PathFollowQueixa) -> &'static str {
    match q {
        PathFollowQueixa::SemNome => "panel.inspector.path_follow.type_the_name_of_a_drawn_shape",
        PathFollowQueixa::NomeDesconhecido => {
            "panel.inspector.path_follow.no_object_in_the_scene_has_that_name"
        }
        PathFollowQueixa::SemForma => "panel.inspector.path_follow.that_object_has_no_drawn_shape",
        PathFollowQueixa::SemRelogio => "panel.inspector.path_follow.there_is_no_timer_at_slot_n",
        PathFollowQueixa::RelogioSemDuracao => {
            "panel.inspector.path_follow.timer_n_has_no_duration"
        }
    }
}

/// ⭐⭐⭐ **O bloco do RELÓGIO** — a duração e os dois interruptores, dentro desta secção.
///
/// ⚠️ **A legenda NOMEIA o timer** porque ele não é privado deste seguidor: ele pode estar a
/// arrancar uma cutscene, a alimentar uma fábrica ou a publicar um sinal.
#[allow(clippy::too_many_arguments)]
fn relogio(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorPathFollowInfo,
) -> f32 {
    let mut cur_y = warn(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &tr_with(
            "panel.inspector.path_follow.clock_is_timer",
            &[("n", &(usize::from(i.relogio) + 1))],
        ),
        ColorToken::Text3,
    );
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.path_follow.duration_seconds")],
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.path_follow.duration_seconds"),
        &[crate::ids::INSP_PF_DURACAO],
        0.1, // LITERAL-PX-OK: passo de scrub em SEGUNDOS, como o da secção TIMERS
        None,
        sec,
    );
    // ⚠️ **O valor das caixas vem do SNAPSHOT**, nunca do store: ler dali faria a caixa sobreviver
    // à troca de objecto.
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
                crate::ids::INSP_PF_REPEAT,
                tr("panel.inspector.path_follow.repeat"),
                i.repeat,
            ),
            (
                crate::ids::INSP_PF_AUTOSTART,
                tr("panel.inspector.path_follow.autostart"),
                i.autostart,
            ),
        ],
        sec,
    )
}

/// O corpo da secção — os CONTROLOS.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorPathFollowInfo,
) -> f32 {
    let mut cur_y = y;
    // ⚠️ **A QUEIXA primeiro** — quem não vê nada mexer não quer afinar uma curva.
    if let Some(q) = i.queixa() {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with(chave_da_queixa(q), &[("n", &(usize::from(i.relogio) + 1))]),
            ColorToken::Text3,
        );
    }
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.path_follow.timer_slot"),
            tr("panel.inspector.path_follow.start_at"),
            tr("panel.inspector.path_follow.angle"),
            tr("panel.inspector.path_follow.side_offset"),
            // ⭐ O nome da LINHA DE TEXTO entra na medição da coluna, como os irmãos.
            tr("panel.inspector.path_follow.shape_label"),
        ],
    );
    // ⭐ **O NOME da forma** — a referência durável desta casa.
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.path_follow.shape_label"),
        crate::ids::INSP_PF_CAMINHO,
        TextInput::new(crate::ids::INSP_PF_CAMINHO, "")
            .placeholder(tr("panel.inspector.path_follow.drawn_shape_name_u")),
        seccao,
    );

    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.path_follow.timer_slot"),
        &[crate::ids::INSP_PF_RELOGIO],
        1.0, // LITERAL-PX-OK: índice
        None,
        seccao,
    );
    // ⛔ **O bloco do relógio desaparece quando não há relógio**, porque aí quem fala é a queixa —
    // *duas superfícies sobre a mesma ausência ensinam que são dois problemas*.
    if i.duracao_us.is_some() {
        cur_y = relogio(scene, text_system, theme, hit_index, store, x, w, cur_y, i);
    }

    let ciclos: Vec<&str> = ph2d_tween::Ciclo::ALL
        .iter()
        .map(|c| tr(c.label_key()))
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.path_follow.cycle"),
        &crate::ids::INSP_PF_CICLO,
        &ciclos,
        i.ciclo as usize,
    );
    let familias: Vec<&str> = ph2d_anim::EasingFamily::ALL
        .iter()
        .map(|f| ph2d_i18n::tr(f.label_key()))
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.path_follow.curve"),
        &crate::ids::INSP_PF_FAMILIA,
        &familias,
        i.familia as usize,
    );
    let modos: Vec<&str> = ph2d_anim::EasingMode::ALL
        .iter()
        .map(|m| ph2d_i18n::tr(m.label_key()))
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.path_follow.ease"),
        &crate::ids::INSP_PF_MODO,
        &modos,
        i.modo as usize,
    );
    let fins: Vec<&str> = ph2d_tween::AoAcabar::ALL
        .iter()
        .map(|a| tr(a.label_key()))
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.path_follow.when_done"),
        &crate::ids::INSP_PF_AO_ACABAR,
        &fins,
        i.ao_acabar as usize,
    );

    // ⚠️ **A UNIDADE é um CHIP do campo, nunca texto no rótulo** — há gate a prová-lo, e a razão é
    // que dois sítios a dizer a mesma unidade divergem no dia em que um deles mudar.
    for (label, id, step, unidade) in [
        (
            tr("panel.inspector.path_follow.start_at"),
            crate::ids::INSP_PF_DESLOCAMENTO,
            0.01, // LITERAL-PX-OK: fracção do percurso
            None,
        ),
        (
            tr("panel.inspector.path_follow.angle"),
            crate::ids::INSP_PF_ANGULO,
            5.0, // LITERAL-PX-OK: graus
            Some(ph2d_editor_core::widget::Unit::Degrees),
        ),
        (
            tr("panel.inspector.path_follow.side_offset"),
            crate::ids::INSP_PF_LADO,
            0.1, // LITERAL-PX-OK: metros
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
    ] {
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
            step,
            unidade,
            seccao,
        );
    }

    // ⚠️ **O ÂNGULO só tem sujeito quando ele ALINHA** — sem alinhar, somar graus a uma rotação que
    // este motor não escreve é um controlo morto. ⛔ A caixa fica sempre, porque é ela que liga.
    ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[(
            crate::ids::INSP_PF_ALINHA,
            tr("panel.inspector.path_follow.face_path"),
            i.alinha,
        )],
        seccao,
    )
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_path_follow_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorPathFollowInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PATHFOLLOW_SECTION,
        tr("panel.inspector.path_follow.path_follow"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PATHFOLLOW_SECTION,
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
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.path_follow.editing_the_primary_selection_only"),
            ColorToken::Text3,
        );
    }
    cur_y = corpo(
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
    // ⚠️ **O relógio parado NÃO é queixa** (ver o vocabulário): ele é o estado normal de uma cena em
    // edição, e por isso é uma linha de ESTADO, no fim, e não um aviso no topo.
    if info.queixa().is_none() && !info.clock_playing {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.path_follow.it_moves_while_the_clock_plays"),
            ColorToken::Text3,
        );
    }
    fold.finish(store, scene, hit_index, cur_y)
}
