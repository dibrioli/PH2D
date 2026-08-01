//! Gates do modelo de guias.

use super::*;

#[test]
fn a_horizontal_guide_locks_the_y_and_a_vertical_one_locks_the_x() {
    // ⚠️ O gate que existe porque o compilador NÃO ajuda: os dois eixos são `usize`, e o nome
    // descreve a DIREÇÃO da linha enquanto o índice descreve a coordenada CONGELADA. Trocar os
    // dois compila, e o sintoma seria uma guia que encaixa no eixo errado — indistinguível de
    // "o snap não funciona" para quem olha.
    assert_eq!(GuideAxis::Horizontal.locked_axis(), 1, "horizontal fixa Y");
    assert_eq!(GuideAxis::Vertical.locked_axis(), 0, "vertical fixa X");

    // E a distância confirma pela outra ponta: uma guia horizontal é indiferente ao X.
    let h = Guide::horizontal(10.0);
    assert!((h.distance_to([0.0, 12.0]) - 2.0).abs() < 1e-12);
    assert!(
        (h.distance_to([9999.0, 12.0]) - 2.0).abs() < 1e-12,
        "andar 9999 no X não aproxima nem afasta de uma linha horizontal"
    );
}

#[test]
fn two_guides_can_share_a_position_and_dragging_one_reveals_the_other() {
    // A decisão de NÃO deduplicar, enunciada como comportamento: soltar uma guia sobre outra
    // acrescenta; arrastar uma delas deixa a outra onde estava.
    let mut set = GuideSet::default();
    let a = set.push(Guide::vertical(50.0));
    let b = set.push(Guide::vertical(50.0));
    assert_eq!(set.len(), 2, "a segunda guia não foi engolida em silêncio");

    set.set_pos(b, 120.0);
    assert_eq!(set.get(a).unwrap().pos, 50.0);
    assert_eq!(set.get(b).unwrap().pos, 120.0);
}

#[test]
fn removing_a_guide_keeps_the_order_of_the_others() {
    // ⚠️ O índice É a identidade de uma guia durante um arrasto. Um `swap_remove` seria mais
    // barato e faria o gesto seguinte pegar OUTRA linha — o defeito silencioso que este gate
    // pina (a mutação `remove` → `swap_remove` o derruba).
    let mut set = GuideSet::default();
    for x in [0.0, 10.0, 20.0, 30.0] {
        set.push(Guide::vertical(x));
    }
    set.remove(1);
    let left: Vec<f64> = set.iter().map(|g| g.pos).collect();
    assert_eq!(left, vec![0.0, 20.0, 30.0], "a ordem sobrevive à remoção");
}

#[test]
fn the_nearest_guide_is_the_one_under_the_cursor_and_nothing_beyond_the_tolerance() {
    let mut set = GuideSet::default();
    set.push(Guide::vertical(0.0));
    set.push(Guide::horizontal(100.0));

    assert_eq!(set.nearest([0.4, 500.0], 1.0), Some(0), "pega a vertical");
    assert_eq!(
        set.nearest([500.0, 99.6], 1.0),
        Some(1),
        "pega a horizontal"
    );
    assert_eq!(
        set.nearest([50.0, 50.0], 1.0),
        None,
        "longe das duas: nada é pego"
    );
}

#[test]
fn a_tie_goes_to_the_most_recent_guide() {
    // Duas empilhadas: a de cima é a que o artista acabou de soltar, e é a que a mão espera
    // pegar. Qualquer outra escolha lê como aleatória.
    let mut set = GuideSet::default();
    set.push(Guide::vertical(50.0));
    let newer = set.push(Guide::vertical(50.0));
    assert_eq!(set.nearest([50.0, 0.0], 1.0), Some(newer));
}

#[test]
fn a_guide_survives_the_round_trip() {
    let mut set = GuideSet::default();
    set.push(Guide::vertical(-3.5));
    set.push(Guide::horizontal(1234.25));
    let bytes = postcard::to_allocvec(&set).expect("serializa");
    let back: GuideSet = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(back, set);
}

/// **O número que substitui um teto** (§0: um limite tem de dizer de que recurso ele é).
///
/// Não há teto no número de guias. O recurso que um teto protegeria seria o custo de consulta,
/// e este teste o mede: o `nearest` sobre um documento absurdo (1000 guias) contra o gesto real.
/// Se um dia isto sair da ordem dos microssegundos, o número está aqui para ser reconferido.
#[test]
fn the_cost_of_a_guide_is_a_comparison() {
    let mut set = GuideSet::default();
    for i in 0..1000 {
        set.push(Guide::vertical(f64::from(i)));
    }
    let t = std::time::Instant::now();
    let mut hits = 0usize;
    for i in 0..1000 {
        if set.nearest([f64::from(i) + 0.25, 0.0], 0.5).is_some() {
            hits += 1;
        }
    }
    let us = t.elapsed().as_secs_f64() * 1e6 / 1000.0;
    assert_eq!(hits, 1000, "cada consulta acha a sua guia");
    eprintln!("[guides] nearest sobre 1000 guias: {us:.3} us por consulta");
    assert!(
        us < 200.0,
        "uma varredura linear de 1000 guias custa {us:.3} us — se isto disparar, \
         o documento saiu do regime que a ausência de teto pressupõe"
    );
}
