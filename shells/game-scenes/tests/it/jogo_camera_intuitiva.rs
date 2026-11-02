//! ⭐ Os gates da câmera intuitiva ([plano GEF 20](../../../../docs/GameEngineFuncional/20_camera_intuitiva_plano.md)):
//! o Esc, o duplo-clique e o Shift das alças (C4), a etiqueta de cada alça, o Eyedropper e o largar no Target
//! (C2), os limites que nascem no contorno do nível (C6) e o canvas útil sem as réguas (C5).

use ph2d_camera_frame::*;
use ph2d_ecs::{CameraLimits, Entity, Name, SimWorld};
use ph2d_editor_core::CameraFieldEdit;
use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
use ph2d_editor_core::project::DisplayUnit;

fn segue() -> Moldura {
    Moldura {
        camera: 7,
        nome: "Camera".into(),
        pose: [2.0, 1.0],
        centro: [2.0, 1.0],
        meia: [8.0, 4.5],
        ativa: true,
        selecionada: true,
        limites: Some(([-20.0, -5.0], [30.0, 15.0])),
        mira: [2.0, 1.0],
        offset: [0.0, 0.0],
        segue: Some(Segue {
            nome: "Player".into(),
            alvo: Some([2.0, 1.0]),
            offset: [0.5, 1.0],
            zona: [0.25, 0.5],
        }),
    }
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5
}

/// ⭐ **O Esc devolve cada alça ao valor do `Down`**, o duplo-clique à fábrica, e o Shift encaixa no passo do
/// Inspector (C4).
///
/// **Mutações que devem sangrar:** `restaura` do canto com a meia-LARGURA · a zona morta de fábrica ≠ 0 · os limites
/// sem contorno a escrever alguma coisa · o encaixe da altura com o passo dos offsets.
#[test]
fn o_esc_o_duplo_clique_e_o_shift_das_alcas() {
    let m = segue();
    let novo = |a| Arrasto::novo(&m, a, [0.0, 0.0], 16.0 / 9.0, m.pose, (0.0, 0.0));
    assert_eq!(novo(Alca::Canto(2)).restaura(), Escrita::Altura(9.0));
    assert_eq!(
        novo(Alca::Dentro).restaura(),
        Escrita::FollowOffset([0.5, 1.0])
    );
    let sem = Moldura {
        segue: None,
        ..segue()
    };
    let pose = Arrasto::novo(&sem, Alca::Dentro, [0.0; 2], 1.0, sem.pose, (0.0, 0.0));
    assert_eq!(pose.restaura(), Escrita::Pose([2.0, 1.0]));
    assert_eq!(
        novo(Alca::LimiteBorda(1)).restaura(),
        Escrita::Limites([-20.0, -5.0], [30.0, 15.0])
    );
    assert_eq!(
        novo(Alca::ZonaMorta(0)).restaura(),
        Escrita::ZonaMorta([0.25, 0.5])
    );

    let nivel = Some(([-13.0, -1.0], [17.0, 7.0]));
    assert_eq!(
        padrao(&m, Alca::Canto(0), None),
        Some(Escrita::Altura(10.0))
    );
    assert_eq!(
        padrao(&m, Alca::ZonaMorta(3), None),
        Some(Escrita::ZonaMorta([0.0, 0.0]))
    );
    assert_eq!(
        padrao(&m, Alca::Dentro, None),
        Some(Escrita::FollowOffset([0.0, 0.0]))
    );
    assert_eq!(padrao(&sem, Alca::Dentro, None), None);
    assert_eq!(
        padrao(&m, Alca::LimiteCentro, nivel),
        Some(Escrita::Limites([-13.0, -1.0], [17.0, 7.0]))
    );
    assert_eq!(padrao(&m, Alca::LimiteCanto(1), None), None);

    assert_eq!(encaixa(Escrita::Altura(7.3)), Escrita::Altura(7.5));
    assert_eq!(encaixa(Escrita::Altura(0.1)), Escrita::Altura(PASSO_ALTURA));
    let Escrita::ZonaMorta(z) = encaixa(Escrita::ZonaMorta([0.27, 0.33])) else {
        panic!("a zona fica zona")
    };
    assert!(perto(z, [0.25, 0.35]), "{z:?}");
    let Escrita::Limites(lo, hi) = encaixa(Escrita::Limites([-13.2, -1.1], [17.3, -1.0])) else {
        panic!("os limites ficam limites")
    };
    assert!(
        perto(lo, [-13.0, -1.0]) && perto(hi, [17.5, -0.5]),
        "{lo:?} {hi:?}"
    );
    let Escrita::FollowOffset(o) = encaixa(Escrita::FollowOffset([0.26, -0.04])) else {
        panic!("o offset fica offset")
    };
    assert!(perto(o, [0.3, 0.0]), "{o:?}");
}

/// ⭐ **O duplo-clique é o 2.º press na MESMA alça dentro da janela** (o canvas não emite `DoubleClick`).
///
/// **Mutações que devem sangrar:** esquecer a alça · esquecer a câmera · não zerar depois do duplo (o 3.º press
/// seria outro duplo).
#[test]
fn o_duplo_clique_conta_se_por_alca() {
    let janela = ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS;
    let mut u = UltimoPress::default();
    assert!(!u.duplo(0, 7, Alca::Canto(0)));
    assert!(u.duplo(1, 7, Alca::Canto(0)));
    assert!(!u.duplo(2, 7, Alca::Canto(0)), "o 3.º press recomeça");
    assert!(!u.duplo(3, 7, Alca::Canto(1)), "outra alça");
    assert!(!u.duplo(4, 8, Alca::Canto(1)), "outra câmera");
    assert!(!u.duplo(4 + janela, 8, Alca::Canto(1)), "fora da janela");
}

