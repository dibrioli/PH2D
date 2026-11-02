//! Os gates de Vector ▸ Edit (spec/06 F3 ▸ Vector, 2.ª volta: cada forma é um objecto) — as leis
//! puras, a família com a ferramenta do vetor DE VERDADE, e o fluxo do dono pelo MESMO quadro que a
//! shell corre (`mode_drive::drive`).

use super::*;
use ph2d_editor_core::floating_panel::FloatingPanel;
use ph2d_editor_core::screens::hero::mode_drive::drive;
use ph2d_editor_core::toast::ToastQueue;
use ph2d_editor_core::tool::Tool;
use ph2d_tool_vector::DrawMode;
use ph2d_vec_entities::entities::sync;

/// A ferramenta de omissão.
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

pub(super) struct Cena {
    pub(super) sim: SimWorld,
    pub(super) scene: ph2d_vec_scene::VecScene,
    pub(super) vec: VecState,
    pub(super) tools: ToolRegistry,
    pub(super) hero: HeroScreen,
    pub(super) toasts: ToastQueue,
}

pub(super) fn cena() -> Cena {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(Box::new(VectorTool::default()));
    tools.activate_default();
    Cena {
        sim: SimWorld::default(),
        scene: ph2d_vec_scene::VecScene::new(),
        vec: VecState::default(),
        tools,
        hero: HeroScreen::new(ph2d_editor_core::NodeId(1)),
        toasts: ToastQueue::new(),
    }
}

