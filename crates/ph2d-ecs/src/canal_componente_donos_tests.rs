//! Os gates de [`super`] — a tabela `DONOS` aponta campos reais e canais da mesma espécie, e uma
//! faixa com dono deixa de ser canal do catálogo.

use std::collections::BTreeSet;

use ph2d_component_desc::{FieldKind, View};

use super::*;
use crate::canal::Especie;
use crate::canal_componente::{
    chave, chaves_do_rotulo, faixas, info, props_do_rotulo, resolver, todas,
};

const PROPS: [u64; 5] = [
    PROP_OPACIDADE,
    PROP_MOTOR_ALVO,
    PROP_MOTOR_TAXA,
    PROP_COMPRIMENTO_REPOUSO,
    PROP_COMPRIMENTO_MAX,
];

/// A espécie do dono serve o valor que o catálogo guarda: contínuo ↔ número real ou faixa de cor,
/// inteiro ↔ `Numero`, caixa ↔ `Ligado` ou duas opções, escolhas ↔ o MESMO número de opções.
fn compativel(kind: FieldKind, view: View, e: Especie) -> bool {
    match (kind, view) {
        (FieldKind::Enum, View::Options(o)) => {
            matches!(e, Especie::Opcoes(c) if c.len() == o.len())
        }
        (FieldKind::Toggle, _) => {
            matches!(e, Especie::Ligado) || matches!(e, Especie::Opcoes(c) if c.len() == 2)
        }
        (FieldKind::Int, _) => matches!(e, Especie::Numero),
        (
            FieldKind::Scalar
            | FieldKind::Angle
            | FieldKind::Color
            | FieldKind::Vec2
            | FieldKind::Vec4,
            _,
        ) => matches!(e, Especie::Continuo),
        _ => false,
    }
}

/// ⭐ Toda entrada aponta um campo/faixa do catálogo que existe e um canal que existe, da espécie
/// compatível; nenhuma faixa nem nenhum dono aparece duas vezes.
#[test]
fn todo_dono_aponta_um_campo_real_e_um_canal_compativel() {
    let mut faixas_vistas = BTreeSet::new();
    let mut donos_vistos = BTreeSet::new();
    for &(c, fid, x, d) in DONOS {
        let desc = ph2d_component_desc::desc_for(c).unwrap_or_else(|| panic!("{c} não existe"));
        let f = (desc.fields.iter())
            .find(|f| f.field_id == fid)
            .unwrap_or_else(|| panic!("{c}.{fid} não existe"));
        assert!(
            x < faixas(f),
            "{c}.{fid} faixa {x}: o campo tem {}",
            faixas(f)
        );
        assert!(faixas_vistas.insert((c, fid, x)), "{c}.{fid}.{x} repetida");
        assert!(donos_vistos.insert(d), "{d:?} dono de duas faixas");
        match d {
            Dono::Canal(k) => {
                let i = k.info().unwrap_or_else(|| panic!("{k:?} sem info"));
                assert!(
                    compativel(f.kind, f.view, i.especie),
                    "{c}.{fid}.{x} ({:?}/{:?}) ↔ {k:?} ({:?})",
                    f.kind,
                    f.view,
                    i.especie
                );
            }
            Dono::Prop(p) => {
                assert!(
                    PROPS.contains(&p),
                    "{c}.{fid}.{x}: PropKind {p} fora da lista"
                );
                assert!(
                    matches!(f.kind, FieldKind::Scalar | FieldKind::Color),
                    "{c}.{fid}: um PropKind é contínuo"
                );
            }
        }
    }
    // O censo (plano F27 F (d)): 27 do objecto, 28 da física, 26 da junta.
    assert_eq!(DONOS.len(), 81);
}

/// ⭐ Uma faixa com dono não é canal do catálogo: sem chave em `todas`, sem `resolver`/`info`; e o
/// rótulo do catálogo aponta o dono.
#[test]
fn uma_faixa_com_dono_nao_e_canal_do_catalogo() {
    let todas: BTreeSet<_> = todas().collect();
    for &(c, fid, x, d) in DONOS {
        let k = chave(c, fid, x).expect("cabe");
        assert!(
            !todas.contains(&k),
            "{c}.{fid}.{x} ainda é canal do catálogo"
        );
        assert!(
            resolver(k).is_none() && info(k).is_none(),
            "{c}.{fid}.{x} resolve"
        );
        let rotulo = (ph2d_component_desc::desc_for(c)
            .expect("existe")
            .fields
            .iter())
        .find(|f| f.field_id == fid)
        .expect("existe")
        .label_key;
        match d {
            Dono::Canal(o) => assert!(chaves_do_rotulo(rotulo).contains(&o), "{rotulo} ↛ {o:?}"),
            Dono::Prop(p) => assert!(props_do_rotulo(rotulo).contains(&p), "{rotulo} ↛ {p}"),
        }
        assert!(
            !chaves_do_rotulo(rotulo).contains(&k),
            "{rotulo} ainda dá a chave do catálogo"
        );
    }
}

/// O que não tem dono continua do catálogo — inclusive no mesmo componente (o `flip_x` da sprite).
#[test]
fn a_faixa_sem_dono_continua_do_catalogo() {
    let k = chave("ph2d::render::Sprite", 2, 0).expect("cabe");
    assert!(resolver(k).is_some() && info(k).is_some());
    assert!(todas().any(|t| t == k));
    let ks = chaves_do_rotulo("component.field.sprite.2");
    assert_eq!(ks, &[k]);
}
