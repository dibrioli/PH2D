//! **A ponta que é o CENÁRIO tem figura** (W-WorldPinGlyph).
//!
//! A geometria de um pino de mundo sempre esteve certa e o overlay o desenhava
//! **de graça** — cedo demais para alguém notar que o desenho não dizia nada.
//! Medido antes de uma linha ser escrita: um pino de mundo e um pino entre dois
//! corpos produziam caminhos **byte-idênticos**.
//!
//! ⚠️ **E o que o produto entregava era uma AUSÊNCIA, não uma diferença:** a
//! ponte põe `centre_b == anchor_b` num pino de mundo (medido: os dois em
//! `[0,0; 2,0]`), então a tracejada de posse do lado B tinha comprimento zero e
//! não pintava. *Não desenhar nada* não é uma frase — é a falta de uma, e um
//! joint entre dois corpos cujo B esteja centrado na âncora desenha a mesma
//! ausência.

use super::joint_tests::{G, camera, view, window};
use super::{JOINT_DIM_RGBA, joint_marks};
use ph2d_physics_ecs::{JointKind, JointView};
use ph2d_vector::{BezPath, PathEl};

/// Os pontos de cada camada desenhada — a régua destes gates.
fn pts(paths: &[(BezPath, [f32; 4])]) -> Vec<usize> {
    paths
        .iter()
        .map(|(p, _)| {
            p.elements()
                .iter()
                .filter(|e| matches!(e, PathEl::MoveTo(_) | PathEl::LineTo(_)))
                .count()
        })
        .collect()
}

fn marks(v: &JointView, g: [f32; 2]) -> Vec<(BezPath, [f32; 4])> {
    joint_marks(
        true,
        std::slice::from_ref(v),
        &[],
        &[],
        g,
        &camera(),
        window(),
    )
}

fn body_pin() -> JointView {
    let mut v = view(JointKind::Pin);
    v.centre_b = [1.0, 0.0];
    v.body_b = Some(ph2d_ecs::Entity::from_bits(2));
    v
}

fn world_pin() -> JointView {
    let mut v = body_pin();
    v.body_b = None;
    v
}

/// **A afirmação da wave:** as duas figuras DIFEREM na tela.
///
/// Mutação (não ramificar em `body_b`) ⇒ caminhos byte-idênticos, que é o mundo
/// pré-wave.
#[test]
fn a_world_pin_does_not_read_like_a_pin_between_two_bodies() {
    let (two, world) = (marks(&body_pin(), G), marks(&world_pin(), G));
    assert_eq!(two.len(), world.len(), "as duas desenham as mesmas camadas");
    let identical = two
        .iter()
        .zip(world.iter())
        .all(|((pa, ca), (pb, cb))| ca == cb && pa.to_svg() == pb.to_svg());
    assert!(
        !identical,
        "o pino de mundo desenhou exatamente o que um pino entre corpos desenha: \
         pontos {:?}",
        pts(&world)
    );
}

/// **A hachura veste a banda de POSSE**, não uma cor própria — ela responde *de
/// quem é esta ponta*, que é a pergunta daquela camada, e por isso apaga junto
/// num joint desligado e avermelha num rompido sem uma linha a mais.
#[test]
fn the_ground_hatch_wears_the_ownership_band() {
    let world = marks(&world_pin(), G);
    assert_eq!(
        world[0].1, JOINT_DIM_RGBA,
        "a hachura saiu fora da camada de posse"
    );
    // E ela ACRESCENTA à camada em vez de substituí-la: a linha sólida até o
    // centro de A continua lá (uma ponta é do mundo, a outra não).
    //
    // ⚠️ **O oráculo é o CRESCIMENTO com a gravidade, não uma contagem mínima.**
    // A primeira versão pedia `> 2 pontos` e passava com a wave inteira
    // desligada — a tracejada do mundo pré-wave tem 22 —, uma afirmação que não
    // podia falhar pelo motivo que alegava. O que só a hachura explica é a camada
    // de posse MUDAR quando existe um "para baixo".
    let flat = marks(&world_pin(), [0.0, 0.0]);
    assert_eq!(pts(&flat)[0], 2, "sem gravidade sobra só a linha de A");
    assert!(
        pts(&world)[0] > pts(&flat)[0],
        "a camada de posse não cresceu com a gravidade -- a hachura nao foi desenhada"
    );
}

/// **A hachura desce pela GRAVIDADE, não pelo −Y da tela.**
///
/// *Chão* é o lado para onde as coisas caem. Com gravidade LATERAL — que este
/// módulo suporta desde o W2b — uma hachura presa ao eixo da tela apontaria para
/// baixo com toda a convicção enquanto os corpos caem para o lado.
///
/// Mutação (cravar `(0.0, 1.0)` em vez do `g_screen`) ⇒ as duas gravidades
/// desenham a MESMA hachura.
#[test]
fn the_ground_hatch_falls_with_gravity_not_with_the_screen() {
    let down = marks(&world_pin(), [0.0, -9.81]);
    let sideways = marks(&world_pin(), [9.81, 0.0]);
    assert_eq!(
        pts(&down)[0],
        pts(&sideways)[0],
        "a mesma figura, virada -- a contagem de pontos nao muda"
    );
    assert_ne!(
        down[0].0.to_svg(),
        sideways[0].0.to_svg(),
        "a hachura ignorou a gravidade e desenhou presa ao eixo da tela"
    );
}

