//! ⭐ **Os pontos verdes da forma ficam EM CIMA dela** (defeito do dono, 04/11: mover pelo Transform deixava
//! as alças para trás; `docs/Vector Module/BUGS_vector.md`). Gesto real no cartão (escrever e arrastar a pega)
//! ou a porta do verbo; a régua é o ECRÃ. O oráculo [`Bancada::confere`] monta a MESMA forma recém-nascida no
//! sítio em que a real está, confere que os pontos das duas coincidem, e exige as mesmas alças.
#![cfg(feature = "panel-vector")]

use ph2d_app_vec::shape_handles_live::{self as alcas, Gate};
use ph2d_app_vec::shape_live::recook_shape;
use ph2d_card_stack::metrics as m;
use ph2d_ecs::{ChildOf, Entity, SimWorld, Transform, VecShape};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{LengthDisplay, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, cards, ids as p, state};
use ph2d_tool_vector::ids as t;
use ph2d_tool_vector::shape_handles::{Mods, handles_of};
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vec_edit::PenTool;
use ph2d_vec_entities::campos_transform::{self as ct, VecTransformField as F};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_entities::transform::{build, move_origin_to, world_transform, xform_of_transform};
use ph2d_vec_scene::{FlipAxis, Rotate90, ShapeKind, VecPathId, VecScene, VecXforms, Xform};

const PX: f64 = 0.01;
const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1340.0,
    h: 3200.0,
};
/// A folga no ecrã: muito abaixo de um pixel, acima do arredondamento `f32` da pose.
const FOLGA_PX: f64 = 0.02;

/// A câmara do teste: 100 px por metro, Y para baixo no ecrã.
fn tela(w: [f64; 2]) -> [f64; 2] {
    [960.0 + w[0] / PX, 520.0 - w[1] / PX]
}

