//! ⭐⭐ **O clique no CANVAS com um conta-gotas armado não selecciona** (dono 15/11) — o irmão do
//! [`super::jogo_conta_gotas_selecao`] (a Hierarchy). O rato real em píxeis da janela, pela MESMA porta da shell
//! (`conta_gotas::premir_no_canvas`: as guardas do despacho, `Camera2d::screen_to_world` e `pick_sprites_at_world`
//! sobre o mundo de apresentação), e o que ela devolve vai a `conta_gotas::escolher` como no `conta_gotas_clique`.
//! O press que a porta NÃO consome cai no pick de selecção da shell (`ramo_gizmo_pick_selecao`: o sprite de cima
//! substitui a selecção) — modelado aqui por `selecciona`; o Up não selecciona sozinho (só fecha o que o Down abriu).

use ph2d_app_components::{conta_gotas, projectile_inspector, ray_inspector, ray_topo};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, GlobalTransform, Name, PresentWorld, SimRef, SimWorld, StableId};
use ph2d_editor_core::action_bus::{ActionBus, ComponentEdit, EditorAction};
use ph2d_editor_core::projectile_edits::ProjectileFieldEdit;
use ph2d_editor_core::ray_edits::RayFieldEdit;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{BodyKind, RaySensor, RigidBody};
use ph2d_render::{Camera2d, RenderInstance};
use ph2d_tags::TagTree;

const JANELA: WindowSize = WindowSize {
    width: 1200,
    height: 600,
};

/// O quadrado `lado` do objecto `e` no mundo de apresentação, centrado em `(x, y)`.
fn sprite(present: &mut PresentWorld, e: Entity, x: f32, y: f32, lado: f32) {
    let gt =
        GlobalTransform::from_transform(ph2d_ecs::Transform::from_translation(Vec2::new(x, y)));
    let ri = RenderInstance {
        world_pos: [x, y],
        size: [lado, lado],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    };
    present.world_mut().spawn((SimRef(e), gt, ri));
}

struct Palco {
    sim: SimWorld,
    present: PresentWorld,
    camera: Camera2d,
    bus: ActionBus,
    /// A selecção do canvas (o `gizmo` da shell): só o press NÃO consumido a muda.
    seleccao: Option<u64>,
}

impl Palco {
    /// O pixel da janela onde o objecto em `mundo` aparece.
    fn pixel(&self, mundo: [f32; 2]) -> (f32, f32) {
        self.camera.world_to_screen(mundo, JANELA)
    }

    /// Down + Up do botão primário no pixel `cursor`, pela cadeia do despacho da shell.
    fn clica(&mut self, cursor: (f32, f32)) {
        for desce in [true, false] {
            let press = conta_gotas::PressNoCanvas {
                armado: conta_gotas::armado(self.sim.world()),
                primario_desce: desce,
                menu_aberto: false,
                sobre_canvas: true,
                cursor,
            };
            if let Some((_, debaixo)) =
                conta_gotas::premir_no_canvas(press, JANELA, &self.camera, self.present.world_mut())
            {
                conta_gotas::escolher(self.sim.world_mut(), &mut self.bus, &debaixo);
                continue;
            }
            if desce {
                let p = self.camera.screen_to_world(cursor, JANELA);
                let debaixo = ph2d_render::pick_sprites_at_world(self.present.world_mut(), p);
                self.seleccao = debaixo.first().copied();
            }
        }
    }
}

