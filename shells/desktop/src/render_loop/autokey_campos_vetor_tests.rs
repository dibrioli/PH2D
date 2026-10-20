//! ⭐⭐⭐ **O AUTO KEY GRAVA O DESENHO** (plano 35, ordem do dono 19/10) — o quadro real: o dreno da
//! timeline, a família vetor a aplicar o que a curva mandou, a mão a editar o documento, a família a
//! publicar e o auto-key a decidir. Os 11 canais de sempre não entram: é o traço, a cor e os pontos.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as cv;
use ph2d_ecs::{FieldKey, SimWorld, VecPathRef};
use ph2d_timeline::{PropKind, TimelineState};
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecScene, VecVertex};

struct Cena {
    sim: SimWorld,
    scene: VecScene,
    id: u64,
    e: u64,
    st: TimelineState,
    ph: Playhead,
    ak: AutokeyState,
    cond: ph2d_app_vec::canais::ConducaoVetor,
}

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, i, j)
}

/// Uma estrela torta de 4 pontos, vermelha, traço 4, o relógio parado em 0,5 s, a aba Keys.
fn cena() -> Cena {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::smooth([40.0, 5.0], [30.0, -4.0], [48.0, 12.0]),
            VecVertex::corner([35.0, 30.0]),
            VecVertex::corner([3.0, 22.0]),
        ],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 4.0)),
        ..VecPath::default()
    });
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, VecPathRef(id)))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    st.keys_mode = true;
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    ph.seek(0.5);
    Cena {
        sim,
        scene,
        id,
        e,
        st,
        ph,
        ak: AutokeyState::default(),
        cond: Default::default(),
    }
}

fn animar(c: &mut Cena, campo: FieldKey, a: f32, b: f32) {
    for (t, v) in [(0.0, a), (1.0, b)] {
        ph2d_timeline::apply_intent(
            &mut c.st,
            &mut c.ph,
            ph2d_timeline::TimelineIntent::AddKey {
                entity: c.e,
                prop: PropKind::Field(campo),
                t: RationalTime::from_seconds(t),
                value: AnimValue::Float(v),
                interp: Interp::Linear,
            },
        );
    }
    c.ph.pause();
    c.ph.seek(0.5);
}

/// Um quadro inteiro, com a mão (`mao`) a editar o documento entre o dreno e o auto-key.
fn quadro(c: &mut Cena, armed: bool, mao: impl FnOnce(&mut VecScene, u64)) {
    let solo = c.st.keys_mode;
    super::super::timeline_bridge::run(
        c.sim.world_mut(),
        &mut c.st,
        &mut c.ph,
        &mut Vec::new(),
        &[],
        &mut c.ak,
        solo,
        None,
        &mut super::super::timeline_bridge::SignalEmitter::default(),
        &mut ph2d_preview_drive::PreviewDrive::default(),
    );
    ph2d_app_vec::canais::aplicar(&mut c.sim, &mut c.scene, &mut c.cond);
    mao(&mut c.scene, c.id);
    ph2d_app_vec::canais::publicar(&mut c.sim, &c.scene, &[c.e]);
    let mundo = super::super::timeline_bridge::Mundo::com_vetor(c.sim.world(), &c.scene);
    let samples = vec![(c.e, sample_pose(mundo, c.e))];
    c.ak.campos = samples.iter().map(|&(e, _)| (e, campos_lidos(&c.sim, e))).collect();
    apply_samples(
        &mut c.st,
        &c.ph,
        &samples,
        false,
        armed,
        false,
        &mut c.ak,
        &mut ph2d_editor_core::ToastQueue::new(),
    );
}

