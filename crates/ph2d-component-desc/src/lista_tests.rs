//! Os gates do vocabulário das listas (F30 §A1).

use super::*;
use crate::Hidden;
use crate::catalog::listas::{DE_SECCAO, SEM_CANAL};

/// ⭐⭐ **Toda lista do catálogo tem código** (ou diz porque não): um componente novo com lista
/// reprova até escolher o seu lado.
#[test]
fn toda_lista_do_catalogo_tem_codigo() {
    for d in crate::all().filter(|d| d.hidden == Some(Hidden::List)) {
        let tem = do_componente(d.canonical_name).next().is_some();
        let isento = SEM_CANAL.contains(&d.canonical_name);
        assert!(
            tem ^ isento,
            "{}: sem código de LISTA nem motivo",
            d.canonical_name
        );
    }
    for (c, chave) in DE_SECCAO {
        let d = crate::desc_for(c).expect("o componente existe");
        assert!(
            d.hidden_keys
                .iter()
                .any(|(k, h)| k == chave && *h == Hidden::OwnSection),
            "{c}.{chave}"
        );
        assert!(do_componente(c).any(|l| l.caminho == *chave), "{c}.{chave}");
    }
    for l in todas() {
        assert!(crate::desc_for(l.componente).is_some(), "{}", l.componente);
    }
}

type LinhaFixa<'a> = (u16, &'a str, &'a str, Vec<(u16, &'a str)>);

/// ⭐⭐ **O vocabulário das listas não renumera** — cada número está gravado nas chaves de animação.
#[test]
fn o_vocabulario_das_listas_nao_renumera() {
    let agora: Vec<LinhaFixa<'_>> = todas()
        .iter()
        .map(|l| {
            let campos = l.campos.iter().map(|f| (f.field_id, f.key)).collect();
            (l.codigo, l.componente, l.caminho, campos)
        })
        .collect();
    let fixo: Vec<LinhaFixa<'_>> = vec![
        (
            1,
            "ph2d::ecs::Timers",
            ".",
            vec![
                (1, "name"),
                (2, "duration_us"),
                (3, "repeat"),
                (4, "autostart"),
                (5, "signal"),
            ],
        ),
        (
            2,
            "ph2d::ecs::Tweens",
            ".",
            vec![
                (1, "canal"),
                (2, "de"),
                (3, "para"),
                (4, "easing/family"),
                (5, "easing/mode"),
                (6, "ao_acabar"),
                (7, "ciclo"),
            ],
        ),
        (
            3,
            "ph2d::ecs::StateMachine",
            "states",
            vec![(1, "name"), (2, "on_enter"), (3, "on_exit")],
        ),
        (
            4,
            "ph2d::ecs::StateMachine",
            "transitions",
            vec![(4, "from"), (5, "on"), (6, "to")],
        ),
        (
            5,
            "ph2d::ecs::CounterWatch",
            ".",
            vec![
                (1, "counter"),
                (2, "compare"),
                (3, "value"),
                (4, "signal"),
                (5, "once"),
                (6, "scope"),
            ],
        ),
        (
            6,
            "ph2d::ecs::SignalActions",
            ".",
            vec![
                (1, "on"),
                (2, "target"),
                (3, "verb"),
                (4, "arg"),
                (5, "target_by"),
                (6, "from"),
            ],
        ),
        (
            7,
            "ph2d::ecs::SignalOnAction",
            ".",
            vec![(1, "action"), (2, "edge"), (3, "signal")],
        ),
        (
            8,
            "ph2d::ecs::ShakeEmitter",
            ".",
            vec![
                (0, "on"),
                (1, "de"),
                (2, "forca"),
                (3, "dentro"),
                (4, "fora"),
            ],
        ),
        (
            9,
            "ph2d::physics::Health",
            "resistances",
            vec![(1, "kind"), (2, "rate"), (3, "absorbs")],
        ),
        (
            10,
            "ph2d::ecs::NamedAnchorList",
            ".",
            vec![
                (1, "name"),
                (2, "transform/translation"),
                (3, "transform/rotation"),
                (4, "bounds"),
                (5, "center"),
            ],
        ),
    ];
    assert_eq!(
        agora, fixo,
        "um código, um campo ou uma chave mudou: só APPEND"
    );
}

/// O item descrito aqui é o MESMO campo que o descritor do componente já nomeia (rótulo e id).
#[test]
fn o_item_e_o_mesmo_campo_do_componente() {
    for l in todas() {
        let d = crate::desc_for(l.componente).expect("existe");
        // Só quando os campos do descritor SÃO os do item (o componente é a lista).
        if d.hidden != Some(Hidden::List) {
            continue;
        }
        for f in l.campos {
            if let Some(g) = d.fields.iter().find(|g| g.field_id == f.field_id) {
                assert_eq!(f.label_key, g.label_key, "{} #{}", l.componente, f.field_id);
            }
        }
    }
}

/// Cada campo de item é canal, e os números cabem na chave.
#[test]
fn todo_campo_de_item_cabe_na_chave_e_e_canal() {
    let mut vistos = std::collections::BTreeSet::new();
    for l in todas() {
        assert!(vistos.insert(l.codigo), "código repetido {}", l.codigo);
        for f in l.campos {
            assert!(
                f.faixas_de_canal() > 0,
                "{} #{} não é canal",
                l.componente,
                f.field_id
            );
            let k = campo(l.codigo, f.field_id, 3).expect("cabe");
            assert_eq!(decompor(k), (l.codigo, f.field_id, 3));
        }
    }
    assert_eq!(
        chaves(codigo::TIMERS, 2, 0x0001_0002),
        vec![[FAMILIA, 1 << 10 | 2 << 2, 1, 2]]
    );
    assert_eq!(
        chaves(codigo::ANCORAS, 2, 7).len(),
        2,
        "a posição tem x e y"
    );
}
