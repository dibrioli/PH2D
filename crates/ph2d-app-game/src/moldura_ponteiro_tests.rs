//! Os gates da câmera no canvas sobre a cena 4 REAL (onda 17b, auditoria A1–A3, A6, A7): o céu cobre a
//! moldura, a câmera segue o Player e tem limites, e cada quadro corre `camera_2d::update` como o app.

use super::*;
use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{CameraFollow, CameraRuntime, SimWorld, Transform};

const DT: f64 = 1.0 / 60.0;
const ASPECTO: f32 = 16.0 / 9.0;

/// A cena 4 (= o nível 2) com a câmera selecionada.
fn cena() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let _ = crate::smoke::montar(&mut sim, &crate::registo_de_teste(), 4);
    let cam = crate::smoke::por_nome(&mut sim, "Camera").expect("a cena 4 tem câmera");
    (sim, cam)
}

/// Um quadro do editor parado: o rebobinar tira o estado vivo e o passe da câmera corre sem tiques.
fn quadro(sim: &mut SimWorld, cam: Entity) -> Moldura {
    sim.world_mut().entity_mut(cam).remove::<CameraRuntime>();
    let (vista, _) = ph2d_app_components::camera_2d::update(sim, ASPECTO, 0, DT, [0.0, 0.0]);
    molduras(
        sim.world_mut(),
        vista.map(|v| v.center),
        ASPECTO,
        Some(cam.to_bits()),
    )
    .into_iter()
    .find(|m| m.camera == cam.to_bits())
    .expect("a moldura da câmera")
}

/// Escreve no mundo o que o shell escreveria: a pose directa, o resto pela porta do Inspector e a fila.
fn escreve(sim: &mut SimWorld, cam: Entity, e: Escrita) {
    if let Escrita::Pose(xy) = e
        && let Some(mut t) = sim.world_mut().get_mut::<Transform>(cam)
    {
        t.translation.x = xy[0];
        t.translation.y = xy[1];
    }
    let reg = crate::registo_de_teste();
    let fila = EditorCommandQueue::new();
    if let Some(edit) = edicoes(e) {
        ph2d_app_components::camera_inspector::apply_camera_edit(
            sim,
            cam.to_bits(),
            &edit,
            &fila,
            &reg,
        );
    }
    apply_editor_commands(sim.world_mut(), &fila, &reg).expect("a fila aplica");
}

fn pose_local(sim: &SimWorld, cam: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(cam).expect("pose");
    [t.translation.x, t.translation.y]
}

/// ⭐⭐ **A1: o miolo agarra com o céu por baixo.** O céu (40 × 16 m) cobre a moldura inteira da cena 4; o
/// miolo da câmera selecionada tem de agarrar na mesma — o clique sem arrasto é que segue para a seleção.
#[test]
fn o_miolo_agarra_com_o_ceu_por_baixo() {
    let (mut sim, cam) = cena();
    let m = quadro(&mut sim, cam);
    let p = [m.centro[0] + 1.0, m.centro[1] + 0.5];
    assert_eq!(alca_em(&m, p, 0.1), Some(Alca::Dentro));
}

/// ⭐⭐ **A2: com Follow, arrastar o miolo leva a moldura** — a moldura fica onde a lei põe o alvo + o novo
/// offset, apertada pela cerca: (−8; 1,5) + (0; 1) + (6; 0) ⇒ (−2; 2,5) ⇒ y preso em 3.
#[test]
fn com_follow_arrastar_o_miolo_leva_a_moldura() {
    let (mut sim, cam) = cena();
    let m = quadro(&mut sim, cam);
    let antes = m.centro;
    let p0 = [m.centro[0] + 1.0, m.centro[1]];
    let a = Arrasto::novo(
        &m,
        Alca::Dentro,
        p0,
        ASPECTO,
        pose_local(&sim, cam),
        (0.0, 0.0),
    );
    escreve(&mut sim, cam, arrastar(&a, [p0[0] + 6.0, p0[1]]));
    let depois = quadro(&mut sim, cam).centro;
    assert!(
        (depois[0] + 2.0).abs() < 1e-3 && (depois[1] - 3.0).abs() < 1e-3,
        "a moldura ficou em {depois:?} (antes {antes:?}); a lei diz (-2; 3)"
    );
}

/// ⭐⭐ **A3: cada canto e cada borda dos limites chegam INTEIROS ao mundo** — pela porta do Inspector e pela
/// fila, como no app. Dois lados no mesmo quadro não se podem apagar um ao outro.
#[test]
fn as_alcas_dos_limites_chegam_inteiras_ao_mundo() {
    let (mut sim, cam) = cena();
    let m = quadro(&mut sim, cam);
    let (min, max) = m.limites.expect("a cena 4 tem limites");
    let meio = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let mut alcas: Vec<(Alca, [f32; 2])> = cantos(min, max)
        .into_iter()
        .enumerate()
        .map(|(i, c)| (Alca::LimiteCanto(i as u8), c))
        .collect();
    alcas.extend([
        (Alca::LimiteBorda(0), [min[0], meio[1]]),
        (Alca::LimiteBorda(1), [max[0], meio[1]]),
        (Alca::LimiteBorda(2), [meio[0], min[1]]),
        (Alca::LimiteBorda(3), [meio[0], max[1]]),
        (Alca::LimiteCentro, meio),
    ]);
    drop(sim);
    for (alca, p0) in alcas {
        let (mut sim, cam) = cena();
        let m = quadro(&mut sim, cam);
        let a = Arrasto::novo(&m, alca, p0, ASPECTO, pose_local(&sim, cam), (0.0, 0.0));
        let Escrita::Limites(lo, hi) = arrastar(&a, [p0[0] - 1.5, p0[1] - 0.75]) else {
            panic!("{alca:?} escreve os limites");
        };
        escreve(&mut sim, cam, Escrita::Limites(lo, hi));
        let l = sim
            .world()
            .get::<ph2d_ecs::CameraLimits>(cam)
            .expect("limites");
        assert_eq!((l.min, l.max), (lo, hi), "{alca:?}");
    }
}

