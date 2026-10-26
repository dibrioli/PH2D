//! ⭐ **As formas guardadas DENTRO de componentes (Envelope, pele) são regravadas na forma nova do
//! traço** quando um ficheiro anterior ao `PROJECT_SCHEMA` 190 abre (F16c).
//!
//! Os bytes v22 de um `VecPath` saem da cena gravada pelo código v22
//! (`ph2d-vec-scene/tests/it/fixtures/vec_scene_v22_traco.bin`), e não de um escritor de hoje.

use ph2d_ecs::StableId;
use ph2d_ecs::scene::{EntitySnapshotRow, WorldSnapshot, stable_type_id};
use ph2d_vec_scene::{DashPattern, VecPath};

const CENA_V22: &[u8] =
    include_bytes!("../../ph2d-vec-scene/tests/it/fixtures/vec_scene_v22_traco.bin");

/// Os bytes v22 da 1.ª forma da cena (a seta tracejada): `(versão, n.º de formas, forma 0, …)`.
fn seta_v22() -> Vec<u8> {
    let corpo = &CENA_V22[2..];
    assert_eq!(&CENA_V22[..2], &[22, 3], "versão 22, três formas");
    let resto = ph2d_vec_scene::with_wire_layout(22, || {
        postcard::take_from_bytes::<VecPath>(corpo)
            .expect("a forma 0 lê-se na forma v22")
            .1
    });
    corpo[..corpo.len() - resto.len()].to_vec()
}

fn mundo_com(nome: &str, data: Vec<u8>) -> WorldSnapshot {
    let mut w = WorldSnapshot::new();
    w.entities.push(std::sync::Arc::new(EntitySnapshotRow {
        id: StableId(StableId::FIRST),
        components: vec![ph2d_asset::ComponentBlob {
            type_id: stable_type_id(nome),
            data,
        }],
        parent: None,
    }));
    w
}

fn assert_seta(p: &VecPath) {
    let s = p.stroke.as_ref().expect("traço");
    assert_eq!(s.dash, Some(DashPattern::single(3.0, 1.5)));
    assert_eq!((s.marker_scale_start, s.marker_scale_end), (1.7, 1.7));
}

#[test]
fn an_envelope_child_source_is_rewritten_in_the_new_layout() {
    let filho = ph2d_ecs::VecEnvelopeChild {
        path: 7,
        source: seta_v22(),
    };
    let env = ph2d_ecs::VecEnvelope::at_rest(vec![filho], [[0.0; 2]; 4], [[[0.0; 2]; 2]; 4]);
    let mut w = mundo_com(
        ph2d_ecs::VEC_ENVELOPE_TYPE_NAME,
        postcard::to_allocvec(&env).unwrap(),
    );
    assert_eq!(crate::envelope_live::migra_fio(&mut w, 22), 1);
    let env: ph2d_ecs::VecEnvelope =
        postcard::from_bytes(&w.entities[0].components[0].data).expect("o envelope lê-se");
    let fonte: VecPath = postcard::from_bytes(&env.children[0].source).expect("a fonte lê-se VIVA");
    assert_seta(&fonte);
}

#[test]
fn a_skin_source_is_rewritten_in_the_new_layout() {
    // `SkinnedPath` = caminho + pesos + campo + efeitos — o postcard concatena, e um `None` é um
    // byte qualquer que seja o tipo.
    let mut fonte = seta_v22();
    fonte.extend(postcard::to_allocvec(&(vec![0.25_f64, 0.75], None::<u8>, true)).unwrap());
    let skin = ph2d_skeleton_ecs::SkinBind::new(fonte, Vec::new());
    let mut w = mundo_com(
        ph2d_skeleton_ecs::SKIN_TYPE_NAME,
        postcard::to_allocvec(&skin).unwrap(),
    );
    assert_eq!(ph2d_skeleton_live::skinned_mesh::migra_fio(&mut w, 22), 1);
    let skin: ph2d_skeleton_ecs::SkinBind =
        postcard::from_bytes(&w.entities[0].components[0].data).expect("a pele lê-se");
    let g = ph2d_skeleton_live::skinned_mesh::le(&skin.source).expect("a fonte lê-se VIVA");
    assert_seta(&g.path);
    assert_eq!(
        (g.pesos.as_slice(), g.efeitos_cozidos),
        (&[0.25, 0.75][..], true)
    );
}

const CENA_V23: &[u8] =
    include_bytes!("../../ph2d-vec-scene/tests/it/fixtures/vec_scene_v23_geometria.bin");

/// Os bytes v23 da 1.ª forma da cena da W4.2 («dentes», 10 vértices, sem id).
fn dentes_v23() -> Vec<u8> {
    let corpo = &CENA_V23[2..];
    assert_eq!(&CENA_V23[..2], &[23, 2], "versão 23, duas formas");
    let resto = ph2d_vec_scene::with_wire_layout(23, || {
        postcard::take_from_bytes::<VecPath>(corpo)
            .expect("a forma 0 lê-se na forma v23")
            .1
    });
    corpo[..corpo.len() - resto.len()].to_vec()
}

fn assert_dentes(p: &VecPath) {
    let ids: Vec<u32> = p.verts.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, (1..=10).collect::<Vec<_>>(), "abre numerada");
    assert_eq!(p.verts[3].corner_radius, 2.5);
}

/// ⭐ W4.2: a fonte v23 de um envelope (geometria sem id) regrava-se VIVA e numerada.
#[test]
fn a_v23_envelope_child_source_is_rewritten_with_ids() {
    let filho = ph2d_ecs::VecEnvelopeChild {
        path: 7,
        source: dentes_v23(),
    };
    assert!(
        postcard::from_bytes::<VecPath>(&filho.source).map_or(true, |p| p.verts.len() != 10),
        "⛔ CONTROLO: a fonte v23 NÃO se lê com a forma viva"
    );
    let env = ph2d_ecs::VecEnvelope::at_rest(vec![filho], [[0.0; 2]; 4], [[[0.0; 2]; 2]; 4]);
    let mut w = mundo_com(
        ph2d_ecs::VEC_ENVELOPE_TYPE_NAME,
        postcard::to_allocvec(&env).unwrap(),
    );
    assert_eq!(crate::envelope_live::migra_fio(&mut w, 23), 1);
    let env: ph2d_ecs::VecEnvelope =
        postcard::from_bytes(&w.entities[0].components[0].data).expect("o envelope lê-se");
    let fonte: VecPath = postcard::from_bytes(&env.children[0].source).expect("a fonte lê-se VIVA");
    assert_dentes(&fonte);
}

#[test]
fn a_v23_skin_source_is_rewritten_with_ids() {
    let mut fonte = dentes_v23();
    fonte.extend(postcard::to_allocvec(&(vec![0.5_f64], None::<u8>, false)).unwrap());
    let skin = ph2d_skeleton_ecs::SkinBind::new(fonte, Vec::new());
    let mut w = mundo_com(
        ph2d_skeleton_ecs::SKIN_TYPE_NAME,
        postcard::to_allocvec(&skin).unwrap(),
    );
    assert_eq!(ph2d_skeleton_live::skinned_mesh::migra_fio(&mut w, 23), 1);
    let skin: ph2d_skeleton_ecs::SkinBind =
        postcard::from_bytes(&w.entities[0].components[0].data).expect("a pele lê-se");
    let g = ph2d_skeleton_live::skinned_mesh::le(&skin.source).expect("a fonte lê-se VIVA");
    assert_dentes(&g.path);
}
