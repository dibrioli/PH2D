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
    area: Rect,
}

impl Mesa {
    /// A cena do kart solto e a aba aberta pelo pedido do Inspector (`0` = nenhuma montagem).
    fn aberta() -> Self {
        let mut m = Self {
            sim: SimWorld::new(),
            hero: HeroScreen::new(NodeId(1)),
            ts: TextSystem::without_system_fonts(),
            area: AREA,
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
            f.paint(id, self.area, &mut scene, &mut self.ts, &Theme::default());
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
        let line = self.tab().doc.dropping_line().unwrap_or_default();
        if ok {
            assert!(surface_input::drop(&mut self.hero, Self::at(x, y, &names)));
        }
        surface_input::drop_hover(&mut self.hero, None);
        self.frame();
        (ok, line)
    }

    fn click(&mut self, h: &Hit) {
        let r = self
            .tab()
            .doc
            .hits_for(h)
            .unwrap_or_else(|| panic!("{h:?} não pintado"));
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.ptr(PointerKind::Down, x, y);
        self.ptr(PointerKind::Up, x, y);
    }

    /// Um evento do rato pela porta da aba, e o quadro a seguir.
    fn ptr(&mut self, kind: PointerKind, x: f32, y: f32) {
        let ts = &mut self.ts;
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            let input = PointerInput {
                text: ts,
                mods: Modifiers::default(),
                now_ns: 0,
                primary_held: kind != PointerKind::Up,
            };
            f.pointer(id, input, kind, PointerButton::Primary, x, y);
        }
        self.frame();
    }

    /// Prende o centro de `h`, arrasta `(dx, dy)` e solta.
    fn drag(&mut self, h: &Hit, dx: f32, dy: f32) {
        let r = self
            .tab()
            .doc
            .hits_for(h)
            .unwrap_or_else(|| panic!("{h:?}"));
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.ptr(PointerKind::Down, x, y);
        self.ptr(PointerKind::Move, x + dx, y + dy);
        self.ptr(PointerKind::Up, x + dx, y + dy);
    }

    fn rect(&self, h: &Hit) -> Rect {
        self.tab()
            .doc
            .hits_for(h)
            .unwrap_or_else(|| panic!("{h:?} pintado"))
    }

