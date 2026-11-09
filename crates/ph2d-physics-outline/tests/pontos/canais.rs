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

/// Uma forma com os pontos chaveados em 0 s (`a`) e 1 s (`b`).
fn duas_chaves(a: &[Vec<[i32; 2]>], b: &[Vec<[i32; 2]>]) -> ph2d_timeline::TimelineDoc {
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    let mut doc = ph2d_timeline::TimelineDoc::new();
    for (t, r) in [(0.0, a), (1.0, b)] {
        for (kk, v) in publicados(&contorno(r.to_vec(), true)) {
            let (p, t) = (
                ph2d_timeline::PropKind::Field(kk),
                RationalTime::from_seconds(t),
            );
            doc.upsert_key(7, p, t, AnimValue::Float(v), Interp::Linear);
        }
    }
    doc
}

/// Os anéis de `doc` na chave `t` (a forma 7, `n` pontos no anel 0).
fn na_chave(doc: &ph2d_timeline::TimelineDoc, t: f64, n: usize) -> Vec<[i32; 2]> {
    let v = |c, j| {
        let kk = FieldKey::new(ph2d_ecs::canal::familia::FISICA, c, 0, j);
        ph2d_physics_outline::canais_pontos::na_chave_anterior(doc, 7, kk, t).expect("chave") as i32
    };
    (0..n as u16)
        .map(|j| [v(f::CONTORNO_PONTO_X, j), v(f::CONTORNO_PONTO_Y, j)])
        .collect()
}

/// ⭐ Acrescentar um ponto (duplo clique na aresta) numa forma com DUAS chaves: as duas ganham o ponto,
/// no mesmo lugar do segmento (na chave de 1 s, o meio da aresta deslocada); apagar um ponto sai das
/// duas, e a trilha dele também; tocar entre as chaves continua válido. Lei do dono 19/10.
/// *Mutação: propagar só à chave do instante ⇒ vermelho.*
#[test]
fn acrescentar_e_apagar_um_ponto_vale_nas_duas_chaves() {
    use ph2d_physics_outline::canais_pontos::{mudou, propagar};
    let q = quadrado();
    let longe: Vec<Vec<[i32; 2]>> = vec![q[0].iter().map(|p| [p[0] + 32, p[1]]).collect()];
    let mut doc = duas_chaves(&q, &longe);
    let antes: std::collections::BTreeMap<_, _> =
        publicados(&contorno(q.clone(), true)).into_iter().collect();
    let mais = vec![vec![[0, 0], [80, 0], [160, 0], [160, 160], [0, 160]]];
    let agora = publicados(&contorno(mais.clone(), true));
    assert!(mudou(&antes, &agora));
    assert!(propagar(&mut doc, 7, &antes, &agora));
    assert_eq!(
        na_chave(&doc, 0.0, 5),
        mais[0],
        "a chave de 0 s ganha o ponto"
    );
    assert_eq!(
        na_chave(&doc, 1.0, 5),
        vec![[32, 0], [112, 0], [192, 0], [192, 160], [32, 160]],
        "a de 1 s também, no meio da aresta dela"
    );
    let meio = sample_em(&doc, 0.5, 5);
    assert!(
        escritos(&mais, &meio, |_| None).is_some(),
        "tocar a meio: um anel válido"
    );

    let antes: std::collections::BTreeMap<_, _> = agora.into_iter().collect();
    let menos = vec![vec![[0, 0], [80, 0], [160, 160], [0, 160]]];
    let agora = publicados(&contorno(menos.clone(), true));
    assert!(propagar(&mut doc, 7, &antes, &agora), "apagar");
    assert_eq!(na_chave(&doc, 0.0, 4), menos[0]);
    assert_eq!(
        na_chave(&doc, 1.0, 4),
        vec![[32, 0], [112, 0], [192, 160], [32, 160]]
    );
    let x4 = FieldKey::new(ph2d_ecs::canal::familia::FISICA, f::CONTORNO_PONTO_X, 0, 4);
    assert!(
        doc.binding_for(7, ph2d_timeline::PropKind::Field(x4))
            .is_none(),
        "a trilha do 5.º sai"
    );
}

/// Os canais dos pontos de `doc` amostrados em `t` (a curva a tocar).
fn sample_em(doc: &ph2d_timeline::TimelineDoc, t: f64, n: u16) -> Vec<(FieldKey, f32)> {
    let mut v = Vec::new();
    for j in 0..n {
        for c in [f::CONTORNO_PONTO_X, f::CONTORNO_PONTO_Y] {
            let kk = FieldKey::new(ph2d_ecs::canal::familia::FISICA, c, 0, j);
            let b = doc
                .binding_for(7, ph2d_timeline::PropKind::Field(kk))
                .expect("trilha");
            let tr = doc.active_clip().track(b.target).expect("trilha");
            if let ph2d_anim::AnimValue::Float(x) = ph2d_anim::AttributeEvaluator::sample(tr, t) {
                v.push((kk, x));
            }
        }
    }
    v
}

/// ⭐ Regra B (dono 09/11): «Delete animation» apaga TODAS as trilhas dos pontos da forma (só as dela) e deixa
/// o Detail; `com_trilha` dá as formas com pontos animados (o Auto Key decide só o mover, em `travados`).
/// *Mutação: `apagar_pontos` sem `unbind` ⇒ vermelho; `com_trilha` sem o filtro dos pontos ⇒ vermelho.*
#[test]
fn delete_animation_apaga_as_trilhas_dos_pontos_e_com_trilha_da_as_formas() {
    use ph2d_physics_outline::canais_pontos::{animados, apagar_pontos, com_trilha};
    use ph2d_timeline::PropKind;
    let mut doc = duas_chaves(&quadrado(), &quadrado());
    let detalhe = FieldKey::new(ph2d_ecs::canal::familia::FISICA, f::CONTORNO_DETALHE, 0, 0);
    doc.upsert_key(
        7,
        PropKind::Field(detalhe),
        ph2d_anim::RationalTime::ZERO,
        ph2d_anim::AnimValue::Float(2.0),
        ph2d_anim::Interp::Linear,
    );
    assert!(animados(&doc, 7) && !animados(&doc, 8));
    let so_o_detalhe = {
        let mut d = ph2d_timeline::TimelineDoc::new();
        d.upsert_key(
            8,
            PropKind::Field(detalhe),
            ph2d_anim::RationalTime::ZERO,
            ph2d_anim::AnimValue::Float(2.0),
            ph2d_anim::Interp::Linear,
        );
        d
    };
    assert_eq!(com_trilha(&doc), [7].into_iter().collect());
    assert!(com_trilha(&so_o_detalhe).is_empty(), "o Detail não é ponto");
    assert!(apagar_pontos(&mut doc, 7), "apagou");
    assert!(!animados(&doc, 7), "nenhuma trilha de ponto fica");
    assert!(
        doc.binding_for(7, PropKind::Field(detalhe)).is_some(),
        "o Detail fica"
    );
    assert!(
        !apagar_pontos(&mut doc, 7),
        "a segunda vez não tem o que apagar"
    );
}
