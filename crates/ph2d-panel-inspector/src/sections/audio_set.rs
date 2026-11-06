//! **A lista de sons do objeto no cartão Audio Source** (SFX Lab W1): os sons, o «Pick», o «Mode», «No repeat»
//! e as duas variações. O dado é o `SoundSet`; quem sorteia é só o `SceneAudio::play`.
//!
//! Os nomes vêm do snapshot, nunca do teclado: o «Pick» oferece os sons do projeto que ainda não estão na lista.

use super::*;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::cards::corpo;
use ph2d_editor_core::property_row::Seccao;
use ph2d_editor_core::screens::hero::InspectorSoundSet;
use ph2d_editor_core::widget::card_ink::card_body;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, DropdownState, paint_dropdown_chip};
use ph2d_i18n::{TextKey, tr, tr_with};

/// Os nomes que esta lista pinta na coluna da secção.
pub(super) const NOMES: [&str; 4] = [
    "card.audio.pick",
    "card.audio.mode",
    "card.audio.pitch_variation",
    "card.audio.volume_variation",
];

/// A caixa do peso: o bastante para `0.25` ao lado da letra de arrasto.
const WEIGHT_W: f32 = m::FIELD_H * 2.5; // LITERAL-PX-OK: múltiplo da altura de campo

/// As opções do «Mode», uma por `PickStrategy::ALL`. ⚠️ A posição é a tag.
pub(crate) fn mode_options(labels: &[String]) -> Vec<DropdownOption<u8>> {
    ids::INSP_AUDIO_MODE_OPT
        .iter()
        .enumerate()
        .zip(labels)
        .map(|((i, &id), label)| {
            DropdownOption::new(id, u8::try_from(i).unwrap_or(0), label.clone())
        })
        .collect()
}

/// As opções do «Pick»: o que o snapshot oferece, por posição.
pub(crate) fn pick_options(offer: &[String]) -> Vec<DropdownOption<usize>> {
    ids::INSP_AUDIO_PICK_OPT
        .iter()
        .enumerate()
        .zip(offer)
        .map(|((i, &id), name)| DropdownOption::new(id, i, name.clone()))
        .collect()
}

/// O menu `id` está aberto?
fn is_open(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// Uma linha da lista: a caixa com o nome, o peso e o ×, na mesma fileira num cartão.
#[allow(clippy::too_many_arguments)]
fn sound_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: usize,
    row: &ph2d_editor_core::screens::hero::InspectorSoundRow,
    seccao: Seccao,
) -> f32 {
    let (on, weight, del) = (
        ids::INSP_AUDIO_SET_ON[i],
        ids::INSP_AUDIO_SET_WEIGHT[i],
        ids::INSP_AUDIO_SET_DEL[i],
    );
    let Some(ink) = card_body() else {
        let side = ROW_H_PX;
        let gap = ph2d_tokens::control_gap_px();
        super::anim::icon_button(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            Rect::new(x + w - side, y, side, side),
            del,
            IconId::Close,
            tr("card.audio.remove_sound"),
        );
        let y = ph2d_editor_core::property_row::paint_check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w - side - gap,
            y,
            (on, Rotulo::vista_de(&row.name), row.enabled),
            seccao,
        );
        return super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y,
            Rotulo::vista(TextKey::new("card.audio.weight")),
            &[weight],
            0.1, // LITERAL-PX-OK: passo do peso
            None,
            seccao,
        );
    };
    let (gap, h) = (m::FIELD_GAP, m::FIELD_H);
    let y = y + m::FIELD_ROW_GAP;
    let nome_w = (w - WEIGHT_W - h - gap * 2.0).max(0.0); // LITERAL-PX-OK: dois vãos
    corpo::check_in(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        Rect::new(x, y, nome_w, h),
        on,
        &row.name,
        row.enabled,
        false,
    );
    corpo::fields_in(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        Rect::new(x + nome_w + gap, y, WEIGHT_W, h),
        tr("card.audio.weight"),
        &[weight],
        0.1, // LITERAL-PX-OK: passo do peso
    );
    super::anim::icon_button(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        Rect::new(x + w - h, y, h, h),
        del,
        IconId::Close,
        tr("card.audio.remove_sound"),
    );
    y + h
}

/// O título «Sounds» e, sobre a coluna da caixa, «Weight». Fora do cartão a linha do peso traz o rótulo.
#[allow(clippy::too_many_arguments)]
fn head(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    com_peso: bool,
) -> f32 {
    let fim =
        super::rows::titulo_de_grupo(scene, text_system, theme, x, w, y, tr("card.audio.sounds"));
    if let Some(ink) = card_body().filter(|_| com_peso) {
        let coluna = x + w - m::FIELD_H - m::FIELD_GAP - WEIGHT_W;
        ph2d_editor_core::paint::paint_text(
            text_system,
            scene,
            tr("card.audio.weight"),
            coluna,
            y + m::GROUP_TOP,
            TypeToken::Sm.px(),
            WEIGHT_W,
            ink.color(ColorToken::Text3),
        );
    }
    fim
}

