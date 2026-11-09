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
    // Parou: uma leitura quando fica QUADROS_PARADOS quadros parada, e a folha 0 mostra-a.
    for _ in 1..QUADROS_PARADOS {
        follow_live(&mut w, Some((e, (0, 3))), || {
            leu += 1;
            px(9)
        });
    }
    assert_eq!(
        leu, 0,
        "parada há menos de QUADROS_PARADOS quadros ainda não se lê"
    );
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

/// ⭐ **A água do Wet não dispara a leitura** (dono 09/11: «pintar com wet paint ficou lentíssimo», raw 170 → 14). Depois do
/// traço a água re-revisa a tinta a ~24 Hz — a 60 Hz, um passo a cada 2–3 quadros, com quadros parados entre eles. Cada
/// leitura compõe e acende a tela inteira (61 ms a 1024² Textured Paper), por isso nenhuma pode correr enquanto a água
/// anda; quando ela assenta, uma. *Mutação: ler com um quadro parado (a regra de antes) ⇒ ~40 leituras aqui.*
#[test]
fn a_agua_do_wet_a_andar_nao_le_a_tela_a_cada_passo() {
    let mut w = World::new();
    let e = w.spawn_empty().id().to_bits();
    let mut leu = 0;
    // Dois segundos de água a 60 Hz: a revisão anda a cada 3 quadros (20 Hz) e fica parada nos outros dois.
    for q in 0..120u64 {
        follow_live(&mut w, Some((e, (0, 100 + q / 3))), || {
            leu += 1;
            px(5)
        });
    }
    assert_eq!(leu, 0, "a água a andar fez {leu} leituras da tela inteira");
    // Assentou: meio segundo parada e lê-se uma vez.
    for _ in 0..=QUADROS_PARADOS {
        follow_live(&mut w, Some((e, (0, 999))), || {
            leu += 1;
            px(5)
        });
    }
    assert_eq!(
        leu, 1,
        "assentada, a tinta não se leu (ou leu-se mais de uma vez)"
    );
}
