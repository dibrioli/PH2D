//! A costura da P6 (selecção, moldura solta, menus Select/Edit/Sprite) no painel PIXEL: cada
//! controlo novo carregado no MEIO pelo rato de verdade (ou escrito no campo, com Enter) chega ao
//! barramento como exactamente o pedido dele; o que pede selecção, cópia ou células levantadas não
//! se pinta sem elas.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_panel::{
    PixelOptions, PixelPanelInfo, PixelRequest, PixelTool, PixelTransformField, PixelTransformInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::MockPanelHost;

/// Alto o bastante para o painel inteiro caber sem rolar.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 6000.0,
};

const MOLDURA: PixelTransformInfo = PixelTransformInfo {
    x: 3,
    y: 4,
    w: 10,
    h: 12,
    angle: 0,
    skew: 0,
};

fn info(tool: PixelTool, tudo: bool) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        tool,
        palette: vec![[0, 0, 0, 255], [255, 255, 255, 255]],
        has_selection: tudo,
        has_clip: tudo,
        can_reselect: tudo,
        transform: tudo.then_some(MOLDURA),
        size: (64, 32),
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
            host: MockPanelHost::with_panel::<PixelToolsPanel>(),
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
                .any(|e| matches!(e, WidgetEvent::Click(c) | WidgetEvent::Toggled(c) if *c == id)),
            "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
        );
        self.aplica(ev)
    }

    fn escreve(&mut self, id: ph2d_a11y::NodeId, texto: &str) -> Vec<EditorAction> {
        assert!(self.rect(id).is_some(), "o campo {id:?} não se pintou");
        let ev = self.host.type_into_number(id, texto);
        self.aplica(ev)
    }
}

fn so(a: &[EditorAction], r: PixelRequest) {
    assert_eq!(a, [EditorAction::Pixel(r)]);
}

#[test]
fn os_quatro_modos_da_selecao_chegam_as_opcoes() {
    for (m, id) in ids::SELECT_MODES {
        let mut p = Palco::novo(info(PixelTool::RectMarquee, false));
        let quer = PixelOptions {
            select_mode: m,
            ..PixelOptions::default()
        };
        so(&p.carrega(id), PixelRequest::Options(Box::new(quer)));
    }
    let p = Palco::novo(info(PixelTool::Pencil, false));
    for (_, id) in ids::SELECT_MODES {
        assert!(
            p.rect(id).is_none(),
            "o lápis não mostra o modo de selecção"
        );
    }
}

#[test]
fn a_varinha_tem_tolerancia_contigua_e_conectividade() {
    let mut p = Palco::novo(info(PixelTool::MagicWand, false));
    let quer = |f: fn(&mut PixelOptions)| {
        let mut o = PixelOptions::default();
        f(&mut o);
        PixelRequest::Options(Box::new(o))
    };
    so(
        &p.escreve(ids::WAND_TOLERANCE, "37"),
        quer(|o| o.wand_tolerance = 37),
    );
    let mut p = Palco::novo(info(PixelTool::MagicWand, false));
    so(
        &p.carrega(ids::WAND_CONTIGUOUS),
        quer(|o| o.wand_contiguous = false),
    );
    let mut p = Palco::novo(info(PixelTool::MagicWand, false));
    so(
        &p.carrega(ids::WAND_CONNECT_8),
        quer(|o| o.wand_eight = true),
    );
    let mut p = Palco::novo(info(PixelTool::MagicWand, false));
    so(&p.carrega(ids::WAND_CONNECT_4), quer(|_| {}));
    let mut p = Palco::novo(info(PixelTool::MagicWand, false));
    so(
        &p.carrega(ids::WAND_SAMPLE_VISIBLE),
        quer(|o| o.wand_sample_visible = true),
    );
    let mut p = Palco::novo(info(PixelTool::MagicWand, false));
    so(&p.carrega(ids::WAND_SAMPLE_LAYER), quer(|_| {}));
    let p = Palco::novo(info(PixelTool::MagicWand, false));
    assert!(
        p.rect(ids::SAMPLE_VISIBLE).is_none(),
        "a amostra do balde não é a da varinha"
    );
    let p = Palco::novo(info(PixelTool::RectMarquee, false));
    assert!(p.rect(ids::WAND_TOLERANCE).is_none(), "só a varinha");
    assert!(p.rect(ids::WAND_SAMPLE_VISIBLE).is_none(), "só a varinha");
}

#[test]
fn a_amostragem_fast_e_rotsprite_chega_com_o_transform() {
    for (r, id) in ids::RESAMPLES {
        let mut p = Palco::novo(info(PixelTool::Transform, false));
        let quer = PixelOptions {
            resample: r,
            ..PixelOptions::default()
        };
        so(&p.carrega(id), PixelRequest::Options(Box::new(quer)));
    }
}

