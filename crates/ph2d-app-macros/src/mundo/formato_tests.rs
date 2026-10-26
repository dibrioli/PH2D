use super::*;

/// F32 §5: uma macro gravada antes de os itens terem id (`Timers` na forma 194, campo a campo) e de
/// o tween ter relógio por id (`Tweens` na forma 196) abre com os componentes na forma de agora — no
/// estado inicial e num passo.
#[test]
fn uma_macro_antiga_le_os_componentes_na_forma_de_agora() {
    let timers_v194: Vec<(String, u64, bool, bool, String)> =
        vec![("a".into(), 1_000_000, false, true, String::new())];
    let mut tweens_v196 =
        postcard::to_allocvec(&ph2d_ecs::Tweens(vec![Default::default()])).expect("t");
    assert_eq!(tweens_v196.pop(), Some(0), "sem o relógio do fim");
    let linha = Linha {
        id: 1,
        pai: None,
        componentes: vec![
            Componente {
                nome: "ph2d::ecs::Timers".into(),
                bytes: postcard::to_allocvec(&timers_v194).expect("v194"),
            },
            Componente {
                nome: "ph2d::ecs::Tweens".into(),
                bytes: tweens_v196,
            },
        ],
    };
    let ler = |c: &[Componente]| {
        let t: ph2d_ecs::Timers = postcard::from_bytes(&c[0].bytes).expect("Timers de agora");
        let w: ph2d_ecs::Tweens = postcard::from_bytes(&c[1].bytes).expect("Tweens de agora");
        assert_eq!((t.0[0].duration_us, w.0.len()), (1_000_000, 1));
    };
    let inicial = Inicial {
        linhas: vec![linha.clone()],
        ..Inicial::default()
    };
    ler(&Inicial::le(&inicial.escreve()).expect("lê").linhas[0].componentes);
    let passo = Evento::Passo {
        novos: Vec::new(),
        mudancas: vec![Mudanca::Nasce(linha)],
        recursos: Vec::new(),
    };
    let Evento::Passo { mudancas, .. } = Evento::le(&passo.escreve()).expect("lê") else {
        panic!("passo");
    };
    let Mudanca::Nasce(l) = &mudancas[0] else {
        panic!("nasce");
    };
    ler(&l.componentes);
}