fn chaves(c: &Cena, campo: FieldKey) -> Vec<(f64, f32)> {
    let Some(b) = c.st.doc.binding_for(c.e, PropKind::Field(campo)) else {
        return Vec::new();
    };
    c.st.doc
        .active_clip()
        .track(b.target)
        .map(|t| {
            t.keys()
                .iter()
                .map(|kk| match kk.value {
                    AnimValue::Float(v) => (kk.t.to_seconds(), v),
                    outro => panic!("{outro:?}"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn largura(c: &Cena) -> f64 {
    c.scene.path(c.id).unwrap().stroke.as_ref().unwrap().width
}

/// ⭐ Armado, engrossar o traço de uma forma SEM animação grava a chave do traço, no quadro do
/// relógio e no valor da mão — e nada mais.
#[test]
fn armado_o_traco_engrossado_vira_chave() {
    let mut c = cena();
    quadro(&mut c, true, |_, _| {});
    quadro(&mut c, true, |s, id| s.path_mut(id).unwrap().stroke.as_mut().unwrap().width = 12.0);
    assert_eq!(chaves(&c, k(cv::TRACO_LARGURA, 0, 0)), vec![(0.5, 12.0)]);
    assert!(chaves(&c, k(cv::ANCORA_X, 0, 0)).is_empty(), "a forma não foi tocada");
}

/// ⭐⭐ **Mover os PONTOS do desenho grava a forma INTEIRA** (o report do dono: o arrasto da
/// ferramenta Vetor move os pontos, não o objecto) — pela porta da ferramenta, `translate_path`.
#[test]
fn armado_mover_os_pontos_grava_a_forma_inteira() {
    let mut c = cena();
    quadro(&mut c, true, |_, _| {});
    quadro(&mut c, true, |s, id| {
        assert!(s.translate_path(id, 80.0, 0.0));
    });
    for j in 0..4 {
        for campo in cv::ANCORA_X..=cv::TIPO_VERTICE {
            assert_eq!(chaves(&c, k(campo, 0, j)).len(), 1, "campo {campo} do vértice {j}");
        }
    }
    assert_eq!(chaves(&c, k(cv::ANCORA_X, 0, 0)), vec![(0.5, 80.0)]);
    assert_eq!(chaves(&c, k(cv::FECHADO, 0, 0)).len(), 1, "o contorno inteiro vai na chave");
    assert!(chaves(&c, k(cv::TRACO_LARGURA, 0, 0)).is_empty(), "o traço não mudou");
}

/// ⛔⛔ **Parado a meio de uma cor animada, armado, NÃO nasce chave** — a cor é `u8`, a curva dá
/// 127,5 e o documento 128 (o eco). Cinco quadros, as duas chaves de sempre.
#[test]
fn parado_a_meio_armado_nao_grava_mesmo_com_a_cor_quantizada() {
    let mut c = cena();
    let vermelho = k(cv::TINTA_PREENCH + cv::OFF_SOLIDA, 0, 0);
    animar(&mut c, vermelho, 255.0, 0.0);
    for i in 0..5 {
        quadro(&mut c, true, |_, _| {});
        assert_eq!(chaves(&c, vermelho).len(), 2, "quadro {i}");
    }
    assert_eq!(
        c.scene.path(c.id).unwrap().fill,
        Some(Paint::Solid(Rgba8::new(128, 0, 0, 255))),
        "a tela está na curva"
    );
}

/// Desarmado, a edição do traço de uma forma animada FICA até o relógio mudar; depois a curva volta.
#[test]
fn desarmado_a_edicao_fica_ate_o_relogio_mudar() {
    let mut c = cena();
    animar(&mut c, k(cv::TRACO_LARGURA, 0, 0), 4.0, 12.0);
    quadro(&mut c, false, |_, _| {});
    assert_eq!(largura(&c), 8.0, "a curva a meio");
    quadro(&mut c, false, |s, id| s.path_mut(id).unwrap().stroke.as_mut().unwrap().width = 30.0);
    for _ in 0..3 {
        quadro(&mut c, false, |_, _| {});
        assert_eq!(largura(&c), 30.0, "a mão ficou presa até o tempo andar");
    }
    c.ph.seek(0.25);
    quadro(&mut c, false, |_, _| {});
    assert_eq!(largura(&c), 6.0, "o relógio andou: a curva volta");
    assert_eq!(chaves(&c, k(cv::TRACO_LARGURA, 0, 0)).len(), 2, "desarmado não grava");
}