fn longe(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Todas as formas do catálogo com alças.
fn formas() -> Vec<ShapeKind> {
    (0..64)
        .filter_map(ShapeKind::from_u16)
        .filter(|k| !handles_of(*k).is_empty())
        .collect()
}

/// As três poses que o dono pede: só translação · rodada e escalada · filha de um pai transformado.
#[derive(Copy, Clone, Debug)]
enum Pose {
    Solta,
    Rodada,
    FilhaDePai,
}

const POSES: [Pose; 3] = [Pose::Solta, Pose::Rodada, Pose::FilhaDePai];

struct Mundo {
    sim: SimWorld,
    scene: VecScene,
    map: VecEntityMap,
    pen: PenTool,
    id: VecPathId,
    e: Entity,
}

fn mundo(kind: ShapeKind, pose: Pose) -> Mundo {
    let (mut sim, mut scene, mut map) = (SimWorld::default(), VecScene::new(), VecEntityMap::new());
    let shape = VecShape::Param {
        kind: kind.as_u16(),
        w: 2.0,
        h: 1.4,
        values: kind.defaults(),
    };
    let id = scene.push_path(recook_shape(&shape).expect("cozinha"));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let e = Entity::from_bits(map[&id]);
    let mut local = Transform::default();
    local.translation = ph2d_core::Vec2::new(1.0, 0.5);
    if matches!(pose, Pose::Rodada) {
        local.rotation = 0.5;
        local.scale = ph2d_core::Vec2::new(2.0, 1.5);
    }
    if matches!(pose, Pose::FilhaDePai) {
        let mut pai = Transform::default();
        pai.translation = ph2d_core::Vec2::new(-2.0, 1.0);
        pai.rotation = 0.3;
        pai.scale = ph2d_core::Vec2::new(1.5, 1.5);
        let pai = sim.world_mut().spawn(pai).id();
        sim.world_mut().entity_mut(e).insert(ChildOf(pai));
    }
    sim.world_mut().entity_mut(e).insert((local, shape));
    let mut pen = PenTool::new();
    pen.select(Some(id));
    Mundo {
        sim,
        scene,
        map,
        pen,
        id,
        e,
    }
}

fn caixa_de_ancoras(pts: impl Iterator<Item = [f64; 2]>) -> [f64; 2] {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for a in pts {
        for i in 0..2 {
            lo[i] = lo[i].min(a[i]);
            hi[i] = hi[i].max(a[i]);
        }
    }
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
}

impl Mundo {
    fn xf(&self) -> Xform {
        xform_of_transform(world_transform(&self.sim, self.e))
    }

    fn xforms(&self) -> VecXforms {
        build(&self.sim, &self.map)
    }

    fn alcas(&self) -> Vec<[f64; 2]> {
        alcas::view(
            &self.sim,
            &self.scene,
            &self.map,
            &[self.id],
            Gate::SHAPE,
            PX,
            None,
            None,
        )
        .expect("as alças")
        .points
        .into_iter()
        .map(tela)
        .collect()
    }

    /// O que a forma desenha, no ecrã.
    fn pontos(&self) -> Vec<[f64; 2]> {
        let xf = self.xf();
        (self.scene.path(self.id).expect("o caminho").verts_all())
            .map(|v| tela(xf.apply(v.anchor)))
            .collect()
    }

    fn canto(&self) -> [f64; 2] {
        let (lo, _) = (self.scene)
            .path_world_curve_bbox(&self.xforms(), self.id)
            .expect("a caixa");
        tela(lo)
    }

    /// ⭐ O oráculo: a MESMA receita, nascida centrada, com a pose que a põe sobre os pontos reais.
    fn confere(&self, oque: &str) {
        let forma = self.sim.world().get::<VecShape>(self.e).cloned();
        let forma = forma.expect("a receita");
        let cozida = recook_shape(&forma).expect("cozinha");
        let pose = world_transform(&self.sim, self.e);
        let xf = self.xf();
        let real = self.scene.path(self.id).expect("o caminho");
        let c_real = caixa_de_ancoras(real.verts_all().map(|v| v.anchor));
        let c_coz = caixa_de_ancoras(cozida.verts_all().map(|v| v.anchor));
        let onde = xf.apply(c_real);
        let linear = xf.apply_vec(c_coz);
        let mut nasce = pose;
        nasce.translation =
            ph2d_core::Vec2::new((onde[0] - linear[0]) as f32, (onde[1] - linear[1]) as f32);
        let (mut sim, mut scene, mut map) =
            (SimWorld::default(), VecScene::new(), VecEntityMap::new());
        let id = scene.push_path(cozida);
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
        let e = Entity::from_bits(map[&id]);
        sim.world_mut().entity_mut(e).insert((nasce, forma));
        let novo = Mundo {
            pen: PenTool::new(),
            sim,
            scene,
            map,
            id,
            e,
        };
        for (a, b) in self.pontos().iter().zip(novo.pontos()) {
            assert!(
                longe(*a, b) < FOLGA_PX,
                "{oque}: a régua não é a forma: {a:?} × {b:?}"
            );
        }
        let (a, b) = (self.alcas(), novo.alcas());
        assert_eq!(a.len(), b.len(), "{oque}");
        for (k, (x, y)) in a.iter().zip(&b).enumerate() {
            assert!(
                longe(*x, *y) < FOLGA_PX,
                "{oque}: a alça {k} está a {} px da forma: {x:?} × {y:?}",
                longe(*x, *y)
            );
        }
    }
}

/// O cartão Transform de verdade: o hospedeiro do painel, e o dreno que a shell faz com o que ele publica.
struct Cartao {
    host: MockPanelHost,
    st: VectorPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Cartao {
    fn novo(cartoes: bool) -> Self {
        cards::force_cards_on_this_thread(Some(cartoes));
        state::set_length_suffix("px");
        state::set_current_transform(Some([0.0, 0.0, 100.0, 100.0]));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        host.open_all_sections();
        let mut c = Self {
            host,
            st: VectorPanelState,
            rects: Vec::new(),
        };
        for _ in 0..40 {
            c.pinta();
        }
        c
    }

    fn pinta(&mut self) {
        let (h, st) = (&mut self.host, &mut self.st);
        self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || h.paint::<VectorPanel>(st, COLUNA),
        );
    }

    /// O dreno de `fase_bus_tool_panel` + `fase_transform_ops`.
    fn dreno(&mut self, w: &mut Mundo, eventos: Vec<WidgetEvent>) -> usize {
        let _ = self.host.drained_actions();
        for e in eventos {
            self.host.apply_panel_event::<VectorPanel>(&mut self.st, e);
        }
        let display = LengthDisplay::of(self.host.project_mut());
        let mut n = 0;
        for a in self.host.drained_actions() {
            let EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)) = a else {
                continue;
            };
            let campo = [
                (t::VECTOR_TRANSFORM_X, F::X),
                (t::VECTOR_TRANSFORM_Y, F::Y),
                (t::VECTOR_TRANSFORM_W, F::W),
                (t::VECTOR_TRANSFORM_H, F::H),
            ]
            .into_iter()
            .find_map(|(i, f)| (i == id).then_some(f));
            if let Some(campo) = campo {
                let xf = w.xforms();
                let sel = w.pen.selected();
                ct::apply_vec_transform(
                    &mut w.sim,
                    &w.map,
                    &mut w.scene,
                    sel,
                    &xf,
                    campo,
                    display.to_world(v),
                );
                n += 1;
            } else if id == p::VECTOR_TRANSFORM_R {
                let sel = w.pen.selected();
                ct::apply_vec_rotate_by(&mut w.sim, &mut w.scene, &w.map, sel, v);
                n += 1;
            }
        }
        n
    }

    fn escreve(&mut self, w: &mut Mundo, id: NodeId, valor: f64) {
        self.pinta();
        let ev = self.host.type_into_number(id, &format!("{valor}"));
        assert_eq!(
            self.dreno(w, ev),
            1,
            "o campo escrito não falou com ninguém"
        );
    }

    /// O arrasto real da pega do campo, `dx` px de cursor, largado no fim.
    fn arrasta(&mut self, w: &mut Mundo, id: NodeId, dx: f32) {
        self.pinta();
        let r = (self.rects.iter().rev())
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
            .expect("o campo pintado");
        let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
        let mut ponteiro = |kind, x| {
            self.host.dispatch_pointer_event(PointerEvent {
                kind,
                x,
                y,
                button: PointerButton::Primary,
                source: PointerSource::Mouse,
                pressure: 1.0,
                timestamp_ns: 0,
            })
        };
        let mut ev = ponteiro(PointerKind::Down, x);
        ev.extend(ponteiro(PointerKind::Move, x + dx * 0.5));
        ev.extend(ponteiro(PointerKind::Move, x + dx));
        ev.extend(ponteiro(PointerKind::Up, x + dx));
        assert!(self.dreno(w, ev) >= 1, "o arrasto não falou com ninguém");
    }
}

