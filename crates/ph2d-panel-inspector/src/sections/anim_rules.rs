//! **A lista de regras do cartão «Animation»** ([plano GameEngineFuncional 08](../../../../docs/GameEngineFuncional/08_regras_de_animacao_plano.md)):
//! `[n] [When ▾] → [Play ▾] [×]` por regra, pela ordem (a de cima ganha), «+ Rule», e por baixo
//! «Run From» e «Flip» quando o objecto é um personagem. Arrastar a pega `[n]` reordena
//! (família [`PanelRowFamily::AnimRule`]).
//!
//! ⚠️ Irmão de [`super::anim_card`] pelo tecto de LOC dos painéis.

use super::*;
use ph2d_editor_core::interaction::PanelRowFamily;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{AnimChooser, AnimWhenInfo, InspectorAnimInfo};
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, dropdown_label_budget, paint_dropdown_chip,
};
use ph2d_i18n::{TextKey, tr, tr_with};

const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// As chaves das poses, pela ordem de `CharacterPose::ALL` (o índice de `AnimWhenInfo::Pose`).
const POSE_KEYS: [&str; ph2d_editor_core::screens::hero::ANIM_POSES as usize] = [
    "panel.inspector.anim_card.when_pose_0",
    "panel.inspector.anim_card.when_pose_1",
    "panel.inspector.anim_card.when_pose_2",
    "panel.inspector.anim_card.when_pose_3",
    "panel.inspector.anim_card.when_pose_4",
    "panel.inspector.anim_card.when_pose_5",
];

/// As chaves dos factos de vida, pela ordem de `LifeEvent::ALL` (o índice de `AnimWhenInfo::Life`).
const LIFE_KEYS: [&str; ph2d_editor_core::screens::hero::ANIM_LIFE_EVENTS as usize] = [
    "panel.inspector.anim_card.when_life_0",
    "panel.inspector.anim_card.when_life_1",
    "panel.inspector.anim_card.when_life_2",
    "panel.inspector.anim_card.when_life_3",
];

/// **O texto de uma condição** — a única porta: o chip da regra e as linhas do popover leem esta.
pub(crate) fn when_label(when: &AnimWhenInfo) -> String {
    let indexed = |keys: &[&str], i: u8| {
        keys.get(usize::from(i))
            .map_or_else(|| i.to_string(), |k| tr(k).to_owned())
    };
    match when {
        AnimWhenInfo::Pose(i) => indexed(&POSE_KEYS, *i),
        AnimWhenInfo::Life(i) => indexed(&LIFE_KEYS, *i),
        AnimWhenInfo::Action(name) => {
            tr_with("panel.inspector.anim_card.when_action", &[("name", name)])
        }
        AnimWhenInfo::Signal(name) => {
            tr_with("panel.inspector.anim_card.when_signal", &[("name", name)])
        }
    }
}

/// As opções do menu «When», pela ordem de `AnimCardInfo::when_options` (o índice É a posição).
pub(crate) fn when_options(info: &InspectorAnimInfo) -> Vec<DropdownOption<usize>> {
    ids::INSP_ANIM_RULE_WHEN_OPT
        .iter()
        .enumerate()
        .zip(info.card.when_options())
        .map(|((j, &id), w)| DropdownOption::new(id, j, when_label(&w)))
        .collect()
}

/// As opções do menu «Play»: as animações da lista, pela ordem dela (o índice É a posição).
pub(crate) fn play_options(info: &InspectorAnimInfo) -> Vec<DropdownOption<usize>> {
    ids::INSP_ANIM_RULE_PLAY_OPT
        .iter()
        .enumerate()
        .zip(info.rows.iter())
        .map(|((j, &id), row)| DropdownOption::new(id, j, row.name.clone()))
        .collect()
}

thread_local! {
    /// A regra que a pintura deste quadro acendeu e onde: a costura lê o que foi pintado, não o modelo.
    static ACESA_PINTADA: std::cell::Cell<Option<(usize, Rect)>> = const { std::cell::Cell::new(None) };
}

/// A regra acesa na última pintura do cartão.
#[must_use]
pub fn regra_acesa_pintada() -> Option<(usize, Rect)> {
    ACESA_PINTADA.get()
}

pub(crate) fn esquece_a_acesa() {
    ACESA_PINTADA.set(None);
}

/// O índice da regra acesa, se ela tem linha no cartão.
fn acesa(info: &InspectorAnimInfo) -> Option<usize> {
    info.card
        .live_rule
        .map(usize::from)
        .filter(|i| *i < shown_rules(info))
}

/// O rótulo da fila Who Picks: com uma regra acesa, leva a etiqueta LIVE.
pub(crate) fn rotulo_de_quem_escolhe(info: &InspectorAnimInfo) -> &'static str {
    if acesa(info).is_some() {
        "panel.inspector.anim_card.who_picks_live"
    } else {
        "panel.inspector.anim_card.who_picks"
    }
}

