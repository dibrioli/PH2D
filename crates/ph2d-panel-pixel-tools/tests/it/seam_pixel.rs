//! A costura do painel PIXEL: cada controlo carregado no MEIO, pelo rato de verdade, chega ao
//! barramento como exactamente o pedido dele.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelRequest, PixelTool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 900.0,
};

/// Uma paleta de 32 cores distintas (a DB32 tem 32).
fn paleta() -> Vec<[u8; 4]> {
    (0..32u8)
        .map(|i| [i * 7, 255 - i * 5, i * 3, 255])
        .collect()
}

fn info(editing: bool) -> PixelPanelInfo {
    PixelPanelInfo {
        editing,
        tool: PixelTool::Pencil,
        fg: [255, 255, 255, 255],
        bg: [0, 0, 0, 255],
        palette: paleta(),
    }
}

struct Palco {
    host: MockPanelHost,
    st: PixelToolsPanelState,
    rects: Vec<(ph2d_a11y::NodeId, Rect)>,
}

impl Palco {
    fn novo(i: PixelPanelInfo) -> Self {
        let mut p = Self {
            host: MockPanelHost::with_panel::<PixelToolsPanel>(),
            st: PixelToolsPanelState,
            rects: Vec::new(),
        };
        set_current_pixel(i);
        p.rects = p.host.paint::<PixelToolsPanel>(&mut p.st, VIEWPORT);
        p
    }

    fn rect(&self, id: ph2d_a11y::NodeId) -> Option<Rect> {
        self.rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
    }

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<EditorAction> {
        for ev in eventos {
            let _ = self
                .host
                .apply_panel_event::<PixelToolsPanel>(&mut self.st, ev);
        }
        self.host.drained_actions()
    }

    fn carrega(&mut self, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
        let ev = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
        );
        self.aplica(ev)
    }

    fn carrega_direito(&mut self, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
        let r = self.rect(id).expect("pintado");
        let ev = self.host.right_click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.aplica(ev)
    }
}

fn so(a: &[EditorAction], r: PixelRequest) {
    assert_eq!(a, [EditorAction::Pixel(r)]);
}

/// As ferramentas moram na COLUNA da esquerda (dono 25/10), que a pinta o chrome: o clique dela
/// chega a este painel pelo mesmo `apply_event` (com o painel fechado também) e vira o pedido.
#[test]
fn cada_ferramenta_da_coluna_chega_ao_barramento() {
    for (t, id) in ids::TOOLS {
        let mut p = Palco::novo(info(true));
        p.host
            .set_panel_visible(ph2d_editor_core::screens::hero::PANEL_PIXEL, false);
        so(
            &p.aplica(vec![WidgetEvent::Click(id)]),
            PixelRequest::Tool(t),
        );
        assert!(
            p.rect(id).is_none(),
            "{id:?} continua pintado no painel direito"
        );
    }
}

/// A coluna: as quatro ferramentas (a da mão acesa) e as duas amostras com o trocar; sem sprite em
/// Draw, nada.
#[test]
fn a_coluna_tem_as_quatro_ferramentas_e_as_duas_cores() {
    use ph2d_editor_core::interaction::BarFace;
    let mut i = info(true);
    i.tool = PixelTool::Eyedropper;
    let col = ph2d_panel_pixel_tools::column::column(&i);
    assert_eq!(col.len(), 2);
    let ferr: Vec<_> = col[0].iter().map(|b| (b.id, b.on)).collect();
    let quer: Vec<_> = ids::TOOLS
        .iter()
        .map(|(t, id)| (*id, *t == PixelTool::Eyedropper))
        .collect();
    assert_eq!(ferr, quer);
    let cores: Vec<_> = col[1].iter().map(|b| (b.id, b.face.clone())).collect();
    assert!(
        matches!(cores[0], (id, BarFace::Swatch { rgba: Some(c), ring: false, .. }) if id == ids::PIXEL_FG && c == i.fg)
    );
    assert!(
        matches!(cores[1], (id, BarFace::Swatch { rgba: Some(c), ring: false, .. }) if id == ids::PIXEL_BG && c == i.bg)
    );
    assert_eq!(cores[2].0, ids::PIXEL_SWAP);
    assert!(ph2d_panel_pixel_tools::column::column(&info(false)).is_empty());
}

#[test]
fn o_clique_esquerdo_na_paleta_e_a_frente_e_o_direito_o_fundo() {
    let pal = paleta();
    let mut p = Palco::novo(info(true));
    so(&p.carrega(ids::palette_id(5)), PixelRequest::Fg(pal[5]));
    let mut p = Palco::novo(info(true));
    so(
        &p.carrega_direito(ids::palette_id(9)),
        PixelRequest::Bg(pal[9]),
    );
    // O botão direito não abriu o menu de notas por cima da escolha.
    assert!(p.host.store().context_menu().is_none());
    // Todas as 32 células pintaram.
    assert!((0..32).all(|i| p.rect(ids::palette_id(i)).is_some()));
}

#[test]
fn trocar_chega_como_swap() {
    let mut p = Palco::novo(info(true));
    so(&p.carrega(ids::PIXEL_SWAP), PixelRequest::Swap);
}

/// Sem sprite em Edit: só a frase e o `×` — nenhum controlo morto pintado.
#[test]
fn sem_sprite_em_edicao_nao_ha_controlo_nenhum() {
    let p = Palco::novo(info(false));
    for (_, id) in ids::TOOLS {
        assert!(p.rect(id).is_none(), "{id:?} pintado sem sprite em edição");
    }
    assert!(p.rect(ids::PIXEL_SWAP).is_none());
    assert!(p.rect(ids::palette_id(0)).is_none());
    assert!(p.rect(ids::PIXEL_CLOSE).is_some());
}

#[test]
fn o_x_fecha_o_painel() {
    let mut p = Palco::novo(info(true));
    let _ = p.carrega(ids::PIXEL_CLOSE);
    assert!(
        !p.host
            .panel_visible(ph2d_editor_core::screens::hero::PANEL_PIXEL)
    );
}

#[test]
fn o_hex_e_o_do_aseprite() {
    assert_eq!(ph2d_panel_pixel_tools::hex([255, 255, 255, 255]), "#FFFFFF");
    assert_eq!(ph2d_panel_pixel_tools::hex([0, 0, 0, 255]), "#000000");
    assert_eq!(ph2d_panel_pixel_tools::hex([1, 2, 3, 4]), "#01020304");
}
