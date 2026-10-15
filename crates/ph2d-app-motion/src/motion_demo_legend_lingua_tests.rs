//! ⭐⭐⭐ **TODA PALAVRA QUE UMA CENA PINTA NO CANVAS É INGLÊS** — legenda e título de cartão, em
//! TODAS as cenas do roteador.
//!
//! O gate irmão `as_cenas_da_conferencia_pintam_ingles` lê o FONTE de cinco cenas, e só os rótulos
//! de coluna; as legendas (`Caption`) ficavam fora — medido em 14/10: metade das cenas de ciclo
//! pintava legenda em português sem acento. As cenas ficam isentas da TABELA de strings por desenho
//! (o texto é o dado da cena), mas não da LÍNGUA (ordem do dono, 2026-09-19: «tudo em inglês»).
//! ⇒ esta régua lê o PRODUTO: monta cada nível pela porta dos testes e lê o que ele publicou, o que
//! alcança literal, tabela, `format!` e closure da mesma maneira.

use ph2d_editor_core::ProjectSettings;
use ph2d_label_census::portuguese_tokens;

/// O que o nível `level` pinta: as legendas e os títulos dos cartões.
fn pintado(level: &str) -> (Vec<String>, Vec<String>) {
    let mut m = crate::motion_state::MotionState::new();
    let (_, caps) = super::monta(level, &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ProjectSettings::default(),
        &mut snap,
    );
    (
        caps.into_iter().map(|c| c.text).collect(),
        snap.nodes.iter().map(|v| v.display_name.clone()).collect(),
    )
}

#[test]
fn every_word_a_scene_paints_on_the_canvas_is_english() {
    let mut maus = Vec::new();
    let (mut legendas, mut cartoes, mut cenas_com_legenda) = (0usize, 0usize, 0usize);
    for n in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let level = n.to_string();
        let (caps, cards) = pintado(&level);
        cenas_com_legenda += usize::from(!caps.is_empty());
        legendas += caps.len();
        cartoes += cards.len();
        for (onde, t) in caps
            .iter()
            .map(|t| ("legenda", t))
            .chain(cards.iter().map(|t| ("cartão", t)))
        {
            let pt = portuguese_tokens(t);
            if !pt.is_empty() {
                maus.push(format!("={level} {onde}: {t:?} {pt:?}"));
            }
        }
    }
    // ⛔ PISOS (14/10: 42 cenas com legenda, 192 legendas, 3 884 cartões): um leitor que deixe de
    // montar lê zero e passa.
    assert!(
        cenas_com_legenda >= 42 && legendas >= 192 && cartoes >= 3800,
        "o censo leu {cenas_com_legenda} cenas com legenda, {legendas} legendas e {cartoes} cartões \
         — ou as cenas mudaram, ou a montagem partiu-se"
    );
    assert!(
        maus.is_empty(),
        "{} texto(s) pintados no canvas em português — o artista lê-os assim:\n  {}",
        maus.len(),
        maus.join("\n  ")
    );
}

// O controlo POSITIVO da régua sobre ESTAS legendas mora com ela: `portugues_cenas_2026_10_14.txt`
// é o que as cenas pintavam antes da tradução, e `ela_acusa_o_portugues_que_as_cenas_pintavam`
// exige que a régua acuse 184 dos 195 (e nomeia os 11 que lhe fogem).
