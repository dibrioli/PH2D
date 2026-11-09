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

/// ⭐ W5: **cada gesto dos pontos do «Outline» é UM passo** — arrastar (Down, dois Move, Up), duplo clique
/// numa aresta (acrescenta), Alt+clique (tira) — e cada Ctrl+Z devolve os anéis de antes desse gesto.
/// *Mutações: sem a supressão do botão em baixo ⇒ o arrasto vira 3 passos; `sobe` a devolver o `Down` ⇒ 0.*
#[test]
fn cada_gesto_dos_pontos_do_outline_e_um_passo_de_desfazer() {
    use ph2d_physics_ecs::{Collider, ColliderOutline, ColliderShape};
    use ph2d_shape_handles::{Porta, Teclas, pontos as p};
    let mut reg = registry();
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let (mut sim, vec) = scene();
    let o = ColliderOutline {
        rings: vec![vec![[0, 0], [64, 0], [64, 64], [0, 64]]].into(),
        picture_px: [4, 4],
        ..ColliderOutline::default()
    };
    let shape = ColliderShape::Outline {
        half_x: 2.0,
        half_y: 2.0,
    };
    let c = Collider {
        shape,
        ..Collider::default()
    };
    let e = sim.world_mut().spawn((Transform::default(), c, o)).id();
    ph2d_shape_handles::alterna(sim.world_mut(), e.to_bits());
    let (camera, window) = (
        ph2d_render::Camera2d::new([0.0, 0.0], 10.0),
        ph2d_host::WindowSize {
            width: 1400,
            height: 900,
        },
    );
    let ecra = |w: [f32; 2]| camera.world_to_screen(w, window);
    let pts = |sim: &SimWorld| {
        let (pose, m, o) = p::do_alvo(sim.world(), e.to_bits()).expect("pontos");
        p::locais(&pose, &m, &o)[0].clone()
    };
    let aneis = |s: &ProjectState| s.world.clone();
    let mut app = crate::App::new();
    let mut fila = ProjectUndo::default();
    let mut base = capture(&mut sim, &vec, &reg);
    let mut fotos = vec![base.clone()];
    // Um gesto: `passos` com o botão em baixo (um quadro depois de cada), e o Up com o quadro dele.
    let mut gesto =
        |sim: &mut SimWorld, passos: &dyn Fn(&mut Porta<'_>, usize) -> bool, n: usize| {
            app.held_button = Some(PointerButton::Primary);
            for i in 0..n {
                let mut bus = ph2d_editor_core::action_bus::ActionBus::default();
                let mut porta = Porta {
                    world: sim.world_mut(),
                    camera,
                    window,
                    bus: &mut bus,
                };
                assert!(passos(&mut porta, i), "o passo {i} foi consumido");
                quadro(&app, &mut fila, &mut base, capture(sim, &vec, &reg));
            }
            let mut bus = ph2d_editor_core::action_bus::ActionBus::default();
            let _ = Porta {
                world: sim.world_mut(),
                camera,
                window,
                bus: &mut bus,
            }
            .sobe();
            app.held_button = None;
            let depois = capture(sim, &vec, &reg);
            quadro(&app, &mut fila, &mut base, depois.clone());
            (depois, fila.depth())
        };
    let t = Teclas::default();
    let p0 = pts(&sim)[0];
    let (s0, s1, s2) = (
        ecra(p0),
        ecra([p0[0] - 0.5, p0[1]]),
        ecra([p0[0] - 1.0, p0[1]]),
    );
    let (d, n1) = gesto(
        &mut sim,
        &|po, i| match i {
            0 => po.desce_com(s0, t),
            1 => po.move_(s1, t),
            _ => po.move_(s2, t),
        },
        3,
    );
    fotos.push(d);
    assert_eq!(n1, 1, "o arrasto do ponto tem de ser UM passo");
    let v = pts(&sim);
    let meio = ecra([(v[1][0] + v[2][0]) / 2.0, (v[1][1] + v[2][1]) / 2.0]);
    let (d, n2) = gesto(&mut sim, &|po, _| po.desce_com(meio, t), 1);
    fotos.push(d);
    assert_eq!(pts(&sim).len(), 5, "o duplo clique acrescentou");
    assert_eq!(n2, 2, "o duplo clique é UM passo");
    let alt = Teclas { alt: true, ..t };
    let (d, n3) = gesto(&mut sim, &|po, _| po.desce_com(meio, alt), 1);
    fotos.push(d);
    assert_eq!(pts(&sim).len(), 4, "o Alt+clique tirou");
    assert_eq!(n3, 3, "o Alt+clique é UM passo");
    let (d, _) = gesto(&mut sim, &|po, _| po.desce_com(meio, t), 1);
    fotos.push(d);
    let (d, n5) = gesto(&mut sim, &|po, _| po.apaga(), 1);
    fotos.push(d);
    assert_eq!((pts(&sim).len(), n5), (4, 5), "o Delete é UM passo");
    let (d, _) = gesto(&mut sim, &|po, _| po.desce_com(meio, t), 1);
    fotos.push(d);
    let alvo = ecra(pts(&sim)[3]);
    let (d, n7) = gesto(
        &mut sim,
        &|po, i| match i {
            0 => po.desce_com(meio, t),
            1 => po.move_(((meio.0 + alvo.0) / 2.0, (meio.1 + alvo.1) / 2.0), t),
            _ => po.move_(alvo, t),
        },
        3,
    );
    fotos.push(d);
    assert_eq!((pts(&sim).len(), n7), (4, 7), "a fusão é UM passo");
    let v = pts(&sim);
    let ao = |t: f32| ecra([0, 1].map(|k| v[2][k] + t * (v[3][k] - v[2][k])));
    let (d, n8) = gesto(
        &mut sim,
        &|po, i| match i {
            0 => po.desce_com(ao(0.5), t),
            _ => po.move_(ao(0.5 - 0.1 * i as f32), t),
        },
        3,
    );
    fotos.push(d);
    assert_eq!(
        (pts(&sim).len(), n8),
        (5, 8),
        "acrescentar e arrastar é UM passo"
    );
    let mut bus = ph2d_editor_core::action_bus::ActionBus::default();
    let mut po = Porta {
        world: sim.world_mut(),
        camera,
        window,
        bus: &mut bus,
    };
    assert!(
        !po.desce_com(ecra([30.0, 30.0]), t),
        "o vazio limpa a seleção"
    );
    // «Merge by distance» (dono 09/11): o cacho num passo, o Merge do cartão noutro, um só.
    let (bits, q) = (e.to_bits(), ph2d_ecs::scene::EditorCommandQueue::new());
    p::escreve(
        sim.world_mut(),
        bits,
        vec![vec![[0, 0], [8, 0], [64, 0], [64, 64], [0, 64]]],
        true,
    );
    for m in [None, Some(ph2d_editor_core::OutlineEdit::Merge(3.0))] {
        if let Some(m) = m.map(ph2d_editor_core::PhysicsFieldEdit::Outline) {
            ph2d_app_physics::inspector::body::apply_physics_edit(&mut sim, bits, m, &q, &reg);
            ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
        }
        fotos.push(capture(&mut sim, &vec, &reg));
        quadro(&app, &mut fila, &mut base, fotos[fotos.len() - 1].clone());
    }
    assert_eq!(
        (pts(&sim).len(), fila.depth()),
        (4, 10),
        "o Merge é UM passo"
    );
    let mut agora = fotos.pop().expect("foto");
    while let Some(antes) = fotos.pop() {
        let (volta, _) = fila
            .undo(agora, SelectionMark::default())
            .expect("há um passo");
        assert_eq!(
            aneis(&volta),
            aneis(&antes),
            "cada Ctrl+Z devolve o gesto inteiro"
        );
        agora = volta;
    }
    assert_eq!(fila.depth(), 0);
}