impl Cena {
    /// Uma forma nova (o que um traço deixa), já com entidade — devolve os bits dela.
    fn forma(&mut self, x: f64) -> u64 {
        let id = self
            .scene
            .push_path(ph2d_vec_scene::rectangle([x, 0.0], [x + 1.0, 1.0]));
        sync(&mut self.sim, &mut self.scene, &mut self.vec.entities);
        self.vec.entities[&id]
    }
    /// ⚠️ O lado INDEPENDENTE do tipo: escrito aqui, pelo marcador, e não lido do `kind_of`.
    pub(super) fn quadro(&mut self, req: Option<ModeRequest>) {
        let sim = &self.sim;
        let kind_of = |b: u64| {
            if sim
                .world()
                .get::<ph2d_ecs::VecPathRef>(Entity::from_bits(b))
                .is_some()
            {
                ObjectKind::Vector
            } else {
                ObjectKind::Empty
            }
        };
        let name_of = |_| String::new();
        let mut fam = Family::new(&mut self.vec, sim);
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
    pub(super) fn na_mao(&mut self) -> bool {
        in_hand(&mut self.tools)
    }
    pub(super) fn em_edit(&self) -> Option<u64> {
        self.hero
            .gizmo
            .mode
            .active()
            .filter(|a| a.mode == ObjectMode::Draw)
            .map(|a| a.entity)
    }
    pub(super) fn avisos_de_entrada(&self) -> usize {
        let entered = ph2d_i18n::tr_with("object_mode.entered", &[("mode", &"")]);
        let marca = entered.trim();
        self.toasts
            .iter()
            .filter(|t| t.message.contains(marca))
            .count()
    }
}

/// ⭐ GATE — as leis, puras: «tem em mãos», a porta antiga, a ferramenta que segue o modo e o
/// NASCIMENTO (uma só forma nova, fora de gesto, depois do 1.º quadro).
#[test]
fn the_pure_laws() {
    assert!(holds(ObjectMode::Draw, true, true));
    assert!(!holds(ObjectMode::Draw, false, true), "outra forma");
    assert!(!holds(ObjectMode::Draw, true, false), "sem a ferramenta");
    assert!(!holds(ObjectMode::Object, true, true));
    assert!(adopt(true, false));
    assert!(!adopt(true, true), "já seguia");
    assert!(!adopt(false, false), "sem a ferramenta");
    assert!(releases(false, true));
    assert!(!releases(true, true) && !releases(false, false));
    assert_eq!(newborn(&[1, 2], Some(&[1]), &[]), Some(2));
    assert_eq!(newborn(&[1, 2], Some(&[1]), &[2]), None, "ainda em gesto");
    assert_eq!(
        newborn(&[1, 2, 3], Some(&[1]), &[]),
        None,
        "várias de uma vez"
    );
    assert_eq!(newborn(&[1, 2], None, &[]), None, "antes do 1.º quadro");
    assert_eq!(newborn(&[1], Some(&[1]), &[]), None, "nada de novo");
    assert_eq!(
        heir(&[9, 1], &[1, 2], &[2, 1]),
        Some(1),
        "a última da caneta"
    );
    assert_eq!(heir(&[9, 1], &[1, 2], &[]), Some(1), "senão uma do Edit");
    assert_eq!(heir(&[9], &[1, 2], &[]), Some(2), "senão qualquer forma");
    assert_eq!(heir(&[1], &[1, 2], &[2]), None, "a trancada vive");
    assert_eq!(heir(&[9], &[], &[]), None, "não resta forma");
}

/// ⭐⭐ GATE (escolha do dono, 05/10) — **o fluxo**: *Add ▸ Vector Drawing* não cria NADA e põe a
/// ferramenta na mão; a 1.ª forma nasce objecto e entra em Edit; a 2.ª é OUTRO objecto e o Edit passa
/// a ela SEM largar a ferramenta (o `DrawMode` escolhido sobrevive) e sem repetir o aviso.
/// (Mutação: o `leave` voltar a largar a ferramenta ⇒ o `DrawMode` volta ao de fábrica ⇒ RED.)
#[test]
fn add_arms_the_tool_and_each_drawn_shape_becomes_the_object_in_edit() {
    let mut c = cena();
    c.quadro(None);
    let antes = c.sim.world().entities().len();
    assert!(crate::object_add::add(
        crate::object_add::VECTOR,
        &mut c.vec
    ));
    c.quadro(None);
    assert!(c.na_mao(), "o Add não pôs a ferramenta na mão");
    assert_eq!(
        c.sim.world().entities().len(),
        antes,
        "o Add criou uma entidade"
    );
    assert_eq!(c.em_edit(), None, "sem forma não há Edit");
    tool_mut(&mut c.tools)
        .expect("na mão")
        .set_mode(DrawMode::Pencil);
    let a = c.forma(0.0);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(a), "a 1.ª forma não entrou em Edit");
    assert_eq!(c.hero.gizmo.selection, Some(a));
    let b = c.forma(3.0);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b), "o Edit não passou à 2.ª forma");
    assert_eq!(c.hero.gizmo.selection, Some(b));
    assert!(c.na_mao(), "a passagem largou a ferramenta");
    assert_eq!(
        tool_mut(&mut c.tools).expect("na mão").mode(),
        DrawMode::Pencil,
        "a ferramenta foi largada e retomada (o modo de desenho perdeu-se)"
    );
    assert_eq!(c.avisos_de_entrada(), 1, "o aviso do Edit repetiu-se");
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), None);
    assert!(!c.na_mao(), "Object não largou a ferramenta");
}

/// ⭐⭐ GATE — **uma forma que nasce SEM a ferramenta na mão não pede o Edit** (colar, desfazer, a
/// Hierarquia em Object): o Edit é do gesto do vetor. CONTROLO: com a ferramenta, a mesma forma pede.
#[test]
fn a_shape_born_without_the_tool_does_not_ask_for_edit() {
    let mut c = cena();
    c.quadro(None);
    let a = c.forma(0.0);
    c.quadro(None);
    assert_eq!(c.em_edit(), None, "pediu o Edit sem a ferramenta");
    assert!(!c.na_mao());
    c.vec.edit.arm();
    c.quadro(None);
    let b = c.forma(3.0);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b), "controlo: com a ferramenta pede");
    assert_ne!(a, b);
}

