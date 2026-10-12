//! ⭐⭐ «Send to board…» (MiroClone W15) pelo caminho REAL do ecrã, da linha da Hierarquia ao pedido
//! que a shell drena: botão direito numa linha pintada → o menu (com a linha só para quem um quadro
//! recebe) → o clique em «Send to board…» chega a ESTE painel → o pedido → o modal (o que a shell faz
//! a seguir: `send_to_board::open`) → o clique numa linha do modal → a escolha do quadro.
//! A entrega no quadro tem o seu teste na família do quadro (`ph2d_app_board::send_to_board`).

use bumpalo::Bump;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::screens::hero::send_to_board;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::sync::Once;

const PICTURE: NodeId = NodeId(100_000);
const BONES: NodeId = NodeId(100_001);

fn ensure_typed_registry() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<HierarchyPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
}

fn entry(name: &str, bits: u64) -> HierarchyEntity {
    HierarchyEntity {
        name: name.into(),
        icon: IconId::Sprite,
        indent: 0,
        badge: None,
        swatch: None,
        visible: true,
        selected: false,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
        role: Default::default(),
        components: Vec::new(),
        content: Vec::new(),
        has: Default::default(),
        entity: Some(bits),
    }
}

struct T {
    hero: HeroScreen,
    text: TextSystem,
}

impl T {
    /// Uma cena com uma imagem e um esqueleto na Hierarquia, e dois quadros abertos (a aba da cena
    /// activa). A shell publica a lista das linhas que vão para um quadro: só a imagem.
    fn new() -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        ensure_typed_registry();
        let mut hero = HeroScreen::new(NodeId(1));
        hero.panel_visibility.insert(HierarchyPanel::ID, true);
        for n in ["Moodboard", "Ideas"] {
            let b = hero.documents.create(n.to_owned());
            ph2d_editor_core::screens::hero::document_tabs::register_board(&mut hero.store, b);
        }
        hero.documents.activate(None);
        let entries = [(PICTURE, entry("Hero", 7)), (BONES, entry("Rig", 8))];
        ph2d_panel_hierarchy::sync_from_hierarchy(
            &mut hero.store,
            &[PICTURE, BONES],
            entries.into_iter().collect(),
        );
        hero.store
            .set_hierarchy_board_rows(std::iter::once(PICTURE).collect());
        let mut t = Self {
            hero,
            text: TextSystem::without_system_fonts(),
        };
        t.paint();
        t
    }

    fn paint(&mut self) {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.text);
    }

    /// O centro do alvo `id` que o pintor registou (passo de 2 px).
    fn locate(&self, id: NodeId) -> Option<(f32, f32)> {
        let vp = self.hero.last_viewport;
        let mut hits = Vec::new();
        let mut y = vp.y;
        while y < vp.y + vp.h {
            let mut x = vp.x;
            while x < vp.x + vp.w {
                if self.hero.hit_index.hit(x, y) == Some(id) {
                    hits.push((x, y));
                }
                x += 2.0;
            }
            y += 2.0;
        }
        let n = hits.len() as f32;
        (!hits.is_empty()).then(|| {
            let sx: f32 = hits.iter().map(|p| p.0).sum();
            let sy: f32 = hits.iter().map(|p| p.1).sum();
            (sx / n, sy / n)
        })
    }

    /// Down + Up com `button` sobre `id`, pelo despacho do hero e cada evento pelo `apply_event`.
    fn press(&mut self, id: NodeId, button: PointerButton) {
        let (x, y) = self
            .locate(id)
            .unwrap_or_else(|| panic!("{id:?} não está pintado"));
        let arena = Bump::new();
        let mut events: Vec<WidgetEvent> = Vec::new();
        for kind in [PointerKind::Down, PointerKind::Up] {
            let e = PointerEvent {
                x,
                y,
                pressure: 1.0,
                kind,
                source: PointerSource::Mouse,
                button,
                timestamp_ns: 0,
            };
            events.extend_from_slice(self.hero.handle_pointer(e, &arena));
        }
        for e in events {
            self.hero.apply_event(e);
        }
        self.paint();
    }
}

#[test]
fn the_row_menu_the_modal_and_the_pick_by_real_clicks() {
    let mut t = T::new();
    // Botão direito no esqueleto: o menu abre SEM a linha (não é para um quadro).
    t.press(BONES, PointerButton::Secondary);
    assert!(matches!(
        t.hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::HierarchyRow { row: BONES })
    ));
    assert!(
        t.locate(ids::CTX_MENU_HIER_DUPLICATE).is_some(),
        "o menu da linha"
    );
    assert!(
        t.locate(ids::CTX_MENU_HIER_SEND_TO_BOARD).is_none(),
        "oferecido a um esqueleto"
    );
    t.hero.store.close_context_menu();
    t.paint();
    // Botão direito na imagem: a linha está lá, e o clique nela chega ao painel ⇒ o pedido.
    t.press(PICTURE, PointerButton::Secondary);
    t.press(ids::CTX_MENU_HIER_SEND_TO_BOARD, PointerButton::Primary);
    let asked: Vec<_> = t.hero.bus.drain().collect();
    assert!(
        asked.contains(&EditorAction::Hierarchy(HierRequest::SendToBoard {
            row: PICTURE
        })),
        "{asked:?}"
    );
    // A shell confere o tipo e abre o modal; o clique na 2.ª linha escolhe o 2.º quadro.
    send_to_board::open(&mut t.hero.store, PICTURE);
    t.paint();
    t.press(ids::CTX_SEND_BOARD_ROWS[1], PointerButton::Primary);
    let ideas = t.hero.documents.boards().boards()[1].id;
    let picked: Vec<_> = t.hero.bus.drain().collect();
    assert_eq!(
        picked,
        vec![EditorAction::Hierarchy(HierRequest::SendToBoardChosen {
            row: PICTURE,
            board: Some(ideas.0),
        })]
    );
    assert!(
        t.hero.store.context_menu().is_none(),
        "o modal ficou aberto"
    );
}
