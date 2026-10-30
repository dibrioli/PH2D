//! ⭐ **A caixa do cartão (`View::Toggle`) no canal: 0 = desligada, 1 = ligada** (v25). Até à v24 o
//! canal guardava o índice da opção pela ordem da declaração (`Toggle(ligado, desligado)`), então a
//! chave de uma caixa MARCADA era `0` e a timeline escrevia «Off». A migração v24 → v25 troca
//! `v → 1 − v` em cada chave, no `default` e no `rest` das trilhas de caixa — o estado da caixa em
//! cada chave não muda (fixtura `timeline_doc_v24.bin`, gravada pelo código de `c531ef677c`, ANTES
//! da cura, por um teste `#[ignore]` de uma corrida só, apagado a seguir: as três trilhas no dono
//! `wire_id` 44 — a caixa do Image, a caixa de um item da vigia e o tamanho do 9-slice — com chaves
//! em 0 s, 1 s e 2 s: caixa marcada, desmarcada, marcada; `rest` = a caixa marcada).

use ph2d_anim::AnimValue;
use ph2d_ecs::canal_componente as cc;
use ph2d_ecs::{FieldKey, ItemId};
use ph2d_timeline::{DOC_VERSION, PropKind, TimelineDoc};

/// O «9-slice» do Image (`Toggle(Sliced, Simple)`, componente).
fn caixa_imagem() -> FieldKey {
    cc::chaves_do_rotulo("component.field.slice_nine.8")[0]
}

/// O «só este objecto» de um item da vigia (`Toggle(Own, World)`, lista; código 5 = `VIGIA`).
fn caixa_lista() -> FieldKey {
    ph2d_ecs::canal_lista::chave(5, 6, 0, ItemId(9)).expect("o campo 6 da vigia")
}

/// Um contínuo do mesmo componente (o controlo: não pode mudar).
fn controlo() -> FieldKey {
    cc::chaves_do_rotulo("component.field.slice_nine.7")[0]
}

/// O estado da caixa em cada chave: marcada?
const MARCADA: [bool; 3] = [true, false, true];

fn chaves(doc: &TimelineDoc, k: FieldKey) -> (Vec<f32>, Option<f32>) {
    let b = (doc.bindings().iter())
        .find(|b| b.prop == PropKind::Field(k))
        .expect("o binding");
    let t = (doc.active_clip().tracks().iter())
        .find(|(t, _)| *t == b.target)
        .map(|(_, t)| t)
        .expect("a trilha");
    let v = (t.keys().iter())
        .map(|k| match k.value {
            AnimValue::Float(v) => v,
            o => panic!("{o:?}"),
        })
        .collect();
    (v, b.rest)
}

/// O estado da caixa que o valor `x` do canal escreve no campo: o cartão marca a 1.ª opção.
fn marcada(k: FieldKey, x: f32) -> bool {
    let (f, ligado, desligado) = match cc::resolver(k) {
        Some((_, f, _)) => (f, "Sliced", "Simple"),
        None => (
            ph2d_ecs::canal_lista::resolver(k).expect("o campo").1,
            "Own",
            "World",
        ),
    };
    let mut v = serde_json::Value::String(desligado.to_owned());
    cc::escrever_valor(f, 0, &mut v, x);
    v == serde_json::Value::String(ligado.to_owned())
}

/// Gate 1: uma chave numa caixa MARCADA vale 1 e diz «Ligado».
#[test]
fn a_caixa_marcada_e_1_e_diz_ligado() {
    for k in [caixa_imagem(), caixa_lista()] {
        assert!(marcada(k, 1.0), "{k:?}: 1 = marcada");
        assert!(!marcada(k, 0.0), "{k:?}: 0 = desmarcada");
        assert_eq!(
            cc::chave_da_opcao(k, 1),
            Some(ph2d_ecs::canal_palavra::LIGADO)
        );
        assert_eq!(
            cc::chave_da_opcao(k, 0),
            Some(ph2d_ecs::canal_palavra::DESLIGADO)
        );
    }
}

/// Gate 2: a v24 gravada antes da cura abre com a caixa no MESMO estado em cada chave.
#[test]
fn a_v24_abre_com_a_caixa_no_mesmo_estado() {
    const V24: &[u8] = include_bytes!("../fixtures/timeline_doc_v24.bin");
    let doc = TimelineDoc::from_bytes(V24).expect("a v24 abre");
    assert_eq!(doc.version, DOC_VERSION);
    for k in [caixa_imagem(), caixa_lista()] {
        let (v, rest) = chaves(&doc, k);
        let lido: Vec<bool> = v.iter().map(|&x| marcada(k, x)).collect();
        assert_eq!(lido, MARCADA, "{k:?}: {v:?}");
        assert_eq!(v, [1.0, 0.0, 1.0], "{k:?}: marcada = 1");
        assert_eq!(rest, Some(1.0), "{k:?}: o rest era a caixa marcada");
    }
    assert_eq!(
        chaves(&doc, controlo()),
        (vec![10.0, 11.0, 12.0], Some(3.0)),
        "o contínuo não muda"
    );
    // E a v25 vai e volta sem migrar outra vez.
    let lido = TimelineDoc::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert_eq!(chaves(&lido, caixa_imagem()).0, [1.0, 0.0, 1.0]);
}