/// ⭐⭐ GATE (dono, 05/10, 24/10 e a EXCEÇÃO de 02/11: *«se estou num objeto vector modo draw e
/// seleciono outro objeto vetor ou vários objetos vector, não deve sair do modo draw»*) — **o Draw é
/// do TIPO**: a outra forma seleccionada por uma porta que não é gesto é parte do modo e ele fica; o
/// `Tab` de volta deixa a selecção como estava; escolher um objecto de OUTRO tipo pela porta de
/// selecção volta a Object com ele seleccionado e larga a ferramenta.
/// (Mutação: `holds_the_whole_kind` = `false` ⇒ a outra forma por outra porta larga o modo ⇒ RED.)
#[test]
fn the_draw_holds_every_shape_and_choosing_another_kind_leaves_it() {
    use ph2d_editor_core::screens::hero::mode_drive::chosen;
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    let sprite = c.sim.world_mut().spawn_empty().id().to_bits();
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(a));
    assert!(c.hero.gizmo.mode.whole_kind());
    c.hero.gizmo.replace_selection(Some(b));
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        Some(a),
        "a outra forma por uma porta que não é gesto largou o Draw"
    );
    assert!(c.na_mao());
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), None);
    assert_eq!(
        (c.hero.gizmo.selection, c.hero.gizmo.selected_len()),
        (Some(b), 1),
        "sair do Draw mudou a selecção"
    );
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(b));
    chosen(&mut c.hero, Some(sprite), false);
    c.hero.gizmo.replace_selection(Some(sprite));
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        None,
        "o objecto de outro tipo não saiu do Draw"
    );
    assert_eq!(c.hero.gizmo.selection, Some(sprite));
    assert!(!c.na_mao(), "Object não largou a ferramenta");
}

/// ⭐⭐⭐ GATE (dono, 02/11 — a EXCEÇÃO Vector → Vector à regra de 24/10: *«temos boolean com vector e
/// não é legal sair do modo draw ao selecionar mais de um objeto»*) — **no Draw do vetor, escolher
/// OUTRA forma, ou várias, FICA no Draw**, por cada porta: o clique no canvas e a linha da Hierarquia
/// (`chosen` com o alvo), o laço (`lasso_chosen`) e o intervalo da Hierarquia (`range`); a caneta segue
/// a selecção (as booleanas operam sobre ela). Forma + objecto de outro tipo (laço ou intervalo) SAI,
/// com a selecção do artista. (Mutação: o ramo `stays_on_its_kind` de `switches` fora ⇒ RED.)
#[test]
fn choosing_other_vector_shapes_in_the_draw_stays_in_the_draw() {
    use ph2d_editor_core::screens::hero::mode_drive::{chosen, lasso_chosen};
    let mut c = cena();
    let (a, b, d) = (c.forma(0.0), c.forma(3.0), c.forma(6.0));
    let sprite = c.sim.world_mut().spawn_empty().id().to_bits();
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(a));
    // 1. Uma forma — o clique no canvas / a linha da Hierarquia.
    chosen(&mut c.hero, Some(b), false);
    c.hero.gizmo.replace_selection(Some(b));
    c.quadro(None);
    assert!(c.em_edit().is_some(), "escolher OUTRA forma saiu do Draw");
    assert!(c.na_mao(), "a ferramenta do vetor saiu da mão");
    assert_eq!(c.hero.gizmo.selection, Some(b));
    // 2. Várias — o Ctrl+clique (mesma porta), o laço e o intervalo da Hierarquia.
    chosen(&mut c.hero, Some(d), false);
    c.hero.gizmo.add_to_selection(d);
    c.quadro(None);
    assert!(c.em_edit().is_some(), "acrescentar uma forma saiu do Draw");
    assert_eq!(c.hero.gizmo.selected_len(), 2);
    lasso_chosen(&mut c.hero, &[a, b, d]);
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.hero.gizmo.add_to_selection(d);
    c.quadro(None);
    assert!(c.em_edit().is_some(), "o laço de três formas saiu do Draw");
    assert_eq!(c.hero.gizmo.selected_len(), 3);
    chosen(&mut c.hero, None, true);
    c.hero.gizmo.replace_selection(Some(b));
    c.hero.gizmo.add_to_selection(d);
    c.quadro(None);
    assert!(
        c.em_edit().is_some(),
        "o intervalo da Hierarquia só com formas saiu do Draw"
    );
    // 3. Forma + objecto de OUTRO tipo — pelo intervalo (a rede decide pela selecção que ficou)…
    chosen(&mut c.hero, None, true);
    c.hero.gizmo.add_to_selection(sprite);
    c.quadro(None);
    assert_eq!(c.em_edit(), None, "o intervalo com um sprite ficou no Draw");
    assert_eq!(c.hero.gizmo.selected_len(), 3, "a selecção do artista");
    assert!(!c.na_mao(), "Object não largou a ferramenta");
    // … e pelo laço.
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(a));
    lasso_chosen(&mut c.hero, &[b, sprite]);
    c.hero.gizmo.replace_selection(Some(b));
    c.hero.gizmo.add_to_selection(sprite);
    c.quadro(None);
    assert_eq!(c.em_edit(), None, "o laço com um sprite ficou no Draw");
    assert_eq!(
        (c.hero.gizmo.selection, c.hero.gizmo.selected_len()),
        (Some(b), 2)
    );
    assert!(!c.na_mao(), "Object não largou a ferramenta");
}