/// Quantas regras têm linha (e pega arrastável) no cartão.
pub(crate) fn shown_rules(info: &InspectorAnimInfo) -> usize {
    if info.card.chooser == AnimChooser::Rules && !info.rows.is_empty() {
        info.card.rules.len().min(ids::MAX_ANIM_RULE_ROWS)
    } else {
        0
    }
}

/// **As pegas arrastáveis deste quadro** — republicadas pelo `sync` a cada quadro (o conjunto vazio
/// quando o cartão não mostra regras: uma regra apagada não pode ficar arrastável).
pub(crate) fn sync_rule_rows(host: &mut dyn PanelHostInternal, info: Option<&InspectorAnimInfo>) {
    let n = info.map_or(0, shown_rules);
    host.store_mut().set_panel_row_ids(
        PanelRowFamily::AnimRule,
        ids::INSP_ANIM_RULE_GRIP[..n].iter().copied().collect(),
    );
}

/// O menu `id` está aberto?
fn is_open(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// Onde moram as peças de uma regra.
struct RuleRects {
    grip: Rect,
    when: Rect,
    arrow: Rect,
    play: Rect,
    del: Rect,
    next_y: f32,
}

/// **Uma regra numa fileira, ou em duas quando algum nome não cabe** (plano 22: «Cro…» e «crou…» liam-se
/// iguais). A pega ocupa as duas fileiras: é o rectângulo que o arrastar mede para largar.
fn rule_rects(x: f32, control_w: f32, y: f32, stacked: bool) -> RuleRects {
    let gap = ph2d_tokens::control_gap_px();
    let pitch = ph2d_tokens::row_pitch_px();
    let del = Rect::new(x + control_w - ROW_H, y, ROW_H, ROW_H);
    let left = x + ROW_H + gap;
    if stacked {
        let right = del.x - gap;
        let play_x = left + ROW_H + gap;
        RuleRects {
            grip: Rect::new(x, y, ROW_H, pitch + ROW_H),
            when: Rect::new(left, y, right - left, ROW_H),
            arrow: Rect::new(left, y + pitch, ROW_H, ROW_H),
            play: Rect::new(play_x, y + pitch, right - play_x, ROW_H),
            del,
            next_y: y + pitch * 2.0, // LITERAL-PX-OK: duas fileiras
        }
    } else {
        let chip_w = ((control_w - 3.0 * ROW_H - 4.0 * gap) * 0.5).max(0.0); // LITERAL-PX-OK: 3 células e 4 vãos, metade para cada chip
        let arrow_x = left + chip_w + gap;
        RuleRects {
            grip: Rect::new(x, y, ROW_H, ROW_H),
            when: Rect::new(left, y, chip_w, ROW_H),
            arrow: Rect::new(arrow_x, y, ROW_H, ROW_H),
            play: Rect::new(arrow_x + ROW_H + gap, y, chip_w, ROW_H),
            del,
            next_y: y + pitch,
        }
    }
}

/// O que sobra entre o fim das peças de uma regra e a regra seguinte.
fn gap_da_fila() -> f32 {
    ph2d_tokens::row_pitch_px() - ROW_H
}

/// O texto do chip «Play» de uma regra.
fn play_label(info: &InspectorAnimInfo, play: &str) -> String {
    if info.rows.iter().any(|r| r.name == play) {
        play.to_owned()
    } else {
        tr_with("panel.inspector.anim_card.missing", &[("name", &play)])
    }
}

/// As regras, «+ Rule», «Run From» e «Flip». Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rule_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    if info.rows.is_empty() {
        return super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            tr("panel.inspector.anim_card.no_animations"),
            ColorToken::Text3,
        );
    }
    let font = TypeToken::Sm.px();
    // A coluna do losango (a lista não é animável: o ponto só reserva o MESMO `x` das outras linhas).
    let (control_w, _) = ph2d_editor_core::widget::form_row_columns(x, w, y, ROW_H);
    let budget = dropdown_label_budget(rule_rects(x, control_w, y, false).when);
    let stacked = info
        .card
        .rules
        .iter()
        .take(ids::MAX_ANIM_RULE_ROWS)
        .any(|rule| {
            [when_label(&rule.when), play_label(info, &rule.play)]
                .iter()
                .any(|t| {
                    ph2d_editor_core::paint::label_column_width(
                        text_system,
                        TypeToken::Base.px(),
                        [t.as_str()],
                    ) > budget
                })
        });
    let options = info.card.when_options();
    let mut cur_y = y;
    let mut grips = Vec::new();
    for (i, rule) in info
        .card
        .rules
        .iter()
        .take(ids::MAX_ANIM_RULE_ROWS)
        .enumerate()
    {
        let rects = rule_rects(x, control_w, cur_y, stacked);
        let grip = rects.grip;
        let acesa = acesa(info) == Some(i);
        let raio = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
        if acesa {
            let fila = Rect::new(x, cur_y, control_w, rects.next_y - cur_y - gap_da_fila());
            fill_rounded_rect(scene, fila, raio, resolve(ColorToken::AccentSoft, theme));
            ACESA_PINTADA.set(Some((i, fila)));
        }
        hit_index.register(ids::INSP_ANIM_RULE_GRIP[i], grip);
        let (fundo, tinta) = if acesa {
            (ColorToken::Accent, ColorToken::AccentFg)
        } else {
            (ColorToken::Bg2, ColorToken::Text2)
        };
        fill_rounded_rect(scene, grip, raio, resolve(fundo, theme));
        ph2d_editor_core::paint::paint_text_centered(
            text_system,
            scene,
            &(i + 1).to_string(),
            grip,
            font,
            resolve(tinta, theme),
        );
        grips.push(grip);

        let when_rect = rects.when;
        let when_id = ids::INSP_ANIM_RULE_WHEN[i];
        hit_index.register(when_id, when_rect);
        let open = is_open(store, when_id);
        let mut dd = Dropdown::new(when_id, "", when_options(info))
            .open(open)
            .placeholder(when_label(&rule.when))
            .visual(store.dropdown_visual(when_id));
        if let Some(j) = options.iter().position(|o| *o == rule.when) {
            dd.select(j);
        }
        paint_dropdown_chip(&dd, when_rect, scene, text_system, theme);
        if open {
            crate::state_popovers::set_pending_anim_rule_dd(Some((true, i as u8, when_rect)));
        }

        ph2d_editor_core::paint::paint_icon(
            scene,
            IconId::ChevronRight,
            rects.arrow,
            resolve(ColorToken::Text3, theme),
            ph2d_tokens::StrokeToken::Default.px(),
        );

        let play_rect = rects.play;
        let play_id = ids::INSP_ANIM_RULE_PLAY[i];
        hit_index.register(play_id, play_rect);
        let open = is_open(store, play_id);
        let known = info.rows.iter().position(|r| r.name == rule.play);
        let mut dd = Dropdown::new(play_id, "", play_options(info))
            .open(open)
            .placeholder(play_label(info, &rule.play))
            .visual(store.dropdown_visual(play_id));
        // ⚠️ A escolha vem do SNAPSHOT (o nome guardado), nunca do store.
        if let Some(j) = known {
            dd.select(j);
        }
        paint_dropdown_chip(&dd, play_rect, scene, text_system, theme);
        if open {
            crate::state_popovers::set_pending_anim_rule_dd(Some((false, i as u8, play_rect)));
        }

        super::anim::icon_button(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            rects.del,
            ids::INSP_ANIM_RULE_DEL[i],
            IconId::Close,
            tr("panel.inspector.anim_card.rule_del_tip"),
        );
        let (_, dot) = ph2d_editor_core::widget::form_row_columns(x, w, cur_y, ROW_H);
        ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
        cur_y = rects.next_y;
    }
    paint_drop_bar(scene, theme, store, x, w, &grips);
    let hidden = info
        .card
        .rules
        .len()
        .saturating_sub(ids::MAX_ANIM_RULE_ROWS);
    cur_y = if hidden > 0 {
        super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with("panel.inspector.anim_card.rules_hidden", &[("n", &hidden)]),
            ColorToken::Text3,
        )
    } else {
        super::anim_card::button_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            ids::INSP_ANIM_RULE_ADD,
            tr("panel.inspector.anim_card.rule_add"),
        )
    };
    character_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        sec,
    )
}

