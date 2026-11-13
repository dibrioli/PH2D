//! **A costura da aba Joint Editor** (plano §W3): a família ligada a um `HeroScreen` de teste e dirigida
//! pelas portas reais — o pedido do Inspector, o `frame`, a pintura, a queda de um objeto pela porta da
//! queda entre painéis (`surface_input::drop_hover` / `drop`, a mesma do arrasto da Hierarchy) e o clique.

use ph2d_a11y::NodeId;
use ph2d_app_joint_editor::{Hit, JointEditorTab, frame, install, smoke};
use ph2d_ecs::{Name, SimWorld, StableId};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::doc_surface::{DropAt, Dropping, PointerInput};
use ph2d_editor_core::screens::hero::surface_input;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_physics_ecs::{JointKind, PhysicsJoint};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1400.0,
    h: 900.0,
};

struct Mesa {
    sim: SimWorld,
    hero: HeroScreen,
    ts: TextSystem,
}

impl Mesa {
    /// A cena do kart solto e a aba aberta pelo pedido do Inspector (`0` = nenhuma montagem).
    fn aberta() -> Self {
        let mut m = Self {
            sim: SimWorld::new(),
            hero: HeroScreen::new(NodeId(1)),
            ts: TextSystem::without_system_fonts(),
        };
        smoke::spawn_kart(&mut m.sim);
        install(&mut m.hero);
        m.hero.store.request_joint_editor(0);
        m.frame();
        m
    }

    fn frame(&mut self) {
        frame(&mut self.sim, &mut self.hero);
        self.hero.documents.follow_surfaces();
        let mut scene = VectorScene::new();
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            f.paint(id, AREA, &mut scene, &mut self.ts, &Theme::default());
        }
    }

    fn tab(&self) -> &JointEditorTab {
        self.hero
            .documents
            .surfaces()
            .find::<JointEditorTab>()
            .expect("a família instalada")
    }

    /// O centro, no ecrã, da forma do papel `role` no desenho pintado.
    fn shape(&self, role: u16) -> (f32, f32) {
        let r = self
            .tab()
            .doc
            .hits_for(&Hit::Shape(role, 0))
            .expect("a forma pintada");
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    fn at<'a>(x: f32, y: f32, names: &'a [String]) -> DropAt<'a> {
        DropAt {
            x,
            y,
            what: Dropping::Objects(names),
            mods: Modifiers::default(),
        }
    }

    /// Arrasta `name` da Hierarchy até ao papel e larga: devolve o que a aba prometeu antes de soltar.
    fn drop(&mut self, name: &str, role: u16) -> (bool, String) {
        let (x, y) = self.shape(role);
        let names = vec![name.to_owned()];
        let ok = surface_input::drop_hover(&mut self.hero, Some(Self::at(x, y, &names)));
        let line = self
            .tab()
            .doc
            .dropping_line()
            .unwrap_or_default();
        if ok {
            assert!(surface_input::drop(&mut self.hero, Self::at(x, y, &names)));
        }
        surface_input::drop_hover(&mut self.hero, None);
        self.frame();
        (ok, line)
    }

    fn click(&mut self, h: &Hit) {
        let r = self.tab().doc.hits_for(h).unwrap_or_else(|| panic!("{h:?} não pintado"));
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        for kind in [PointerKind::Down, PointerKind::Up] {
            let ts = &mut self.ts;
            if let Some((f, id)) = self.hero.documents.active_surface_mut() {
                let input = PointerInput {
                    text: ts,
                    mods: Modifiers::default(),
                    now_ns: 0,
                    primary_held: kind == PointerKind::Down,
                };
                f.pointer(id, input, kind, PointerButton::Primary, x, y);
            }
        }
        self.frame();
    }

    fn joints(&mut self) -> Vec<PhysicsJoint> {
        let w = self.sim.world_mut();
        let mut q = w.query::<&PhysicsJoint>();
        let mut v: Vec<PhysicsJoint> = q.iter(w).copied().collect();
        v.sort_by_key(|j| j.role);
        v
    }

    fn sid(&mut self, name: &str) -> u64 {
        let w = self.sim.world_mut();
        let mut q = w.query::<(&Name, &StableId)>();
        q.iter(w).find(|(n, _)| n.as_str() == name).map(|(_, s)| s.0).expect(name)
    }
}

