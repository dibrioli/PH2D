use super::*;
use crate::canal::{FieldKey, familia};

fn k(campo: u16) -> FieldKey {
    FieldKey::new(familia::MESA, campo, 0, 0)
}

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_da_mesa_nao_renumera() {
    assert_eq!(familia::MESA, 12);
    let fixos = [
        (GANHO, 1),
        (CORTE_ALTO, 2),
        (CORTE_BAIXO, 3),
        (MUDO, 4),
        (PAN, 5),
        (LIMITADOR, 6),
        (REVERB, 7),
        (REVERB_TAMANHO, 8),
        (REVERB_RETORNO, 9),
        (EQ_GRAVE, 10),
        (EQ_MEDIO, 11),
        (EQ_AGUDO, 12),
        (DELAY, 13),
        (DELAY_TEMPO, 14),
        (DELAY_FEEDBACK, 15),
        (DELAY_RETORNO, 16),
        (DUCKING, 17),
        (DUCKING_PROFUNDIDADE, 18),
        (DUCKING_CHAVE, 19),
    ];
    assert_eq!(fixos.len(), MASTER.len(), "um campo sem número fixado");
    for (atual, gravado) in fixos {
        assert_eq!(atual, gravado, "o campo {gravado} da mesa foi renumerado");
    }
    let do_bus = [
        (B_GANHO, 0),
        (B_MUDO, 1),
        (B_SOLO, 2),
        (B_PAN, 3),
        (B_TOM, 4),
        (B_CORTE_BAIXO, 5),
        (B_ENVIO_REVERB, 6),
        (B_ENVIO_DELAY, 7),
        (B_COMP, 8),
    ];
    assert_eq!(do_bus.len(), DO_BUS.len());
    for (atual, gravado) in do_bus {
        assert_eq!(
            atual, gravado,
            "o campo {gravado} do sub-barramento foi renumerado"
        );
    }
    assert_eq!((BUS_BASE, BUS_PASSO, BUSES), (32, 16, 4));
    assert_eq!(bus(3, B_COMP), 88, "o comp do Voice");
}

/// Todo campo diz o que é, é da Cena e é o seu próprio grupo; interruptores e a chave são
/// discretos; o resto interpola-se.
#[test]
fn cada_campo_da_mesa_diz_o_que_e() {
    let todos: Vec<u16> = campos().collect();
    assert_eq!(todos.len(), MASTER.len() + 4 * DO_BUS.len());
    for &campo in &todos {
        assert!(k(campo).info().is_some(), "campo {campo} sem info");
        assert!(k(campo).membro().is_some(), "campo {campo} sem nome");
        assert!(k(campo).da_cena(), "campo {campo} é da Cena");
        assert_eq!(k(campo).grupo(), Some((familia::MESA, campo, 0)));
    }
    for d in [
        MUDO,
        LIMITADOR,
        REVERB,
        DELAY,
        DUCKING,
        DUCKING_CHAVE,
        bus(1, B_SOLO),
    ] {
        assert!(k(d).info().is_some_and(|i| i.discreto()), "{d}");
    }
    for cont in [GANHO, PAN, EQ_MEDIO, bus(0, B_GANHO), bus(2, B_ENVIO_DELAY)] {
        assert!(k(cont).info().is_some_and(|i| !i.discreto()), "{cont}");
    }
    let rotulos: std::collections::BTreeSet<_> = todos
        .iter()
        .filter_map(|&c| k(c).info())
        .map(|i| i.rotulo)
        .collect();
    assert_eq!(rotulos.len(), todos.len(), "dois campos com o mesmo rótulo");
    for buraco in [0, 20, 31, bus(0, 9), bus(4, 0)] {
        assert!(k(buraco).info().is_none(), "buraco {buraco} no vocabulário");
    }
}