/// ⭐⭐ **Ray «Ignore up to» e Target da bala, pelo canvas**: armado, o clique sobre um objecto alimenta o
/// conta-gotas e a selecção fica no dono (o raio, a bala). A Crate (fora da cadeia) escreve «Crate» e a caixa mostra
/// o erro; o Truck (antepassado) escreve limpo; a bala recebe a Crate como Target.
///
/// **Mutações que sangram:** `premir_no_canvas` não consumir o press armado (a shell sem a guarda) · o press ler o
/// ponto do mundo sem a câmara.
#[test]
fn o_clique_no_canvas_do_conta_gotas_nao_seleciona() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let corpo = RigidBody {
        kind: BodyKind::Static,
    };
    let truck = w.spawn((Name::new("Truck"), corpo, StableId(41))).id();
    let raio = w
        .spawn((
            Name::new("Ray Sensor"),
            RaySensor::default(),
            StableId(42),
            ChildOf(truck),
        ))
        .id();
    let caixa = w.spawn((Name::new("Crate"), corpo, StableId(43))).id();
    let bala = w.spawn((Name::new("Bullet"), StableId(44))).id();
    let mut present = PresentWorld::new();
    let [m_truck, m_caixa] = [[-3.0, 0.0], [3.0, 0.0]];
    sprite(&mut present, truck, m_truck[0], m_truck[1], 2.0);
    sprite(&mut present, caixa, m_caixa[0], m_caixa[1], 2.0);
    let camera = Camera2d::new([0.5, 0.0], 10.0);
    let mut p = Palco {
        sim,
        present,
        camera,
        bus: ActionBus::default(),
        seleccao: Some(raio.to_bits()),
    };
    let (px_truck, px_caixa) = (p.pixel(m_truck), p.pixel(m_caixa));
    let escrito = |n: &str, dono: Entity, edit| EditorAction::InspectorComponentEdit {
        entity_bits: dono.to_bits(),
        edit: match edit {
            0 => ComponentEdit::Ray(RayFieldEdit::IgnoreUpTo(n.into())),
            _ => ComponentEdit::Projectile(ProjectileFieldEdit::HomingTarget(n.into())),
        },
    };
    let aplica = |p: &mut Palco| {
        for a in p.bus.drain().collect::<Vec<_>>() {
            if let EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Ray(e),
            } = a
            {
                assert!(ray_inspector::apply_ray_edit(
                    p.sim.world_mut(),
                    entity_bits,
                    &e
                ));
            }
        }
    };
    let caixa_do_raio = |p: &Palco| {
        let i = ray_inspector::build_ray_info(
            p.sim.world(),
            &TagTree::new(),
            raio.to_bits(),
            1,
            true,
            &[],
        )
        .expect("o raio");
        (i.ignore_up_to, i.ignore_up_to_off)
    };

    // O raio armado: a Crate (fora da cadeia) escreve-se, desarma, e a caixa mostra o erro.
    ray_topo::arma(p.sim.world_mut(), Some(raio.to_bits()));
    p.clica(px_caixa);
    assert_eq!(p.seleccao, Some(raio.to_bits()), "a Crate mudou a selecção");
    assert!(!conta_gotas::armado(p.sim.world()), "escreve e desarma");
    let saiu: Vec<_> = p.bus.drain().collect();
    assert_eq!(saiu, vec![escrito("Crate", raio, 0)]);
    p.bus.push(saiu[0].clone());
    aplica(&mut p);
    assert_eq!(caixa_do_raio(&p), ("Crate".to_owned(), true), "o erro");

    // O Truck (o antepassado): limpo.
    ray_topo::arma(p.sim.world_mut(), Some(raio.to_bits()));
    p.clica(px_truck);
    assert_eq!(p.seleccao, Some(raio.to_bits()), "o Truck mudou a selecção");
    let saiu: Vec<_> = p.bus.drain().collect();
    assert_eq!(saiu, vec![escrito("Truck", raio, 0)]);
    p.bus.push(saiu[0].clone());
    aplica(&mut p);
    assert_eq!(caixa_do_raio(&p), ("Truck".to_owned(), false), "limpo");

    // A bala armada (seleccionada, dona do Inspector): a Crate vira o Target.
    p.seleccao = Some(bala.to_bits());
    p.sim
        .world_mut()
        .insert_resource(projectile_inspector::AlvoArmado(Some(bala.to_bits())));
    p.clica(px_caixa);
    assert_eq!(p.seleccao, Some(bala.to_bits()), "a Crate mudou a selecção");
    assert_eq!(
        p.bus.drain().collect::<Vec<_>>(),
        vec![escrito("Crate", bala, 1)]
    );

    // Desarmado: o mesmo pixel volta a seleccionar (controlo positivo da régua).
    assert!(!conta_gotas::armado(p.sim.world()));
    p.clica(px_caixa);
    assert_eq!(p.seleccao, Some(caixa.to_bits()), "desarmado, selecciona");
    assert!(p.bus.drain().next().is_none());
}