/// ⭐⭐ **Arrastar as três peças para os papéis do gabarito Car faz o carro**: a montagem nasce na
/// primeira, e cada roda nasce com a junta dela, do tipo Wheel, com o papel da regra e as duas peças
/// certas. Antes de soltar, a aba diz o que vai acontecer.
///
/// Mutações que devem sangrar: trocar o papel em `drop_into` · não criar a montagem na 1.ª peça.
#[test]
fn arrastar_as_pecas_para_os_papeis_monta_o_carro() {
    let mut m = Mesa::aberta();
    assert!(m.tab().is_open(), "o pedido do Inspector abre a aba");
    let (ok, line) = m.drop("Kart Body", 0);
    assert!(ok, "{line}");
    assert!(line.contains("Kart Body") && line.contains("Body"), "a promessa: {line}");
    assert!(m.joints().is_empty(), "o corpo sozinho não é junta nenhuma");
    assert!(m.tab().doc.assembly().is_some(), "a aba abre a montagem que nasceu");
    m.drop("Kart Front Wheel", 2);
    m.drop("Kart Rear Wheel", 1);
    let j = m.joints();
    assert_eq!(j.len(), 2, "duas rodas, duas juntas");
    let (body, rear, front) = (m.sid("Kart Body"), m.sid("Kart Rear Wheel"), m.sid("Kart Front Wheel"));
    assert_eq!((j[0].role, j[0].kind, j[0].body_a, j[0].body_b), (0, JointKind::Wheel, body, rear));
    assert_eq!((j[1].role, j[1].kind, j[1].body_a, j[1].body_b), (1, JointKind::Wheel, body, front));
    assert!(j[0].motor_enabled && !j[1].motor_enabled, "o motor é o de fábrica: a roda de trás puxa");
}

/// ⛔ A queda RECUSA o que não pode — e diz porquê antes de soltar: a roda que já está no carro não
/// entra noutro papel, e nada muda no mundo.
#[test]
fn a_queda_recusada_diz_porque_e_nao_muda_nada() {
    let mut m = Mesa::aberta();
    m.drop("Kart Body", 0);
    m.drop("Kart Front Wheel", 2);
    let before = m.joints();
    let (ok, line) = m.drop("Kart Front Wheel", 1);
    assert!(!ok);
    assert!(line.contains("already"), "{line}");
    assert_eq!(m.joints(), before);
}

/// ⭐ O painel de baixo edita as DUAS rodas pelo «Both wheels» e uma só pela escolha dela; «Open in
/// Inspector» seleciona a junta na cena.
#[test]
fn o_painel_edita_as_duas_rodas_ou_uma_e_abre_o_inspector() {
    let mut m = Mesa::aberta();
    m.drop("Kart Body", 0);
    m.drop("Kart Rear Wheel", 1);
    m.drop("Kart Front Wheel", 2);
    // «Front wheel» (a 3.ª opção): desliga o «Same as» e o número vale só para ela.
    m.click(&Hit::Scope(2));
    m.tab_mut().doc.commit(ph2d_app_joint_editor::doc::Field::Stiffness, 77.0);
    m.frame();
    let j = m.joints();
    assert!((j[1].stiffness - 77.0).abs() < 1e-3 && (j[0].stiffness - 77.0).abs() > 1e-3, "só a da frente");
    m.click(&Hit::Scope(0));
    m.tab_mut().doc.commit(ph2d_app_joint_editor::doc::Field::Stiffness, 55.0);
    m.frame();
    let j = m.joints();
    assert!(j.iter().all(|j| (j.stiffness - 55.0).abs() < 1e-3), "as duas");
    m.click(&Hit::Inspect);
    let sel = m.hero.gizmo.selection.expect("uma seleção");
    let e = ph2d_ecs::Entity::from_bits(sel);
    assert!(m.sim.world().get::<PhysicsJoint>(e).is_some(), "a junta fica selecionada no Inspector");
}

impl Mesa {
    fn tab_mut(&mut self) -> &mut JointEditorTab {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<JointEditorTab>()
            .expect("a família instalada")
    }
}
