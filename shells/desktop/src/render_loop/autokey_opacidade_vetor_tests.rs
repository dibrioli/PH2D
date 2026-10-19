//! ⭐⭐⭐ **A OPACIDADE DE UM VETOR ANIMADO, PARADA FORA DE UMA CHAVE** — o quadro real (o dreno da
//! timeline, a projecção da tela com o `settle_to_authored`, o auto-key), sobre uma forma gravada a
//! 100 % cuja curva diz 20 % → 100 %. Plano 34 §1.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{SimWorld, VecPathRef};
use ph2d_timeline::{PropKind, TimelineState};
use ph2d_vec_scene::{VecPath, VecScene};

struct Cena {
    sim: SimWorld,
    scene: VecScene,
    map: ph2d_app_vec::entity_map::VecEntityMap,
    e: u64,
    st: TimelineState,
    ph: Playhead,
    ak: AutokeyState,
}

/// Uma forma a 100 % com a opacidade keyada `0,2 → 1,0` em `0..1 s`, o relógio parado a meio
/// (a curva vale `0,6`).
fn cena() -> Cena {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath::default());
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, VecPathRef(id)))
        .id()
        .to_bits();
    let mut map = ph2d_app_vec::entity_map::VecEntityMap::new();
    map.insert(id, e);
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.2_f32), (1.0, 1.0)] {
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            ph2d_timeline::TimelineIntent::AddKey {
                entity: e,
                prop: PropKind::Opacity,
                t: RationalTime::from_seconds(t),
                value: AnimValue::Float(v),
                interp: Interp::Linear,
            },
        );
    }
    ph.pause();
    ph.seek(0.5);
    // A aba Keys: o clip activo a solo, onde o artista anima (a pilha está vazia).
    st.keys_mode = true;
    Cena {
        sim,
        scene,
        map,
        e,
        st,
        ph,
        ak: AutokeyState::default(),
    }
}

/// Um quadro: dreno → a tela lê a opacidade conduzida → `settle` → auto-key sobre a forma
/// seleccionada. Devolve a opacidade que a tela desenhou.
fn quadro(c: &mut Cena, armed: bool) -> f32 {
    quadro_com(c, armed, None)
}

/// O mesmo quadro, com o artista a digitar `edicao` na Opacity (a fase do painel corre entre o
/// dreno e a tela, como `fase_transform_ops`).
fn quadro_com(c: &mut Cena, armed: bool, edicao: Option<f32>) -> f32 {
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
    if let Some(v) = edicao {
        let sel: Vec<_> = c.map.keys().copied().collect();
        crate::vec_appearance::set_opacity(&mut c.scene, &sel, v);
        crate::vec_driven_style::follow_the_hand(&mut c.sim, &c.map, &sel, v);
    }
    let tela = crate::vec_driven_style::resolve(&c.sim, &c.map)
        .first()
        .map_or(1.0, |&(_, a)| a);
    crate::vec_driven_style::settle_to_authored(&mut c.sim, &c.map, &c.scene, |b| {
        c.st.doc.binding_for(b, PropKind::Opacity).is_some()
    });
    let mundo = super::super::timeline_bridge::Mundo::com_vetor(c.sim.world(), &c.scene);
    let samples = vec![(c.e, sample_pose(mundo, c.e))];
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
    tela
}

fn chaves(c: &Cena) -> usize {
    let b = c.st.doc.binding_for(c.e, PropKind::Opacity).expect("track");
    c.st.doc
        .active_clip()
        .track(b.target)
        .expect("track")
        .keys()
        .len()
}

/// ⛔ **Sem ninguém tocar, a tela continua na curva** — desarmado, cinco quadros parados.
#[test]
fn parada_e_desarmada_a_tela_fica_na_curva() {
    let mut c = cena();
    for i in 0..5 {
        let tela = quadro(&mut c, false);
        assert!(
            (tela - 0.6).abs() < 1e-3,
            "quadro {i}: a tela desenhou {tela}, a curva diz 0,6 (prendeu no gravado?)"
        );
    }
    assert!(
        !c.ak.displaced.contains(&c.e),
        "a forma ficou PRESA sem ninguem tocar"
    );
}

