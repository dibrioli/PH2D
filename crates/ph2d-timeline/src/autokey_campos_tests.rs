use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::canal::familia;
use ph2d_ecs::canal_vetor as v;

const E: u64 = 7;

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(familia::VETOR, campo, i, j)
}

/// A chave da entrada da pilha com o id `1` (F31: o endereço é o id, não a posição).
fn entrada(campo: u16) -> FieldKey {
    FieldKey::do_item(familia::VETOR, campo, ph2d_ecs::ItemId(1), 0)
}

/// Dois vértices de uma forma (âncoras e um canto), a largura do traço e a cor de base e da
/// entrada 0 da pilha — o bastante para ver quem entra no grupo e quem fica de fora.
fn campos() -> Vec<(FieldKey, f32)> {
    vec![
        (k(v::ANCORA_X, 0, 0), 0.0),
        (k(v::ANCORA_Y, 0, 0), 0.0),
        (k(v::ANCORA_X, 0, 1), 10.0),
        (k(v::CANTO, 0, 1), 2.0),
        (k(v::TRACO_LARGURA, 0, 0), 4.0),
        (k(v::TINTA_PREENCH + v::OFF_SOLIDA, 0, 0), 255.0),
        (k(v::TINTA_PREENCH + v::OFF_SOLIDA + 1, 0, 0), 0.0),
        (entrada(v::TINTA_PREENCH + v::OFF_SOLIDA), 9.0),
    ]
}

fn base_de(l: &[(FieldKey, f32)]) -> BTreeMap<FieldKey, f32> {
    l.iter().copied().collect()
}

fn mudar(l: &mut [(FieldKey, f32)], kk: FieldKey, x: f32) {
    l.iter_mut().find(|(c, _)| *c == kk).unwrap().1 = x;
}

fn chaves(plan: &AutokeyPlan) -> BTreeSet<FieldKey> {
    plan.keys
        .iter()
        .map(|(p, _)| match p {
            PropKind::Field(k) => *k,
            outro => panic!("{outro:?} não é campo"),
        })
        .collect()
}

#[test]
fn nada_mudou_nada_grava() {
    let l = campos();
    let plan = autokey_campos(&TimelineDoc::new(), E, 0.0, &l, &base_de(&l), true, false);
    assert!(plan.is_empty());
}

/// ⭐ Mexer UM ponto de uma forma sem animação grava a FORMA INTEIRA (decisão do dono) — e nada mais.
#[test]
fn mexer_um_ponto_grava_a_forma_inteira_e_so_ela() {
    let antes = campos();
    let mut l = antes.clone();
    mudar(&mut l, k(v::ANCORA_X, 0, 0), 5.0);
    let plan = autokey_campos(
        &TimelineDoc::new(),
        E,
        0.0,
        &l,
        &base_de(&antes),
        true,
        false,
    );
    let quer: BTreeSet<FieldKey> = [
        k(v::ANCORA_X, 0, 0),
        k(v::ANCORA_Y, 0, 0),
        k(v::ANCORA_X, 0, 1),
        k(v::CANTO, 0, 1),
    ]
    .into();
    assert_eq!(chaves(&plan), quer);
    let x0 = plan
        .keys
        .iter()
        .find(|(p, _)| *p == PropKind::Field(k(v::ANCORA_X, 0, 0)));
    assert_eq!(x0.map(|c| c.1), Some(5.0), "o valor da mão");
}

/// A tinta parte-se por entrada da pilha: mudar o vermelho da entrada não grava a cor de base.
#[test]
fn a_cor_grava_inteira_mas_so_a_da_sua_entrada() {
    let antes = campos();
    let mut l = antes.clone();
    mudar(&mut l, entrada(v::TINTA_PREENCH + v::OFF_SOLIDA), 10.0);
    let plan = autokey_campos(
        &TimelineDoc::new(),
        E,
        0.0,
        &l,
        &base_de(&antes),
        true,
        false,
    );
    assert_eq!(
        chaves(&plan),
        [entrada(v::TINTA_PREENCH + v::OFF_SOLIDA)].into()
    );

    let mut l = antes.clone();
    mudar(&mut l, k(v::TINTA_PREENCH + v::OFF_SOLIDA + 1, 0, 0), 30.0);
    let plan = autokey_campos(
        &TimelineDoc::new(),
        E,
        0.0,
        &l,
        &base_de(&antes),
        true,
        false,
    );
    assert_eq!(
        chaves(&plan),
        [
            k(v::TINTA_PREENCH + v::OFF_SOLIDA, 0, 0),
            k(v::TINTA_PREENCH + v::OFF_SOLIDA + 1, 0, 0)
        ]
        .into(),
        "o verde mudou ⇒ a cor de base inteira (os quatro canais que existem)"
    );
}