/// ⭐ O defeito da foto: a forma do dono, o X escrito e o X arrastado; com o cartão e com o painel de
/// antes (`PH2D_CARDS=0`): o ponto verde mantém a distância ao canto da caixa, ao centésimo de pixel.
#[test]
fn o_ponto_verde_acompanha_o_canto_quando_o_x_muda() {
    for cartoes in [true, false] {
        let mut c = Cartao::novo(cartoes);
        let mut w = mundo(ShapeKind::RoundRect, Pose::Solta);
        c.host
            .store_mut()
            .set_number_drag_rate(t::VECTOR_TRANSFORM_X, 0.5);
        let distancia = |w: &Mundo| {
            let (a, k) = (w.alcas()[0], w.canto());
            [a[0] - k[0], a[1] - k[1]]
        };
        let antes = distancia(&w);
        let k0 = w.canto();
        c.escreve(&mut w, t::VECTOR_TRANSFORM_X, 138.861);
        assert!(
            longe(k0, w.canto()) > 20.0,
            "o X escrito não moveu a caixa ({cartoes})"
        );
        assert!(
            longe(antes, distancia(&w)) < FOLGA_PX,
            "cartões {cartoes}: o ponto ficou para trás: {antes:?} × {:?}",
            distancia(&w)
        );
        let k1 = w.canto();
        c.arrasta(&mut w, t::VECTOR_TRANSFORM_X, 60.0);
        assert!(longe(k1, w.canto()) > 1.0, "o arrasto não moveu a caixa");
        assert!(
            longe(antes, distancia(&w)) < FOLGA_PX,
            "cartões {cartoes}: o arrasto deixou o ponto para trás"
        );
    }
}