/// ⭐⭐ **A7: o canto vai para onde o ponteiro o leva, com a cerca a empurrar.** A câmera da cena 4 está presa na
/// borda esquerda dos limites; crescer pelo canto de cima-direita empurra o centro para a direita a cada
/// quadro — o canto tem de cair no ponteiro no eixo que manda (x).
#[test]
fn o_canto_com_limites_cai_no_ponteiro() {
    let (mut sim, cam) = cena();
    let m = quadro(&mut sim, cam);
    let (_, hi) = caixa(&m);
    let alca = alca_em(&m, hi, 0.1).expect("o canto de cima-direita");
    let a = Arrasto::novo(&m, alca, hi, ASPECTO, pose_local(&sim, cam), (0.0, 0.0));
    let alvo = [hi[0] + 3.0, m.centro[1]];
    escreve(&mut sim, cam, arrastar(&a, alvo));
    let (_, hi2) = caixa(&quadro(&mut sim, cam));
    assert!(
        (hi2[0] - alvo[0]).abs() < 1e-3,
        "o canto ficou em x {} e o ponteiro está em {}",
        hi2[0],
        alvo[0]
    );
}

/// ⭐ **A6: os dois «Offset» do cartão Camera têm nomes distintos** — o da vista e o do seguimento fazem coisas
/// diferentes (um soma depois da cerca, o outro entra na mira).
#[test]
fn os_dois_offset_tem_nomes_distintos() {
    use ph2d_i18n::tr;
    assert_ne!(
        tr("component.field.camera_fields.1"),
        tr("component.field.follow_fields.4")
    );
    let _ = CameraFollow::default();
}

/// ⭐⭐ **A9: o dedo é o ícone pintado, NO CENTRO do quadro (dono 28/10), e uma câmera escondida não tem ícone
/// nem moldura.** Com Follow o objecto fica longe do que filma: o ícone pega no centro e não na pose; dentro do
/// quadrado do lado do projecto e não fora dele; com o dobro do lado pega onde antes falhava.
#[test]
fn o_icone_pega_no_centro_e_a_camera_escondida_some() {
    use crate::moldura_tinta::icones_em;
    let (mut sim, cam) = cena();
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(cam) {
        t.translation.x = 20.0;
        t.translation.y = 20.0;
    }
    let ms = vec![quadro(&mut sim, cam)];
    let janela = ph2d_host::WindowSize {
        width: 1600,
        height: 900,
    };
    let vista = ph2d_render::Camera2d::new(ms[0].centro, 15.0);
    let (x, y) = vista.world_to_screen(ms[0].centro, janela);
    let lado = ph2d_editor_core::game_format::CAMERA_ICON_PX;
    let dentro = (x + lado * 0.5 - 1.0, y);
    let fora = (x + lado * 0.5 + 1.0, y);
    assert_eq!(
        icones_em(&ms, dentro, lado, &vista, janela),
        vec![cam.to_bits()]
    );
    assert!(icones_em(&ms, fora, lado, &vista, janela).is_empty());
    assert_eq!(
        icones_em(&ms, fora, lado * 2.0, &vista, janela),
        vec![cam.to_bits()]
    );
    assert!(
        icones_em(
            &ms,
            vista.world_to_screen([20.0, 20.0], janela),
            lado,
            &vista,
            janela
        )
        .is_empty()
    );
    sim.world_mut()
        .entity_mut(cam)
        .insert(ph2d_ecs::Visibility { hidden: true });
    let (vista_jogo, _) =
        ph2d_app_components::camera_2d::update(&mut sim, ASPECTO, 0, DT, [0.0, 0.0]);
    let ms = molduras(
        sim.world_mut(),
        vista_jogo.map(|v| v.center),
        ASPECTO,
        Some(cam.to_bits()),
    );
    assert!(ms.iter().all(|m| m.camera != cam.to_bits()));
    assert!(icones_em(&ms, dentro, lado, &vista, janela).is_empty());
}

/// ⭐ **Com Follow a moldura conhece o alvo** (o fio tracejado e a cruz da mira saem daqui).
#[test]
fn a_moldura_com_follow_sabe_o_alvo() {
    let (mut sim, cam) = cena();
    let m = quadro(&mut sim, cam);
    let s = m.segue.expect("a câmera da cena 4 segue o Player");
    assert_eq!(s.nome, "Player");
    assert_eq!(s.alvo, Some([-8.0, 1.5]));
    assert_eq!(s.offset, [0.0, 1.0]);
    assert_eq!(m.mira, [-8.0, 2.5]);
}
