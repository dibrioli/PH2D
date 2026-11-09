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
        follow_live(&mut w, Some((e, (0, r))), 0, || {
            leu += 1;
            px(9)
        });
    }
    assert_eq!(leu, 0, "a meio do traço não se lê");
    // Parou: uma leitura quando fica QUADROS_PARADOS quadros parada, e a folha 0 mostra-a.
    for _ in 1..QUADROS_PARADOS {
        follow_live(&mut w, Some((e, (0, 3))), 0, || {
            leu += 1;
            px(9)
        });
    }
    assert_eq!(
        leu, 0,
        "parada há menos de QUADROS_PARADOS quadros ainda não se lê"
    );
    for _ in 0..3 {
        follow_live(&mut w, Some((e, (0, 3))), 0, || {
            leu += 1;
            px(9)
        });
    }
    assert_eq!(leu, 1, "uma leitura por traço");
    let c = w.resource::<SheetCache>();
    assert_eq!(c.of_sheet(e, 0).map(|x| x.sheet.rgba[0]), Some(9));
    assert!(c.of_sheet(e, 1).is_none(), "só a folha visível");
    // Assou: a gravada volta.
    follow_live(&mut w, None, 0, || px(1));
    assert!(w.resource::<SheetCache>().of_sheet(e, 0).is_none());
}

/// ⭐ **O selo da prévia** (dono 09/11): gira enquanto a tinta anda — também com a água do Wet, que pára 2 de cada 3
/// quadros e não pode fazê-lo piscar para «encher» —, enche de verdade pelos [`QUADROS_PARADOS`] quando ela pára, vira
/// o check verde na leitura e some; outro objeto não o vê; ao assar some sem check. *Mutações: `QUADROS_A_GIRAR` 1 ⇒
/// enche com a água a andar; `feito` não armado na leitura ⇒ sem check; check ao assar ⇒ `Done` depois do `None`.*
#[test]
fn o_selo_gira_com_a_tinta_a_andar_enche_parada_e_acaba_no_check() {
    let mut w = World::new();
    let e = w.spawn_empty().id().to_bits();
    let sync = |w: &World, b| w.resource::<SheetCache>().sync(b);
    // Um quadro a 60 Hz: o relógio da interface anda com cada chamada.
    let mut agora = 0u128;
    let mut quadro = |w: &mut World, vivo| {
        agora += 16_666_667;
        follow_live(w, vivo, agora, || px(5));
    };
    for q in 0..120u64 {
        quadro(&mut w, Some((e, (0, 100 + q / 3))));
        assert_eq!(
            sync(&w, e),
            Some(ProgressPhase::Indeterminate),
            "quadro {q} com a água a andar"
        );
    }
    assert_eq!(sync(&w, e + 1), None, "outro objeto");
    // O quadro em que ela muda pela última vez e os parados que se seguem, até à véspera da leitura.
    let (mut antes, mut enchendo) = (-1.0, 0);
    for q in 0..QUADROS_PARADOS {
        quadro(&mut w, Some((e, (0, 999))));
        match sync(&w, e) {
            Some(ProgressPhase::Indeterminate) => assert!(q < QUADROS_A_GIRAR, "parada há {q}"),
            Some(ProgressPhase::Filling(f)) => {
                assert!(f > antes && f < 1.0, "parada há {q}: {f}");
                (antes, enchendo) = (f, enchendo + 1);
            }
            outro => panic!("parada há {q}: {outro:?}"),
        }
    }
    assert_eq!(
        enchendo,
        QUADROS_PARADOS - QUADROS_A_GIRAR,
        "encheu em todos os quadros parados depois de girar"
    );
    quadro(&mut w, Some((e, (0, 999))));
    assert_eq!(
        sync(&w, e),
        Some(ProgressPhase::Done(0.0)),
        "a folha atualizou-se"
    );
    // A vida do check é de RELÓGIO: a 60 Hz, 1,2 s são 72 quadros (a 30 Hz seriam 36 — a mesma vida).
    let quadros_da_vida = VIDA_DO_CHECK_NS.div_ceil(16_666_667);
    for _ in 1..quadros_da_vida {
        quadro(&mut w, Some((e, (0, 999))));
        assert!(matches!(sync(&w, e), Some(ProgressPhase::Done(t)) if t > 0.0 && t < 1.0));
    }
    quadro(&mut w, Some((e, (0, 999))));
    assert_eq!(sync(&w, e), None, "o check some");
    // Outro traço e assar antes de assentar: some sem check (nada se leu).
    quadro(&mut w, Some((e, (0, 1000))));
    assert_eq!(sync(&w, e), Some(ProgressPhase::Indeterminate));
    quadro(&mut w, None);
    assert_eq!(sync(&w, e), None, "assado: sem selo nem check");
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
        follow_live(&mut w, Some((e, (0, 100 + q / 3))), 0, || {
            leu += 1;
            px(5)
        });
    }
    assert_eq!(leu, 0, "a água a andar fez {leu} leituras da tela inteira");
    // Assentou: meio segundo parada e lê-se uma vez.
    for _ in 0..=QUADROS_PARADOS {
        follow_live(&mut w, Some((e, (0, 999))), 0, || {
            leu += 1;
            px(5)
        });
    }
    assert_eq!(
        leu, 1,
        "assentada, a tinta não se leu (ou leu-se mais de uma vez)"
    );
}
