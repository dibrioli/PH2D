//! Os gates da família COMPONENTE: a chave é estável, não colide, e ida e volta pelo catálogo.

use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// ⛔ **Dois componentes com os mesmos 32 bits seriam o MESMO canal** — gravado num ficheiro, uma
/// curva da câmera a escrever na vida. O catálogo inteiro, sem colisão.
#[test]
fn nenhum_componente_partilha_os_32_bits() {
    let mut vistos = BTreeSet::new();
    let mut n = 0;
    for d in ph2d_component_desc::all() {
        n += 1;
        assert!(
            vistos.insert(id32(d.canonical_name)),
            "{} colide nos 32 bits altos do stable_type_id",
            d.canonical_name
        );
    }
    assert!(
        n > 100,
        "controlo: o catálogo tem {n} componentes — a varredura mediu o nada"
    );
}

/// O `field_id` cabe nos 14 bits do `campo`, e é único dentro do componente.
#[test]
fn todo_field_id_cabe_e_e_unico() {
    for d in ph2d_component_desc::all() {
        let mut ids = BTreeSet::new();
        for f in d.fields {
            assert!(
                f.field_id <= MAX_FIELD_ID,
                "{}.{}",
                d.canonical_name,
                f.field_id
            );
            assert!(
                ids.insert(f.field_id),
                "{} repete o field_id {}",
                d.canonical_name,
                f.field_id
            );
        }
    }
}

/// ⭐ **Ida e volta**: toda chave de todo campo animável resolve para o mesmo componente, campo e
/// faixa; tem rótulo e membro; e as faixas de um campo são UM grupo (um losango), distinto de
/// todos os outros.
#[test]
fn toda_chave_resolve_para_o_seu_campo() {
    let mut grupos = BTreeSet::new();
    let mut canais = 0;
    for d in ph2d_component_desc::all() {
        for f in d.fields {
            let mut do_campo = BTreeSet::new();
            for faixa in 0..faixas(f) {
                let k = chave(d.canonical_name, f.field_id, faixa).expect("cabe");
                let (d2, f2, faixa2) = resolver(k).expect("resolve");
                assert_eq!(
                    (d2.canonical_name, f2.field_id, faixa2),
                    (d.canonical_name, f.field_id, faixa)
                );
                let i = info(k).expect("info");
                assert_eq!(i.rotulo, f.label_key);
                assert!(membro(k).is_some());
                assert_eq!(
                    k.grupo().map(|g| g.2),
                    Some(0),
                    "por_i: o rótulo ganharia «i»"
                );
                do_campo.insert(k.grupo());
                canais += 1;
            }
            if faixas(f) > 0 {
                assert_eq!(
                    do_campo.len(),
                    1,
                    "{}.{}: faixas em grupos diferentes",
                    d.canonical_name,
                    f.field_id
                );
                assert!(grupos.insert(do_campo.into_iter().next()), "grupo repetido");
            }
        }
    }
    assert!(
        canais > 300,
        "controlo: {canais} canais — o catálogo encolheu ou a regra recusa tudo"
    );
}

/// Uma faixa que o campo não tem, ou outra família, não resolve.
#[test]
fn o_que_nao_existe_nao_resolve() {
    let d = ph2d_component_desc::all()
        .find(|d| d.fields.iter().any(|f| faixas(f) == 1))
        .expect("um campo escalar");
    let f = d.fields.iter().find(|f| faixas(f) == 1).expect("acima");
    assert!(resolver(chave(d.canonical_name, f.field_id, 1).expect("cabe")).is_none());
    let k = chave(d.canonical_name, f.field_id, 0).expect("cabe");
    assert!(resolver(FieldKey { familia: 2, ..k }).is_none());
    assert!(chave(d.canonical_name, MAX_FIELD_ID + 1, 0).is_none());
}

fn campo(kind: FieldKind, view: View) -> FieldDesc {
    FieldDesc {
        field_id: 0,
        label_key: "x",
        kind,
        policy: ph2d_component_desc::Propagation::Propagate,
        is_ref: None,
        name: "x",
        key: "",
        view,
    }
}