/// X, Y, W, H e Rotation por gesto, em cada forma com alças e nas três poses.
#[test]
fn os_campos_do_transform_deixam_as_alcas_em_cima_em_toda_forma_e_pose() {
    let mut c = Cartao::novo(true);
    for kind in formas() {
        for pose in POSES {
            let mut w = mundo(kind, pose);
            let oque = |campo: &str| format!("{kind:?} {pose:?} {campo}");
            w.confere(&oque("nasceu"));
            for (id, nome, valor) in [
                (t::VECTOR_TRANSFORM_X, "X", 138.861),
                (t::VECTOR_TRANSFORM_Y, "Y", -9.071),
                (t::VECTOR_TRANSFORM_W, "W", 94.344),
                (t::VECTOR_TRANSFORM_H, "H", 104.287),
            ] {
                c.escreve(&mut w, id, valor);
                w.confere(&oque(nome));
            }
            c.escreve(&mut w, p::VECTOR_TRANSFORM_R, 30.0);
            w.confere(&oque("Rotation"));
        }
    }
}

/// Os verbos: girar, espelhar, setas, Set Center, o Reset, Ctrl+Z e Ctrl+Y.
#[test]
fn os_verbos_do_transform_deixam_as_alcas_em_cima() {
    for kind in formas() {
        for pose in POSES {
            let mut w = mundo(kind, pose);
            let oque = |v: &str| format!("{kind:?} {pose:?} {v}");
            let (id, e) = (w.id, w.e);
            ct::espelhar(&mut w.sim, &mut w.scene, &w.map, id, FlipAxis::Horizontal);
            w.confere(&oque("espelhar H"));
            ct::espelhar(&mut w.sim, &mut w.scene, &w.map, id, FlipAxis::Vertical);
            w.confere(&oque("espelhar V"));
            ct::quarto_de_volta(&mut w.sim, &mut w.scene, &w.map, id, Rotate90::Cw);
            w.confere(&oque("quarto horário"));
            ct::seta(&mut w.sim, &mut w.scene, &mut w.pen, &w.map, [0.25, -0.1]);
            w.confere(&oque("seta"));

            let foto = (
                w.sim.world().get::<Transform>(e).copied(),
                w.scene.path(id).cloned(),
            );
            let alvo = w.xf().apply([0.2, 0.1]);
            let antes = w.pontos();
            assert!(move_origin_to(
                &mut w.sim,
                &mut w.scene,
                e,
                id,
                [alvo[0] as f32, alvo[1] as f32]
            ));
            for (a, b) in antes.iter().zip(w.pontos()) {
                assert!(longe(*a, b) < FOLGA_PX, "o Set Center moveu a forma");
            }
            w.confere(&oque("Set Center"));

            // Ctrl+Z e Ctrl+Y: o documento volta (pose e pontos), as alças são derivadas dele.
            let depois = (
                w.sim.world().get::<Transform>(e).copied(),
                w.scene.path(id).cloned(),
            );
            for (estado, nome) in [(&foto, "Ctrl+Z"), (&depois, "Ctrl+Y")] {
                *w.sim.world_mut().get_mut::<Transform>(e).expect("pose") = estado.0.expect("pose");
                *w.scene.path_mut(id).expect("caminho") = estado.1.clone().expect("caminho");
                w.confere(&oque(nome));
            }

            // O Reset e a Timeline escrevem a pose por fora do cartão.
            let mut t = Transform::default();
            t.translation = ph2d_core::Vec2::new(-0.7, 0.9);
            *w.sim.world_mut().get_mut::<Transform>(e).expect("pose") = t;
            w.confere(&oque("a pose escrita por fora"));
        }
    }
}

