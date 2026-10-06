//! Os gates da família do esqueleto (A14) — as leis puras e o quadro do modo conduzido pelo `drive`
//! real, com a ferramenta de osso registada (o arnês é o da família do vetor).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::object_mode::ModeRequest;
use ph2d_editor_core::screens::hero::mode_drive::drive;
use ph2d_editor_core::toast::ToastQueue;
use ph2d_editor_core::tool::Tool;

struct Move;

impl Tool for Move {
    fn id(&self) -> ToolId {
        ToolId::new("move")
    }
    fn label(&self) -> &str {
        "move"
    }
    fn icon_slug(&self) -> &str {
        "move"
    }
    fn build_panel(&self) -> FloatingPanel {
        FloatingPanel::new(self.id(), "move")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct Cena {
    sim: SimWorld,
    state: SkeletonState,
    tools: ToolRegistry,
    hero: HeroScreen,
    toasts: ToastQueue,
}

fn cena() -> Cena {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(ph2d_tool_bone::make());
    tools.activate_default();
    Cena {
        sim: SimWorld::default(),
        state: SkeletonState::default(),
        tools,
        hero: HeroScreen::new(ph2d_editor_core::NodeId(1)),
        toasts: ToastQueue::new(),
    }
}

impl Cena {
    /// Um esqueleto pelo menu Add — devolve `(esqueleto, osso)`.
    fn esqueleto(&mut self) -> (u64, u64) {
        let s = crate::object_add::add(crate::object_add::SKELETON, &mut self.sim, &mut self.state)
            .expect("é desta família")
            .expect("nasce");
        let osso = ph2d_skeleton_ecs::bones_of(self.sim.world(), Entity::from_bits(s))[0];
        (s, osso.to_bits())
    }
    /// ⚠️ O lado INDEPENDENTE do tipo: escrito aqui, pelo marcador, e não lido do `kind_of`.
    fn quadro(&mut self, req: Option<ModeRequest>) {
        let world = self.sim.world();
        let kind_of = |b: u64| {
            if world
                .get::<ph2d_skeleton_ecs::Skeleton>(Entity::from_bits(b))
                .is_some()
            {
                ObjectKind::Skeleton
            } else {
                ObjectKind::Empty
            }
        };
        let name_of = |_| String::new();
        let selected = self.hero.gizmo.selection;
        let mut fam = Family::new(&mut self.state, world, selected);
        drive(
            &mut [&mut fam],
            &kind_of,
            &name_of,
            &mut self.tools,
            &mut self.hero,
            &mut self.toasts,
            req,
        );
    }
    fn modo(&self) -> Option<(u64, ObjectMode)> {
        self.hero.gizmo.mode.active().map(|a| (a.entity, a.mode))
    }
    fn verbo(&mut self) -> Option<BoneAction> {
        verb_in_hand(&mut self.tools)
    }
    fn selecao(&self) -> Vec<u64> {
        self.hero.gizmo.iter_selected().collect()
    }
}

/// ⭐⭐ GATE (A16) — **entrar no Edit/Pose com um OSSO escolhido mantém o osso** (ele é o pai do
/// próximo osso e o que o painel mostra): pelo `Tab`, pelo seletor e pela ferramenta posta na mão
/// em Object. Controlo: com o ESQUELETO escolhido a selecção é o esqueleto.
#[test]
fn entering_edit_or_pose_keeps_the_chosen_bone() {
    let mut c = cena();
    let (s, osso) = c.esqueleto();
    c.quadro(None);
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.modo(), None, "controlo: em Object");
    c.hero.gizmo.replace_selection(Some(s));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.modo(), Some((s, ObjectMode::Edit)));
    assert_eq!(c.selecao(), vec![s], "controlo: o esqueleto escolhido");
    c.quadro(Some(ModeRequest::Toggle));
    c.hero.gizmo.replace_selection(Some(osso));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.modo(), Some((s, ObjectMode::Edit)));
    assert_eq!(
        c.selecao(),
        vec![osso],
        "o Tab trocou o osso pelo esqueleto"
    );
    c.quadro(None);
    assert_eq!(
        c.modo(),
        Some((s, ObjectMode::Edit)),
        "o modo caiu com o osso"
    );
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Pose)));
    assert_eq!(c.modo(), Some((s, ObjectMode::Pose)));
    assert_eq!(c.selecao(), vec![osso], "o seletor trocou o osso");
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.modo(), None);
    c.hero.gizmo.replace_selection(Some(osso));
    bone_bridge::arm(&mut c.tools, BoneAction::Weight);
    c.quadro(None);
    assert_eq!(c.modo(), Some((s, ObjectMode::Pose)));
    assert_eq!(c.selecao(), vec![osso], "a ferramenta na mão trocou o osso");
}

