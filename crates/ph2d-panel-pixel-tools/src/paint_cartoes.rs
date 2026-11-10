//! ⭐ **O painel PIXEL em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): as mesmas
//! secções e os MESMOS ids; o `seam` não muda. A paleta, o seletor e os pincéis guardados são os widgets
//! de sempre, dentro do cartão e na tinta dele. O corpo corre o [`Corpo`] de sempre: os ajudantes de
//! linha já sabem pintar-se num cartão (T29), e os que não sabem perguntam [`Corpo::cartao`].

use super::*;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, paint_card_list};
use std::cell::RefCell;

/// O que os corpos escrevem: o corpo não tem store mutável e corre mais de uma vez por quadro.
struct Vista<'a> {
    seletor: &'a mut crate::state::Seletor,
    paleta: &'a mut crate::state::PaletaVista,
    pincel: &'a mut crate::state::PincelVista,
    blend: Option<(Rect, ph2d_editor_core::pixel_panel::PixelBlend)>,
    menu: Option<(Rect, paleta_menu::MenuPaleta)>,
    fonte: Option<Rect>,
}

fn corre(
    c: &mut CardCtx<'_>,
    vista: &RefCell<Vista<'_>>,
    y: f32,
    f: impl FnOnce(&mut Corpo<'_>, f32) -> f32,
) -> f32 {
    let mut v = vista.borrow_mut();
    let v = &mut *v;
    let mut k = Corpo {
        scene: &mut *c.scene,
        text_system: &mut *c.text_system,
        store: c.store,
        hit_index: &mut *c.hit_index,
        theme: c.ink.theme,
        x: c.x,
        w: c.w,
        cartao: Some((c.ink, y)),
        blend_aberto: None,
        menu_aberto: None,
        fonte_aberta: None,
        seletor: &mut *v.seletor,
        paleta_vista: &mut *v.paleta,
        pincel: &mut *v.pincel,
    };
    let fim = f(&mut k, y);
    // Vale o 1.º passe: o 2.º é o voo ou o solto, noutro sítio.
    let (b, me, fo) = (k.blend_aberto, k.menu_aberto, k.fonte_aberta);
    v.blend = v.blend.or(b);
    v.menu = v.menu.or(me);
    v.fonte = v.fonte.or(fo);
    fim
}

fn cartao<'a, 'v: 'a>(
    section: NodeId,
    title: &'a str,
    summary: Option<String>,
    vista: &'a RefCell<Vista<'v>>,
    f: impl Fn(&mut Corpo<'_>, f32) -> f32 + 'a,
) -> CardDef<'a> {
    CardDef {
        section,
        title,
        actions: Vec::new(),
        add: None,
        summary,
        body: Some(Box::new(move |c, y| corre(c, vista, y, &f))),
    }
}

