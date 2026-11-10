use super::*;
use std::time::Instant;

fn pede(ativos: &[Produtor], focada: bool) -> Pedido {
    quer_quadro(&Estado { ativos, focada })
}

/// G2 (a lei): cada produtor contínuo, sozinho, pede quadro já com a janela em foco; desfocada,
/// Documento e Isento continuam já e só a UI desce a 10 q/s.
#[test]
fn cada_produtor_continuo_pede_quadro() {
    for p in Produtor::TODOS {
        assert_eq!(pede(&[p], true), Pedido::Ja(p), "{p:?} em foco");
        let desfocada = match p.classe() {
            Classe::Documento | Classe::Isento => Pedido::Ja(p),
            Classe::Ui => Pedido::Ate {
                prazo: DESFOCADA,
                porque: Parada::Desfocada,
            },
        };
        assert_eq!(pede(&[p], false), desfocada, "{p:?} desfocada");
    }
}

/// ⛔ G3 (a lei): o papel molhado e o traço parado seguram o laço contra a UI desfocada à frente
/// na lista — o byte da secagem depende do passo, não pode dormir.
#[test]
fn o_papel_molhado_e_o_traco_parado_seguram_o_laco() {
    for doc in [Produtor::PapelMolhado, Produtor::TracoAberto] {
        for focada in [true, false] {
            assert_eq!(
                pede(&[Produtor::Toast, Produtor::Entrada, doc], focada),
                Pedido::Ja(doc)
            );
        }
    }
}

/// G9: sem ninguém, o laço dorme o [`PRAZO`] com a parada nomeada; nenhum sono passa do prazo.
#[test]
fn o_sono_e_parada_nomeada() {
    for focada in [true, false] {
        assert_eq!(
            pede(&[], focada),
            Pedido::Ate {
                prazo: PRAZO,
                porque: Parada::Ocioso
            }
        );
    }
    assert!(DESFOCADA > Duration::ZERO && DESFOCADA <= PRAZO);
    assert_eq!(Produtor::TODOS.len(), 14);
}

/// O laço: a entrada segura o quadro só o [`PRAZO`]; depois dorme com a parada nomeada.
#[test]
fn a_entrada_segura_o_laco_so_o_prazo() {
    let t0 = Instant::now();
    let mut l = Laco::novo(false);
    l.entrada(t0);
    let antes = t0 + PRAZO / 2;
    assert_eq!(l.pedido(&[], antes), Pedido::Ja(Produtor::Entrada));
    assert_eq!(l.acorda(&[], antes), None);
    let depois = t0 + PRAZO * 2;
    assert_eq!(l.acorda(&[], depois), Some(depois + PRAZO));
    l.foco(false);
    assert_eq!(l.acorda(&[], antes), Some(antes + DESFOCADA));
    assert_eq!(
        l.pedido(&[Produtor::PapelMolhado], depois),
        Pedido::Ja(Produtor::PapelMolhado)
    );
}

/// Um smoke (que conta quadros) nunca dorme, nem desfocado: `PH2D_EXIT_AFTER_FRAMES` não pendura.
#[test]
fn o_smoke_nao_dorme_nem_desfocado() {
    let mut l = Laco::novo(true);
    l.foco(false);
    assert_eq!(l.pedido(&[], Instant::now()), Pedido::Ja(Produtor::Smoke));
    for (k, v) in [
        ("PH2D_EXIT_AFTER_FRAMES", "120"),
        ("PH2D_COMPOSITE_SMOKE", "1"),
        ("PH2D_GPU_COOK_DEMO", "3"),
        ("PH2D_QUADRO_A_PEDIDO", "0"),
    ] {
        assert!(e_smoke(k, v), "{k}");
    }
    assert!(!e_smoke("PH2D_QUADRO_A_PEDIDO", "1"));
    assert!(!e_smoke("PH2D_CARDS", "0"));
}

/// B23.3: oculta, a UI (a entrada incluída) dorme o [`PRAZO`]; o documento e o Play não param.
#[test]
fn a_janela_oculta_so_acorda_pelo_documento_e_pelo_play() {
    let t0 = Instant::now();
    let mut l = Laco::novo(false);
    l.oculta(true);
    l.entrada(t0);
    let oculta = Pedido::Ate {
        prazo: PRAZO,
        porque: Parada::Oculta,
    };
    assert_eq!(l.pedido(&[Produtor::UiAnima, Produtor::Toast], t0), oculta);
    for p in [Produtor::PapelMolhado, Produtor::Play, Produtor::Som] {
        assert_eq!(l.pedido(&[Produtor::Toast, p], t0), Pedido::Ja(p));
    }
    l.oculta(false);
    assert_eq!(l.pedido(&[], t0), Pedido::Ja(Produtor::Entrada));
}

/// B23.3: oculta, o quadro não apresenta (o documento avança na mesma); visível ou num smoke, apresenta.
#[test]
fn a_janela_oculta_nao_apresenta_salvo_o_smoke() {
    for smoke in [false, true] {
        let mut l = Laco::novo(smoke);
        assert!(l.apresenta(), "visível, smoke={smoke}");
        l.foco(false);
        assert!(l.apresenta(), "desfocada mas à vista, smoke={smoke}");
        l.oculta(true);
        assert_eq!(l.apresenta(), smoke, "oculta, smoke={smoke}");
        l.oculta(false);
        assert!(l.apresenta(), "de volta à vista, smoke={smoke}");
    }
    for (forca, apresenta) in [(Some(true), true), (Some(false), false)] {
        let mut l = Laco {
            forca_oculta: forca,
            ..Laco::novo(true)
        };
        l.oculta(true);
        assert_eq!(l.apresenta(), apresenta, "PH2D_OCULTA_APRESENTA={forca:?}");
    }
}
