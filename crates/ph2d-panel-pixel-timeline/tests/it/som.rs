//! P13 T: as linhas de som sob os quadros, pelo rato de verdade (`ph2d_ui_testkit`): **Add Audio…**
//! ⇒ a faixa no documento num passo, Ctrl+Z tira; Mute e Remove pedem a edição DA linha.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::{PixelFrameCol, PixelPanelInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_timeline::ids::header_id;
use ph2d_panel_pixel_timeline::som::{ADD_AUDIO, faixa_id, mute_id, remove_id};
use ph2d_panel_pixel_timeline::{PixelTimelinePanel, PixelTimelinePanelState, set_current_pixel};
use ph2d_pixel_audio::canal::{self, Pedido};
use ph2d_pixel_audio::faixas::{self, Edicao};
use ph2d_pixel_core::{History, NewSprite, Sprite};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

fn info(s: &Sprite) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        frames: s
            .frames()
            .iter()
            .map(|f| PixelFrameCol {
                id: f.id.0,
                duration_ms: f.duration_ms,
            })
            .collect(),
        active_frame: s.frames().first().map(|f| f.id.0),
        ..PixelPanelInfo::default()
    }
}

/// Pinta, carrega no MEIO de `id` e devolve o que o painel pediu.
fn carrega(s: &Sprite, id: ph2d_a11y::NodeId) -> Vec<Pedido> {
    let mut host = MockPanelHost::with_panel::<PixelTimelinePanel>();
    let mut st = PixelTimelinePanelState::default();
    set_current_pixel(info(s));
    let rects = host.paint::<PixelTimelinePanel>(&mut st, VIEWPORT);
    let r = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
    let ev = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        ev.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
    );
    for e in ev {
        let _ = host.apply_panel_event::<PixelTimelinePanel>(&mut st, e);
    }
    canal::toma()
}

/// O que a família faz com um pedido (`ph2d-app-pixel::som`): o ficheiro escolhido, um passo.
fn aplica(s: &mut Sprite, h: &mut History, p: Pedido, ficheiro: &str) {
    let e = canal::edicao(p, None, || Some(ficheiro.into())).expect("edição");
    let op = faixas::op(s, &e).expect("muda");
    h.commit(s, op).expect("aplica");
}

#[test]
fn add_audio_pelo_rato_poe_a_faixa_num_passo_e_ctrl_z_tira() {
    let mut s = Sprite::new(NewSprite::default()).expect("sprite");
    let mut h = History::default();
    canal::publica(Some(&s), None);
    let antes = s.clone();
    let pedidos = carrega(&s, ADD_AUDIO);
    assert_eq!(pedidos, [Pedido::Escolher]);
    let p = pedidos.into_iter().next().expect("pedido");
    aplica(&mut s, &mut h, p, "/sons/passo.wav");
    assert_eq!(h.undo_len(), 1, "um passo");
    assert_eq!(s.audio().len(), 1);
    assert_eq!(s.audio()[0].asset, "/sons/passo.wav");
    // A linha aparece (o asset não existe: «not found», sem pânico) com o Mute e o Remove dela.
    canal::publica(Some(&s), None);
    assert_eq!(canal::linhas().len(), 1);
    assert_eq!(
        carrega(&s, mute_id(0)),
        [Pedido::Edita(Edicao::Mudo(0, true))]
    );
    assert_eq!(
        carrega(&s, remove_id(0)),
        [Pedido::Edita(Edicao::Remove(0))]
    );
    assert!(h.undo(&mut s), "Ctrl+Z");
    assert_eq!(s, antes);
}

/// Um sprite de `n` quadros de 100 ms com uma faixa (o asset não existe: a linha desenha-se igual).
fn com_faixa(n: usize) -> (Sprite, History) {
    let mut s = Sprite::new(NewSprite::default()).expect("sprite");
    let mut h = History::default();
    for _ in 1..n {
        let f = s.frames().last().expect("quadro").id;
        let op =
            ph2d_pixel_core::frames::new_frame(&mut s, f, ph2d_pixel_core::frames::NewFrame::Empty)
                .expect("novo")
                .0;
        ph2d_pixel_core::apply(&mut s, op).expect("aplica");
    }
    let op = faixas::op(&s, &Edicao::Adiciona("/sons/passo.wav".into())).expect("faixa");
    h.commit(&mut s, op).expect("aplica");
    (s, h)
}

/// **Arrastar a onda 10 quadros** pelo rato de verdade ⇒ `inicio_ms` += 1000 num passo; Ctrl+Z volta.
#[test]
fn arrastar_a_faixa_dez_quadros_muda_o_inicio_num_passo_e_ctrl_z_volta() {
    let (mut s, mut h) = com_faixa(16);
    canal::publica(Some(&s), None);
    let mut host = MockPanelHost::with_panel::<PixelTimelinePanel>();
    let mut st = PixelTimelinePanelState::default();
    set_current_pixel(info(&s));
    let rects = host.paint::<PixelTimelinePanel>(&mut st, VIEWPORT);
    let rect = |id| rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r);
    let onda = rect(faixa_id(0)).expect("a onda da faixa 0 pintada e registada");
    let (h0, h1) = (s.frames()[0].id.0, s.frames()[1].id.0);
    let col_w = rect(header_id(h1)).expect("cab 1").x - rect(header_id(h0)).expect("cab 0").x;
    assert!(col_w > 0.0);
    let (x, y) = (onda.x + col_w * 0.5, onda.y + onda.h * 0.5);
    let _ = host.drag_at(x, y, x + 10.0 * col_w, y);
    let _ = host.paint::<PixelTimelinePanel>(&mut st, VIEWPORT);
    let pedidos = canal::toma();
    assert_eq!(pedidos, [Pedido::Edita(Edicao::Move(0, 1000))]);
    let antes = s.clone();
    let p = pedidos.into_iter().next().expect("pedido");
    aplica(&mut s, &mut h, p, "");
    assert_eq!(s.audio()[0].inicio_ms, 1000);
    assert_eq!(
        h.undo_len(),
        2,
        "o arrasto é UM passo (o outro é o Add Audio)"
    );
    assert!(h.undo(&mut s), "Ctrl+Z");
    assert_eq!(s.audio()[0].inicio_ms, 0);
    assert_eq!(s, antes, "Ctrl+Z devolve o sprite de antes do arrasto");
}
