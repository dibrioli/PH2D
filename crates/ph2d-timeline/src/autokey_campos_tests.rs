use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::canal::familia;
use ph2d_ecs::canal_vetor as v;

const E: u64 = 7;

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(familia::VETOR, campo, i, j)
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
        (k(v::TINTA_PREENCH + v::OFF_SOLIDA, 1, 0), 9.0),
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

/// A tinta parte-se por entrada da pilha: mudar o vermelho da entrada 0 não grava a cor de base.
#[test]
fn a_cor_grava_inteira_mas_so_a_da_sua_entrada() {
    let antes = campos();
    let mut l = antes.clone();
    mudar(&mut l, k(v::TINTA_PREENCH + v::OFF_SOLIDA, 1, 0), 10.0);
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
        [k(v::TINTA_PREENCH + v::OFF_SOLIDA, 1, 0)].into()
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
