//! (Auto Key dos pontos, fase 1) Os pontos de um «Outline» à mão são canais FISICA 51/52 (`i` = anel,
//! `j` = ponto): o leitor só publica o `custom`; o escritor segura o anel que a interpolação cruzaria.

use ph2d_ecs::FieldKey;
use ph2d_ecs::canal_fisica as f;
use ph2d_physics_ecs::ColliderOutline;
use ph2d_physics_outline::canais_pontos::{escritos, publicados, valor};
use std::sync::Arc;

fn quadrado() -> Vec<Vec<[i32; 2]>> {
    vec![vec![[0, 0], [160, 0], [160, 160], [0, 160]]]
}

fn contorno(rings: Vec<Vec<[i32; 2]>>, custom: bool) -> ColliderOutline {
    ColliderOutline {
        rings: Arc::from(rings),
        custom,
        ..ColliderOutline::default()
    }
}

/// Entre a chave `a` e a `b`, a fracção `s` de cada canal (a interpolação linear da timeline).
fn entre(a: &[(FieldKey, f32)], b: &[(FieldKey, f32)], s: f32) -> Vec<(FieldKey, f32)> {
    a.iter()
        .zip(b)
        .map(|(&(k, x), &(_, y))| (k, x + (y - x) * s))
        .collect()
}

/// ⭐ Um Outline à mão publica TODOS os pontos (X e Y de cada anel), num grupo; o traçado da imagem
/// não publica nenhum. *Mutação: publicar com `custom = false` ⇒ vermelho.*
#[test]
fn so_o_outline_a_mao_publica_os_pontos() {
    let mao = publicados(&contorno(quadrado(), true));
    assert_eq!(mao.len(), 8, "4 pontos × (X, Y)");
    assert!(
        mao.iter().all(|(k, _)| k.mesmo_grupo(mao[0].0)),
        "uma chave"
    );
    let y2 = FieldKey::new(ph2d_ecs::canal::familia::FISICA, f::CONTORNO_PONTO_Y, 0, 2);
    assert_eq!(mao.iter().find(|(k, _)| *k == y2).map(|p| p.1), Some(160.0));
    assert_eq!(valor(&quadrado(), y2), Some(160.0));
    assert!(
        publicados(&contorno(quadrado(), false)).is_empty(),
        "o traçado é derivado: as chaves dele são Detail/Alpha/Specks"
    );
}

/// ⭐ A curva leva os anéis; a meio de duas chaves válidas cujo caminho se cruza (o quadrado rodado
/// 180°: a meio colapsa num ponto), o quadro SEGURA o que tem. *Mutação: escrever sem a guarda de
/// `validos` ⇒ vermelho.*
#[test]
fn o_anel_interpolado_que_se_cruza_segura_o_que_tem() {
    let q = quadrado();
    let rodado = vec![vec![[160, 160], [0, 160], [0, 0], [160, 0]]];
    let a = publicados(&contorno(q.clone(), true));
    let b = publicados(&contorno(rodado.clone(), true));
    let quarto = escritos(&q, &entre(&a, &b, 0.25)).expect("a um quarto o anel é válido");
    assert_eq!(quarto[0][0], [40, 40], "a curva leva o ponto");
    assert_eq!(
        escritos(&q, &entre(&a, &b, 0.5)),
        None,
        "a meio, colapsa: segura"
    );
    assert_eq!(escritos(&q, &b), Some(rodado), "a chave B chega inteira");
    assert_eq!(escritos(&q, &a), None, "nada mudou");
}