    /// O carro do kart montado inteiro.
    fn carro() -> Self {
        let mut m = Self::aberta();
        m.drop("Kart Body", 0);
        m.drop("Kart Rear Wheel", 1);
        m.drop("Kart Front Wheel", 2);
        m
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
        q.iter(w)
            .find(|(n, _)| n.as_str() == name)
            .map(|(_, s)| s.0)
            .expect(name)
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
    assert!(
        line.contains("Kart Body") && line.contains("Body"),
        "a promessa: {line}"
    );
    assert!(m.joints().is_empty(), "o corpo sozinho não é junta nenhuma");
    assert!(
        m.tab().doc.assembly().is_some(),
        "a aba abre a montagem que nasceu"
    );
    m.drop("Kart Front Wheel", 2);
    m.drop("Kart Rear Wheel", 1);
    let j = m.joints();
    assert_eq!(j.len(), 2, "duas rodas, duas juntas");
    let (body, rear, front) = (
        m.sid("Kart Body"),
        m.sid("Kart Rear Wheel"),
        m.sid("Kart Front Wheel"),
    );
    assert_eq!(
        (j[0].role, j[0].kind, j[0].body_a, j[0].body_b),
        (0, JointKind::Wheel, body, rear)
    );
    assert_eq!(
        (j[1].role, j[1].kind, j[1].body_a, j[1].body_b),
        (1, JointKind::Wheel, body, front)
    );
    assert!(
        j[0].motor_enabled && !j[1].motor_enabled,
        "o motor é o de fábrica: a roda de trás puxa"
    );
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
    m.tab_mut()
        .doc
        .commit(ph2d_app_joint_editor::doc::Field::Stiffness, 77.0);
    m.frame();
    let j = m.joints();
    assert!(
        (j[1].stiffness - 77.0).abs() < 1e-3 && (j[0].stiffness - 77.0).abs() > 1e-3,
        "só a da frente"
    );
    m.click(&Hit::Scope(0));
    m.tab_mut()
        .doc
        .commit(ph2d_app_joint_editor::doc::Field::Stiffness, 55.0);
    m.frame();
    let j = m.joints();
    assert!(
        j.iter().all(|j| (j.stiffness - 55.0).abs() < 1e-3),
        "as duas"
    );
    m.click(&Hit::Inspect);
    let sel = m.hero.gizmo.selection.expect("uma seleção");
    let e = ph2d_ecs::Entity::from_bits(sel);
    assert!(
        m.sim.world().get::<PhysicsJoint>(e).is_some(),
        "a junta fica selecionada no Inspector"
    );
}

/// ⭐ **O painel é uma coluna à direita** (dono 13/11): da largura do Inspector, a toda a altura da aba, à
/// direita do palco, que fica com a altura inteira (sem faixa em cima nem painel em baixo). Os números do
/// carro estão todos à vista e nenhum passa de metade do painel (dois por fila).
#[test]
fn o_painel_e_uma_coluna_a_direita_e_os_campos_ocupam_meia_largura() {
    use ph2d_app_joint_editor::doc::Field;
    use ph2d_card_stack::metrics as cm;
    let mut m = Mesa::aberta();
    m.drop("Kart Body", 0);
    m.drop("Kart Rear Wheel", 1);
    m.drop("Kart Front Wheel", 2);
    let (p, st, area) = (m.tab().doc.panel, m.tab().doc.stage, m.tab().doc.area);
    let full = area.h - cm::CARD_GAP * 2.0;
    let ok = |a: f32, b: f32| (a - b).abs() < 0.5;
    assert!(
        p.x >= st.x + st.w,
        "o painel ({p:?}) à direita do palco ({st:?})"
    );
    assert!(
        ok(p.w, m.tab().doc.panel_width()) && ok(p.w, ph2d_tokens::INSPECTOR_W_PX),
        "nasce com a largura do Inspector: {p:?}"
    );
    assert!(
        ok(p.h, full) && ok(st.h, full) && p.y == st.y,
        "a toda a altura {full}: {p:?} {st:?}"
    );
    use Field::{Damping, LimitMax, LimitMin, MotorForce, MotorSpeed, Stiffness, Wheelbase};
    let fields = [
        Stiffness, Damping, LimitMin, LimitMax, MotorSpeed, MotorForce, Wheelbase,
    ];
    let more = [
        Hit::OpenMenu(ph2d_app_joint_editor::doc::Menu::Kind),
        Hit::Inspect,
        Hit::Scope(0),
    ];
    for h in fields.map(Hit::Field).into_iter().chain(more) {
        let r = m
            .tab()
            .doc
            .hits_for(&h)
            .unwrap_or_else(|| panic!("{h:?} pintado"));
        assert!(
            r.x >= p.x && r.x + r.w <= p.x + p.w,
            "{h:?} dentro do painel"
        );
        let narrow = !matches!(h, Hit::Field(_)) || r.w <= p.w * 0.5;
        assert!(narrow, "{h:?} mede {} px", r.w);
    }
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

/// ⭐ **O idioma do Inspector** (dono 13/11, W3d): cada caixa tem o rótulo pequeno por cima (o vão entre o
/// título do cartão e a 1.ª caixa leva-o), Joint e Template na MESMA fila, os números aos pares e o resumo
/// com «Inspector» numa fila só.
#[test]
fn cada_caixa_tem_o_rotulo_por_cima_e_joint_e_template_partilham_a_fila() {
    use ph2d_app_joint_editor::doc::{Field, Menu};
    use ph2d_card_stack::metrics as cm;
    let m = Mesa::carro();
    let (k, t) = (
        m.rect(&Hit::OpenMenu(Menu::Kind)),
        m.rect(&Hit::OpenMenu(Menu::Template)),
    );
    assert!(
        k.y == t.y && k.x + k.w < t.x,
        "Joint {k:?} e Template {t:?} lado a lado"
    );
    let (s, d) = (
        m.rect(&Hit::Field(Field::Stiffness)),
        m.rect(&Hit::Field(Field::Damping)),
    );
    assert!(
        s.y == d.y && s.x + s.w < d.x,
        "Stiffness e Damping na mesma fila"
    );
    let head = m.rect(&Hit::Fold("joint_editor.group.suspension"));
    assert!(
        s.y - (head.y + head.h) >= cm::LABEL_PX + cm::LABEL_BOTTOM,
        "o rótulo cabe entre o título {head:?} e a caixa {s:?}"
    );
    let assembly = m.rect(&Hit::Fold("joint_editor.group.assembly"));
    assert!(
        k.y - (assembly.y + assembly.h) >= cm::LABEL_PX,
        "o rótulo Joint por cima"
    );
    let i = m.rect(&Hit::Inspect);
    let p = m.tab().doc.panel;
    assert!(
        i.w < p.w * 0.5 && i.y > k.y,
        "o botão é compacto, à direita do resumo: {i:?}"
    );
}

/// ⭐ **A divisória muda a largura** dentro de [a do Inspector, metade da aba]; o palco cede.
#[test]
fn arrastar_a_divisoria_muda_a_largura_dentro_dos_limites() {
    let mut m = Mesa::carro();
    let (lo, hi) = m.tab().doc.panel_bounds();
    let w0 = m.tab().doc.panel.w;
    assert!(
        (w0 - ph2d_tokens::INSPECTOR_W_PX).abs() < 0.5,
        "nasce com a do Inspector"
    );
    assert!(
        (lo - ph2d_tokens::PANEL_MIN_W_PX).abs() < 0.5,
        "estreita até ao piso dos painéis, como o Inspector"
    );
    m.drag(&Hit::Split, -120.0, 0.0);
    let w1 = m.tab().doc.panel.w;
    assert!((w1 - (w0 + 120.0)).abs() < 0.5, "{w0} → {w1}");
    let st = m.tab().doc.stage;
    assert!(st.x + st.w <= m.tab().doc.panel.x, "o palco cede");
    m.drag(&Hit::Split, -5000.0, 0.0);
    assert!(
        (m.tab().doc.panel.w - hi).abs() < 0.5,
        "presa a metade da aba"
    );
    m.drag(&Hit::Split, 5000.0, 0.0);
    assert!(
        (m.tab().doc.panel.w - lo).abs() < 0.5,
        "presa ao piso dos painéis"
    );
}

/// ⭐ **A coluna que não cabe rola** pela roda e pela barra arrastada, e o que passa da borda não se clica
/// (recortado no painel). Recolher um cartão esconde os campos dele.
#[test]
fn a_roda_e_a_barra_rolam_e_recolher_esconde_os_campos() {
    use ph2d_app_joint_editor::doc::Field;
    let mut m = Mesa::carro();
    m.area = Rect { h: 420.0, ..AREA };
    m.frame();
    let p = m.tab().doc.panel;
    let inside = |m: &Mesa| {
        m.tab()
            .doc
            .hits_for(&Hit::Field(Field::Wheelbase))
            .is_none_or(|r| r.y >= p.y && r.y + r.h <= p.y + p.h)
    };
    assert!(
        m.tab().doc.hits_for(&Hit::Bar).is_some(),
        "não cabe: há barra"
    );
    assert!(inside(&m));
    let (x, y) = (p.x + p.w * 0.5, p.y + p.h * 0.5);
    if let Some((f, id)) = m.hero.documents.active_surface_mut() {
        assert!(f.wheel(id, x, y, -60.0, false), "a roda rola");
    }
    m.frame();
    let s1 = m.tab().doc.scroll();
    assert!(s1 > 0.0 && inside(&m));
    m.drag(&Hit::Bar, 0.0, 40.0);
    assert!(
        m.tab().doc.scroll() > s1,
        "a barra arrastada rola: {s1} → {}",
        m.tab().doc.scroll()
    );
    m.area = AREA;
    m.frame();
    assert!(
        m.tab()
            .doc
            .hits_for(&Hit::Field(Field::Stiffness))
            .is_some()
    );
    m.click(&Hit::Fold("joint_editor.group.suspension"));
    assert!(m.tab().doc.is_folded("joint_editor.group.suspension"));
    for f in [
        Field::Stiffness,
        Field::Damping,
        Field::LimitMin,
        Field::LimitMax,
    ] {
        assert!(
            m.tab().doc.hits_for(&Hit::Field(f)).is_none(),
            "{f:?} recolhido"
        );
    }
    m.click(&Hit::Fold("joint_editor.group.suspension"));
    assert!(
        m.tab()
            .doc
            .hits_for(&Hit::Field(Field::Stiffness))
            .is_some(),
        "reabre"
    );
}