/// ⭐ GATE — **multi-objecto**: `Tab` com duas formas seleccionadas = Edit das duas (a activa à
/// frente), e as duas agarram-se.
#[test]
fn tab_with_two_shapes_edits_both() {
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(b), "o activo é o ÚLTIMO");
    assert_eq!(c.vec.edit.objects, vec![b, a]);
}

/// ⭐ GATE — **a porta antiga**: a ferramenta que chega à mão SEM o modo (a aba de cima, as cenas
/// `PH2D_*_SMOKE`) pede o Edit sobre a forma que a caneta tem seleccionada; sem selecção, espera.
#[test]
fn the_vector_tool_arriving_without_the_mode_asks_for_the_selected_shape() {
    let mut c = cena();
    let a = c.forma(0.0);
    c.quadro(None);
    assert!(c.tools.set_active(&ToolId::new(VECTOR)));
    c.quadro(None);
    assert_eq!(c.em_edit(), None, "sem selecção, espera");
    let id = *c.vec.entities.keys().next().expect("a forma");
    c.vec.pen.select_many(&[id]);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(a));
}

/// ⭐ GATE — **o Add não entra no Edit de uma forma VELHA**: a caneta com uma selecção antiga não
/// pode puxar o Edit pela porta antiga quando a ferramenta chega à mão — o Add espera o 1.º traço.
/// CONTROLO: a mesma selecção com a ferramenta posta à mão (sem o Add) entra
/// (`the_vector_tool_arriving_without_the_mode_asks_for_the_selected_shape`).
/// (Mutação: o `follow` não limpar a caneta ao armar ⇒ RED.)
#[test]
fn add_never_enters_the_edit_of_a_stale_pen_selection() {
    let mut c = cena();
    c.forma(0.0);
    c.quadro(None);
    let velha = *c.vec.entities.keys().next().expect("a forma");
    c.vec.pen.select_many(&[velha]);
    c.vec.edit.arm();
    c.quadro(None);
    c.quadro(None);
    assert!(c.na_mao(), "o Add não pôs a ferramenta na mão");
    assert_eq!(c.em_edit(), None, "o Add entrou no Edit da forma velha");
}

/// ⭐⭐ GATE (dono, 05/10: *«em edit mode se clicar no canvas vazio (desselecionar) sai do modo Edit.
/// Não permita isso»* e *«permita desselecionar mesmo sem sair do modo edit»*) — **desseleccionar num
/// Edit não sai dele, e a selecção FICA vazia** (num Edit de uma forma e num de duas). Apagar a
/// trancada passa o Edit à outra forma ([`heir`], sem tocar a selecção); CONTROLO: sem forma
/// nenhuma o modo cai a Object.
/// (Mutação: o `still_holds` voltar a exigir a entidade seleccionada ⇒ RED.)
#[test]
fn clearing_the_selection_in_edit_keeps_the_edit() {
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(a));
    c.hero.gizmo.replace_selection(None);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(a), "o clique no vazio saiu do Edit");
    assert_eq!(c.hero.gizmo.selection, None, "a desselecção foi desfeita");
    c.quadro(Some(ModeRequest::Toggle));
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(Some(ModeRequest::Toggle));
    c.hero.gizmo.replace_selection(None);
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        Some(b),
        "o Edit de duas caiu ao desseleccionar"
    );
    assert_eq!(c.hero.gizmo.selected_len(), 0);
    c.sim.world_mut().despawn(Entity::from_bits(b));
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        Some(a),
        "apagar a trancada largou o Edit (há outra forma)"
    );
    assert_eq!(
        c.hero.gizmo.selected_len(),
        0,
        "a herdeira mexeu na selecção"
    );
    c.sim.world_mut().despawn(Entity::from_bits(a));
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        None,
        "controlo: sem forma nenhuma o modo não larga"
    );
}