#[test]
fn os_campos_da_moldura_so_existem_com_celulas_levantadas_e_pedem_o_valor() {
    let p = Palco::novo(info(PixelTool::RectMarquee, false));
    for (_, id, _) in ids::XFORM_FIELDS {
        assert!(p.rect(id).is_none(), "{id:?} sem moldura");
    }
    for (f, id, _) in ids::XFORM_FIELDS {
        let mut p = Palco::novo(info(PixelTool::RectMarquee, true));
        let v = match f {
            PixelTransformField::Angle => 30,
            PixelTransformField::Skew => -15,
            _ => 21,
        };
        so(
            &p.escreve(id, &v.to_string()),
            PixelRequest::TransformField { field: f, value: v },
        );
    }
    // os campos mostram a moldura
    let p = Palco::novo(info(PixelTool::RectMarquee, true));
    let x = ids::XFORM_FIELDS[0].1;
    assert_eq!(p.host.store().number_value(x), Some(3.0));
}

#[test]
fn cada_botao_dos_menus_select_edit_sprite_chega_ao_barramento() {
    let tabelas = ids::SELECT_ACTIONS
        .iter()
        .chain(&ids::EDIT_ACTIONS)
        .chain(&ids::SPRITE_ACTIONS);
    for (id, r, _) in tabelas {
        let mut p = Palco::novo(info(PixelTool::RectMarquee, true));
        so(&p.carrega(*id), r.clone());
    }
}

#[test]
fn sem_selecao_copia_ou_reselect_os_botoes_que_os_pedem_nao_se_pintam() {
    let p = Palco::novo(info(PixelTool::RectMarquee, false));
    let s = &ids::SELECT_ACTIONS;
    let e = &ids::EDIT_ACTIONS;
    let sp = &ids::SPRITE_ACTIONS;
    let escondidos = [
        s[1].0, s[2].0, s[5].0, s[6].0, s[7].0, e[0].0, e[1].0, e[2].0, e[3].0, e[11].0, e[12].0,
        e[13].0, e[14].0, e[15].0, e[16].0, sp[7].0,
    ];
    for id in escondidos {
        assert!(p.rect(id).is_none(), "{id:?} pintado sem o que pede");
    }
    let sempre = [
        s[0].0, s[3].0, s[4].0, e[4].0, e[5].0, e[6].0, e[10].0, sp[0].0, sp[8].0,
    ];
    for id in sempre {
        assert!(p.rect(id).is_some(), "{id:?} devia pintar-se sempre");
    }
}

/// P6b (smoke do dono 26/10): as quatro selecções num botão de grupo com o triângulo — a cara é a
/// última usada, e o flyout aberto tem as quatro com o atalho; a varinha e o Transform seguem
/// soltos, com os ícones deles. P6c: o Move saiu e o laço poligonal tem ícone próprio.
#[test]
fn as_ferramentas_de_selecao_estao_na_coluna_com_o_atalho() {
    use ph2d_editor_core::icons::IconId;
    use ph2d_editor_core::interaction::BarFace;
    let mut i = info(PixelTool::MagicWand, false);
    i.groups.chose(PixelTool::Lasso);
    let col = ph2d_panel_pixel_tools::column::column(&i);
    let sel: Vec<_> = col[0].iter().map(|b| (b.id, b.on)).collect();
    let quer = [
        (ids::PIXEL_SELECT_GROUP, false),
        (ids::TOOLS[4].1, true),
        (ids::TOOLS[5].1, false),
    ];
    assert_eq!(sel, quer);
    assert!(
        col[0][0].label.contains("(Q)"),
        "a cara é a última: {}",
        col[0][0].label
    );
    assert_eq!(col[0][0].face, BarFace::Icon(IconId::MarqueeLasso));
    assert_eq!(col[0][1].face, BarFace::Icon(IconId::WandSparkles));
    assert_eq!(col[0][2].face, BarFace::Icon(IconId::Transform));
    assert!(col[0][2].label.contains("(V)"), "{}", col[0][2].label);
    // Com uma das quatro na mão o botão de grupo acende.
    i.tool = PixelTool::PolygonLasso;
    let col = ph2d_panel_pixel_tools::column::column(&i);
    assert!(col[0][0].on);
    // O triângulo e o flyout.
    let fechado = ph2d_panel_pixel_tools::column::bars(&i);
    assert_eq!(
        fechado.grouped,
        [ids::PIXEL_SELECT_GROUP, ids::PIXEL_SHAPE_GROUP]
    );
    assert!(fechado.flyout.is_none());
    i.groups.open = Some(ph2d_editor_core::pixel_panel::PixelToolGroup::Select);
    let bars = ph2d_panel_pixel_tools::column::bars(&i);
    let (ancora, quatro) = bars.flyout.expect("aberto");
    assert_eq!(ancora, ids::PIXEL_SELECT_GROUP);
    let ids4: Vec<_> = quatro.iter().map(|b| b.id).collect();
    assert_eq!(
        ids4,
        ids::TOOLS[..4]
            .iter()
            .map(|(_, id)| *id)
            .collect::<Vec<_>>()
    );
    assert!(
        quatro[3].on && quatro[3].label.contains("Shift+Q"),
        "{}",
        quatro[3].label
    );
    assert_eq!(quatro[3].face, BarFace::Icon(IconId::PolygonLasso));
}
