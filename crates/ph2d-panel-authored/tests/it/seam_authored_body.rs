//! Seam do **CORPO** (F18, decisão «A» do dono): o painel gerado veste a cor que o artista deu à
//! forma, pela porta REAL de pintura do painel — lida na GRAVAÇÃO da cena e nas tintas dos glifos.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{PaintCtx, Panel, PanelHost, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::slot::SlotSet;
use ph2d_editor_core::widget::{ButtonState, WidgetKind};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_authored::state::AuthoredPanelState;
use ph2d_panel_authored::{AuthoredPanel, rows};
use ph2d_tokens::Color;
use ph2d_tokens::skin_body::readable_on;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::{Brush, DrawOp, Shape, VectorScene};

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const BLUE: [u8; 4] = [40, 120, 240, 255];
const KEY: &str = "go";

fn row(body: Option<[u8; 4]>) -> rows::Row {
    rows::Row {
        kind: WidgetKind::Button,
        label: "Go".to_string(),
        key: KEY.to_string(),
        id: ph2d_panel_authored::ids::authored_row_id(KEY),
        rgba: None,
        body,
        icon: None,
        icon_id: None,
        options: Vec::new(),
    }
}

struct Painted {
    ops: Vec<DrawOp>,
    /// A tinta de cada corrida de glifos, `[r, g, b, a]` pré-multiplicada.
    inks: Vec<[u8; 4]>,
    /// O rect que a row registou para o clique.
    rect: Rect,
}

/// Pinta UMA vez pelo `Panel::paint` do painel — o `hover` encena o rato por cima da row.
fn paint(h: &mut MockPanelHost, hover: bool) -> Painted {
    let id = ph2d_panel_authored::ids::authored_row_id(KEY);
    let mut st = AuthoredPanelState;
    // 1.ª pintura: o `adopt` regista a row no store, como no app.
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
    if hover {
        let store = h.store_mut();
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Hovered,
            },
        );
        store.set_hover_live(id, 1.0);
    }
    let rect = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, id)
        .expect("a row nao foi pintada");
    let layout = HeroLayout::for_viewport(VIEWPORT);
    let mut scene = VectorScene::new();
    scene.record();
    let mut text_system = ph2d_text::TextSystem::without_system_fonts();
    {
        let mut ctx = PaintCtx {
            host: h,
            layout: &layout,
            slot: layout
                .slot_rects(SlotSet::of(AuthoredPanel::DEFAULT_SLOT))
                .get(AuthoredPanel::DEFAULT_SLOT),
            viewport: VIEWPORT,
            scene: &mut scene,
            text_system: &mut text_system,
        };
        AuthoredPanel::paint(&mut st, &mut ctx);
    }
    let ops = scene.take_record();
    let e = scene.inner().encoding();
    let inks = e
        .resources
        .glyph_runs
        .iter()
        .map(|r| e.draw_data[r.stream_offsets.draw_data].to_ne_bytes())
        .collect();
    Painted { ops, inks, rect }
}

fn host_with(body: Option<[u8; 4]>) -> MockPanelHost {
    rows::set_live_rows(None);
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    rows::set_live_rows(Some(vec![row(body)]));
    h
}

/// A cor sólida de cada preenchimento que cobre `r` (folga de 1 px), na ordem da pintura.
fn fills_covering(ops: &[DrawOp], r: Rect) -> Vec<[u8; 4]> {
    ops.iter()
        .filter_map(|op| {
            let DrawOp::Fill {
                brush: Brush::Solid(c),
                path,
                transform,
                ..
            } = op
            else {
                return None;
            };
            let b = (*transform * path.clone()).bounding_box();
            let covers = b.x0 <= f64::from(r.x) + 1.0
                && b.y0 <= f64::from(r.y) + 1.0
                && b.x1 >= f64::from(r.x + r.w) - 1.0
                && b.y1 >= f64::from(r.y + r.h) - 1.0;
            covers.then(|| c.to_rgba8().to_u8_array())
        })
        .collect()
}

fn token(c: [u8; 4]) -> Color {
    Color {
        r: c[0],
        g: c[1],
        b: c[2],
        a: c[3],
    }
}

/// **O painel gerado veste a cor do desenho**: em repouso o corpo É o azul do artista e o rótulo é
/// a tinta legível sobre ele; com o rato por cima, a camada de estado M3 aparece sobre o azul.
#[test]
fn o_painel_gerado_veste_a_cor_do_desenho() {
    let mut h = host_with(Some(BLUE));
    let theme = h.theme();
    let rest = paint(&mut h, false);
    let fills = fills_covering(&rest.ops, rest.rect);
    assert!(
        fills.contains(&BLUE),
        "em repouso nenhum preenchimento azul cobre a row {:?} — cobrem: {fills:?}",
        rest.rect
    );
    let ink = readable_on(token(BLUE), theme);
    assert!(
        rest.inks.iter().any(|c| token(*c) == ink),
        "o rotulo nao e' a tinta legivel {ink:?} sobre o azul — tintas: {:?}",
        rest.inks
    );

    let hovered = paint(&mut h, true);
    let fills = fills_covering(&hovered.ops, hovered.rect);
    let top = fills.last().copied();
    assert!(top.is_some(), "com o rato por cima a row perdeu o corpo");
    assert_ne!(
        top,
        Some(BLUE),
        "com o rato por cima o corpo continua o azul liso — a camada de estado nao aparece: \
         {fills:?}"
    );
    rows::set_live_rows(None);
}

/// **CONTROLO:** sem corpo, o painel pinta o tema — nenhum azul do artista em lado nenhum.
#[test]
fn sem_corpo_o_painel_gerado_nao_pinta_o_azul() {
    let mut h = host_with(None);
    let p = paint(&mut h, false);
    let blue = p.ops.iter().any(|op| {
        matches!(op, DrawOp::Fill { brush: Brush::Solid(c), .. } if c.to_rgba8().to_u8_array() == BLUE)
    });
    assert!(!blue, "uma row sem corpo pintou o azul do artista");
    assert!(
        !fills_covering(&p.ops, p.rect).is_empty(),
        "a regua nao ve o corpo do tema — o controlo passaria por vacuo"
    );
    rows::set_live_rows(None);
}