/// ⭐⭐ GATE (report do dono, 05/10: *«ao fazer um boolean o gizmo já não aparece»*) — **a forma que a
/// booleana deixa fica em Edit COM o gizmo**: as duas do Edit somem, nasce uma, o Edit passa a ela e
/// o gizmo de objecto mostra-se (a forma trancada é a própria selecção).
/// (Mutação: o `object_gizmo_shows` voltar a excluir o objecto trancado ⇒ RED.)
#[test]
fn the_shape_a_boolean_leaves_is_in_edit_with_its_gizmo() {
    use ph2d_editor_core::screens::hero::mode_drive::object_gizmo_shows;
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(b));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(b));
    // O caminho do programa: a CANETA selecciona as duas no Edit (e a shell copia-a para a
    // selecção) — elas juntam-se ao Edit, e o cadeado não o larga.
    let ids: Vec<VecPathId> = c.vec.entities.keys().copied().collect();
    c.vec.pen.select_many(&ids);
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b), "a selecção da caneta largou o Edit");
    assert!(
        c.hero.gizmo.mode.parts().is_some_and(|p| p.contains(&a)),
        "a forma da caneta não é parte do Edit"
    );
    assert!(object_gizmo_shows(&c.hero), "controlo: duas formas em Edit");
    // A booleana: as duas saem da cena (e do mapa), nasce o resultado.
    let ids: Vec<VecPathId> = c.vec.entities.keys().copied().collect();
    for id in ids {
        let e = c.vec.entities.remove(&id).expect("a forma");
        c.sim.world_mut().despawn(Entity::from_bits(e));
        c.scene.remove_path(id);
    }
    // ⚠️ No programa a forma nova só ganha entidade no quadro SEGUINTE (a sincronia): o Edit cai
    // num quadro e o resultado nasce no outro — com tudo num quadro só este gate passava e a foto não.
    c.quadro(None);
    assert!(
        c.na_mao(),
        "o Edit caiu com as formas consumidas e largou a ferramenta"
    );
    let u = c.forma(1.0);
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        Some(u),
        "o Edit não passou à forma da booleana"
    );
    assert!(c.na_mao());
    assert!(
        object_gizmo_shows(&c.hero),
        "a forma da booleana ficou sem gizmo"
    );
}

/// ⭐ GATE — **uma selecção VELHA da caneta não entra no Edit seguinte**: as formas que a caneta
/// selecciona juntam-se ao Edit, mas só as que ela seleccionou DENTRO dele — o `Tab` noutra forma
/// abre o Edit dessa forma só.
/// (Mutação: o `enter_with` não podar a caneta ⇒ RED.)
#[test]
fn a_stale_pen_selection_never_joins_the_next_edit() {
    let mut c = cena();
    let (a, b) = (c.forma(0.0), c.forma(3.0));
    c.quadro(None);
    let id_a = c
        .vec
        .entities
        .iter()
        .find(|(_, x)| **x == a)
        .map(|(id, _)| *id)
        .expect("a");
    c.vec.pen.select_many(&[id_a]);
    c.hero.gizmo.replace_selection(Some(b));
    c.quadro(Some(ModeRequest::Toggle));
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(b));
    assert_eq!(
        c.vec.edit.objects,
        vec![b],
        "a selecção velha entrou no Edit"
    );
}