/// **Sem gravidade não há hachura** — e o joint continua sendo desenhado.
///
/// Não há *"para baixo"* a afirmar num mundo sem gravidade, e inventar um seria o
/// desenho escolhendo uma física que o mundo não tem. A metade que importa é a
/// segunda: a ausência da marca não pode levar o resto do joint junto.
#[test]
fn without_gravity_there_is_no_ground_to_draw_and_the_joint_survives() {
    let none = marks(&world_pin(), [0.0, 0.0]);
    assert_eq!(
        pts(&none)[0],
        2,
        "sobrou algo além da linha de posse de A: {:?}",
        pts(&none)
    );
    assert_eq!(none.len(), marks(&world_pin(), G).len(), "o joint sumiu");
}

/// **O teste de POSIÇÃO que a wave recusou erraria, e este gate é o porquê.**
///
/// A ponte põe `centre_b == anchor_b` num pino de mundo, então *"as duas
/// coincidem"* parece o predicado óbvio — e um joint entre dois corpos cujo B
/// esteja **centrado na própria âncora** satisfaz a MESMA igualdade. Ele ganharia
/// um chão que não existe.
///
/// Mutação (trocar `body_b.is_none()` por `centre_b == anchor_b`) ⇒ este corpo
/// ganha hachura.
#[test]
fn a_two_body_pin_centred_on_its_own_anchor_gets_no_ground() {
    let mut v = body_pin();
    v.centre_b = v.anchor_b;
    let drawn = marks(&v, G);
    assert_eq!(
        pts(&drawn)[0],
        2,
        "um corpo B centrado na âncora ganhou hachura de chão: {:?}",
        pts(&drawn)
    );
}

/// **SONDA:** o pino de MUNDO e o pino entre dois corpos desenham a mesma coisa?
/// `cargo test -p ph2d-host-desktop --release probe_world_pin_vs_body_pin --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_world_pin_vs_body_pin() {
    let a = marks(&body_pin(), G);
    let b = marks(&world_pin(), G);
    println!("\n=== pino entre CORPOS  vs  pino de MUNDO ===");
    println!("   corpos: {} camada(s), pontos {:?}", a.len(), pts(&a));
    println!("   mundo : {} camada(s), pontos {:?}", b.len(), pts(&b));
}

/// **A figura é uma HACHURA, não uma barra.**
///
/// ⚠️ **Este gate existe porque a mutação que apaga os riscos SOBREVIVEU aos
/// outros cinco.** Eles pinam que o lado do mundo *difere* e que ele *segue a
/// gravidade* — e uma barra nua satisfaz as duas coisas. Mas o valor inteiro da
/// wave é a marca ser **reconhecível**: a hachura é a notação de apoio fixo dos
/// diagramas de mecanismo, e uma barra sozinha lê como parede, batente ou fim de
/// curso, que são três outras coisas que este overlay já desenha.
///
/// O oráculo é a GEOMETRIA desenhada, não a constante: a camada de posse tem de
/// trazer segmentos em **duas direções diferentes** depois da linha de A — a
/// barra, e os riscos atravessados nela.
#[test]
fn the_world_mark_is_a_hatch_and_not_a_bare_bar() {
    let world = marks(&world_pin(), G);
    // Os segmentos da camada de posse, como pares (início, fim).
    let mut segs: Vec<(ph2d_vector::Point, ph2d_vector::Point)> = Vec::new();
    let mut cur = None;
    for el in world[0].0.elements() {
        match el {
            PathEl::MoveTo(p) => cur = Some(*p),
            PathEl::LineTo(p) => {
                if let Some(s) = cur {
                    segs.push((s, *p));
                }
                cur = Some(*p);
            }
            _ => {}
        }
    }
    // A primeira é a linha sólida até o centro de A; a marca de chão é o resto.
    let hatch = &segs[1..];
    assert!(
        hatch.len() >= 3,
        "a marca de chão tem {} segmento(s) -- uma barra sozinha nao e' hachura",
        hatch.len()
    );
    let dir = |s: &(ph2d_vector::Point, ph2d_vector::Point)| {
        let (dx, dy) = (s.1.x - s.0.x, s.1.y - s.0.y);
        let l = dx.hypot(dy).max(1e-9);
        (dx / l, dy / l)
    };
    let bar = dir(&hatch[0]);
    // `|cos|`, para que um risco desenhado ao contrário conte como paralelo.
    let slanted = hatch[1..]
        .iter()
        .filter(|s| {
            let d = dir(s);
            (bar.0 * d.0 + bar.1 * d.1).abs() < 0.9
        })
        .count();
    assert!(
        slanted >= 2,
        "nenhum risco atravessa a barra -- a figura desenhada e' uma barra, \
         nao a notacao de apoio fixo"
    );
}
