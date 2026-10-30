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

/// ⭐ F41: os componentes de uma macro `1` (antes da v25 da caixa) passam pela conversão das caixas do
/// registo — no inicial e num passo; os de uma `2` não. A lei (o Follow Value) prova-se no
/// `ph2d-app-constraints` (`valor_caixa_v209`); aqui, a porta da versão, com uma conversão de prova.
#[test]
fn uma_macro_1_passa_os_componentes_pela_conversao_das_caixas() {
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    reg.register_caixas_v209("ph2d::ecs::Transform", |_| Some(vec![42]));
    let linha = Linha {
        id: 1,
        pai: None,
        componentes: vec![Componente {
            nome: "ph2d::ecs::Transform".into(),
            bytes: vec![7],
        }],
    };
    let inicial = Inicial {
        objetos: Vec::new(),
        linhas: vec![linha.clone()],
        recursos: Vec::new(),
    };
    let passo = Evento::Passo {
        novos: Vec::new(),
        mudancas: vec![
            Mudanca::Nasce(linha),
            Mudanca::Muda {
                id: 1,
                pai: None,
                componentes: vec![("ph2d::ecs::Transform".into(), Some(vec![7]))],
            },
        ],
        recursos: Vec::new(),
    };
    for (versao, esperado) in [(1u16, 42u8), (VERSAO, 7)] {
        let com_versao = |mut b: Vec<u8>| {
            b[..2].copy_from_slice(&versao.to_le_bytes());
            b
        };
        let t = ph2d_macro::Track {
            family: FAMILIA.into(),
            target: 0,
            initial: com_versao(inicial.escreve()),
            events: vec![ph2d_macro::Event {
                t: ph2d_macro::MacroTime(0),
                payload: com_versao(passo.escreve()),
            }],
            edicoes: Vec::new(),
            nasce: None,
            continua: None,
        };
        let lida = super::super::estado::Lida::de(&t, &reg).expect("lê");
        assert_eq!(
            lida.inicial.linhas[0].componentes[0].bytes,
            [esperado],
            "v{versao}: inicial"
        );
        let Evento::Passo { mudancas, .. } = &lida.eventos[0].1 else {
            panic!("o passo")
        };
        let Mudanca::Nasce(l) = &mudancas[0] else {
            panic!("nasce")
        };
        assert_eq!(l.componentes[0].bytes, [esperado], "v{versao}: nasce");
        let Mudanca::Muda { componentes, .. } = &mudancas[1] else {
            panic!("muda")
        };
        assert_eq!(
            componentes[0].1.as_deref(),
            Some(&[esperado][..]),
            "v{versao}: muda"
        );
    }
}