/// O «Pick»: um chip que abre o que o projeto oferece, ou parado com a razão dita. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn pick_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    set: &InspectorSoundSet,
    seccao: Seccao,
) -> f32 {
    let razao = if set.project_sounds == 0 {
        Some("card.audio.pick_empty_project")
    } else if set.offer.is_empty() {
        Some("card.audio.pick_all_in_list")
    } else if set.rows.len() >= ids::MAX_SOUND_SET_ROWS {
        Some("card.audio.pick_full")
    } else {
        None
    };
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        tr("card.audio.pick"),
        seccao,
    );
    let rect = linha.control;
    // A lista não é animável: o ponto só reserva o MESMO `x` das outras linhas.
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, linha.dot);
    let id = ids::INSP_AUDIO_PICK;
    let opcoes = pick_options(&set.offer);
    if let Some(razao) = razao {
        let dd = Dropdown::new(id, "", opcoes)
            .placeholder(tr("card.audio.pick_none"))
            .state(DropdownState::Disabled);
        paint_dropdown_chip(&dd, rect, scene, text_system, theme);
        let fim = linha.fim(ph2d_tokens::row_pitch_px());
        return super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            fim,
            tr(razao),
            ColorToken::Text3,
        );
    }
    hit_index.register(id, rect);
    let open = is_open(store, id);
    let dd = Dropdown::new(id, "", opcoes)
        .open(open)
        .placeholder(tr("card.audio.pick_placeholder"))
        .visual(store.dropdown_visual(id));
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    if open {
        crate::state_popovers::set_pending_sound_set_dd(Some((false, 0, rect)));
    }
    let fim = linha.fim(ph2d_tokens::row_pitch_px());
    let fora = set.offer.len().saturating_sub(ids::MAX_SOUND_OFFER);
    if fora == 0 {
        return fim;
    }
    super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        fim,
        &tr_with("card.audio.offer_hidden", &[("n", &fora)]),
        ColorToken::Warn,
    )
}

/// «Mode» e «No repeat»: a mesma fileira num cartão. No modo Sequence a caixa fica parada e a razão é dita.
#[allow(clippy::too_many_arguments)]
fn mode_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    set: &InspectorSoundSet,
    seccao: Seccao,
) -> f32 {
    let cartao = card_body();
    let [(x0, w0), (x1, w1)] = corpo::colunas_iguais(x, w);
    let (xm, wm) = if cartao.is_some() { (x0, w0) } else { (x, w) };
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        xm,
        wm,
        y,
        ROW_H_PX,
        tr("card.audio.mode"),
        seccao,
    );
    let rect = linha.control;
    let id = ids::INSP_AUDIO_MODE;
    hit_index.register(id, rect);
    let open = is_open(store, id);
    let mut dd = Dropdown::new(id, "", mode_options(&set.mode_labels))
        .open(open)
        .visual(store.dropdown_visual(id));
    dd.select(set.strategy_tag);
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    if open {
        crate::state_popovers::set_pending_sound_set_dd(Some((true, set.strategy_tag, rect)));
    }
    let mut fim = linha.fim(ph2d_tokens::row_pitch_px());
    let decide = set.no_repeat_decides();
    let nome = tr("card.audio.no_repeat");
    if let Some(ink) = cartao {
        corpo::check_in(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            Rect::new(x1, rect.y, w1, rect.h),
            ids::INSP_AUDIO_NO_REPEAT,
            nome,
            set.no_repeat && decide,
            !decide,
        );
    } else if decide {
        fim = ph2d_editor_core::property_row::paint_check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            fim,
            (
                ids::INSP_AUDIO_NO_REPEAT,
                Rotulo::vista(TextKey::new("card.audio.no_repeat")),
                set.no_repeat,
            ),
            seccao,
        );
    }
    if decide {
        return fim;
    }
    super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        fim,
        tr("card.audio.no_repeat_sequence"),
        ColorToken::Text3,
    )
}

/// A lista inteira. ⚠️ Sem som nenhum só o «Pick» aparece: o «Mode» e as variações não decidiriam nada.
#[allow(clippy::too_many_arguments)]
pub(super) fn sound_set_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    set: &InspectorSoundSet,
    seccao: Seccao,
) -> f32 {
    let mut cur_y = head(scene, text_system, theme, x, w, y, !set.rows.is_empty());
    for (i, row) in set.rows.iter().take(ids::MAX_SOUND_SET_ROWS).enumerate() {
        cur_y = sound_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            i,
            row,
            seccao,
        );
    }
    let fora = set.rows.len().saturating_sub(ids::MAX_SOUND_SET_ROWS);
    if fora > 0 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with("card.audio.rows_hidden", &[("n", &fora)]),
            ColorToken::Warn,
        );
    }
    cur_y = pick_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        set,
        seccao,
    );
    if set.rows.is_empty() {
        return cur_y;
    }
    cur_y = mode_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        set,
        seccao,
    );
    for (key, id) in [
        ("card.audio.pitch_variation", ids::INSP_AUDIO_PITCH_JITTER),
        ("card.audio.volume_variation", ids::INSP_AUDIO_GAIN_JITTER),
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
            Rotulo::vista(TextKey::new(key)),
            &[id],
            0.5, // LITERAL-PX-OK: passo em semitons e em dB
            None,
            seccao,
        );
    }
    cur_y
}
