use super::*;
use crate::mundo::formato::{Componente, Linha};

fn c(nome: &str, b: u8) -> Componente {
    Componente {
        nome: nome.into(),
        bytes: vec![b],
    }
}

fn lida() -> Lida {
    let inicial = Inicial {
        objetos: vec![
            Objeto {
                id: 0,
                original: 10,
                quem: Quem::Macro,
            },
            Objeto {
                id: 1,
                original: 11,
                quem: Quem::Trilha(0),
            },
        ],
        linhas: vec![
            Linha {
                id: 0,
                pai: None,
                componentes: vec![],
            },
            Linha {
                id: 1,
                pai: None,
                componentes: vec![c("Transform", 1)],
            },
        ],
        recursos: vec![],
    };
    let eventos = vec![
        (
            MacroTime(100),
            Evento::Passo {
                novos: vec![],
                mudancas: vec![Mudanca::Muda {
                    id: 1,
                    pai: None,
                    componentes: vec![("Collider".into(), Some(vec![7]))],
                }],
                recursos: vec![],
            },
        ),
        (
            MacroTime(200),
            Evento::Passo {
                novos: vec![Objeto {
                    id: 2,
                    original: 12,
                    quem: Quem::Mundo,
                }],
                mudancas: vec![Mudanca::Nasce(Linha {
                    id: 2,
                    pai: Some(1),
                    componentes: vec![c("Name", 2)],
                })],
                recursos: vec![],
            },
        ),
        (
            MacroTime(300),
            Evento::Passo {
                novos: vec![],
                mudancas: vec![
                    Mudanca::Morre(2),
                    Mudanca::Muda {
                        id: 1,
                        pai: None,
                        componentes: vec![("Collider".into(), None)],
                    },
                ],
                recursos: vec![],
            },
        ),
    ];
    Lida { inicial, eventos }
}

#[test]
fn o_mundo_da_macro_em_cada_instante() {
    let l = lida();
    let antes = l.em(MacroTime(99));
    assert!(
        !antes[&1].componentes.contains_key("Collider"),
        "o collider ainda não foi posto"
    );
    assert!(!antes.contains_key(&2), "o 2 ainda não nasceu");
    let posto = l.em(MacroTime(100));
    assert_eq!(posto[&1].componentes["Collider"], [7]);
    assert_eq!(posto[&1].quem, Quem::Trilha(0));
    let nascido = l.em(MacroTime(250));
    assert!(nascido[&2].vivo);
    assert_eq!(nascido[&2].pai, Some(1));
    assert_eq!(nascido[&2].original, 12);
    let fim = l.no_fim();
    assert!(!fim[&2].vivo, "morreu");
    assert!(!fim[&1].componentes.contains_key("Collider"), "tirado");
    assert_eq!(fim[&1].componentes["Transform"], [1]);
}