/// A FORMA do valor: um número, uma faixa de array ou de objecto, um booleano, um inteiro, uma
/// opção — lida e escrita pela mesma lei.
#[test]
fn a_forma_do_valor_ida_e_volta() {
    let esc = campo(FieldKind::Scalar, View::Same);
    let mut v = serde_json::json!(1.5);
    assert_eq!(ler_valor(&esc, 0, &v), Some(1.5));
    assert!(escrever_valor(&esc, 0, &mut v, 2.25));
    assert_eq!(ler_valor(&esc, 0, &v), Some(2.25));
    assert!(
        !escrever_valor(&esc, 0, &mut v, 2.25),
        "o mesmo valor não é escrita"
    );

    let par = campo(FieldKind::Vec2, View::Same);
    let mut v = serde_json::json!([1.0, 2.0]);
    assert!(escrever_valor(&par, 1, &mut v, 5.0));
    assert_eq!(v, serde_json::json!([1.0, 5.0]));
    let mut o = serde_json::json!({"x": 1.0, "y": 2.0});
    assert_eq!(ler_valor(&par, 1, &o), Some(2.0));
    assert!(escrever_valor(&par, 0, &mut o, 3.0));
    assert_eq!(o, serde_json::json!({"x": 3.0, "y": 2.0}));

    let liga = campo(FieldKind::Toggle, View::Same);
    let mut b = serde_json::json!(false);
    assert!(escrever_valor(&liga, 0, &mut b, 0.8));
    assert_eq!(b, serde_json::json!(true));

    let int = campo(FieldKind::Int, View::Same);
    let mut n = serde_json::json!(3);
    assert!(escrever_valor(&int, 0, &mut n, -1.6));
    assert_eq!(n, serde_json::json!(-2), "inteiro com sinal, arredondado");

    let opc = campo(
        FieldKind::Enum,
        View::Options(&[("full", "Full"), ("none", "Nothing"), ("inherit", "null")]),
    );
    let mut e = serde_json::json!("Nothing");
    assert_eq!(ler_valor(&opc, 0, &e), Some(1.0));
    assert!(escrever_valor(&opc, 0, &mut e, 2.2));
    assert_eq!(e, Value::Null);
    assert!(escrever_valor(&opc, 0, &mut e, 0.0));
    assert!(escrever_valor(&opc, 0, &mut e, 9.0));
    assert_eq!(e, Value::Null, "preso à última opção");
    assert!(!escrever_valor(
        &esc,
        0,
        &mut serde_json::json!(1.0),
        f32::NAN
    ));
}

/// ⛔ **Um rótulo nomeia UM conceito** — o Inspector acha o canal pelo rótulo que pinta (F27 B2).
/// Partilhado só entre campos do MESMO tipo e faixas (os `LIMITS_FIELDS`), e as chaves dele são as
/// de todos; os partilhados contam-se (um novo par é uma decisão, não um acaso).
#[test]
fn cada_rotulo_nomeia_um_conceito_so() {
    let mut vistos: BTreeMap<&str, (FieldKind, u16, usize)> = BTreeMap::new();
    let mut n = 0;
    for d in ph2d_component_desc::all() {
        for f in d.fields {
            n += 1;
            let e = vistos.entry(f.label_key).or_insert((f.kind, faixas(f), 0));
            assert_eq!((e.0, e.1), (f.kind, faixas(f)), "{} · {}", d.canonical_name, f.label_key);
            e.2 += 1;
            assert!(campo_do_rotulo(f.label_key).is_some(), "{}", f.label_key);
        }
    }
    for (r, &(_, fx, quantos)) in &vistos {
        let ks = chaves_do_rotulo(r);
        assert_eq!(ks.len(), usize::from(fx) * quantos, "{r}");
        assert!(ks.iter().all(|&k| resolver(k).is_some_and(|(_, g, _)| g.label_key == *r)));
    }
    let partilhados: Vec<_> = (vistos.iter())
        .filter(|(_, v)| v.2 > 1 && v.1 > 0)
        .map(|(r, _)| *r)
        .collect();
    assert_eq!(
        partilhados,
        ["component.field.limits_fields.0", "component.field.limits_fields.1"],
        "rótulos de canal partilhados entre componentes"
    );
    assert!(n > 300, "controlo: {n} campos — a varredura mediu o nada");
}

/// ⭐ **O «Visible» é o canal do que o artista vê** (F27 B2): o campo guarda `hidden` (`View::Not`),
/// e a curva lê e escreve `1 = visível` — senão o gráfico diria o contrário do cartão.
#[test]
fn o_visible_le_e_escreve_o_que_se_ve() {
    let d = ph2d_component_desc::all()
        .find(|d| d.fields.iter().any(|f| f.label_key == "component.field.visibility.1"))
        .expect("Visibility");
    let f = &d.fields[0];
    assert_eq!(faixas(f), 1);
    assert_eq!(ler_valor(f, 0, &Value::Bool(false)), Some(1.0), "não escondido = visível");
    let mut v = Value::Bool(false);
    assert!(escrever_valor(f, 0, &mut v, 0.0));
    assert_eq!(v, Value::Bool(true), "0 = escondido");
}
