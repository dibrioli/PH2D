//! ⭐ **R5: o popover do Planner fica por cima do HUD** (`panel/camada_das_listas.rs`). As caixas partilham
//! a porta; esta prova a pergunta do alcance, a mesma forma das outras três.

use ph2d_app_planner::ScopePop;
use ph2d_app_planner::scope::{Ask, Change};
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::panel::camada_das_listas::{abre, fecha};
use ph2d_kanban_model::{EventId, KanbanId, Moment};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// *Mutação: `ScopePop::paint` a pintar direto ⇒ vermelho.*
#[test]
fn a_pergunta_do_alcance_vai_para_a_camada() {
    let pop = ScopePop::new(
        Ask {
            kanban: KanbanId::AGENDA,
            series: EventId(1),
            at: Moment(0),
            change: Change::Delete,
        },
        (10.0, 10.0),
    );
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    scene.record();
    abre(&scene);
    pop.paint(&mut scene, &mut ts, Theme::default());
    assert!(
        scene.take_record().is_empty(),
        "o popover ficou na cena da aba"
    );
    fecha(&mut scene, &mut HitIndex::new());
    assert!(!scene.take_record().is_empty(), "o popover perdeu-se");
}
