//! **O arrasto da paleta vê-se** (smoke do dono 01/11, a lei do cartão do Kanban): a meio do
//! arrasto a grelha pinta a ordem prevista — a MESMA conta da família — com o vão, o bloco voa e
//! nada se pede; ao largar sai o `DragSwatch` de sempre; Esc cancela sem pedido.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{KEY_ESCAPE, WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::{NewSpriteMode, PixelPanelInfo, PixelRequest, PixelTool};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_pixel_core::palette::{copy_insert, drag_permutation};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 4000.0,
};

fn info() -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        tool: PixelTool::Pencil,
        fg: [255, 255, 255, 255],
        bg: [0, 0, 0, 255],
        palette: (0..32u8)
            .map(|i| [i * 7, 255 - i * 5, i * 3, 255])
            .collect(),
        mode: NewSpriteMode::Indexed,
        selected: vec![3, 4],
        anchor: Some(4),
        ..Default::default()
    }
}

struct Palco {
    host: MockPanelHost,
    st: PixelToolsPanelState,
    rects: Vec<(ph2d_a11y::NodeId, Rect)>,
    relogio: u128,
}

impl Palco {
    fn novo() -> Self {
        let mut p = Self {
            host: MockPanelHost::with_panel::<PixelToolsPanel>(),
            st: PixelToolsPanelState::default(),
            rects: Vec::new(),
            relogio: 0,
        };
        set_current_pixel(info());
        p.pinta();
        p
    }

    fn pinta(&mut self) {
        self.rects = self.host.paint::<PixelToolsPanel>(&mut self.st, VIEWPORT);
    }

    fn meio(&self, i: usize) -> (f32, f32) {
        let id = ids::palette_id(i);
        let r = self
            .rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("a amostra {i} não se pintou"));
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    fn ponteiro(&mut self, kind: PointerKind, (x, y): (f32, f32)) -> Vec<WidgetEvent> {
        self.relogio += 1_000_000_000;
        self.host.dispatch_pointer_event(PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: self.relogio,
        })
    }

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<EditorAction> {
        for ev in eventos {
            let _ = self
                .host
                .apply_panel_event::<PixelToolsPanel>(&mut self.st, ev);
        }
        self.host.drained_actions()
    }

    /// Prime a amostra `de` e leva o rato até ao meio de `a`, sem largar; pinta.
    fn arrasta_ate(&mut self, de: usize, a: usize) -> Vec<EditorAction> {
        let (p0, p1) = (self.meio(de), self.meio(a));
        let mut ev = self.ponteiro(PointerKind::Down, p0);
        ev.extend(self.ponteiro(PointerKind::Move, p1));
        let acoes = self.aplica(ev);
        self.pinta();
        acoes
    }

    fn larga_em(&mut self, a: usize) -> Vec<EditorAction> {
        let p = self.meio(a);
        let ev = self.ponteiro(PointerKind::Up, p);
        self.aplica(ev)
    }
}

/// A ordem que a grelha deve pintar para um MOVER: a lei da família, com o bloco no vão.
fn movido(sel: &[usize], to: usize) -> Vec<Option<usize>> {
    drag_permutation(32, sel, to)
        .into_iter()
        .map(|i| (!sel.contains(&i)).then_some(i))
        .collect()
}

#[test]
fn mid_drag_the_palette_paints_the_order_where_the_swatch_will_land() {
    let mut p = Palco::novo();
    let identidade: Vec<Option<usize>> = (0..32).map(Some).collect();
    assert_eq!(p.st.palette_painted(), identidade.as_slice());
    assert!(p.st.palette_flight().is_none(), "sem arrasto nada voa");
    assert!(
        p.st.palette_drop_marker().is_none(),
        "sem arrasto não há marca"
    );

    let a = p.arrasta_ate(9, 12);
    assert!(a.is_empty(), "a meio do arrasto nada se pede: {a:?}");
    assert_eq!(p.st.palette_painted(), movido(&[9], 12).as_slice());
    marca_a_esquerda_do_vao(&p);
    let voo = p.st.palette_flight().expect("a amostra voa");
    let (cx, cy) = p.meio(12);
    assert!(
        voo.contains(cx, cy),
        "o bloco voa sob o cursor ({cx}, {cy}) ∉ {voo:?}"
    );
    assert_eq!(
        p.larga_em(12),
        [EditorAction::Pixel(PixelRequest::DragSwatch {
            from: 9,
            to: 12,
            copy: false
        })]
    );
    p.pinta();
    assert!(p.st.palette_flight().is_none(), "largado, nada voa");
    assert!(
        p.st.palette_drop_marker().is_none(),
        "largado, a marca some"
    );
}

