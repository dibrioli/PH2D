//! **Os gates do READOUT de joint** (W-J7b) — o número que torna um teto de
//! ruptura ajustável em vez de adivinhável.
//!
//! Todos aqui afirmam a mesma frase por ângulos diferentes: *o artista consegue
//! LER a carga*. Sem ela, escolher um teto é busca binária feita à mão, que foi
//! exatamente o report que abriu esta wave.

use super::joint_readouts;
use crate::render_loop::physics_overlay_joints::{JOINT_BROKEN_RGBA, JOINT_RGBA};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{JointKind, JointLoad, JointView};
use ph2d_render::Camera2d;

fn window() -> WindowSize {
    WindowSize {
        width: 800,
        height: 600,
    }
}

fn camera() -> Camera2d {
    Camera2d {
        height_world: 15.0,
        ..Camera2d::default()
    }
}

const E1: fn() -> ph2d_ecs::Entity = || ph2d_ecs::Entity::from_bits(1);

/// Uma view segurando `load`, com marca d'água `peak` e teto `cap`
/// (`f32::INFINITY` = não quebrável).
fn view(load: f32, peak: f32, cap: f32, broken: bool) -> JointView {
    JointView {
        entity: E1(),
        kind: JointKind::Rope,
        anchor_a: [0.0, 0.0],
        anchor_b: [0.0, -1.0],
        centre_a: [0.0, 0.0],
        centre_b: [0.0, -1.0],
        body_b: ph2d_ecs::Entity::from_bits(3),
        angle_a: 0.0,
        angle_b: 0.0,
        limits: None,
        motor_speed: None,
        length: Some(1.0),
        axis: None,
        broken,
        load: JointLoad {
            force: load,
            torque: 0.0,
        },
        peak: JointLoad {
            force: peak,
            torque: 0.0,
        },
        break_force: cap,
        break_torque: f32::INFINITY,
    }
}

fn texts(v: &JointView, selected: bool) -> Vec<String> {
    joint_readouts(
        true,
        std::slice::from_ref(v),
        selected.then(E1),
        &camera(),
        window(),
    )
    .into_iter()
    .map(|r| r.text)
    .collect()
}

/// **A carga que o joint segura está NA TELA, ao lado do teto que ele aguenta.**
///
/// O par é o produto inteiro desta wave: com os dois números juntos, escolher o
/// teto deixa de ser um chute e vira uma leitura.
///
/// Mutação: o readout mostrar só o teto — o `58.9` some e o gate cai, que é
/// exatamente o estado que o Enio reportou (*"enorme quantidade de tentativas"*).
#[test]
fn a_breakable_joint_shows_what_it_carries_next_to_what_it_can_take() {
    assert_eq!(
        texts(&view(58.9, 58.9, 60.0, false), false),
        ["58.9 / 60 N"]
    );
}

/// **Um joint SELECIONADO mostra a carga mesmo sem teto armado.**
///
/// A metade do *bootstrap*, e sem ela o laço continua começando por um chute:
/// para escolher um teto é preciso ler a carga ANTES de armar qualquer coisa.
/// O controle está no mesmo gate — o mesmo joint NÃO selecionado e sem teto não
/// desenha nada, senão uma cena de quarenta joints vira quarenta números.
///
/// Mutação: o gate de seleção trocado por `true` — o controle passa a desenhar e
/// a 2ª metade cai.
#[test]
fn the_selected_joint_shows_its_load_even_with_no_threshold_armed() {
    assert_eq!(
        texts(&view(41.2, 41.2, f32::INFINITY, false), true),
        ["41.2 N"]
    );
    assert!(
        texts(&view(41.2, 41.2, f32::INFINITY, false), false).is_empty(),
        "e um joint sem teto e sem selecao nao desenha numero nenhum"
    );
}

/// **O `max` só aparece quando diz algo que a carga viva não diz.**
///
/// Num rig parado o pico da corrida É a carga viva, e repetir o mesmo número
/// duas vezes é ruído; ele nasce quando o joint levou um tranco, que é
/// precisamente quando a carga viva não serve para escolher um teto.
///
/// Mutação: `PEAK_MARGIN` em `1.0` — o rig parado ganha uma 2ª linha redundante
/// e a 1ª metade cai.
#[test]
fn the_high_water_mark_appears_only_when_it_says_something_new() {
    assert_eq!(
        texts(&view(58.9, 59.0, 60.0, false), false),
        ["58.9 / 60 N"],
        "parado: o pico e a carga viva, entao nao ha 2a linha"
    );
    assert_eq!(
        texts(&view(12.4, 87.2, 60.0, false), false),
        ["12.4 / 60 N", "max 87.2"],
        "depois de um tranco: o numero que se DIGITA e o pico"
    );
}

/// **Num joint rompido o número é a carga que PROVOCOU a fratura, em vermelho.**
///
/// O pedido literal do Enio, e ele se congela sozinho: o wrapper pula um joint
/// desabilitado, então a carga viva de um rompido lê zero enquanto a marca
/// d'água guarda o que cruzou. Sem essa troca o rótulo diria `0.0 / 60 N`, que é
/// verdade e não serve para nada.
///
/// Mutação: o rompido usar `load` em vez de `peak` — o rótulo vira `0.0 / 60 N`.
#[test]
fn a_broken_joint_shows_the_load_that_broke_it_not_the_zero_it_carries_now() {
    let broken = view(0.0, 87.2, 60.0, true);
    assert_eq!(texts(&broken, false), ["87.2 / 60 N"]);
    let colours: Vec<[f32; 4]> = joint_readouts(
        true,
        std::slice::from_ref(&broken),
        None,
        &camera(),
        window(),
    )
    .into_iter()
    .map(|r| r.rgba)
    .collect();
    assert_eq!(colours, [JOINT_BROKEN_RGBA], "e em vermelho");
    // O controle: segurando, o mesmo joint escreve em âmbar.
    let holding = view(58.9, 58.9, 60.0, false);
    let colours: Vec<[f32; 4]> = joint_readouts(
        true,
        std::slice::from_ref(&holding),
        None,
        &camera(),
        window(),
    )
    .into_iter()
    .map(|r| r.rgba)
    .collect();
    assert_eq!(colours, [JOINT_RGBA]);
}

/// **O torque só ganha linha quando há um teto de torque** — fora do Pin ele é
/// estruturalmente zero, e um `0.0 N.m` permanente seria um número que não
/// responde a nada.
#[test]
fn the_torque_line_exists_only_where_a_torque_threshold_does() {
    let mut v = view(9.8, 9.8, 100.0, false);
    v.kind = JointKind::Pin;
    v.load.torque = 4.9;
    v.peak.torque = 4.9;
    assert_eq!(
        texts(&v, false),
        ["9.8 / 100 N"],
        "sem teto de torque, sem linha"
    );
    v.break_torque = 20.0;
    assert_eq!(texts(&v, false), ["9.8 / 100 N", "4.9 / 20 N.m"]);
}

/// **O readout obedece ao interruptor do overlay** (tecla `B`), como todo o
/// resto deste módulo: ele é ANOTAÇÃO de algo que existe, não feedback de um
/// gesto em andamento (que é a única coisa aqui desenhada com o overlay off).
#[test]
fn the_readout_respects_the_overlay_switch() {
    assert!(
        joint_readouts(
            false,
            std::slice::from_ref(&view(58.9, 58.9, 60.0, false)),
            Some(E1()),
            &camera(),
            window(),
        )
        .is_empty()
    );
}