/// A forma movida não salta de volta quando um parâmetro muda (o re-cook cozinhava-a no local 0), e o
/// arrasto da alça continua a agarrar em cima do ponto.
#[test]
fn editar_a_alca_depois_de_mover_nao_faz_a_forma_saltar() {
    let mut c = Cartao::novo(true);
    let mut w = mundo(ShapeKind::RoundRect, Pose::Rodada);
    c.escreve(&mut w, t::VECTOR_TRANSFORM_W, 150.0);
    let (id, sel) = (w.id, vec![w.id]);
    let move_centro = w.xf().apply([0.3, -0.2]);
    assert!(move_origin_to(
        &mut w.sim,
        &mut w.scene,
        w.e,
        id,
        [move_centro[0] as f32, move_centro[1] as f32]
    ));
    let centro = |w: &Mundo| caixa_de_ancoras(w.pontos().into_iter());
    let (centro0, alca0) = (centro(&w), w.alcas()[0]);
    let onde = alcas::view(&w.sim, &w.scene, &w.map, &sel, Gate::SHAPE, PX, None, None)
        .expect("alças")
        .points[0];
    let mut g = alcas::press(&w.sim, &w.scene, &w.map, &sel, Gate::SHAPE, onde, PX)
        .expect("o press em cima do ponto verde agarra");
    assert!(alcas::drag(
        alcas::World {
            sim: &mut w.sim,
            scene: &mut w.scene,
            map: &w.map,
            selection: &sel,
            gate: Gate::SHAPE,
        },
        &mut g,
        [onde[0] + 0.1, onde[1]],
        Mods::default(),
        PX,
    ));
    w.confere("depois do arrasto");
    assert!(
        longe(centro0, centro(&w)) < 0.5,
        "a forma saltou: {centro0:?} × {:?}",
        centro(&w)
    );
    assert!(
        longe(alca0, w.alcas()[0]) > 1.0,
        "o arrasto não mexeu no raio"
    );
}

/// Mover pelo campo leva o pivô e os filhos, como o gizmo.
#[test]
fn o_campo_x_leva_o_pivo_e_os_filhos() {
    let mut c = Cartao::novo(true);
    let mut w = mundo(ShapeKind::RoundRect, Pose::Solta);
    let filho = w
        .sim
        .world_mut()
        .spawn((Transform::default(), ChildOf(w.e)))
        .id();
    let pivo0 = world_transform(&w.sim, w.e).translation;
    let filho0 = world_transform(&w.sim, filho).translation;
    let (k0, _) = (w.scene)
        .path_world_curve_bbox(&w.xforms(), w.id)
        .expect("caixa");
    let alvo = 200.0;
    c.escreve(&mut w, t::VECTOR_TRANSFORM_X, alvo);
    let dx = (alvo * PX - k0[0]) as f32;
    let pivo = world_transform(&w.sim, w.e).translation;
    let fi = world_transform(&w.sim, filho).translation;
    assert!(
        (pivo.x - pivo0.x - dx).abs() < 1e-4,
        "o pivô ficou para trás"
    );
    assert!(
        (fi.x - filho0.x - dx).abs() < 1e-4,
        "o filho ficou para trás"
    );
}

/// Na multi-seleção o campo move a primária (a última escolhida) e as alças são da primeira forma viva:
/// cada coisa em cima da sua forma, e a que o campo não moveu não anda.
#[test]
fn na_multi_selecao_o_campo_move_a_primaria_e_as_alcas_ficam_na_sua_forma() {
    let mut c = Cartao::novo(true);
    let mut w = mundo(ShapeKind::RoundRect, Pose::Solta);
    let receita = w
        .sim
        .world()
        .get::<VecShape>(w.e)
        .cloned()
        .expect("receita");
    let outra = w.scene.push_path(recook_shape(&receita).expect("cozinha"));
    ph2d_vec_entities::entities::sync(&mut w.sim, &mut w.scene, &mut w.map);
    let outra_e = Entity::from_bits(w.map[&outra]);
    w.sim.world_mut().entity_mut(outra_e).insert(receita);
    w.pen.select_many(&[outra, w.id]);
    assert_eq!(w.pen.selected(), Some(w.id), "a primária");
    let par = [outra, w.id];
    let vistas = |w: &Mundo| {
        (alcas::view(&w.sim, &w.scene, &w.map, &par, Gate::SHAPE, PX, None, None))
            .expect("as alças da primeira forma viva")
            .points
            .into_iter()
            .map(tela)
            .collect::<Vec<_>>()
    };
    let antes = world_transform(&w.sim, outra_e).translation;
    let (k0, alcas0) = (w.canto(), vistas(&w));
    c.escreve(&mut w, t::VECTOR_TRANSFORM_X, 200.0);
    assert!(longe(k0, w.canto()) > 1.0);
    assert_eq!(world_transform(&w.sim, outra_e).translation, antes);
    assert_eq!(vistas(&w), alcas0, "as alças são as da forma que ficou");
    w.confere("a primária");
}

