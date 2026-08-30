//! **O X da tira do Flip FECHA a tira.**
//!
//! ## O defeito, e por que nenhum gate desta casa o via
//!
//! O X era pintado, registado no índice de hit, e **encaminhado** — o `event.rs` punha-o na lista
//! `BUTTONS`, que empurra um `PanelEvent::Click(id)` no barramento e devolve `Consumed`. Isso é o
//! bastante para passar em todo gate de registo (o widget existe), em todo gate de costura (o
//! clique CHEGA à ferramenta) e no `architecture_panel_wiring_parity` (ele é focalizável).
//!
//! ⛔ **O que faltava era o passo seguinte:** nenhum dos três drenos do shell —
//! `flip_layers.rs`, `flip_strip.rs`, `ph2d-tool-flip/tool.rs` — tinha braço para aquele id. O
//! botão acendia sob o dedo, comia o clique, e terminava no vazio. *Um botão que consome o gesto
//! e não faz nada é pior que um botão ausente: o artista conclui que a tira não fecha.*
//!
//! ## A cura é a lei que o painel vizinho já escrevia
//!
//! Fechar um painel não é edição de documento nem de transporte — é do próprio painel. O
//! `TimelinePanel` faz `host.set_panel_visible(ID, false)` no seu X desde 2026-07-16, e este
//! ficheiro é o irmão do `close_button_seam.rs` dele, de propósito: *duas respostas à mesma
//! pergunta divergem; uma lei em dois painéis com o mesmo teste, não.*
//!
//! ⚠️ O gesto é o **ponteiro real** (`click_at` → `dispatch_pointer`), não um `WidgetEvent`
//! sintético: só ele mede que o rect pintado é o rect que recebe o dedo.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_flip_frames::state::FlipStripState;
use ph2d_panel_flip_frames::{FlipCell, FlipFramesPanel, FlipStripSnapshot, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

fn strip_snapshot() -> FlipStripSnapshot {
    let cell = |key: i32| FlipCell {
        key,
        exposure: 4,
        breakdown: false,
        instanced: false,
        selected: false,
        pinned: false,
        weight: 1.0,
    };
    FlipStripSnapshot {
        has_layer: true,
        cells: vec![cell(0), cell(4), cell(8)],
        ..Default::default()
    }
}

#[test]
fn the_close_button_is_painted_registered_and_actually_closes_the_strip() {
    ph2d_panel_flip_frames::set_current_flip_strip(strip_snapshot());
    let mut host = MockPanelHost::with_panel::<FlipFramesPanel>();
    let mut state = FlipStripState::default();
    let regs = host.paint::<FlipFramesPanel>(&mut state, VIEWPORT);

    // ⚠️ Metade JUSTA: sem ela um painel que já nascesse fechado tornaria o resto vacuamente
    // verdadeiro — o gate estaria a medir a sua própria fixtura.
    assert!(
        host.panel_visible(FlipFramesPanel::ID),
        "a tira começa aberta — senão a asserção do fecho não prova nada"
    );

    let r = regs
        .iter()
        .find(|(w, _)| *w == ids::FLIP_STRIP_CLOSE)
        .map(|(_, r)| *r)
        .expect("o X foi pintado mas nunca registado: ele clica no nada");
    assert!(
        r.w > 0.0 && r.h > 0.0,
        "o X não tem área para clicar: {r:?}"
    );

    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let evs = host.click_at(cx, cy);
    assert!(
        evs.contains(&WidgetEvent::Click(ids::FLIP_STRIP_CLOSE)),
        "o ponteiro caiu em {:?}, não no X — got {evs:?}",
        host.hit_at(cx, cy)
    );
    for ev in evs {
        host.apply_panel_event::<FlipFramesPanel>(&mut state, ev);
    }

    assert!(
        !host.panel_visible(FlipFramesPanel::ID),
        "clicar no X tem de FECHAR a tira do Flip. Se ele voltou à lista BUTTONS, o clique vai \
         para o barramento — e nenhum dreno do shell tem braço para ele: o botão morre calado."
    );
}
