use super::*;

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
