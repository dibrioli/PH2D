//! **§14 Platform Player** — o comportamento de personagem (W5).
//!
//! ⚠️ **A face VAZIA é a metade importante**, e é a lição que a §11 do W2a já
//! pagou: antes dela não existia gesto nenhum no editor que tornasse um sprite
//! físico. Aqui é a mesma coisa um degrau acima — um corpo Dynamic sem o
//! componente vê **um botão**, e é ele que faz o comportamento existir.
//!
//! ⚠️ **Nada aqui é oferecido a um corpo que não é Dynamic**, e é FÍSICA: a mola
//! é um impulso, e um impulso não move massa infinita. A recusa mora no
//! construtor do info (a shell), que é quem sabe o kind — o pintor decide se
//! oferece a partir da MESMA resposta.
//!
//! ⚠️ **Os `hit_index.register` são escritos com id LITERAL, um por botão**, e
//! isso não é verbosidade: é a única forma que o `architecture_panel_wiring_parity`
//! consegue ver (ele coleta `.register(ids::<LITERAL>` e pula um primeiro
//! argumento variável). Dobrá-los num laço apagaria a cobertura de paridade dos
//! botões em silêncio — a cicatriz que a §11 já carrega escrita.

use super::*;
use ph2d_editor_core::screens::hero::InspectorPlayerInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

/// A tabela dos cards da §14 — irmã por RESPONSABILIDADE (ver o topo dela).
///
/// ⚠️ **Este doc dizia «os nove cards» e a tabela tem doze** desde que `LEDGE`/`GLIDE`/`FALL`
/// entraram. ⛔ Não escreva o número aqui: a fonte é `PLAYER_CARDS.len()`, e uma contagem
/// copiada para um comentário envelhece sozinha (auditoria `docs/Sprite_projeto/20` §8).
///
/// ⚠️ **`table`, e não `rows`:** a `sections::rows` já existe (os primitivos
/// `card_frame`/`num_row`/`seg_row` que este pintor consome), e um segundo
/// `rows` aqui dentro a sombrearia — o pintor compilaria a chamar outra coisa.
#[path = "player_rows.rs"]
mod table;

/// A face VAZIA — ver [`door::paint_empty_face`].
#[path = "player_door.rs"]
mod door;

/// A leitura AO VIVO (postura, `facing`, velocidade) — irmã por RESPONSABILIDADE: é a única parte
/// da secção que não se edita.
#[path = "player_cartoes.rs"]
mod cartoes;

#[path = "player_live.rs"]
mod live;
use live::paint_live_readout;

pub(crate) use table::{PLAYER_CARDS, player_row_count};

/// As dicas dos QUATRO BOTÕES da seção — a mesma lei das rows, num lugar onde não
/// cabe uma tupla de row.
///
/// ⚠️ **Foram QUATRO entre a F3 e 2026-09-14**: o `INSP_PLAYER_ADD` saiu com a face vazia que o
/// continha, e voltou com ela por ordem do dono — *«todas as opções aparecem com ele»*.
pub(crate) const PLAYER_BUTTON_TIPS: [(ph2d_a11y::NodeId, TextKey); 5] = [
    (
        ids::INSP_PLAYER_ADD,
        TextKey::new("panel.inspector.player.turn_this_body_into_a"),
    ),
    (
        ids::INSP_PLAYER_FIT,
        TextKey::new("panel.inspector.player.set_float_height_from_the"),
    ),
    (
        ids::INSP_PLAYER_REMOVE,
        TextKey::new("panel.inspector.player.give_the_behaviour_back_it"),
    ),
    (
        ids::INSP_PLAYER_CLEAR_RUN,
        TextKey::new("panel.inspector.player.throw_away_the_recorded_run"),
    ),
    (
        ids::INSP_PLAYER_FIT_CROUCH,
        TextKey::new("panel.inspector.player.set_crouch_height_to_the"),
    ),
];

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_player_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorPlayerInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let header = section_header(
        store,
        core_ids::INSP_LIVE_PLAYER_SECTION,
        tr("component.platform_player.name"),
    );
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    // ⚠️ **A DOBRA do corpo** — ver `SectionFold`, e o `t` no lugar do `is_collapsed`.
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_PLAYER_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };

    let mut yy = y + header_h;

    // ⭐⭐ **A FACE VAZIA — a porta, no ficheiro irmão** (ver `player_door.rs` para o porquê).
    //
    // ⚠️ **A guarda continua a ser LEI, e não decoração:** sem ela um info com `has_player =
    // false` pintaria a secção inteira de knobs sobre um player que não existe — foi o que
    // aconteceu quando a face saiu, e os dois gates do `seam_player` foram quem o disse.
    if !info.has_player {
        let fim = door::paint_empty_face(scene, text_system, theme, hit_index, store, x, w, yy);
        return fold.finish(store, scene, hit_index, fim);
    }

    // **COMO ele é movido** (W-KinMove) — a primeira coisa da seção, porque toda
    // row abaixo dela é interpretada por este modo (a `LEG` inteira é a mola, e
    // sob Snap não há mola).
    if info.live.is_some() {
        yy = paint_live_readout(scene, text_system, theme, x, w, yy, info.live);
    }
    yy = cartoes::paint(
        &mut cartoes::Pintor {
            scene,
            text: text_system,
            theme,
            hit: hit_index,
            store,
            x,
            w,
            info,
        },
        yy,
    );

    let out = paint_verbs(scene, text_system, theme, hit_index, store, x, w, yy, info);
    fold.finish(store, scene, hit_index, out)
}

