//! A geometria de uma fila de abas e o salto de id.
//!
//! ⚠️ **As leis que precisam do REGISTRY de painéis não moram aqui** — nesta crate o
//! `test_support::ensure_panel_registry` é um `{}`, e uma varredura sobre zero painéis passa sobre
//! nada. Elas vivem em `ph2d-panel-registry-init/tests/`.

use super::*;

/// A coluna da direita **no mínimo** (`PANEL_MIN_W_PX`), que é a largura em que o gesto de fechar a
/// deixa — e a largura exacta do report de 2026-09-07.
fn bar() -> Rect {
    Rect::new(1146.0, 28.0, 220.0, TAB_BAR_H)
}

fn occupant(id: &'static str, node: u64, title: &'static str) -> Occupant {
    Occupant {
        id,
        node: NodeId(node),
        title,
    }
}

/// Os três ocupantes da foto do dono, na ordem do registo.
fn three() -> [Occupant; 3] {
    [
        occupant("audio_editor", 11, "Audio Editor"),
        occupant("audio_mixer", 12, "Audio Mixer"),
        occupant("inspector", 13, "Inspector"),
    ]
}

#[test]
fn the_tabs_touch_and_never_overlap() {
    let mut text = TextSystem::without_system_fonts();
    let occ = three();
    let laid = tab_layout(&occ, None, bar(), &mut text);
    assert!(!laid.is_empty());
    for w in laid.windows(2) {
        // ⭐ **Encostam**, sem vão: a lei do grupo — o que separa duas peças é a QUINA.
        assert!(
            (w[0].1.x + w[0].1.w - w[1].1.x).abs() < 0.001,
            "as abas deixaram de encostar: {:?} / {:?}",
            w[0].1,
            w[1].1
        );
    }
    let last = laid.last().unwrap().1;
    assert!(
        last.x + last.w <= bar().x + bar().w + 0.001,
        "a última aba sai da faixa ({last:?})"
    );
}

/// ⭐⭐⭐ **DOIS PAINÉIS DISTINTOS NUNCA MOSTRAM O MESMO RÓTULO** — o defeito do report.
///
/// Com a fila repartida em partes iguais, *«Audio Editor»* e *«Audio Mixer»* elidiam **as duas**
/// para `Audio …` nesta exacta largura de coluna. A cura é a aba medir o próprio nome, e a
/// consequência mede-se aqui: **o rótulo cabe inteiro**, logo não há elisão que o possa colapsar.
///
/// ⚠️ **A asserção do meio é o controlo de vacuidade**: ela prova que esta fixtura É a que falhava
/// — com largura igual, o orçamento fica ABAIXO do nome, que é a condição do defeito.
#[test]
fn a_tab_is_as_wide_as_its_own_name() {
    let mut text = TextSystem::without_system_fonts();
    let occ = three();
    let font = TypeToken::Sm.px();
    let inset = tab_pad_x() * 2.0;

    let equal_share = bar().w / occ.len() as f32;
    let widest = occ
        .iter()
        .map(|o| text.prefix_width(o.title, font))
        .fold(0.0_f32, f32::max);
    assert!(
        equal_share - inset < widest,
        "controlo partido: a partes iguais cada aba teria {equal_share} px e o maior nome mede \
         {widest} px — se ele coubesse, esta fixtura não reproduz o defeito"
    );

    let widths = tab_widths(&occ, &mut text);
    for (o, w) in occ.iter().zip(&widths) {
        let full = text.prefix_width(o.title, font);
        assert!(
            *w - inset >= full - 0.001,
            "«{}» não cabe na própria aba ({w} px, com {inset} de recuo, para {full} px de nome)",
            o.title
        );
    }
    assert!(
        (widths[0] - widths[1]).abs() > 0.001,
        "dois nomes diferentes deram a MESMA largura de aba"
    );
}

/// ⭐⭐⭐ **Quando não cabem todas, o que desliza é a JANELA — as abas não se mexem.**
///
/// > *«quando se clica na aba ela troca de lugar com a outra aba. não permita isso»* — Enio,
/// > 2026-09-07, no smoke da wave 34.
///
/// A fila está na ordem do REGISTO e o escolhido é o topo do z — duas perguntas, duas respostas.
/// Este teste mede a consequência no transbordo: seja qual for o escolhido, ele aparece, **e os
/// vizinhos dele aparecem na mesma ordem relativa**.
#[test]
fn the_window_slides_to_the_chosen_and_never_reorders_the_row() {
    let mut text = TextSystem::without_system_fonts();
    let many: Vec<Occupant> = (0..12)
        .map(|i| occupant("p", 100 + i, "Background Removal"))
        .collect();

    let all = tab_layout(&many, Some(many[0].node), bar(), &mut text);
    assert!(all.len() < many.len(), "doze abas couberam em 220 px");
    assert!(!all.is_empty(), "o transbordo comeu a fila inteira");

    for pick in [0usize, 5, 11] {
        let laid = tab_layout(&many, Some(many[pick].node), bar(), &mut text);
        let shown: Vec<u64> = laid.iter().map(|(o, _)| o.node.0).collect();
        assert!(
            shown.contains(&many[pick].node.0),
            "o escolhido ({pick}) não foi pintado: {shown:?}"
        );
        // ⭐ **Contígua e na ordem** — uma janela que reordenasse seria a troca de lugar do report.
        let first = many.iter().position(|o| o.node.0 == shown[0]).unwrap();
        let esperado: Vec<u64> = many[first..first + shown.len()]
            .iter()
            .map(|o| o.node.0)
            .collect();
        assert_eq!(
            shown, esperado,
            "a janela mexeu na ordem da fila ao escolher {pick}"
        );
    }
}

/// ⛔ **Um nome mais largo que a coluna inteira é APARADO, nunca deitado fora.**
#[test]
fn a_name_wider_than_the_whole_row_is_trimmed_not_dropped() {
    let mut text = TextSystem::without_system_fonts();
    let occ = [occupant(
        "long",
        7,
        "Um nome absurdamente comprido que nunca caberia numa coluna estreita",
    )];
    let laid = tab_layout(&occ, None, bar(), &mut text);
    assert_eq!(laid.len(), 1, "a única aba desapareceu");
    assert!(
        laid[0].1.w <= bar().w + 0.001,
        "a aba aparada continua mais larga que a faixa ({:?})",
        laid[0].1
    );
}

#[test]
fn an_empty_row_has_no_tabs() {
    let mut text = TextSystem::without_system_fonts();
    assert!(tab_layout(&[], None, bar(), &mut text).is_empty());
    assert!(
        tab_layout(&three(), None, Rect::new(0.0, 0.0, 0.0, 0.0), &mut text).is_empty(),
        "uma faixa de área zero pintou abas"
    );
}

/// ⛔ **O controlo de que o salto de id é uma BIJECÇÃO** — ele não pode criar colisões que o
/// espaço de ids de painel já não tivesse, e não pode devolver o próprio id do painel.
#[test]
fn the_tab_id_is_a_bijection_and_never_the_panels_own_id() {
    let ids = [NodeId(1), NodeId(2), NodeId(0xdead_beef), NodeId(u64::MAX)];
    for a in ids {
        assert_ne!(tab_node_id(a), a, "a aba herdou o id do painel ({a:?})");
        assert_eq!(tab_node_id(tab_node_id(a)), a, "o salto não é involutivo");
        for b in ids {
            if a != b {
                assert_ne!(
                    tab_node_id(a),
                    tab_node_id(b),
                    "dois painéis distintos deram a mesma aba"
                );
            }
        }
    }
}
