//! A costura da P2 (cor, paleta, tinta, modo) no painel PIXEL: cada controlo novo carregado no
//! MEIO, pelo rato de verdade, chega ao barramento como exactamente o pedido dele.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::{
    NewSpriteMode, PixelOptions, PixelPanelInfo, PixelRequest, PixelSelect, PixelTool,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::MockPanelHost;

/// Alto o bastante para o painel inteiro caber sem rolar.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 4000.0,
};

fn info(tool: PixelTool) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        tool,
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
}

impl Palco {
    fn novo(i: PixelPanelInfo) -> Self {
        let mut p = Self {
            host: crate::host_de_antes(),
            st: PixelToolsPanelState::default(),
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

    /// Abre o menu `menu` da barra da paleta pelo ícone (o rato de verdade) e carrega na linha `id`.
    fn no_menu(&mut self, menu: ph2d_a11y::NodeId, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
        let r = self.rect(menu).expect("o ícone do menu pintou-se na barra");
        let ev = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(self.aplica(ev).is_empty(), "abrir o menu não pede nada");
        self.rects = self.host.paint::<PixelToolsPanel>(&mut self.st, VIEWPORT);
        let a = self.carrega(id);
        self.rects = self.host.paint::<PixelToolsPanel>(&mut self.st, VIEWPORT);
        assert!(self.rect(id).is_none(), "carregar numa linha fecha o menu");
        a
    }
}

fn so(a: &[EditorAction], r: PixelRequest) {
    assert_eq!(a, [EditorAction::Pixel(r)]);
}

#[test]
fn cada_tinta_chega_as_opcoes_e_a_borracha_nao_tem_tinta() {
    for (ink, id) in ids::INKS {
        let mut p = Palco::novo(info(PixelTool::Pencil));
        let quer = PixelOptions {
            ink,
            ..PixelOptions::default()
        };
        so(&p.carrega(id), PixelRequest::Options(Box::new(quer)));
    }
    let p = Palco::novo(info(PixelTool::Bucket));
    assert!(
        p.rect(ids::INKS[4].1).is_some(),
        "o balde também pinta com tinta"
    );
    let p = Palco::novo(info(PixelTool::Eraser));
    for (_, id) in ids::INKS {
        assert!(p.rect(id).is_none(), "a borracha mostra a tinta {id:?}");
    }
}

#[test]
fn shift_e_ctrl_no_clique_da_paleta_sao_a_faixa_e_o_liga_desliga() {
    let mut p = Palco::novo(info(PixelTool::Pencil));
    p.host.store_mut().set_shift_held(true);
    so(
        &p.carrega(ids::palette_id(9)),
        PixelRequest::Swatch {
            index: 9,
            how: PixelSelect::Range,
        },
    );
    let mut p = Palco::novo(info(PixelTool::Pencil));
    p.host.store_mut().set_cmd_held(true);
    so(
        &p.carrega(ids::palette_id(2)),
        PixelRequest::Swatch {
            index: 2,
            how: PixelSelect::Toggle,
        },
    );
}

#[test]
fn cada_accao_da_paleta_e_do_sprite_chega_ao_barramento() {
    // O *Remap Palette* é um botão à vista, só com o remapeamento pendente (o Trial).
    let remap = ids::ACTIONS
        .iter()
        .find(|(_, r, _)| *r == PixelRequest::RemapPalette);
    let (remap, ..) = *remap.expect("o botão Remap Palette");
    assert!(
        Palco::novo(info(PixelTool::Pencil)).rect(remap).is_none(),
        "sem pendente, sem botão"
    );
    let mut i = info(PixelTool::Pencil);
    i.remap_pending = true;
    so(&Palco::novo(i).carrega(remap), PixelRequest::RemapPalette);
    // O resto mora nos dois menus da barra da paleta (como no Aseprite): nenhum à vista no corpo.
    let presets = [6, 7, 8]; // Load · Save · Default palette
    let paleta = [0, 1, 2, 3, 4, 5, 9, 10, 11];
    for (k, menu) in [
        (&presets[..], ids::PRESETS_MENU),
        (&paleta[..], ids::PALETTE_MENU),
    ] {
        for &n in k {
            let (id, ref r, _) = ids::ACTIONS[n];
            let mut p = Palco::novo(info(PixelTool::Pencil));
            assert!(p.rect(id).is_none(), "{id:?} já não é um botão do corpo");
            so(&p.no_menu(menu, id), r.clone());
        }
    }
    for (id, r, _) in ids::PRESET_ACTIONS {
        let mut p = Palco::novo(info(PixelTool::Pencil));
        so(&p.no_menu(ids::PRESETS_MENU, id), r);
    }
    for (k, id) in ids::SORT_KEYS {
        let mut p = Palco::novo(info(PixelTool::Pencil));
        so(&p.no_menu(ids::PALETTE_MENU, id), PixelRequest::SortKey(k));
    }
    let mut i = info(PixelTool::Pencil);
    i.presets = vec!["Warm".to_owned()];
    so(
        &Palco::novo(i).no_menu(ids::PRESETS_MENU, ids::preset_id(0)),
        PixelRequest::LoadPreset("Warm".to_owned()),
    );
}

#[test]
fn o_tamanho_da_paleta_mostra_o_do_sprite_e_pede_o_novo() {
    let mut p = Palco::novo(info(PixelTool::Pencil));
    assert!(p.rect(ids::PALETTE_SIZE).is_some());
    assert_eq!(p.host.store().number_value(ids::PALETTE_SIZE), Some(32.0));
    p.host
        .store_mut()
        .set_number_value(ids::PALETTE_SIZE, 300.0);
    so(
        &p.aplica(vec![WidgetEvent::ValueChanged(ids::PALETTE_SIZE)]),
        PixelRequest::PaletteSize(256),
    );
}

#[test]
fn color_mode_abre_o_dialogo_sobre_o_modo_do_sprite() {
    let mut p = Palco::novo(info(PixelTool::Pencil));
    assert!(p.carrega(ids::COLOR_MODE).is_empty(), "abre, não pede");
    let menu = p.host.store().context_menu().map(|r| r.kind);
    assert!(matches!(menu, Some(ContextMenuKind::PixelColorModeDialog)));
    assert_eq!(p.host.store().color_mode_current(), NewSpriteMode::Indexed);
}

/// O *Edit color* existe quando há amostra clicada, e um Down nele abre o seletor partilhado.
#[test]
fn edit_color_abre_o_seletor_partilhado() {
    let mut p = Palco::novo(info(PixelTool::Pencil));
    let r = p.rect(ids::PIXEL_PAL_EDIT).expect("o Edit color pintou");
    let _ = p.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(p.host.store().picker_target(), Some(ids::PIXEL_PAL_EDIT));
    let mut sem = info(PixelTool::Pencil);
    sem.anchor = None;
    let p = Palco::novo(sem);
    assert!(
        p.rect(ids::PIXEL_PAL_EDIT).is_none(),
        "sem amostra clicada não há o que editar"
    );
}

/// Com o Shading o botão direito anda para trás na rampa — a dica do rodapé não pode dizer «pinta a
/// cor de fundo» (a cena 3 abre assim). Controlo: com Simple a dica do lápis é a de sempre.
#[test]
fn the_hint_tells_what_the_shading_ink_does_with_each_button() {
    use ph2d_editor_core::pixel_panel::PixelInk;
    use ph2d_panel_pixel_tools::column::hint;
    let mut info = PixelPanelInfo {
        tool: PixelTool::Pencil,
        ..PixelPanelInfo::default()
    };
    let simple = hint(&info);
    info.options.ink = PixelInk::Shading;
    let shading = hint(&info);
    assert_ne!(simple, shading);
    assert_eq!(
        shading,
        ph2d_i18n::tr_with("pixel.hint.shading", &[("tool", &shading_tool(&simple))])
    );
    info.tool = PixelTool::Eraser;
    assert_ne!(hint(&info), shading, "a borracha é sempre Simple");
}

/// O nome da ferramenta, como a dica do lápis o escreve antes dos dois pontos.
fn shading_tool(pencil_hint: &str) -> String {
    pencil_hint.split(':').next().expect("nome").to_owned()
}