/// **Os quatro botões da §14** — os dois `Fit`, o da corrida gravada e o
/// `Remove`.
///
/// ⚠️ Eles viajam juntos porque respondem à mesma pergunta (*que verbos esta
/// seção oferece?*) e porque três dos quatro são CONDICIONAIS: cada um só é
/// pintado onde tem o que fazer, que é a lei do knob morto desta seção. Separá-los
/// espalharia essa decisão por dois arquivos.
#[allow(clippy::too_many_arguments)]
fn paint_verbs(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorPlayerInfo,
) -> f32 {
    let mut yy = y;
    let run_button = if info.recorded_run_seconds > 0.0 {
        Some((
            ids::INSP_PLAYER_CLEAR_RUN,
            tr_with(
                "panel.inspector.player.clear_run",
                &[("s", &format!("{:.1}", info.recorded_run_seconds))],
            ),
        ))
    } else if info.discarded_run_seconds > 0.0 {
        Some((
            ids::INSP_PLAYER_RESTORE_RUN,
            tr_with(
                "panel.inspector.player.restore_run",
                &[("s", &format!("{:.1}", info.discarded_run_seconds))],
            ),
        ))
    } else {
        None
    };
    if let Some((id, label)) = run_button {
        let rect = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, yy, &label);
        let btn = Button::new(id, &label)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(id, rect);
        yy = ph2d_editor_core::property_row::abaixo_do_botao(rect);
    }

    let rect = ph2d_editor_core::property_row::caixa_do_botao(
        text_system,
        x,
        w,
        yy,
        tr("panel.inspector.player.remove_platform_player"),
    );
    let btn = Button::new(
        ids::INSP_PLAYER_REMOVE,
        tr("panel.inspector.player.remove_platform_player"),
    )
    .kind(ButtonKind::Default)
    .visual(store.button_visual(ids::INSP_PLAYER_REMOVE));
    paint_button(&btn, rect, scene, text_system, theme);
    hit_index.register(ids::INSP_PLAYER_REMOVE, rect);
    ph2d_editor_core::property_row::abaixo_do_botao(rect)
}

/// ⭐ T59 (a lei da T56, smoke do dono 02/11): as linhas VIZINHAS que no cartão dividem uma linha —
/// duas metades de um assunto, um rótulo sobre cada caixa. Fora do cartão, uma por linha.
/// (Air Jumps e Air Jump Height saíram da lista na versão B: moram em partes diferentes, o cartão Jump e a dobra More.)
pub(crate) const PLAYER_PARES: [(ph2d_a11y::NodeId, ph2d_a11y::NodeId); 15] = {
    use crate::ids as i;
    [
        (i::INSP_PLAYER_STIFFNESS, i::INSP_PLAYER_DAMPING),
        (i::INSP_PLAYER_FOOT_SAMPLES, i::INSP_PLAYER_FOOT_SPREAD),
        (i::INSP_PLAYER_SPEED, i::INSP_PLAYER_ACCEL),
        (i::INSP_PLAYER_TAKEOFF_G, i::INSP_PLAYER_TAKEOFF_SPEED),
        (i::INSP_PLAYER_PEAK_G, i::INSP_PLAYER_PEAK_SPEED),
        (i::INSP_PLAYER_FALL_G, i::INSP_PLAYER_CUT_G),
        (i::INSP_PLAYER_COYOTE, i::INSP_PLAYER_BUFFER),
        (i::INSP_PLAYER_CORNER, i::INSP_PLAYER_CORNER_SAMPLES),
        (i::INSP_PLAYER_REACT_MOVEMENT, i::INSP_PLAYER_REACT_PUSH),
        (i::INSP_PLAYER_WALL_JUMP, i::INSP_PLAYER_WALL_PUSH),
        (i::INSP_PLAYER_WALL_SAMPLES, i::INSP_PLAYER_WALL_SPREAD),
        (i::INSP_PLAYER_DASH_TIME, i::INSP_PLAYER_DASH_COOL),
        (i::INSP_PLAYER_CROUCH_HEIGHT, i::INSP_PLAYER_CROUCH_SPEED),
        (i::INSP_PLAYER_SWIM_SPEED, i::INSP_PLAYER_SWIM_ACCEL),
        (i::INSP_PLAYER_LEDGE_REACH_Y, i::INSP_PLAYER_LEDGE_SPAN),
    ]
};