/// ⭐ GATE — as leis, puras.
#[test]
fn the_pure_laws() {
    assert_eq!(mode_of(BoneAction::Create), ObjectMode::Edit);
    assert_eq!(mode_of(BoneAction::Transform), ObjectMode::Pose);
    assert_eq!(mode_of(BoneAction::Weight), ObjectMode::Pose);
    assert_eq!(verb_of(ObjectMode::Edit, None), Some(BoneAction::Create));
    assert_eq!(verb_of(ObjectMode::Pose, None), Some(BoneAction::Transform));
    assert_eq!(
        verb_of(ObjectMode::Pose, Some(BoneAction::Weight)),
        Some(BoneAction::Weight),
        "o Pose volta ao último verbo dele"
    );
    assert_eq!(verb_of(ObjectMode::Object, None), None);
    assert!(holds(ObjectMode::Pose, true, Some(BoneAction::Weight)));
    assert!(
        !holds(ObjectMode::Pose, true, Some(BoneAction::Create)),
        "verbo do Edit"
    );
    assert!(
        !holds(ObjectMode::Pose, false, Some(BoneAction::Transform)),
        "outro esqueleto"
    );
    assert!(!holds(ObjectMode::Edit, true, None), "sem a ferramenta");
    assert_eq!(
        adopt(Some(BoneAction::Create), None),
        Some(ObjectMode::Edit)
    );
    assert_eq!(
        adopt(Some(BoneAction::Create), Some(ObjectMode::Edit)),
        None,
        "já seguia"
    );
    assert_eq!(
        adopt(Some(BoneAction::Weight), Some(ObjectMode::Edit)),
        Some(ObjectMode::Pose),
        "o verbo mudou de modo"
    );
    assert_eq!(adopt(None, None), None);
}

/// ⭐⭐ GATE — **o fluxo do Add**: o esqueleto nasce, entra em Edit com a ferramenta de osso na mão
/// e o verbo *Create*; `Tab` volta a Object e LARGA a ferramenta (o gizmo move o esqueleto); a linha
/// *Pose* do seletor entra no Pose com *Transform*. Controlo: antes do quadro, nada na mão.
#[test]
fn add_enters_edit_with_create_and_tab_and_pose_follow() {
    let mut c = cena();
    let (s, _) = c.esqueleto();
    assert_eq!(c.verbo(), None, "controlo: o Add sozinho não arma nada");
    c.quadro(None);
    assert_eq!(
        c.modo(),
        Some((s, ObjectMode::Edit)),
        "o esqueleto novo não entrou em Edit"
    );
    assert_eq!(c.verbo(), Some(BoneAction::Create));
    assert_eq!(c.state.target, Some(s));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.modo(), None, "o Tab não voltou a Object");
    assert_eq!(c.verbo(), None, "Object não largou a ferramenta de osso");
    assert_eq!(c.state.target, None);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Pose)));
    assert_eq!(c.modo(), Some((s, ObjectMode::Pose)));
    assert_eq!(c.verbo(), Some(BoneAction::Transform));
}

