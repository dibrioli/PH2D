use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::canal_vetor as c;
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc, autokey_campos};
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecVertex};

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, i, j)
}

const VERMELHO: u16 = c::TINTA_PREENCH + c::OFF_SOLIDA;

/// Uma forma vermelha de traço 4, ligada a uma entidade; a curva leva o vermelho 255→0 e o traço
/// 4→12 num segundo.
fn cena() -> (SimWorld, VecScene, u64, TimelineDoc) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([10.0, 0.0]),
            VecVertex::corner([5.0, 8.0]),
        ],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 4.0)),
        ..VecPath::default()
    });
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecPathRef(id)).id().to_bits();
    let mut doc = TimelineDoc::new();
    let s = RationalTime::from_seconds;
    for (campo, a, b) in [(VERMELHO, 255.0, 0.0), (c::TRACO_LARGURA, 4.0, 12.0)] {
        let p = PropKind::Field(k(campo, 0, 0));
        doc.insert_key(e, p, s(0.0), AnimValue::Float(a), Interp::Linear);
        doc.insert_key(e, p, s(1.0), AnimValue::Float(b), Interp::Linear);
    }
    (sim, scene, e, doc)
}

fn lidos(sim: &SimWorld, e: u64) -> Vec<(FieldKey, f32)> {
    sim.world().get_resource::<CanaisLidos>().unwrap().campos(e).collect()
}

fn quadro(sim: &mut SimWorld, scene: &mut VecScene, doc: &mut TimelineDoc, cond: &mut ConducaoVetor, e: u64, t: f64) {
    apply_from_doc(sim.world_mut(), doc, t);
    aplicar(sim, scene, cond);
    publicar(sim, scene, &[e]);
}

/// ⭐⭐ **A curva chega ao desenho e a meio não há chave fantasma.** O vermelho a 0,5 s vale 127,5 na
/// curva e 128 no documento (`u8`): sem o eco o autokey via a mão em todo quadro.
#[test]
fn a_curva_chega_ao_desenho_e_parar_a_meio_nao_grava() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let mut cond = ConducaoVetor::default();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
    let p = &scene.paths()[0];
    assert_eq!(p.fill, Some(Paint::Solid(Rgba8::new(128, 0, 0, 255))));
    assert_eq!(p.stroke.as_ref().unwrap().width, 8.0);

    let l = lidos(&sim, e);
    let base = l.iter().copied().collect();
    let plan = autokey_campos(&doc, e, 0.5, &l, &base, true, false);
    assert!(plan.is_empty(), "parado a meio, na curva, gravou {:?}", plan.keys);
}

/// A mão muda o traço com o Auto Key armado ⇒ a chave do traço, no valor da mão — e só dele.
#[test]
fn a_mao_no_traco_vira_chave_do_traco() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let mut cond = ConducaoVetor::default();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
    let base: BTreeMap<FieldKey, f32> = lidos(&sim, e).into_iter().collect();
    scene.path_mut(scene.paths()[0].id).unwrap().stroke.as_mut().unwrap().width = 20.0;
    publicar(&mut sim, &scene, &[e]);
    let plan = autokey_campos(&doc, e, 0.5, &lidos(&sim, e), &base, true, false);
    assert_eq!(plan.keys, vec![(PropKind::Field(k(c::TRACO_LARGURA, 0, 0)), 20.0)]);
}

/// ⭐ **Undo e save fotografam o AUTORADO enquanto a curva move o desenho** (a lei do
/// `ph2d_preview_drive`); parado, o valor da curva vira documento.
#[test]
fn a_fotografia_ve_o_autorado_e_parado_a_curva_vira_documento() {
    let (mut sim, mut scene, e, mut doc) = cena();
    let mut cond = ConducaoVetor::default();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5);
    let vivo = cond.substituir(&mut sim, &mut scene);
    let p = &scene.paths()[0];
    assert_eq!(p.fill, Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))), "a fotografia vê o autorado");
    assert_eq!(p.stroke.as_ref().unwrap().width, 4.0);
    cond.repor(&mut sim, &mut scene, vivo);
    assert_eq!(scene.paths()[0].stroke.as_ref().unwrap().width, 8.0, "e o vivo volta");

    cond.settle();
    quadro(&mut sim, &mut scene, &mut doc, &mut cond, e, 0.5); // o mesmo instante: nada muda
    cond.settle();
    assert!(cond.is_empty(), "parado, a curva já é documento");
}