/// ⭐ **Cada alça diz o que mexe e quanto** (a ficha do idioma das formas do Vector): textos distintos, cada um
/// com o seu número.
///
/// **Mutação que deve sangrar:** duas famílias de alça com a mesma etiqueta.
#[test]
fn cada_alca_tem_a_sua_etiqueta() {
    let m = segue();
    let alcas = [
        Alca::Canto(0),
        Alca::Dentro,
        Alca::ZonaMorta(0),
        Alca::LimiteBorda(0),
        Alca::LimiteBorda(1),
        Alca::LimiteBorda(2),
        Alca::LimiteBorda(3),
        Alca::LimiteCanto(0),
        Alca::LimiteCentro,
    ];
    let textos: Vec<String> = alcas
        .iter()
        .map(|a| etiqueta(&m, *a, DisplayUnit::Meters, 100.0))
        .collect();
    let mut unicos = textos.clone();
    unicos.sort();
    unicos.dedup();
    assert_eq!(unicos.len(), textos.len(), "{textos:?}");
    assert!(textos[0].contains("9.0 m"), "{}", textos[0]);
    assert!(textos[2].contains("25 %"), "{}", textos[2]);
    assert!(textos[3].contains("-20.0 m"), "{}", textos[3]);
}

/// ⭐ **O Eyedropper escolhe o de CIMA que não é a própria câmera**, pelo nome, pela porta do Inspector; só a
/// câmera debaixo do clique não escolhe nada (C2).
///
/// **Mutações que devem sangrar:** não saltar a câmera · escolher o último em vez do primeiro.
#[test]
fn o_conta_gotas_escolhe_o_de_cima_que_nao_e_a_camera() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let cam = w.spawn(Name("Camera".into())).id().to_bits();
    let player = w.spawn(Name("Player".into())).id().to_bits();
    let chao = w.spawn(Name("Ground".into())).id().to_bits();
    let mut bus = ActionBus::default();
    assert!(escolhe_alvo(
        sim.world(),
        &mut bus,
        cam,
        &[cam, player, chao]
    ));
    assert!(!escolhe_alvo(sim.world(), &mut bus, cam, &[cam]));
    let edits: Vec<_> = bus
        .drain()
        .map(|a| match a {
            EditorAction::InspectorCameraEdit { entity_bits, edit } => (entity_bits, edit),
            outra => panic!("{outra:?}"),
        })
        .collect();
    assert_eq!(edits, vec![(cam, CameraFieldEdit::Target("Player".into()))]);
}

/// ⭐ **Um objecto largado no Target do Follow faz a câmera segui-lo; uma imagem é recusada** (C2).
#[test]
fn largar_um_objecto_no_alvo_da_camera() {
    use ph2d_app_components::asset_drop::{DropAction, DropTarget, resolve};
    use ph2d_editor_core::interaction::drag_payload::DragPayload;
    let alvo = DropTarget::CameraTarget { entity_bits: 9 };
    assert_eq!(
        resolve(DragPayload::Object { stable_id: 42 }, alvo),
        DropAction::SetCameraTarget {
            entity_bits: 9,
            stable_id: 42
        }
    );
    assert_eq!(
        resolve(DragPayload::Image { asset: [0; 32] }, alvo),
        DropAction::Refuse
    );
}

/// ⭐ **Os limites nascem no contorno do nível 2** — as bordas de fora das paredes e do chão, o topo da parede —
/// sem o céu parado nem os fundos em parallax (C6). E a semente escreve-o.
///
/// **Mutações que devem sangrar:** contar os fundos (`ScrollFactor ≠ 1`) · contar o escondido · a semente não
/// escrever.
#[test]
fn os_limites_nascem_no_contorno_do_nivel() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, 2);
    let (lo, hi) = contorno_do_nivel(sim.world_mut()).expect("o nível tem imagens");
    assert!(
        perto(lo, [-13.0, -1.0]) && perto(hi, [17.0, 7.0]),
        "{lo:?} {hi:?}"
    );
    let w = sim.world_mut();
    let cam = w
        .query::<(Entity, &Name)>()
        .iter(w)
        .find(|(_, n)| n.as_str() == "Camera")
        .map(|(e, _)| e)
        .expect("a câmera");
    w.entity_mut(cam).insert(CameraLimits::default());
    semeia_limites(&mut sim, cam.to_bits());
    let l = sim
        .world()
        .get::<CameraLimits>(cam)
        .cloned()
        .expect("limites");
    assert!(perto(l.min, lo) && perto(l.max, hi), "{l:?}");
}

/// ⭐ **O canvas útil tira as réguas** (C5): o rótulo da moldura encostado à esquerda ficava por baixo delas.
#[test]
fn o_canvas_util_tira_as_reguas() {
    let r = ph2d_editor_core::ruler::RULER_PX;
    assert_eq!(
        tinta::canvas_util([10.0, 20.0, 300.0, 200.0], true),
        [10.0 + r, 20.0 + r, 300.0 - r, 200.0 - r]
    );
    assert_eq!(
        tinta::canvas_util([10.0, 20.0, 300.0, 200.0], false),
        [10.0, 20.0, 300.0, 200.0]
    );
}