/// ⭐⭐ GATE — **os ossos são PARTES**: seleccionar um osso do esqueleto em Pose mantém o Pose.
/// Controlo: seleccionar um objecto de fora larga-o.
#[test]
fn selecting_a_bone_of_the_skeleton_keeps_pose_and_a_foreign_object_does_not() {
    let mut c = cena();
    let (s, osso) = c.esqueleto();
    c.quadro(None);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Pose)));
    assert_eq!(c.modo(), Some((s, ObjectMode::Pose)), "controlo: em Pose");
    c.hero.gizmo.replace_selection(Some(osso));
    c.quadro(None);
    assert_eq!(
        c.modo(),
        Some((s, ObjectMode::Pose)),
        "o osso seleccionado tirou do Pose"
    );
    let fora = c
        .sim
        .world_mut()
        .spawn(ph2d_ecs::Transform::IDENTITY)
        .id()
        .to_bits();
    c.hero.gizmo.replace_selection(Some(fora));
    c.quadro(None);
    assert_ne!(
        c.modo(),
        Some((s, ObjectMode::Pose)),
        "um objecto de fora ficou no Pose"
    );
}

/// ⭐⭐ GATE — **um osso seleccionado responde pelo esqueleto dono**: em Object, o `Tab` sobre ele
/// entra no Edit do esqueleto (o `owner_of`).
#[test]
fn tab_on_a_bone_enters_the_edit_of_its_skeleton() {
    let mut c = cena();
    let (s, osso) = c.esqueleto();
    c.quadro(None);
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.modo(), None, "controlo: em Object");
    c.hero.gizmo.replace_selection(Some(osso));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.modo(), Some((s, ObjectMode::Edit)));
}

/// ⭐⭐ GATE — **o modo é a verdade do verbo**: em Edit, o segmento *Transform* do painel (que troca o
/// verbo na ferramenta) leva ao Pose; e a ferramenta posta na mão por outra porta, em Object, abre o
/// modo do verbo dela sobre o esqueleto do osso seleccionado.
#[test]
fn the_verb_in_hand_brings_its_mode() {
    let mut c = cena();
    let (s, osso) = c.esqueleto();
    c.quadro(None);
    assert_eq!(c.modo(), Some((s, ObjectMode::Edit)), "controlo: em Edit");
    bone_bridge::arm(&mut c.tools, BoneAction::Transform);
    c.quadro(None);
    c.quadro(None);
    assert_eq!(
        c.modo(),
        Some((s, ObjectMode::Pose)),
        "o verbo do Pose não levou ao Pose"
    );
    assert_eq!(c.verbo(), Some(BoneAction::Transform));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.modo(), None);
    c.hero.gizmo.replace_selection(Some(osso));
    bone_bridge::arm(&mut c.tools, BoneAction::Weight);
    c.quadro(None);
    assert_eq!(
        c.modo(),
        Some((s, ObjectMode::Pose)),
        "a porta antiga não abriu o Pose"
    );
    assert_eq!(
        c.verbo(),
        Some(BoneAction::Weight),
        "a entrada trocou o verbo que trouxe a ferramenta"
    );
}

/// ⭐⭐ GATE (sobrevivente M6 da mutação) — **o modo que acaba SEM `leave` também larga a
/// ferramenta**: apagar o esqueleto em Pose tira-o do modo, e a ferramenta de osso não pode ficar na
/// mão a criar ossos soltos. Controlo: antes de apagar, em Pose com ela na mão.
#[test]
fn deleting_the_skeleton_in_pose_releases_the_bone_tool() {
    let mut c = cena();
    let (s, osso) = c.esqueleto();
    c.quadro(None);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Pose)));
    assert_eq!(c.verbo(), Some(BoneAction::Transform), "controlo: na mão");
    c.sim.world_mut().despawn(Entity::from_bits(osso));
    c.sim.world_mut().despawn(Entity::from_bits(s));
    c.quadro(None);
    c.quadro(None);
    assert_eq!(c.modo(), None, "o modo sobreviveu ao esqueleto");
    assert_eq!(
        c.verbo(),
        None,
        "a ferramenta de osso ficou na mão sem esqueleto"
    );
}
