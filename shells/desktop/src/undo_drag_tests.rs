//! ⭐ **Um arrasto que pinta interruptores é UM passo de desfazer** (dono 01/11): enquanto o botão
//! está em baixo o gate do quadro ([`crate::App::motivo_da_supressao`]) suprime o passo, e o soltar
//! regista um só — o Ctrl+Z devolve as três linhas / as três células de uma vez.

use super::tests::{capture, registry, scene};
use super::*;
use ph2d_host::PointerButton;

/// Um quadro do `post_frame_undo_steps`: com entrada, regista se o gate deixa e o documento mudou.
fn quadro(app: &crate::App, fila: &mut ProjectUndo, base: &mut ProjectState, agora: ProjectState) {
    if app.motivo_da_supressao(true).is_none() && *base != agora {
        fila.push_undo(std::mem::replace(base, agora), SelectionMark::default());
    }
}

#[test]
fn arrastar_os_olhos_de_tres_linhas_e_um_passo_de_desfazer() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let e: Vec<_> = (0..3)
        .map(|_| {
            sim.world_mut()
                .spawn((Transform::default(), ph2d_ecs::Visibility { hidden: false }))
                .id()
        })
        .collect();
    let mut app = crate::App::new();
    let mut fila = ProjectUndo::default();
    let antes = capture(&mut sim, &vec, &reg);
    let mut base = antes.clone();
    let esconde = |sim: &mut SimWorld, i: usize| {
        ph2d_ecs::row_flag::flip(sim.world_mut(), e[i], ph2d_ecs::row_flag::RowFlag::Visible);
    };
    // Down na 1.ª linha; Move pelas outras duas; Up.
    app.held_button = Some(PointerButton::Primary);
    esconde(&mut sim, 0);
    quadro(&app, &mut fila, &mut base, capture(&mut sim, &vec, &reg));
    esconde(&mut sim, 1);
    quadro(&app, &mut fila, &mut base, capture(&mut sim, &vec, &reg));
    esconde(&mut sim, 2);
    quadro(&app, &mut fila, &mut base, capture(&mut sim, &vec, &reg));
    app.held_button = None;
    let depois = capture(&mut sim, &vec, &reg);
    quadro(&app, &mut fila, &mut base, depois.clone());
    assert_eq!(fila.depth(), 1, "o arrasto inteiro tem de ser UM passo");
    let (volta, _) = fila
        .undo(depois, SelectionMark::default())
        .expect("há um passo");
    assert_eq!(volta, antes, "um Ctrl+Z devolve as três linhas à vista");
}

#[test]
fn arrastar_tres_celulas_da_matriz_e_um_passo_de_desfazer() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let mut app = crate::App::new();
    let mut fila = ProjectUndo::default();
    let mut fisica = ph2d_physics_ecs::PhysicsSettings::default();
    let foto = |sim: &mut SimWorld, f: ph2d_physics_ecs::PhysicsSettings| {
        let mut s = capture(sim, &vec, &reg);
        s.regras.physics = f;
        s
    };
    let antes = foto(&mut sim, fisica);
    let mut base = antes.clone();
    app.held_button = Some(PointerButton::Primary);
    for j in 0..3 {
        let mut m = ph2d_physics_ecs::LayerMatrix::from_rows(fisica.layer_matrix);
        m.set(5, j, false);
        fisica.layer_matrix = m.rows();
        quadro(&app, &mut fila, &mut base, foto(&mut sim, fisica));
    }
    app.held_button = None;
    let depois = foto(&mut sim, fisica);
    quadro(&app, &mut fila, &mut base, depois.clone());
    assert_eq!(fila.depth(), 1, "o arrasto inteiro tem de ser UM passo");
    let (volta, _) = fila
        .undo(depois, SelectionMark::default())
        .expect("há um passo");
    assert_eq!(
        volta.regras.physics, antes.regras.physics,
        "um Ctrl+Z devolve as três células"
    );
}
