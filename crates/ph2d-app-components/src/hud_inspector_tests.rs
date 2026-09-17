//! Os gates do instantâneo e do dreno da secção HUD.

use ph2d_ecs::{Counter, CounterRuntime, Fit, LabelSource, SimWorld, UiButton, UiCanvas, UiLabel};
use ph2d_editor_core::hud_edits::{HudFieldEdit as E, HudNumber as N, HudText as T};
use ph2d_tags::TagTree;

use super::{apply, build_info};

fn cena() -> (SimWorld, u64) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            UiCanvas {
                ref_w: 32.0,
                ref_h: 18.0,
                fit: Fit::Keep,
            },
            UiLabel {
                source: LabelSource::Counter("pontos".into()),
                prefix: "Pontos: ".into(),
                suffix: String::new(),
            },
            UiButton {
                signal: "bonus".into(),
                disabled: false,
            },
        ))
        .id();
    sim.world_mut().spawn((
        Counter {
            name: "pontos".into(),
            start: 0,
        },
        CounterRuntime { value: 12 },
    ));
    let bits = e.to_bits();
    (sim, bits)
}

/// ⛔ **Um objecto sem nenhum dos quatro não tem secção** — ADR-0166.
#[test]
fn um_objecto_sem_hud_nao_tem_seccao() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(ph2d_ecs::Transform::default()).id();
    assert!(build_info(&mut sim, &TagTree::default(), e.to_bits(), true).is_none());
    // O CONTROLO: com um componente do HUD, ela existe.
    let (mut sim, bits) = cena();
    assert!(build_info(&mut sim, &TagTree::default(), bits, true).is_some());
}

/// ⭐ O instantâneo traz o que o rótulo MOSTRA agora — pela mesma porta que o desenho usa.
#[test]
fn o_instantaneo_diz_o_que_o_rotulo_mostra_agora() {
    let (mut sim, bits) = cena();
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert_eq!(i.vivo, "Pontos: 12");
    assert_eq!(i.counter_value, 0, "este objecto não é o contador");
    assert!(i.has_canvas && i.has_label && i.has_button && !i.has_counter);
}

/// ⭐ E diz que NÃO há câmera — a razão de o canvas não se colar a nada.
#[test]
fn o_instantaneo_diz_quando_nao_ha_camera_de_jogo() {
    let (mut sim, bits) = cena();
    let sem = build_info(&mut sim, &TagTree::default(), bits, false).expect("tem HUD");
    assert!(!sem.tem_camera);
    let com = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert!(com.tem_camera, "o CONTROLO");
}

/// ⭐⭐ **Trocar a fonte CONSERVA o nome** — senão um engano custa a digitação.
#[test]
fn trocar_a_fonte_nao_apaga_o_nome() {
    let (mut sim, bits) = cena();
    assert!(apply(&mut sim, bits, &E::Source(2)));
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert_eq!(i.source, 2, "passou a ler um relógio");
    assert_eq!(i.source_name, "pontos", "⛔ o nome sobreviveu à troca");
    // E de volta: o nome continua lá.
    assert!(apply(&mut sim, bits, &E::Source(1)));
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert_eq!((i.source, i.source_name.as_str()), (1, "pontos"));
}

/// ⛔ **Um lado ZERO na caixa de referência é recusado no COMPONENTE**, e não no painel: a lei do
/// `ph2d_hud::Canvas::new` recusa-o, e deixá-lo entrar poria o canvas a conduzir com escala
/// infinita até alguém reparar.
#[test]
fn a_caixa_nunca_fica_com_um_lado_zero() {
    let (mut sim, bits) = cena();
    assert!(apply(&mut sim, bits, &E::Number(N::RefWidth, 0.0)));
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert!(i.ref_w > 0.0, "o zero não entra");
    assert!(
        ph2d_hud::Canvas::new(i.ref_w, i.ref_h, ph2d_hud::Fit::Keep).is_some(),
        "e o que fica é uma caixa que a lei aceita"
    );
}

/// Uma edição de um bloco AUSENTE é inerte, e não um erro.
#[test]
fn uma_edicao_de_um_bloco_ausente_e_inerte() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(ph2d_ecs::Transform::default()).id();
    let bits = e.to_bits();
    for edit in [
        E::Number(N::RefWidth, 4.0),
        E::Number(N::CounterStart, 3.0),
        E::Text(T::Signal, "x".into()),
        E::Fit(1),
        E::Source(1),
        E::Disabled(true),
    ] {
        assert!(!apply(&mut sim, bits, &edit), "{edit:?} não tem dono aqui");
    }
}

/// Cada campo volta ao componente certo — a ida-e-volta pelo instantâneo.
#[test]
fn cada_campo_volta_ao_componente_certo() {
    let (mut sim, bits) = cena();
    assert!(apply(&mut sim, bits, &E::Fit(1)));
    assert!(apply(&mut sim, bits, &E::Text(T::Prefix, "P: ".into())));
    assert!(apply(&mut sim, bits, &E::Text(T::Suffix, " pts".into())));
    assert!(apply(&mut sim, bits, &E::Text(T::Signal, "again".into())));
    assert!(apply(&mut sim, bits, &E::Disabled(true)));
    assert!(apply(&mut sim, bits, &E::Number(N::RefHeight, 9.0)));
    let i = build_info(&mut sim, &TagTree::default(), bits, true).expect("tem HUD");
    assert_eq!(i.fit, 1);
    assert_eq!((i.prefix.as_str(), i.suffix.as_str()), ("P: ", " pts"));
    assert_eq!(i.signal, "again");
    assert!(i.disabled);
    assert!((i.ref_h - 9.0).abs() <= f32::EPSILON);
}