/// ⭐⭐ **«Aim at» e «Also ignore» pelo canvas** (GEF 07 RS-3/RS-12): armado, o clique sobre um objecto escreve-o
/// no campo armado (`AimAt` / `IgnoreAdd`), desarma, e a selecção fica no raio. O clique no PRÓPRIO raio salta:
/// nada sai e o conta-gotas continua armado.
///
/// **Mutações que sangram:** `conta_gotas::armado` esquecer o `gotas::armado` (o press cai no pick e selecciona) ·
/// o `escolher` não filtrar o próprio raio · trocar `AimAt`/`IgnoreAdd` no `match campo`.
#[test]
fn o_clique_no_canvas_escolhe_a_mira_e_a_lista_do_raio() {
    use ph2d_app_components::ray_gotas::{Gotas, arma};
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let corpo = RigidBody {
        kind: BodyKind::Static,
    };
    let truck = w.spawn((Name::new("Truck"), corpo, StableId(41))).id();
    let raio = w
        .spawn((Name::new("Ray Sensor"), RaySensor::default(), StableId(42)))
        .id();
    let caixa = w.spawn((Name::new("Crate"), corpo, StableId(43))).id();
    let mut present = PresentWorld::new();
    let [m_truck, m_raio, m_caixa] = [[-3.0, 0.0], [0.0, 2.0], [3.0, 0.0]];
    sprite(&mut present, truck, m_truck[0], m_truck[1], 2.0);
    sprite(&mut present, raio, m_raio[0], m_raio[1], 1.0);
    sprite(&mut present, caixa, m_caixa[0], m_caixa[1], 2.0);
    let mut p = Palco {
        sim,
        present,
        camera: Camera2d::new([0.5, 0.0], 10.0),
        bus: ActionBus::default(),
        seleccao: Some(raio.to_bits()),
    };
    let (px_truck, px_raio, px_caixa) = (p.pixel(m_truck), p.pixel(m_raio), p.pixel(m_caixa));
    let escrito = |edit| EditorAction::InspectorComponentEdit {
        entity_bits: raio.to_bits(),
        edit: ComponentEdit::Ray(edit),
    };
    let armado = Some((raio.to_bits(), Gotas::Mira));
    arma(p.sim.world_mut(), armado);
    p.clica(px_raio);
    assert!(
        p.bus.drain().next().is_none(),
        "o próprio raio não se escolhe"
    );
    assert!(conta_gotas::armado(p.sim.world()), "e continua armado");
    assert_eq!(
        p.seleccao,
        Some(raio.to_bits()),
        "o raio não se re-selecciona"
    );
    p.clica(px_caixa);
    assert_eq!(p.seleccao, Some(raio.to_bits()), "a Crate mudou a selecção");
    assert!(!conta_gotas::armado(p.sim.world()), "escreve e desarma");
    let saiu: Vec<_> = p.bus.drain().collect();
    assert_eq!(saiu, vec![escrito(RayFieldEdit::AimAt("Crate".into()))]);

    arma(p.sim.world_mut(), Some((raio.to_bits(), Gotas::Lista)));
    p.clica(px_truck);
    assert_eq!(p.seleccao, Some(raio.to_bits()), "o Truck mudou a selecção");
    assert!(!conta_gotas::armado(p.sim.world()));
    let saiu: Vec<_> = p.bus.drain().collect();
    assert_eq!(saiu, vec![escrito(RayFieldEdit::IgnoreAdd("Truck".into()))]);

    // Controlo: desarmado, o mesmo pixel selecciona.
    p.clica(px_truck);
    assert_eq!(p.seleccao, Some(truck.to_bits()), "desarmado, selecciona");
}
