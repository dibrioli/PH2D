//! ⭐ **A caixa do cartão (`View::Toggle`) no canal: 0 = desligada, 1 = ligada** (v25): a chave de
//! uma caixa MARCADA é `1`, e a timeline escreve «On».

use ph2d_ecs::canal_componente as cc;
use ph2d_ecs::{FieldKey, ItemId};

/// O «9-slice» do Image (`Toggle(Sliced, Simple)`, componente).
fn caixa_imagem() -> FieldKey {
    cc::chaves_do_rotulo("component.field.slice_nine.8")[0]
}

/// O «só este objecto» de um item da vigia (`Toggle(Own, World)`, lista; código 5 = `VIGIA`).
fn caixa_lista() -> FieldKey {
    ph2d_ecs::canal_lista::chave(5, 6, 0, ItemId(9)).expect("o campo 6 da vigia")
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