/// Os cartões do painel; `titulo` = o das opções, o nome da ferramenta na mão.
fn cartoes<'a, 'v: 'a>(
    info: &'a PixelPanelInfo,
    titulo: &'a str,
    renaming: Option<u32>,
    vista: &'a RefCell<Vista<'v>>,
) -> Vec<CardDef<'a>> {
    if !info.editing {
        return vec![cartao(
            ids::PIXEL_SEC_OPTIONS,
            titulo,
            None,
            vista,
            |k, y| k.bloco(tr("panel.pixel.empty"), ColorToken::Text3, y),
        )];
    }
    let mut out = vec![cartao(
        ids::PIXEL_SEC_OPTIONS,
        titulo,
        None,
        vista,
        move |k, y| k.opcoes_no_cartao(info, y),
    )];
    if info.tool.uses_brush() {
        out.push(cartao(
            ids::PIXEL_SEC_BRUSHES,
            tr("panel.pixel.brush.saved_title"),
            Some(info.brushes.len().to_string()),
            vista,
            move |k, y| {
                let sec = Seccao::medida(k.text_system, 1, &[tr("panel.pixel.dynamics.min_size")]);
                k.guardados_da_mao(info, sec, y)
            },
        ));
    }
    if info.transform.is_some() {
        out.push(cartao(
            ids::PIXEL_SEC_TRANSFORM,
            tr("panel.pixel.transform"),
            None,
            vista,
            move |k, y| k.moldura(info, y),
        ));
    }
    out.push(cartao(
        ids::PIXEL_SEC_COLORS,
        tr("card.pixel.colors"),
        // O resumo do título diz a cor; o alfa está na linha da cor, e 8 dígitos não cabem a 220 px.
        Some(hex([info.fg[0], info.fg[1], info.fg[2], u8::MAX])),
        vista,
        move |k, y| k.cores(info, y),
    ));
    out.push(cartao(
        ids::PIXEL_SEC_LAYERS,
        tr("panel.pixel.layers"),
        Some(info.layers.len().to_string()),
        vista,
        move |k, y| k.camadas(info, renaming, y),
    ));
    out.push(cartao(
        ids::PIXEL_SEC_PALETTE,
        tr("card.pixel.palette"),
        Some(info.palette.len().to_string()),
        vista,
        move |k, y| {
            let y = k.paleta(info, y);
            let y = k.rampas(info, y);
            k.acoes_da_paleta(info, y)
        },
    ));
    out.push(cartao(
        ids::PIXEL_SEC_PICKER,
        tr("panel.pixel.selector"),
        Some(tr(info.options.selector.label_key()).to_string()),
        vista,
        move |k, y| k.seletor(info, y),
    ));
    if info.tool.paints() {
        out.push(cartao(
            ids::PIXEL_SEC_SYMMETRY,
            tr("panel.pixel.symmetry"),
            None,
            vista,
            move |k, y| {
                let sec = Seccao::medida(k.text_system, 1, &[tr("panel.pixel.sym_h")]);
                k.simetria(info, sec, y)
            },
        ));
    }
    out.push(cartao(
        ids::PIXEL_SEC_SELECT,
        tr("panel.pixel.select"),
        None,
        vista,
        move |k, y| k.menu_select(info, y),
    ));
    out.push(cartao(
        ids::PIXEL_SEC_EDIT,
        tr("panel.pixel.edit"),
        None,
        vista,
        move |k, y| k.menu_edit(info, y),
    ));
    out.push(cartao(
        ids::PIXEL_SEC_SPRITE,
        tr("panel.pixel.sprite"),
        None,
        vista,
        move |k, y| k.sprite(info, y),
    ));
    out
}

pub(super) fn pinta(st: &mut PixelToolsPanelState, ctx: &mut PaintCtx, renaming: Option<u32>) {
    let store = ctx.host.store_mut();
    store.mark_card_list_close(core_ids::PIXEL_PANEL, ids::PIXEL_CLOSE);
    // Os três menus do Aseprite são listas longas de verbos: abrem-se quando se vai a eles, e a paleta
    // fica acima da dobra (smoke do dono 01/11).
    for id in [
        ids::PIXEL_SEC_SELECT,
        ids::PIXEL_SEC_EDIT,
        ids::PIXEL_SEC_SPRITE,
    ] {
        store.set_collapsed_if_unchosen(id, true);
    }
    let theme = ctx.host.theme();
    st.seletor.rects = [None; 3];
    st.paleta = st.paleta.do_quadro_novo();
    st.pincel.trilhas = [None; 2];
    let info = with_current(Clone::clone);
    let titulo = if info.editing {
        info.tool.named(&info.tool_keys)
    } else {
        PixelToolsPanel::TITLE.tr().to_string()
    };
    let vista = RefCell::new(Vista {
        seletor: &mut st.seletor,
        paleta: &mut st.paleta,
        pincel: &mut st.pincel,
        blend: None,
        menu: None,
        fonte: None,
    });
    {
        let defs = cartoes(&info, &titulo, renaming, &vista);
        paint_card_list(
            ctx,
            core_ids::PIXEL_PANEL,
            PixelToolsPanel::TITLE.tr(),
            &defs,
        );
    }
    let v = vista.into_inner();
    let (blend, menu, fonte) = (v.blend, v.menu, v.fonte);
    if let Some(voando) = &st.paleta.voo {
        voo::voa(ctx.scene, theme, voando);
    }
    let viewport = ctx.viewport;
    if let Some((chip, cur)) = blend {
        let dd =
            ph2d_editor_core::widget::Dropdown::new(ids::LAYER_BLEND, "", camadas::blend_options())
                .selected(cur)
                .open(true);
        ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, viewport, theme, None);
    }
    if let Some(chip) = fonte {
        fonte::popover(ctx, chip, theme);
    }
    if let Some((chip, qual)) = menu {
        let dd = paleta_menu::dropdown(qual, &info);
        ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, viewport, theme, None);
    }
}