/// ⛔⛔ **Armado, parar num quadro não grava chave nenhuma** — a curva continua com as duas.
#[test]
fn parada_e_armada_nao_grava_o_gravado_por_cima_da_curva() {
    let mut c = cena();
    for _ in 0..5 {
        quadro(&mut c, true);
    }
    assert_eq!(
        chaves(&c),
        2,
        "o auto-key gravou uma chave sem ninguem tocar"
    );
}

fn chave_em(c: &Cena, t: f64) -> Option<f32> {
    let b = c.st.doc.binding_for(c.e, PropKind::Opacity)?;
    let tr = c.st.doc.active_clip().track(b.target)?;
    tr.keys()
        .iter()
        .find(|k| (k.t.to_seconds() - t).abs() < 1e-6)
        .and_then(|k| match k.value {
            AnimValue::Float(v) => Some(v),
            _ => None,
        })
}

/// ⭐ **Desarmado, a edição fica na tela até o tempo mudar** — e depois a curva volta (a lei do
/// `Transform`: o pino `displaced`). Nenhuma chave.
#[test]
fn desarmada_a_edicao_fica_ate_o_tempo_mudar() {
    let mut c = cena();
    quadro(&mut c, false);
    let tela = quadro_com(&mut c, false, Some(0.3));
    assert!(
        (tela - 0.3).abs() < 1e-3,
        "a edicao nao chegou a tela: {tela}"
    );
    for i in 0..3 {
        let tela = quadro(&mut c, false);
        assert!(
            (tela - 0.3).abs() < 1e-3,
            "quadro {i}: a curva cobriu a edicao ({tela})"
        );
    }
    assert_eq!(chaves(&c), 2, "desarmado nao grava");
    c.ph.seek(0.75);
    quadro(&mut c, false);
    let tela = quadro(&mut c, false);
    assert!(
        (tela - 0.8).abs() < 1e-3,
        "o tempo mudou e a curva nao voltou: {tela}"
    );
}

/// ⭐ **Armado, a edição vira a chave do quadro** com o valor digitado.
#[test]
fn armada_a_edicao_grava_a_chave_com_o_valor_digitado() {
    let mut c = cena();
    quadro(&mut c, true);
    quadro_com(&mut c, true, Some(0.3));
    quadro(&mut c, true);
    assert_eq!(
        chave_em(&c, 0.5),
        Some(0.3),
        "a chave do quadro nao tem o valor digitado"
    );
    assert_eq!(chaves(&c), 3);
    let tela = quadro(&mut c, true);
    assert!(
        (tela - 0.3).abs() < 1e-3,
        "a tela saiu da chave nova: {tela}"
    );
}

/// Uma forma a 40 % SEM track, o relógio parado.
fn cena_sem_track() -> Cena {
    let mut c = cena();
    c.st = TimelineState::new();
    let id = *c.map.keys().next().unwrap();
    crate::vec_appearance::set_opacity(&mut c.scene, &[id], 0.4);
    c
}

/// ⭐ **A tecla K numa forma a 40 % grava 0,4**, nunca 1,0 (plano 34 D4).
#[test]
fn a_tecla_k_grava_a_opacidade_gravada() {
    let c = cena_sem_track();
    let mundo = super::super::timeline_bridge::Mundo::com_vetor(c.sim.world(), &c.scene);
    let v = super::super::timeline_bridge::key_value_for(mundo, &c.st, c.e, PropKind::Opacity, 0.5);
    assert_eq!(v, Some(AnimValue::Float(0.4)));
}

/// ⭐ **Armado, mudar a Opacity de uma forma sem track CRIA a track** com o valor novo.
#[test]
fn armada_a_opacidade_de_uma_forma_sem_track_cria_a_track() {
    let mut c = cena_sem_track();
    quadro(&mut c, true);
    quadro_com(&mut c, true, Some(0.7));
    quadro(&mut c, true);
    assert_eq!(
        chave_em(&c, 0.5),
        Some(0.7),
        "o auto-key nao viu a Opacity mudar"
    );
}
