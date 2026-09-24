//! ⭐ **A leitura AO VIVO do Platform Player** — o que a lei deu no último passo (postura, lado,
//! velocidade). Irmã do `player.rs` por RESPONSABILIDADE: a secção edita, isto só lê.

use super::super::*;
use ph2d_editor_core::screens::hero::PlayerLive;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// **O readout VIVO** — postura, `facing` e velocidade.
///
/// Sem id e sem hit: nada aqui é editável, e *um readout que despacha mente* (a
/// lei da §12, e o desenho exacto do `paint_gear_readout` da §13).
///
/// ⚠️ **Sem leitura, ele DIZ isso** em vez de deixar um vão. Com o toggle
/// **Physics** desmarcado — o default — a lei não deu passo nenhum, e um vão
/// lê-se como *"o app não sabe"*, quando o que ele sabe é que não há corrida.
pub(super) fn paint_live_readout(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    live: Option<PlayerLive>,
) -> f32 {
    let font = TypeToken::Sm.px();
    // ⭐⭐⭐ **As linhas da leitura passam pela PORTA do nome, e a coluna é a do painel** (ordem do
    //    dono, 2026-09-23, *«quero tudo alinhado e padronizado»*). A coluna era `5` alturas de letra
    //    com tecto de `0,42` e o comentário ao lado dizia *«a mesma coluna de rótulo das rows»* —
    //    não era: as rows deste cartão já passavam pela porta, com o nome à DIREITA da coluna do
    //    painel, e esta leitura pintava-o à ESQUERDA numa coluna própria.
    //    ⚠️ A secção mede os QUATRO nomes que ela pode pintar, e não só os do estado em mãos —
    //    senão a coluna mudaria no quadro em que a corrida começa.
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.player.live"),
            tr("panel.inspector.player.posture"),
            tr("panel.inspector.player.facing"),
            tr("panel.inspector.player.speed"),
        ],
    );
    let mut yy = y;
    let mut line = |label: &str, value: &str, scene: &mut VectorScene, ts: &mut TextSystem| {
        let linha =
            super::rows::property_label_row(scene, ts, theme, x, w, yy, ROW_H_PX, label, sec);
        paint_text(
            ts,
            scene,
            value,
            linha.control.x,
            yy + (ROW_H_PX - font) * 0.5,
            font,
            linha.control.w,
            resolve(ColorToken::Text1, theme),
        );
        yy += ROW_H_PX;
    };

    let Some(l) = live else {
        line(
            tr("panel.inspector.player.live"),
            tr("panel.inspector.player.not_simulating"),
            scene,
            text_system,
        );
        return yy;
    };
    // ⚠️ **A tabela é a outra metade do `FootingKind::tag`**, e a ordem dela É o
    // mapeamento: um índice fora dela seria uma postura que este build não
    // conhece, e nomeá-la de qualquer coisa é como um readout passa a mentir.
    const POSTURE: [TextKey; 3] = [
        TextKey::new("panel.inspector.player.posture_air"),
        TextKey::new("panel.inspector.player.posture_steep"),
        TextKey::new("panel.inspector.player.posture_ground"),
    ];
    let posture = POSTURE.get(l.footing_tag as usize).map_or("?", |k| k.tr());
    line(
        tr("panel.inspector.player.posture"),
        posture,
        scene,
        text_system,
    );
    line(
        tr("panel.inspector.player.facing"),
        if l.facing < 0.0 {
            tr("panel.inspector.player.facing_left")
        } else {
            tr("panel.inspector.player.facing_right")
        },
        scene,
        text_system,
    );
    let speed = format!("{:.2}, {:.2} m/s", l.velocity[0], l.velocity[1]);
    line(
        tr("panel.inspector.player.speed"),
        &speed,
        scene,
        text_system,
    );
    yy
}