/// Cartão Transform preso a A, com B em foco: o gesto no preso anda com A e as alças dela, e B e as dele
/// ficam. Do clique ao pedido é o hero real; a escrita é a de `inspector_commits` (fila pede a shell).
#[cfg(feature = "panel-inspector")]
#[test]
fn o_cartao_preso_anda_com_a_sua_forma_e_as_alcas_dela() {
    use super::cartoes_mao::{clica, mao, pinta, solta};
    use super::o_bloco5_abertos::escolhe;
    use ph2d_editor_core::interaction::{loose_pin_of, loose_rect_of};
    use ph2d_editor_core::screens::hero::HeroScreen;
    use ph2d_panel_inspector::{CARD_TRANSFORM, ids as insp_ids};

    let mut w = mundo(ShapeKind::RoundRect, Pose::Solta);
    let receita = (w.sim.world().get::<VecShape>(w.e).cloned()).expect("receita");
    let b = w.scene.push_path(recook_shape(&receita).expect("cozinha"));
    ph2d_vec_entities::entities::sync(&mut w.sim, &mut w.scene, &mut w.map);
    let b_e = Entity::from_bits(w.map[&b]);
    w.sim.world_mut().entity_mut(b_e).insert(receita);
    let (bits_a, bits_b) = (w.e.to_bits(), b_e.to_bits());

    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    escolhe(bits_a, "A", 1.0);
    let mut h = HeroScreen::new(NodeId(1));
    let paineis: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in paineis {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h, 30);
    solta(&mut h, CARD_TRANSFORM);
    clica(&mut h, loose_pin_of(CARD_TRANSFORM));
    escolhe(bits_b, "B", 9.0);
    let pins = h.store.loose_pins_of(ph2d_editor_core::ids::INSP_PANEL);
    ph2d_panel_inspector::com_cada_preso(&pins, |bits| escolhe(bits, "A", 1.0));
    pinta(&mut h, 5);
    let r = (h.store.panel_rect(loose_rect_of(CARD_TRANSFORM))).expect("o preso");
    let dentro = (r.x + r.w * 0.5, r.y + r.h - 3.0);
    mao(&mut h, PointerKind::Down, dentro);
    mao(&mut h, PointerKind::Up, dentro);
    let _ = h.bus.drain().count();
    let x = insp_ids::INSP_TRANSFORM_POS_X;
    h.store.set_number_value(x, 250.0);
    h.apply_event(WidgetEvent::ValueChanged(x));
    let pedido = h.bus.drain().find_map(|a| match a {
        EditorAction::InspectorTransformEdit(i) => Some(i),
        _ => None,
    });
    let pedido = pedido.expect("o gesto no preso não pediu nada");

    let alcas_de_b = |w: &Mundo| {
        (alcas::view(&w.sim, &w.scene, &w.map, &[b], Gate::SHAPE, PX, None, None))
            .expect("as alças de B")
            .points
    };
    let (k0, b0, pose_b) = (
        w.canto(),
        alcas_de_b(&w),
        world_transform(&w.sim, b_e).translation,
    );
    let pose = Transform::de_autoria(
        pedido.translation,
        pedido.rotation_rad,
        pedido.scale,
        pedido.skew_rad,
    );
    let alvo = Entity::from_bits(pedido.entity_bits);
    w.sim.world_mut().entity_mut(alvo).insert(pose);
    assert!(longe(k0, w.canto()) > 1.0, "A não andou");
    assert_eq!(world_transform(&w.sim, b_e).translation, pose_b, "B andou");
    assert_eq!(alcas_de_b(&w), b0, "as alças de B andaram");
    w.confere("a forma do preso");
}

