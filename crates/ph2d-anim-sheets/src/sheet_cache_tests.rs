use super::*;

fn px(v: u8) -> Option<(Vec<u8>, u32, u32)> {
    Some((vec![v; 4 * 4], 2, 2))
}

/// ⭐ Dono 08/11: a folha que o Painter pinta mostra a tinta por assar, lida UMA vez quando o traço acaba; assada
/// (nada por assar), volta a imagem gravada. *Mutações: ler a cada quadro ⇒ a leitura do meio do traço acontece; sem
/// `live` no `of_sheet` ⇒ nada aparece; não largar ao assar ⇒ a tinta velha fica por cima da gravada.*
#[test]
fn a_folha_pintada_mostra_a_tinta_quando_o_traco_acaba() {
    let mut w = World::new();
    let e = w.spawn_empty().id().to_bits();
    let mut leu = 0;
    // O traço anda: a revisão muda a cada quadro, nada se lê.
    for r in 1..4 {
        follow_live(&mut w, Some((e, (0, r))), || {
            leu += 1;
            px(9)
        });
    }
    assert_eq!(leu, 0, "a meio do traço não se lê");
    // Parou: uma leitura, e a folha 0 mostra-a.
    for _ in 0..3 {
        follow_live(&mut w, Some((e, (0, 3))), || {
            leu += 1;
            px(9)
        });
    }
    assert_eq!(leu, 1, "uma leitura por traço");
    let c = w.resource::<SheetCache>();
    assert_eq!(c.of_sheet(e, 0).map(|x| x.sheet.rgba[0]), Some(9));
    assert!(c.of_sheet(e, 1).is_none(), "só a folha visível");
    // Assou: a gravada volta.
    follow_live(&mut w, None, || px(1));
    assert!(w.resource::<SheetCache>().of_sheet(e, 0).is_none());
}