/// A barra de largada durante o arrasto de uma regra — a MESMA partição do despacho
/// (`find_panel_row_drop`, família plana): metade de cima = antes, metade de baixo = depois.
fn paint_drop_bar(
    scene: &mut VectorScene,
    theme: Theme,
    store: &WidgetStore,
    x: f32,
    w: f32,
    grips: &[Rect],
) {
    let Some((PanelRowFamily::AnimRule, d)) = store.panel_row_drag() else {
        return;
    };
    if !d.active {
        return;
    }
    let alvo = grips
        .iter()
        .zip(ids::INSP_ANIM_RULE_GRIP)
        .find(|(r, id)| *id != d.dragged && d.cursor_y >= r.y && d.cursor_y < r.y + r.h);
    if let Some((r, _)) = alvo {
        let y = if d.cursor_y < r.y + r.h * 0.5 {
            r.y
        } else {
            r.y + r.h
        };
        let h = ph2d_tokens::StrokeToken::Thick.px();
        fill_rounded_rect(
            scene,
            Rect::new(x, y - h * 0.5, w, h),
            ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
            resolve(ColorToken::Accent, theme),
        );
    }
}

/// «Run From» e «Flip» — só num personagem (Platform Player). Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn character_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let Some(ch) = info.card.character.as_ref() else {
        return y;
    };
    let cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        TextKey::new("component.field.character_animations.4"),
        &[ids::INSP_ANIM_RUN_FROM],
        0.1, // LITERAL-PX-OK: passo de scrub em m/s, não em pixels
        Some(ph2d_editor_core::widget::Unit::MetersPerSecond),
        sec,
    );
    ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            ids::INSP_ANIM_FLIP,
            TextKey::new("component.field.character_animations.5"),
            ch.flip,
        ),
        sec,
    )
}