/// A barra da queda fica no intervalo logo à esquerda da PRIMEIRA célula do vão, vertical e da
/// altura dela (o pedido do dono no smoke de 01/11).
fn marca_a_esquerda_do_vao(p: &Palco) {
    let j =
        p.st.palette_painted()
            .iter()
            .position(Option::is_none)
            .expect("há vão");
    let id = ids::palette_id(j);
    let c = p
        .rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .expect("célula");
    let m = p.st.palette_drop_marker().expect("a queda tem marca");
    assert!(m.h > m.w, "a marca é vertical: {m:?}");
    assert!(
        m.x + m.w <= c.x + 0.5 && m.x + m.w >= c.x - c.w * 0.5,
        "logo à esquerda de {c:?}: {m:?}"
    );
    assert!(
        m.y <= c.y && m.y + m.h >= c.y + c.h,
        "cobre a altura da célula: {m:?} × {c:?}"
    );
}

#[test]
fn mid_drag_a_selected_swatch_carries_the_whole_block() {
    let mut p = Palco::novo();
    assert!(p.arrasta_ate(3, 10).is_empty());
    assert_eq!(p.st.palette_painted(), movido(&[3, 4], 10).as_slice());
    marca_a_esquerda_do_vao(&p);
    assert_eq!(
        p.larga_em(10),
        [EditorAction::Pixel(PixelRequest::DragSwatch {
            from: 3,
            to: 10,
            copy: false
        })]
    );
}

#[test]
fn mid_drag_with_ctrl_the_palette_opens_the_gap_of_the_inserted_copy() {
    let mut p = Palco::novo();
    p.host.store_mut().set_cmd_held(true);
    assert!(p.arrasta_ate(9, 12).is_empty());
    let (cores, onde) = copy_insert(&info().palette, &[9], 12).expect("cabe");
    let mut quer = vec![None; cores.len()];
    for (antigo, &novo) in onde.iter().enumerate() {
        quer[novo] = Some(antigo);
    }
    assert_eq!(quer[12], None, "o vão da cópia fica na célula do alvo");
    assert_eq!(p.st.palette_painted(), quer.as_slice());
    marca_a_esquerda_do_vao(&p);
    assert_eq!(
        p.larga_em(12),
        [EditorAction::Pixel(PixelRequest::DragSwatch {
            from: 9,
            to: 12,
            copy: true
        })]
    );
}

#[test]
fn escape_mid_drag_cancels_without_a_request() {
    let mut p = Palco::novo();
    assert!(p.arrasta_ate(9, 12).is_empty());
    let arena = bumpalo::Bump::new();
    let esc = ph2d_host::KeyEvent {
        keycode: KEY_ESCAPE,
        modifiers: ph2d_host::Modifiers::default(),
        kind: ph2d_host::KeyKind::Down,
        timestamp_ns: 0,
    };
    let ev = dispatch_key(p.host.store_mut(), esc, &arena).to_vec();
    assert!(p.aplica(ev).is_empty());
    p.pinta();
    let identidade: Vec<Option<usize>> = (0..32).map(Some).collect();
    assert_eq!(
        p.st.palette_painted(),
        identidade.as_slice(),
        "Esc volta à ordem de agora"
    );
    assert!(p.st.palette_flight().is_none(), "Esc: nada voa");
    assert!(p.st.palette_drop_marker().is_none(), "Esc: a marca some");
    let a = p.larga_em(12);
    assert!(a.is_empty(), "o Up depois do Esc não pede nada: {a:?}");
}
