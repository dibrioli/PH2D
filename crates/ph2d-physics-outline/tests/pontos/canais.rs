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
/// 180°: a meio colapsa num ponto), o quadro dá os anéis da chave ANTERIOR (`anterior`).
/// *Mutação: escrever sem a guarda de `validos` ⇒ vermelho.*
#[test]
fn o_anel_interpolado_que_se_cruza_da_a_chave_anterior() {
    let q = quadrado();
    let rodado = vec![vec![[160, 160], [0, 160], [0, 0], [160, 0]]];
    let a = publicados(&contorno(q.clone(), true));
    let b = publicados(&contorno(rodado.clone(), true));
    let na_a = |kk: FieldKey| valor(&q, kk);
    let quarto = escritos(&q, &entre(&a, &b, 0.25), na_a).expect("a um quarto o anel é válido");
    assert_eq!(quarto[0][0], [40, 40], "a curva leva o ponto");
    let meio = escritos(&quarto, &entre(&a, &b, 0.5), na_a);
    assert_eq!(meio, Some(q.clone()), "a meio, colapsa: a chave anterior");
    assert_eq!(
        escritos(&q, &b, na_a),
        Some(rodado),
        "a chave B chega inteira"
    );
    assert_eq!(escritos(&q, &a, na_a), None, "nada mudou");
}

/// ⭐ Determinístico: o quadro inválido dá o MESMO a tocar (passando pelos quadros do meio) e a saltar
/// para ele. *Mutação: segurar o que o mundo tem ⇒ vermelho.*
#[test]
fn tocar_e_saltar_dao_o_mesmo_quadro() {
    let q = quadrado();
    let rodado = vec![vec![[160, 160], [0, 160], [0, 0], [160, 0]]];
    let a = publicados(&contorno(q.clone(), true));
    let b = publicados(&contorno(rodado, true));
    let na_a = |kk: FieldKey| valor(&q, kk);
    let mut tocando = q.clone();
    for s in [0.1, 0.2, 0.3, 0.4, 0.5] {
        if let Some(r) = escritos(&tocando, &entre(&a, &b, s), na_a) {
            tocando = r;
        }
    }
    let saltando = escritos(&q, &entre(&a, &b, 0.5), na_a).unwrap_or_else(|| q.clone());
    assert_eq!(tocando, saltando, "o mesmo quadro, por dois caminhos");
    assert_eq!(saltando, q);
}

/// ⭐ O Fit (custom → traçado) apaga a trilha dos pontos e deixa o resto; antes disso, a trilha dos
/// pontos atravessa o save/load do documento da timeline. *Mutação: `apagar_do_fit` sem `unbind` ⇒ vermelho.*
#[test]
fn o_fit_apaga_a_trilha_dos_pontos_e_o_save_a_leva() {
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    use ph2d_physics_outline::canais_pontos::apagar_do_fit;
    use ph2d_timeline::{PropKind, TimelineDoc};
    let forma = 7;
    let detalhe = FieldKey::new(ph2d_ecs::canal::familia::FISICA, f::CONTORNO_DETALHE, 0, 0);
    let mut antes: std::collections::BTreeMap<_, _> = publicados(&contorno(quadrado(), true))
        .into_iter()
        .collect();
    antes.insert(detalhe, 2.0);
    let mut doc = TimelineDoc::new();
    for (&kk, &v) in &antes {
        let (p, z) = (PropKind::Field(kk), RationalTime::ZERO);
        doc.upsert_key(forma, p, z, AnimValue::Float(v), Interp::Linear);
    }
    let x00 = PropKind::Field(FieldKey::new(
        ph2d_ecs::canal::familia::FISICA,
        f::CONTORNO_PONTO_X,
        0,
        0,
    ));
    let ida = TimelineDoc::from_bytes(&doc.to_bytes().expect("grava")).expect("lê");
    assert!(
        ida.bindings().iter().any(|b| b.prop == x00),
        "o save leva os pontos"
    );
    assert_eq!(ida.bindings().len(), 9);
    assert!(!apagar_do_fit(
        &mut doc,
        forma,
        &antes,
        &antes.clone().into_iter().collect::<Vec<_>>()
    ));
    assert!(
        apagar_do_fit(&mut doc, forma, &antes, &[(detalhe, 2.0)]),
        "o Fit"
    );
    assert!(
        doc.binding_for(forma, x00).is_none(),
        "a trilha dos pontos sai"
    );
    assert!(
        doc.binding_for(forma, PropKind::Field(detalhe)).is_some(),
        "o Detail fica"
    );
}

/// A chave ANTERIOR a `t` (a guarda determinística): entre 0 s e 1 s vale a de 0 s; antes da 1.ª, nada.
/// *Mutação: a chave mais PRÓXIMA ⇒ vermelho.*
#[test]
fn a_chave_anterior_e_a_de_antes_do_instante() {
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    use ph2d_physics_outline::canais_pontos::na_chave_anterior;
    let x = FieldKey::new(ph2d_ecs::canal::familia::FISICA, f::CONTORNO_PONTO_X, 0, 0);
    let mut doc = ph2d_timeline::TimelineDoc::new();
    for (t, v) in [(0.0, 1.0), (1.0, 3.0)] {
        let (p, t) = (
            ph2d_timeline::PropKind::Field(x),
            RationalTime::from_seconds(t),
        );
        doc.upsert_key(7, p, t, AnimValue::Float(v), Interp::Linear);
    }
    assert_eq!(na_chave_anterior(&doc, 7, x, 0.9), Some(1.0));
    assert_eq!(na_chave_anterior(&doc, 7, x, 1.0), Some(3.0));
    assert_eq!(na_chave_anterior(&doc, 7, x, -0.5), None);
}