/// O Set Center num círculo perfeito não volta ao centro no quadro seguinte, e as alças ficam na forma.
#[test]
fn o_set_center_de_um_circulo_fica() {
    let mut w = mundo(ShapeKind::Ellipse, Pose::Solta);
    let (id, e) = (w.id, w.e);
    ph2d_vec_entities::receita::resize_recipe(&mut w.sim, e, 2.0, 2.0);
    let forma = w.sim.world().get::<VecShape>(e).cloned().expect("receita");
    ph2d_app_vec::shape_live::recook_into(&mut w.scene, id, &forma);
    assert!(move_origin_to(&mut w.sim, &mut w.scene, e, id, [1.5, 0.75]));
    ph2d_vec_entities::transform::settle_origins(&mut w.sim, &mut w.scene, &w.map, &[]);
    let t = world_transform(&w.sim, e).translation;
    assert!(
        (t.x - 1.5).abs() < 1e-5 && (t.y - 0.75).abs() < 1e-5,
        "o centro voltou: {t:?}"
    );
    w.confere("Set Center no círculo");
}

/// Conector, blend, morph, envelope e pele são geometria de mundo: a pose não os move.
#[test]
fn quem_nao_anda_pela_pose_continua_nos_pontos() {
    let mut w = mundo(ShapeKind::RoundRect, Pose::Solta);
    w.sim
        .world_mut()
        .entity_mut(w.e)
        .insert(ph2d_ecs::VecBlend::new(Vec::new(), 1));
    let (xf, id) = (w.xforms(), w.id);
    let pose = world_transform(&w.sim, w.e).translation;
    let antes = w.pontos();
    assert!(ct::mover_no_mundo(
        &mut w.sim,
        &mut w.scene,
        &w.map,
        &xf,
        id,
        [1.0, 0.0]
    ));
    assert_eq!(world_transform(&w.sim, w.e).translation, pose);
    assert!(
        longe(antes[0], w.pontos()[0]) > 50.0,
        "os pontos não andaram"
    );
}

fn estado(w: &Mundo) -> (ph2d_vec_scene::VecPath, Transform, VecShape) {
    let g = w.sim.world();
    (
        w.scene.path(w.id).expect("o caminho").clone(),
        *g.get::<Transform>(w.e).expect("a pose"),
        g.get::<VecShape>(w.e).expect("a receita").clone(),
    )
}

fn repoe(w: &mut Mundo, (p, t, s): (ph2d_vec_scene::VecPath, Transform, VecShape)) {
    *w.scene.path_mut(w.id).expect("o caminho") = p;
    w.sim.world_mut().entity_mut(w.e).insert((t, s));
}

/// O que o Ctrl+Z e o Ctrl+Y restauram (caminho, pose e receita): as alças voltam com a forma, por cima
/// dela, nos dois sentidos. A fila da shell precisa de janela; o que ela devolve é este estado.
/// *Mutação: as alças sem `desvio_da_receita` ⇒ vermelho no refazer.*
#[test]
fn desfazer_e_refazer_devolvem_as_alcas_com_a_forma() {
    for kind in formas() {
        for pose in POSES {
            let mut c = Cartao::novo(true);
            let mut w = mundo(kind, pose);
            let (antes, alcas_antes) = (estado(&w), w.alcas());
            c.escreve(&mut w, t::VECTOR_TRANSFORM_X, 138.0);
            c.escreve(&mut w, t::VECTOR_TRANSFORM_W, 90.0);
            let (depois, alcas_depois) = (estado(&w), w.alcas());
            repoe(&mut w, antes);
            w.confere("desfazer");
            for (a, b) in w.alcas().iter().zip(&alcas_antes) {
                assert!(
                    longe(*a, *b) < FOLGA_PX,
                    "{kind:?}: o desfazer {a:?} × {b:?}"
                );
            }
            repoe(&mut w, depois);
            w.confere("refazer");
            for (a, b) in w.alcas().iter().zip(&alcas_depois) {
                assert!(
                    longe(*a, *b) < FOLGA_PX,
                    "{kind:?}: o refazer {a:?} × {b:?}"
                );
            }
        }
    }
}

/// O Reset do cartão Transform escreve a pose de fábrica (`INSP_TRANSFORM_RESET`): as alças vão com ela,
/// mesmo com a forma movida pelo X, que deixou desvio.
#[test]
fn o_reset_leva_as_alcas() {
    for kind in formas() {
        for pose in POSES {
            let mut c = Cartao::novo(true);
            let mut w = mundo(kind, pose);
            c.escreve(&mut w, t::VECTOR_TRANSFORM_X, 138.0);
            c.escreve(&mut w, t::VECTOR_TRANSFORM_H, 75.0);
            w.sim
                .world_mut()
                .entity_mut(w.e)
                .insert(Transform::default());
            w.confere("reset");
        }
    }
}