/// Uma forma animada, o relógio parado a meio e o documento NA curva: nada grava (o F23 D2 para
/// todo campo). Mexer um membro grava o grupo inteiro, cada um no valor que mostra.
#[test]
fn na_curva_nao_grava_e_fora_dela_grava_o_grupo() {
    let mut doc = TimelineDoc::new();
    let s = RationalTime::from_seconds;
    for (kk, a, b) in [
        (k(v::ANCORA_X, 0, 0), 0.0, 10.0),
        (k(v::ANCORA_Y, 0, 0), 0.0, 20.0),
    ] {
        let p = PropKind::Field(kk);
        doc.insert_key(E, p, s(0.0), AnimValue::Float(a), Interp::Linear);
        doc.insert_key(E, p, s(1.0), AnimValue::Float(b), Interp::Linear);
    }
    let na_curva = vec![(k(v::ANCORA_X, 0, 0), 5.0), (k(v::ANCORA_Y, 0, 0), 10.0)];
    let base = base_de(&na_curva);
    assert!(autokey_campos(&doc, E, 0.5, &na_curva, &base, true, true).is_empty());
    assert!(autokey_campos(&doc, E, 0.5, &na_curva, &base, true, false).is_empty());

    let mao = vec![(k(v::ANCORA_X, 0, 0), 7.0), (k(v::ANCORA_Y, 0, 0), 10.0)];
    let plan = autokey_campos(&doc, E, 0.5, &mao, &base, true, true);
    assert_eq!(
        plan.keys,
        vec![
            (PropKind::Field(k(v::ANCORA_X, 0, 0)), 7.0),
            (PropKind::Field(k(v::ANCORA_Y, 0, 0)), 10.0)
        ]
    );
}

/// Desarmado (`allow_create = false`), um campo sem curva não «saiu da curva»: não há curva.
#[test]
fn sem_curva_e_sem_criar_nao_ha_diferenca() {
    let antes = campos();
    let mut l = antes.clone();
    mudar(&mut l, k(v::TRACO_LARGURA, 0, 0), 12.0);
    assert!(
        autokey_campos(
            &TimelineDoc::new(),
            E,
            0.0,
            &l,
            &base_de(&antes),
            false,
            false
        )
        .is_empty()
    );
}

/// ⭐ F30 §B: a gravidade é da CENA — com um clip da Cena activo a chave nasce; com o clip de um
/// objecto activo ela é RECUSADA (com o motivo), nunca gravada no clip do objecto.
#[test]
fn a_chave_da_cena_cai_num_clip_da_cena() {
    use ph2d_ecs::canal_mundo as m;
    const CENA: u64 = 3;
    let g = |c| FieldKey::new(familia::MUNDO, c, 0, 0);
    let antes = vec![(g(m::GRAVIDADE_X), 0.0), (g(m::GRAVIDADE_Y), -9.81)];
    let mut l = antes.clone();
    mudar(&mut l, g(m::GRAVIDADE_Y), 9.81);
    let mut doc = TimelineDoc::new();
    assert!(doc.clip_belongs_to(doc.active_index(), crate::doc::SCENE));
    let na_cena = autokey_campos(&doc, CENA, 0.0, &l, &base_de(&antes), true, false);
    assert_eq!(
        chaves(&na_cena),
        [g(m::GRAVIDADE_X), g(m::GRAVIDADE_Y)].into()
    );
    assert_eq!(na_cena.scene_refused, None);
    // A animação de um objecto em vista: o clip activo é dele.
    let i = doc
        .add_clip_for("Heroi".into(), E)
        .expect("clip do objecto");
    doc.set_active(i);
    let no_objecto = autokey_campos(&doc, CENA, 0.0, &l, &base_de(&antes), true, false);
    assert!(no_objecto.keys.is_empty(), "nada no clip do objecto");
    assert_eq!(
        no_objecto.scene_refused,
        Some(crate::KeyRefusal::SceneNotInView)
    );
    // O campo de um objecto continua a gravar no clip dele.
    let mut o = campos();
    let base = base_de(&o);
    mudar(&mut o, k(v::TRACO_LARGURA, 0, 0), 8.0);
    let obj = autokey_campos(&doc, E, 0.0, &o, &base, true, false);
    assert_eq!(chaves(&obj), [k(v::TRACO_LARGURA, 0, 0)].into());
    assert_eq!(obj.scene_refused, None);
}

/// ⭐⭐ **Mexer a duração de UM timer grava só esse timer** (smoke do dono 25/10, F30 §A1): o grupo
/// de um item é o `i:j` inteiro (`FieldKey::grupo`), e os ids pequenos têm todos `i = 0` — um
/// grupo lido só pelo `i` gravava o vizinho também.
#[test]
fn editar_um_timer_nao_grava_a_chave_do_vizinho() {
    // A duração (`campo` 1032 = TIMERS, field 2, faixa 0) dos timers com ids 4 e 5.
    let d = |j| FieldKey::new(familia::LISTA, 1032, 0, j);
    assert!(
        d(4).info().is_some(),
        "a chave tem de ser de uma lista conhecida"
    );
    let antes = vec![(d(4), 1.0e6), (d(5), 3.0e6)];
    let mut l = antes.clone();
    mudar(&mut l, d(5), 2.0e6);
    let plan = autokey_campos(
        &TimelineDoc::new(),
        E,
        0.0,
        &l,
        &base_de(&antes),
        true,
        false,
    );
    assert_eq!(chaves(&plan), BTreeSet::from([d(5)]));
}