/// ⭐ A Timeline pela fila real: a curva escreve em `CanaisEscritos`, e `canais::aplicar` drena-a como a
/// shell. A forma movida fica no sítio, e as alças por cima, quando o parâmetro muda.
/// *Mutação: `devolver_forma` a chamar `recook_into` ⇒ a forma salta para a origem, vermelho.*
#[test]
fn a_curva_de_um_parametro_deixa_a_forma_no_sitio() {
    use ph2d_ecs::canal::{CanaisEscritos, FieldKey, familia::VETOR};
    for kind in formas() {
        for pose in POSES {
            let mut c = Cartao::novo(true);
            let mut w = mundo(kind, pose);
            c.escreve(&mut w, t::VECTOR_TRANSFORM_X, 138.0);
            let VecShape::Param { values, .. } = estado(&w).2 else {
                panic!("a receita");
            };
            let Some(v) = values.first().copied() else {
                continue;
            };
            let (forma_antes, n_antes) = (estado(&w).2, w.pontos().len());
            let caixa = caixa_de_ancoras(w.pontos().into_iter());
            let chave = FieldKey::new(VETOR, ph2d_ecs::canal_vetor::PARAM_VALOR, 0, 0);
            let bits = w.e.to_bits();
            (w.sim.world_mut())
                .get_resource_or_insert_with(CanaisEscritos::default)
                .escrever(bits, chave, (v * 0.5 + 0.13) as f32);
            let mut conducao = ph2d_app_vec::canais::ConducaoVetor::default();
            let _ = ph2d_app_vec::canais::aplicar(&mut w.sim, &mut w.scene, &mut conducao);
            assert_ne!(estado(&w).2, forma_antes, "{kind:?}: a curva não chegou");
            w.confere("a curva");
            if kind == ShapeKind::RoundRect && w.pontos().len() == n_antes {
                let agora = caixa_de_ancoras(w.pontos().into_iter());
                assert!(
                    longe(caixa, agora) < FOLGA_PX,
                    "{pose:?}: a forma saltou: {caixa:?} × {agora:?}"
                );
            }
        }
    }
}

/// Os pontos do degradê moram no local do caminho: mover pelo campo é pose, logo eles andam com a forma,
/// o mesmo delta no ecrã. *Mutação: o X a assar nos pontos (`mover_no_mundo` sem o ramo da pose) ⇒ vermelho.*
#[test]
fn os_pontos_do_degrade_andam_com_a_forma() {
    use ph2d_vec_scene::{GradientStop, Paint, PaintSlot, Rgba8};
    for pose in POSES {
        let mut c = Cartao::novo(true);
        let mut w = mundo(ShapeKind::RoundRect, pose);
        let inicio = [-0.5, 0.2];
        w.scene.path_mut(w.id).expect("o caminho").fill = Some(Paint::Linear {
            stops: vec![
                GradientStop::new(0.0, Rgba8::new(255, 0, 0, 255)),
                GradientStop::new(1.0, Rgba8::new(0, 0, 255, 255)),
            ],
            start: inicio,
            end: [0.5, -0.2],
        });
        let (k0, h0) = (w.canto(), tela(w.xf().apply(inicio)));
        c.escreve(&mut w, t::VECTOR_TRANSFORM_X, 138.0);
        let (k1, h1) = (w.canto(), tela(w.xf().apply(inicio)));
        assert!(longe(k0, k1) > 1.0, "{pose:?}: o X não moveu");
        assert!(
            longe(
                [k1[0] - k0[0], k1[1] - k0[1]],
                [h1[0] - h0[0], h1[1] - h0[1]]
            ) < FOLGA_PX,
            "{pose:?}: a alça do degradê ficou para trás"
        );
        let p = w.scene.path(w.id).expect("o caminho");
        assert_eq!(
            ph2d_vec_render::hit_gradient_handle(p, PaintSlot::Base, inicio[0], inicio[1], 1e-3),
            Some(ph2d_vec_render::GradHandle::LinearStart),
            "{pose:?}: a alça não responde onde está"
        );
    }
}
